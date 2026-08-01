use super::*;
use omena_cascade::SupportsTargetCapabilityV0;
use omena_parser::{
    ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldModuleMetadataV0,
    ClosedWorldSourcePrecisionSummaryV0, OpenWorldSnapshotV0,
};
use omena_query_transform_runner::{
    EmissionOrderingPolicyV0, LinkedEmissionArtifactV0, LinkedStylesheetWithEmissionItemsV0,
    TransformBundleDependencyResolutionV0, TransformBundleEdgeKind,
    TransformBundleEmissionAdmissionV0, TransformBundleEmissionItemProjectionV0,
    TransformBundleLinkErrorV0, TransformBundleLinkOptionsV0, TransformBundleLinkerProjectionV0,
    TransformBundleParsedModuleInputV0, TransformBundleResolvedDependencyV0,
    TransformBundleSemanticReachabilityInputV0, TransformBundleTransformedModuleV0,
    TransformModuleQualifiedExecutionErrorV0, bundle_edge_is_module_dependency,
    classify_transform_reachability_precision,
    evaluate_omena_transform_bundle_projection_emission_admission_with_resolved_dependencies_and_options,
    execute_transform_passes_on_module_with_dialect_context_policy_and_closed_world_bundle_and_retained_class_names,
    link_omena_transform_bundle_projection_with_resolved_dependencies_and_options,
    materialize_omena_transform_bundle_linked_stylesheet_with_emission_items,
    project_omena_transform_bundle_linker_and_emission_items_from_parsed_modules,
    project_omena_transform_bundle_linker_inputs_from_parsed_modules,
};
#[cfg(test)]
use omena_query_transform_runner::{
    TransformBundleModuleInputV0, link_omena_transform_bundle_modules,
    materialize_omena_transform_bundle_linked_stylesheet,
};
use omena_query_transform_runner::{
    transform_pass_requires_closed_world_bundle, transform_pass_sort_ordinal,
};
use std::path::{Path, PathBuf};

use super::parser_facade::{
    lex_omena_query_omena_parser_style_source, omena_parser_dialect_for_style_path,
    parse_omena_query_omena_parser_style_source,
};

mod context;
mod css_modules;
pub(super) use css_modules::derive_class_name_rewrites_for_transform_context;
mod design_tokens;
mod imports;
mod static_stylesheet;

use context::TransformResolutionContext;
pub use context::{
    derive_omena_query_module_reachability_from_engine_input,
    summarize_omena_query_transform_context_from_engine_input,
};

use context::{
    css_identifier_names_match, derive_omena_query_transform_context_from_engine_input,
    find_target_style_source, merge_target_options_transform_context, merge_transform_context,
    summarize_omena_query_transform_context_from_sources_with_resolution_context,
};
use imports::resolve_import_inline_replacement_for_transform_context;
use static_stylesheet::derive_static_scss_module_configurable_variable_names_for_transform_context;

pub(super) struct StaticScssModuleResolutionConfigurationEvidence {
    pub(super) configuration_signature: String,
    pub(super) configuration_variable_count: usize,
    pub(super) configuration_variable_names: Vec<String>,
    pub(super) module_instance_identity_key: Option<String>,
}

pub(super) fn derive_static_scss_module_resolution_configuration_evidence(
    style_source: &str,
    edge_kind: &str,
    rule_ordinal: usize,
    resolved_style_path: Option<&str>,
) -> StaticScssModuleResolutionConfigurationEvidence {
    let at_keyword = match edge_kind {
        "sassUse" => Some("@use"),
        "sassForward" => Some("@forward"),
        _ => None,
    };
    let variable_overrides = match at_keyword {
        Some("@forward") => {
            omena_semantic::derive_sass_module_forward_variable_override_values_at_ordinal(
                style_source,
                rule_ordinal,
            )
        }
        Some(at_keyword) => omena_semantic::derive_sass_module_rule_variable_overrides_at_ordinal(
            style_source,
            at_keyword,
            rule_ordinal,
        ),
        None => BTreeMap::new(),
    };
    let module_instance_identity_key =
        at_keyword
            .and(resolved_style_path)
            .map(|resolved_style_path| {
                omena_semantic::summarize_sass_module_instance_identity_key(
                    resolved_style_path,
                    &variable_overrides,
                )
            });

    StaticScssModuleResolutionConfigurationEvidence {
        configuration_signature: omena_semantic::summarize_sass_module_configuration_signature(
            &variable_overrides,
        ),
        configuration_variable_count: variable_overrides.len(),
        configuration_variable_names: variable_overrides.keys().cloned().collect(),
        module_instance_identity_key,
    }
}

pub(super) fn derive_static_scss_module_configurable_variable_names_for_resolution(
    style_path: &str,
    style_source: &str,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> BTreeSet<String> {
    derive_static_scss_module_configurable_variable_names_for_transform_context(
        style_path,
        style_source,
        available_style_paths,
        source_by_path,
        TransformResolutionContext {
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            disk_style_path_identities: &[],
        },
    )
}

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
    let mut execution_context = merge_target_options_transform_context(context, target_options);
    execution_context.supports_target_capability = Some(
        supports_target_capability_from_feature_support(target_support),
    );
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
    let vendor_prefix_policy = target_query_plan.vendor_prefix_policy;
    let supports_target_capability =
        supports_target_capability_from_feature_support(target_query_plan.support);
    let target = target_query_plan.transform_plan.clone();
    let mut execution_context = merge_target_options_transform_context(context, target_options);
    execution_context.vendor_prefix_policy = vendor_prefix_policy;
    execution_context.supports_target_capability = Some(supports_target_capability);
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

pub struct OmenaQueryBundlePlanInputV0<'a> {
    pub target_style_path: &'a str,
    pub style_sources: &'a [OmenaQueryStyleSourceInputV0],
    pub source_map_sources: &'a [OmenaQueryStyleSourceInputV0],
    pub requested_pass_ids: &'a [String],
    pub context: &'a TransformExecutionContextV0,
    pub resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    pub asset_rewrites: Vec<TransformBundleAssetUrlRewriteSummaryV0>,
    pub bundle_entry_style_paths: &'a [String],
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
    combined_passes.sort_by_key(|pass| transform_pass_sort_ordinal(*pass));
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
    let combined_pass_ids = combined_plan.ordered_pass_ids.clone();
    let egg_witnesses = execute_egg_rewrite_witnesses_for_css_source(
        parts.style_source,
        parts.dialect,
        &execution.output_css,
        &combined_pass_ids,
    );
    let semantic_removal_count = execution.semantic_removals.len();
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
            "cascadeProofObligations",
            "combinedTransformPassPlan",
        ],
    }
}

pub fn run_omena_query_bundle(
    input: OmenaQueryBundlePlanInputV0<'_>,
) -> Result<OmenaQueryBundleArtifactV0, String> {
    run_omena_query_bundle_with_semantic_inputs(input, &[]).map(|result| result.artifact)
}

pub fn run_omena_query_bundle_with_semantic_inputs(
    input: OmenaQueryBundlePlanInputV0<'_>,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Result<OmenaQueryBundleResultV0, String> {
    run_omena_query_bundle_with_semantic_inputs_and_options(
        input,
        external_sifs,
        &OmenaQueryConsumerBuildOptionsV0::default(),
    )
}

pub fn run_omena_query_bundle_with_semantic_inputs_and_options(
    input: OmenaQueryBundlePlanInputV0<'_>,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> Result<OmenaQueryBundleResultV0, String> {
    run_omena_query_bundle_with_execution_scope_evidence_and_options(input, external_sifs, options)
        .map(|result| result.bundle_result)
}

pub fn run_omena_query_bundle_with_execution_scope_evidence_and_options(
    input: OmenaQueryBundlePlanInputV0<'_>,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> Result<OmenaQueryBundleExecutionScopeResultV0, String> {
    let run = run_omena_query_bundle_with_optional_module_reachability(
        input,
        external_sifs,
        options,
        None,
    )?;
    Ok(OmenaQueryBundleExecutionScopeResultV0 {
        bundle_result: run.bundle_result,
        execution_scope: run.execution_scope,
        reachability_attribution: None,
    })
}

pub fn run_omena_query_bundle_with_module_reachability_and_options(
    input: OmenaQueryBundlePlanInputV0<'_>,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    options: &OmenaQueryConsumerBuildOptionsV0,
    module_reachability: &OmenaQueryEngineInputModuleReachabilityV0,
) -> Result<OmenaQueryModuleAttributedBundleResultV0, String> {
    let result =
        run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options(
            input,
            external_sifs,
            options,
            module_reachability,
        )?;
    let attribution = result.reachability_attribution.ok_or_else(|| {
        "module reachability run did not retain its attribution report".to_string()
    })?;
    Ok(OmenaQueryModuleAttributedBundleResultV0::new(
        result.bundle_result,
        attribution,
    ))
}

pub fn run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options(
    input: OmenaQueryBundlePlanInputV0<'_>,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    options: &OmenaQueryConsumerBuildOptionsV0,
    module_reachability: &OmenaQueryEngineInputModuleReachabilityV0,
) -> Result<OmenaQueryBundleExecutionScopeResultV0, String> {
    if find_target_style_source(input.target_style_path, input.style_sources).is_none() {
        return Err(format!(
            "module-attributed bundle target style path {:?} was not found in workspace style sources",
            input.target_style_path
        ));
    }
    let mut flat_context =
        merge_transform_context(input.context.clone(), module_reachability.context());
    flat_context
        .reachable_class_names
        .extend(module_reachability.projected_class_names().iter().cloned());
    flat_context.reachable_class_names.sort();
    flat_context.reachable_class_names.dedup();
    let style_paths = input
        .style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<Vec<_>>();
    let flat_class_names = module_reachability.flat_class_names_for_style_paths(
        style_paths.iter().copied(),
        flat_context.reachable_class_names.as_slice(),
    );
    let attribution_report = OmenaQueryModuleReachabilityAttributionReportV0::from_style_paths(
        module_reachability,
        style_paths.iter().copied(),
        flat_class_names.as_slice(),
    );
    let run = run_omena_query_bundle_with_optional_module_reachability(
        input,
        external_sifs,
        options,
        Some((module_reachability, &attribution_report)),
    )?;
    Ok(OmenaQueryBundleExecutionScopeResultV0 {
        bundle_result: run.bundle_result,
        execution_scope: run.execution_scope,
        reachability_attribution: Some(attribution_report),
    })
}

struct OmenaQueryBundleExecutionRunV0 {
    bundle_result: OmenaQueryBundleResultV0,
    execution_scope: Option<OmenaQueryBundleExecutionScopeEvidenceV0>,
}

fn run_omena_query_bundle_with_optional_module_reachability(
    input: OmenaQueryBundlePlanInputV0<'_>,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    options: &OmenaQueryConsumerBuildOptionsV0,
    module_reachability: Option<(
        &OmenaQueryEngineInputModuleReachabilityV0,
        &OmenaQueryModuleReachabilityAttributionReportV0,
    )>,
) -> Result<OmenaQueryBundleExecutionRunV0, String> {
    let OmenaQueryBundlePlanInputV0 {
        target_style_path,
        style_sources,
        source_map_sources,
        requested_pass_ids,
        context,
        resolution_inputs,
        asset_rewrites,
        bundle_entry_style_paths,
    } = input;
    let Some(target_source) = find_target_style_source(target_style_path, style_sources) else {
        return Err(format!(
            "target style path {target_style_path:?} was not found in workspace style sources"
        ));
    };
    let supplied_context = context;
    let attributed_context = module_reachability.map(|(reachability, _)| {
        merge_transform_context(supplied_context.clone(), reachability.context())
    });
    let base_context = attributed_context.as_ref().unwrap_or(supplied_context);
    let context = merge_workspace_transform_context(
        target_style_path,
        style_sources,
        base_context,
        TransformResolutionContext::from_resolution_inputs(resolution_inputs),
    );
    let reachability_context = if module_reachability.is_some() {
        supplied_context
    } else {
        &context
    };
    let attribution_report = module_reachability.map(|(_, report)| report);
    let effective_pass_ids = consumer_build_pass_set(requested_pass_ids).effective;
    let legacy_summary =
        (options.bundle_emission_path == OmenaQueryBundleEmissionPathV0::ImportInlineLegacy)
            .then(|| {
                execute_omena_query_consumer_build_style_sources_with_context_resolution_inputs_and_options(
                    target_style_path,
                    style_sources,
                    requested_pass_ids,
                    &context,
                    resolution_inputs,
                    options,
                )
            })
            .transpose()?;
    let bundle = summarize_omena_transform_bundle_from_source(
        target_style_path,
        target_source,
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let source_map_sources = if source_map_sources.is_empty() {
        style_sources
    } else {
        source_map_sources
    };
    let code_split_outputs = summarize_omena_query_bundle_code_split_workspace_plan(
        target_style_path,
        bundle_entry_style_paths,
        style_sources,
        resolution_inputs,
    )?
    .outputs;
    let link_options = match options.bundle_emission_path {
        OmenaQueryBundleEmissionPathV0::ImportInlineLegacy => {
            TransformBundleLinkOptionsV0::default()
        }
        OmenaQueryBundleEmissionPathV0::LinkedOrder => TransformBundleLinkOptionsV0 {
            emission_ordering_policy: EmissionOrderingPolicyV0::ImportOrderPreserving,
        },
    };
    let (legacy_open_decision, linked_result) = link_closed_world_stylesheet_for_style_sources(
        ClosedWorldStylesheetRequestV0 {
            target_style_path,
            style_sources,
            requested_pass_ids: &effective_pass_ids,
            context: &context,
            reachability_context,
            attribution_report,
            resolution_inputs,
            external_sifs,
        },
        link_options,
    )
    .into_parts();
    let closed_world_outcome = closed_world_outcome_from_link_result(
        linked_result.clone().map(|linked| linked.linked_stylesheet),
        &effective_pass_ids,
    );
    let closed_world_decision_parity = OmenaQueryClosedWorldDecisionParityV0 {
        legacy_open_decision,
        typed_outcome_open: closed_world_outcome.is_open(),
        equivalent: legacy_open_decision == closed_world_outcome.is_open(),
    };
    validate_omena_query_closed_world_decision_parity(&closed_world_decision_parity)?;

    let (
        execution,
        linked_materialization,
        emission_path,
        mut execution_scope,
        linked_module_executions,
    ) = match options.bundle_emission_path {
        OmenaQueryBundleEmissionPathV0::LinkedOrder => match linked_result.as_ref() {
            Ok(linked) => {
                let linked_execution = execute_linked_bundle_modules(
                    linked,
                    target_style_path,
                    style_sources,
                    &effective_pass_ids,
                    base_context,
                    resolution_inputs,
                    options,
                )?;
                let execution_scope = summarize_linked_bundle_execution_scope(&linked_execution)?;
                (
                    linked_execution.execution,
                    Some(linked_execution.materialization),
                    OmenaQueryBundleEmissionPathV0::LinkedOrder,
                    Some(execution_scope),
                    Some(linked_execution.module_executions),
                )
            }
            Err(error) => return Err(format!("linked bundle emission failed: {error:?}")),
        },
        OmenaQueryBundleEmissionPathV0::ImportInlineLegacy => {
            let Some(summary) = legacy_summary else {
                return Err("legacy bundle emission requires a consumer build summary".to_string());
            };
            (
                summary.execution,
                None,
                OmenaQueryBundleEmissionPathV0::ImportInlineLegacy,
                None,
                None,
            )
        }
    };
    let source_map_v3 = if let (Some(materialization), Some(module_executions)) = (
        linked_materialization.as_ref(),
        linked_module_executions.as_deref(),
    ) {
        let (source_map, dispositions) = summarize_omena_query_linked_bundle_source_map_v3(
            target_style_path,
            source_map_sources,
            &execution,
            materialization,
            module_executions,
        )?;
        if let Some(scope) = execution_scope.as_mut() {
            scope.source_map_dispositions = dispositions;
        }
        source_map
    } else {
        summarize_omena_query_consumer_build_source_map_v3_with_resolution_inputs(
            target_style_path,
            source_map_sources,
            &execution,
            resolution_inputs,
        )
    };

    let artifact = OmenaQueryBundleArtifactV0 {
        schema_version: "0",
        product: "omena-query.bundle-artifact",
        style_path: target_style_path.to_string(),
        emission_path,
        output_css: execution.output_css.clone(),
        bundle,
        source_map_v3,
        code_split_outputs,
        asset_rewrites,
        per_pass_provenance: execution.outcomes.clone(),
        execution,
        ready_surfaces: vec![
            "bundleOperationFacade",
            "transformBundlePlan",
            "transformExecutionRuntime",
            "sourceMapV3Serializer",
            "bundleCodeSplitPlan",
            "transformPassOutcomeContract",
        ],
    };
    Ok(OmenaQueryBundleExecutionRunV0 {
        bundle_result: OmenaQueryBundleResultV0 {
            artifact,
            closed_world_outcome,
            closed_world_decision_parity,
        },
        execution_scope,
    })
}

pub fn run_omena_query_bundle_for_style_sources_with_context(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundle_entry_style_paths: &[String],
) -> Result<OmenaQueryBundleArtifactV0, String> {
    run_omena_query_bundle_with_evidence_for_style_sources_with_context(
        target_style_path,
        style_sources,
        requested_pass_ids,
        context,
        package_manifests,
        bundle_entry_style_paths,
    )
    .map(|bundle| bundle.artifact)
}

pub fn run_omena_query_bundle_with_evidence_for_style_sources_with_context(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundle_entry_style_paths: &[String],
) -> Result<OmenaQueryBundleWithEvidenceV0, String> {
    let resolution_inputs = resolution_inputs_for_transform_style_sources(
        target_style_path,
        style_sources,
        package_manifests,
    );
    let result = run_omena_query_bundle_with_semantic_inputs(
        OmenaQueryBundlePlanInputV0 {
            target_style_path,
            style_sources,
            source_map_sources: style_sources,
            requested_pass_ids,
            context,
            resolution_inputs: &resolution_inputs,
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths,
        },
        &[],
    )?;
    let evidence = summarize_omena_query_bundle_evidence(&result);
    Ok(OmenaQueryBundleWithEvidenceV0 {
        artifact: result.artifact,
        closed_world_outcome: result.closed_world_outcome,
        closed_world_decision_parity: result.closed_world_decision_parity,
        evidence,
    })
}

pub fn run_omena_query_bundle_with_execution_scope_for_style_sources_with_context_and_options(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundle_entry_style_paths: &[String],
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> Result<OmenaQueryBundleExecutionScopeResultV0, String> {
    let resolution_inputs = resolution_inputs_for_transform_style_sources(
        target_style_path,
        style_sources,
        package_manifests,
    );
    run_omena_query_bundle_with_execution_scope_evidence_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path,
            style_sources,
            source_map_sources: style_sources,
            requested_pass_ids,
            context,
            resolution_inputs: &resolution_inputs,
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths,
        },
        &[],
        options,
    )
}

pub fn summarize_omena_query_bundle_evidence(
    result: &OmenaQueryBundleResultV0,
) -> OmenaQueryBundleEvidenceManifestV0 {
    let artifact = &result.artifact;
    let (outcome_status, reachability, blockers, interface_hashes, source_precision) = match &result
        .closed_world_outcome
    {
        OmenaQueryClosedWorldOutcomeV0::Closed { bundle } => (
            "closed",
            Some(OmenaQueryBundleReachabilityEvidenceV0 {
                guarantee: omena_evidence_graph::GuaranteeKindV0::NotClaimedExactTraversal,
                interpretation: "resolved-world exact BFS reachability; world incompleteness is represented by blockers",
                module_instances: bundle.reachability().module_instances().to_vec(),
                closure_hash: bundle.closure_hash().to_string(),
            }),
            Vec::new(),
            bundle.interface_hashes().entries().to_vec(),
            bundle.source_precision(),
        ),
        OmenaQueryClosedWorldOutcomeV0::Open { blockers } => {
            ("open", None, blockers.clone(), Vec::new(), None)
        }
    };
    OmenaQueryBundleEvidenceManifestV0 {
        schema_version: "0",
        product: "omena-query.bundle-evidence",
        style_path: artifact.style_path.clone(),
        outcome_status,
        reachability,
        gates: vec![
            OmenaQueryBundleEvidenceGateV0 {
                name: "resolvedWorldLink",
                passed: outcome_status == "closed",
            },
            OmenaQueryBundleEvidenceGateV0 {
                name: "closedWorldAdmission",
                passed: outcome_status == "closed" && blockers.is_empty(),
            },
            OmenaQueryBundleEvidenceGateV0 {
                name: "closedWorldDecisionParity",
                passed: result.closed_world_decision_parity.equivalent,
            },
        ],
        blockers,
        interface_hashes,
        source_precision,
    }
}

pub fn validate_omena_query_closed_world_decision_parity(
    parity: &OmenaQueryClosedWorldDecisionParityV0,
) -> Result<(), String> {
    if parity.equivalent && parity.legacy_open_decision == parity.typed_outcome_open {
        return Ok(());
    }
    Err(format!(
        "closed-world decision parity mismatch: legacyOpen={}, typedOutcomeOpen={}",
        parity.legacy_open_decision, parity.typed_outcome_open
    ))
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
    let parse_result = parse_omena_query_omena_parser_style_source(style_source, dialect);
    let runtime_index =
        omena_semantic::summarize_style_runtime_index_facts_from_source(style_path, style_source);
    let (class_selector_count, custom_property_count, keyframe_count, index_ready_surface) =
        if let Some(runtime_index) = runtime_index {
            (
                runtime_index.class_selector_names.len(),
                runtime_index.custom_property_names.len(),
                runtime_index.keyframe_names.len(),
                "semanticRuntimeIndexFacts",
            )
        } else {
            let style_facts = summarize_omena_query_omena_parser_style_facts(style_source, dialect);
            (
                style_facts.class_selector_names.len(),
                style_facts.custom_property_names.len(),
                style_facts.keyframe_names.len(),
                "parserFactSummary",
            )
        };

    OmenaQueryConsumerCheckSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-check-style-source",
        style_path: style_path.to_string(),
        dialect: omena_parser_style_dialect_label(dialect),
        token_count: parse_result.token_count(),
        parser_error_count: parse_result.errors().len(),
        class_selector_count,
        custom_property_count,
        keyframe_count,
        ready_surfaces: vec![
            "consumerCheckFacade",
            index_ready_surface,
            "styleDocumentDiagnostics",
        ],
    }
}

pub fn execute_omena_query_consumer_build_style_source(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
) -> OmenaQueryConsumerBuildSummaryV0 {
    execute_omena_query_consumer_build_style_source_with_context_and_options(
        style_path,
        style_source,
        requested_pass_ids,
        &TransformExecutionContextV0::default(),
        &OmenaQueryConsumerBuildOptionsV0::default(),
    )
}

pub fn execute_omena_query_consumer_build_style_source_with_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    execute_omena_query_consumer_build_style_source_with_context_and_options(
        style_path,
        style_source,
        requested_pass_ids,
        context,
        &OmenaQueryConsumerBuildOptionsV0::default(),
    )
}

pub fn execute_omena_query_consumer_build_style_source_with_context_and_options(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    execute_omena_query_consumer_build_style_source_with_context_and_reachability_precision(
        style_path,
        style_source,
        requested_pass_ids,
        context,
        None,
        false,
        options,
    )
}

fn execute_omena_query_consumer_build_style_source_with_context_and_reachability_precision(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    reachability_precision: Option<FactPrecision>,
    closed_set_enumeration_candidate: bool,
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let context = merge_single_source_transform_context(style_path, style_source, context);
    let pass_set = consumer_build_pass_set(requested_pass_ids);
    let closed_world_outcome =
        pass_ids_require_closed_world_bundle(&pass_set.effective).then(|| {
            build_closed_world_outcome_for_single_style_source_context(
                style_path,
                style_source,
                &pass_set.effective,
                &context,
            )
        });
    if let Some(closed_world_bundle) = closed_world_outcome
        .as_ref()
        .and_then(OmenaQueryClosedWorldOutcomeV0::bundle)
    {
        let reachability_precision = closed_world_bound_reachability_precision(
            &context,
            closed_world_bundle,
            reachability_precision,
            closed_set_enumeration_candidate,
        );
        return execute_omena_query_consumer_build_style_source_with_context_and_closed_world_bundle(
            style_path,
            style_source,
            &pass_set,
            &context,
            closed_world_bundle,
            reachability_precision,
            options,
        );
    }

    execute_omena_query_consumer_build_style_source_with_open_world_context(
        style_path,
        style_source,
        &pass_set,
        &context,
        options,
    )
}

struct ConsumerBuildPassSetV0 {
    requested: Vec<String>,
    effective: Vec<String>,
}

fn consumer_build_pass_set(requested_pass_ids: &[String]) -> ConsumerBuildPassSetV0 {
    ConsumerBuildPassSetV0 {
        requested: requested_pass_ids.to_vec(),
        effective: compute_effective_pass_ids(requested_pass_ids),
    }
}

fn compute_effective_pass_ids(requested_pass_ids: &[String]) -> Vec<String> {
    if !requested_pass_ids.is_empty() {
        return requested_pass_ids.to_vec();
    }

    all_transform_pass_kinds()
        .into_iter()
        .filter(|pass| {
            *pass != TransformPassKind::NativeCssStaticEval
                && !transform_pass_requires_closed_world_bundle(*pass)
        })
        .map(|pass| pass.id().to_string())
        .collect()
}

fn execution_policy_for_build_options(
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> TransformExecutionPolicyV0 {
    match options.verification_profile {
        OmenaQueryBuildVerificationProfileV0::Descriptive => TransformExecutionPolicyV0::default(),
        OmenaQueryBuildVerificationProfileV0::Strict => TransformExecutionPolicyV0::for_profile(
            omena_query_transform_runner::STRICT_VERIFICATION_BUILD_PROFILE_ID_V0,
        )
        .unwrap_or_default(),
    }
}

fn execute_omena_query_consumer_build_style_source_with_open_world_context(
    style_path: &str,
    style_source: &str,
    pass_set: &ConsumerBuildPassSetV0,
    context: &TransformExecutionContextV0,
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let execution_summary =
        execute_omena_query_transform_passes_from_source_with_open_world_context(
            style_path,
            style_source,
            &pass_set.effective,
            context,
            &execution_policy_for_build_options(options),
        );
    let open_world_snapshot = open_world_snapshot_for_closed_world_passes(&pass_set.effective);
    let ready_surfaces = consumer_build_ready_surfaces_with_open_world_snapshot(
        open_world_snapshot.as_ref(),
        vec![
            "consumerBuildFacade",
            "singleSourceTransformContextProducer",
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
        ],
    );

    OmenaQueryConsumerBuildSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-build-style-source",
        style_path: style_path.to_string(),
        dialect: omena_parser_style_dialect_label(omena_parser_dialect_for_style_path(style_path)),
        requested_pass_ids: pass_set.requested.clone(),
        effective_pass_ids: pass_set.effective.clone(),
        target_query: None,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
        bundle_emission_path: None,
        source_map_v3: None,
        open_world_snapshot,
        ready_surfaces,
    }
}

fn execute_omena_query_consumer_build_style_source_with_context_and_closed_world_bundle(
    style_path: &str,
    style_source: &str,
    pass_set: &ConsumerBuildPassSetV0,
    context: &TransformExecutionContextV0,
    closed_world_bundle: &ClosedWorldBundleV0,
    reachability_precision: FactPrecision,
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let context = merge_single_source_transform_context(style_path, style_source, context);
    let execution_summary =
        execute_omena_query_transform_passes_from_source_with_context_and_closed_world_bundle(
            style_path,
            style_source,
            &pass_set.effective,
            &context,
            closed_world_bundle,
            reachability_precision,
            &execution_policy_for_build_options(options),
        );

    OmenaQueryConsumerBuildSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-build-style-source",
        style_path: style_path.to_string(),
        dialect: omena_parser_style_dialect_label(omena_parser_dialect_for_style_path(style_path)),
        requested_pass_ids: pass_set.requested.clone(),
        effective_pass_ids: pass_set.effective.clone(),
        target_query: None,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
        bundle_emission_path: None,
        source_map_v3: None,
        open_world_snapshot: None,
        ready_surfaces: vec![
            "consumerBuildFacade",
            "singleSourceTransformContextProducer",
            "closedWorldBundle",
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
        ],
    }
}

struct ModuleQualifiedExecutionInputsV0<'a> {
    closed_world_bundle: &'a ClosedWorldBundleV0,
    module_instance: &'a omena_parser::ModuleInstanceKeyV0,
    reachability_precision: FactPrecision,
    retained_class_names: &'a [String],
}

fn execute_omena_query_consumer_build_style_module_with_context_and_closed_world_bundle(
    style_path: &str,
    style_source: &str,
    pass_set: &ConsumerBuildPassSetV0,
    context: &TransformExecutionContextV0,
    execution_inputs: ModuleQualifiedExecutionInputsV0<'_>,
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    let context = merge_single_source_transform_context(style_path, style_source, context);
    let execution_policy = execution_policy_for_build_options(options);
    let execution_summary =
        execute_omena_query_transform_passes_from_module_with_context_and_closed_world_bundle(
            style_path,
            style_source,
            &pass_set.effective,
            &context,
            execution_inputs,
            &execution_policy,
        )
        .map_err(|error| format!("module-qualified transform execution failed: {error:?}"))?;

    Ok(OmenaQueryConsumerBuildSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-build-style-source",
        style_path: style_path.to_string(),
        dialect: omena_parser_style_dialect_label(omena_parser_dialect_for_style_path(style_path)),
        requested_pass_ids: pass_set.requested.clone(),
        effective_pass_ids: pass_set.effective.clone(),
        target_query: None,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
        bundle_emission_path: None,
        source_map_v3: None,
        open_world_snapshot: None,
        ready_surfaces: vec![
            "consumerBuildFacade",
            "singleSourceTransformContextProducer",
            "closedWorldBundle",
            "moduleQualifiedReachability",
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
        ],
    })
}

pub fn execute_omena_query_consumer_build_style_source_with_engine_input_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    input: &EngineInputV2,
    closed_world_requested: bool,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let context_derivation = derive_omena_query_transform_context_from_engine_input(
        input,
        style_path,
        closed_world_requested,
    );
    let mut summary =
        execute_omena_query_consumer_build_style_source_with_context_and_reachability_precision(
            style_path,
            style_source,
            requested_pass_ids,
            context_derivation.module_reachability.context(),
            context_derivation.reachability_precision,
            context_derivation.closed_set_enumeration_candidate,
            &OmenaQueryConsumerBuildOptionsV0::default(),
        );
    summary
        .ready_surfaces
        .push("semanticReachabilityTransformContext");
    summary
        .ready_surfaces
        .push("expressionDomainSelectorProjection");
    summary
}

fn closed_world_bound_reachability_precision(
    context: &TransformExecutionContextV0,
    closed_world_bundle: &ClosedWorldBundleV0,
    open_world_precision: Option<FactPrecision>,
    closed_set_enumeration_candidate: bool,
) -> FactPrecision {
    let fallback = open_world_precision.unwrap_or(FactPrecision::Conservative);
    if !closed_set_enumeration_candidate
        || !fallback.satisfies(FactPrecision::Conservative)
        || context.reachable_class_names.is_empty()
    {
        return fallback;
    }

    let closed_world_class_names = closed_world_bundle
        .reachability()
        .class_names()
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let enumerated_class_names = context
        .reachable_class_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if enumerated_class_names
        .iter()
        .any(|name| !closed_world_class_names.contains(name.as_str()))
    {
        return fallback;
    }

    let value = AbstractClassValueV0::FiniteSet {
        values: enumerated_class_names.into_iter().collect(),
    };
    let witness = OmenaAbstractValuePrecisionWitnessV0 {
        direction: OmenaAbstractValueCoverageDirectionV0::SupersetOfProducible,
        basis: OmenaAbstractValuePrecisionBasisV0::ClosedSetEnumeration,
        authority_digest: Some(closed_world_bundle.closure_hash().to_string()),
    };
    fact_precision_from_class_value_with_witness(&value, Some(&witness))
}

pub fn execute_omena_query_consumer_build_style_sources_with_context(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    execute_omena_query_consumer_build_style_sources_with_context_and_options(
        target_style_path,
        style_sources,
        requested_pass_ids,
        context,
        package_manifests,
        &OmenaQueryConsumerBuildOptionsV0::default(),
    )
}

pub fn execute_omena_query_consumer_build_style_sources_with_context_and_options(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    let resolution_inputs = resolution_inputs_for_transform_style_sources(
        target_style_path,
        style_sources,
        package_manifests,
    );
    execute_omena_query_consumer_build_style_sources_with_context_resolution_inputs_and_options(
        target_style_path,
        style_sources,
        requested_pass_ids,
        context,
        &resolution_inputs,
        options,
    )
}

pub fn execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    execute_omena_query_consumer_build_style_sources_with_context_resolution_inputs_and_options(
        target_style_path,
        style_sources,
        requested_pass_ids,
        context,
        resolution_inputs,
        &OmenaQueryConsumerBuildOptionsV0::default(),
    )
}

pub fn execute_omena_query_consumer_build_style_sources_with_context_resolution_inputs_and_options(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    options: &OmenaQueryConsumerBuildOptionsV0,
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
        TransformResolutionContext::from_resolution_inputs(resolution_inputs),
    );
    let pass_set = consumer_build_pass_set(requested_pass_ids);
    let closed_world_outcome =
        pass_ids_require_closed_world_bundle(&pass_set.effective).then(|| {
            build_closed_world_outcome_for_style_sources(ClosedWorldStylesheetRequestV0 {
                target_style_path,
                style_sources,
                requested_pass_ids: &pass_set.effective,
                context: &context,
                reachability_context: &context,
                attribution_report: None,
                resolution_inputs,
                external_sifs: &[],
            })
        });
    let mut summary = if let Some(closed_world_bundle) = closed_world_outcome
        .as_ref()
        .and_then(OmenaQueryClosedWorldOutcomeV0::bundle)
    {
        execute_omena_query_consumer_build_style_source_with_context_and_closed_world_bundle(
            target_style_path,
            target_source,
            &pass_set,
            &context,
            closed_world_bundle,
            closed_world_bundle_reachability_precision(&context, &closed_world_bundle),
            options,
        )
    } else {
        execute_omena_query_consumer_build_style_source_with_open_world_context(
            target_style_path,
            target_source,
            &pass_set,
            &context,
            options,
        )
    };
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
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_and_additional_passes(
        style_path,
        style_source,
        target_query,
        context,
        target_options,
        &[],
    )
}

pub fn execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_and_additional_passes(
    style_path: &str,
    style_source: &str,
    target_query: &str,
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
    additional_pass_ids: &[String],
) -> OmenaQueryConsumerBuildSummaryV0 {
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_additional_passes_and_build_options(
        style_path,
        style_source,
        target_query,
        context,
        target_options,
        additional_pass_ids,
        &OmenaQueryConsumerBuildOptionsV0::default(),
    )
}

pub fn execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_additional_passes_and_build_options(
    style_path: &str,
    style_source: &str,
    target_query: &str,
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
    additional_pass_ids: &[String],
    build_options: &OmenaQueryConsumerBuildOptionsV0,
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
    let mut requested_pass_ids = plan
        .combined_pass_ids
        .iter()
        .map(|pass_id| (*pass_id).to_string())
        .collect::<Vec<_>>();
    extend_unique_pass_ids(&mut requested_pass_ids, additional_pass_ids);
    let mut execution_context = merge_target_options_transform_context(&context, target_options);
    execution_context.vendor_prefix_policy = plan
        .target_query
        .as_ref()
        .and_then(|target_query| target_query.vendor_prefix_policy);
    execution_context.supports_target_capability = plan
        .target_query
        .as_ref()
        .map(|target_query| supports_target_capability_from_feature_support(target_query.support));
    let execution_summary =
        execute_omena_query_consumer_build_style_source_with_context_and_options(
            style_path,
            style_source,
            &requested_pass_ids,
            &execution_context,
            build_options,
        );
    let ready_surfaces = extend_ready_surfaces(
        execution_summary.ready_surfaces.clone(),
        ["targetQueryBuildFacade"],
    );
    let ready_surfaces = consumer_build_ready_surfaces_with_open_world_snapshot(
        execution_summary.open_world_snapshot.as_ref(),
        ready_surfaces,
    );

    OmenaQueryConsumerBuildSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-build-style-source",
        style_path: plan.style_path,
        dialect: plan.dialect,
        requested_pass_ids,
        effective_pass_ids: execution_summary.effective_pass_ids,
        target_query: plan.target_query,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
        bundle_emission_path: None,
        source_map_v3: None,
        open_world_snapshot: execution_summary.open_world_snapshot,
        ready_surfaces,
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
    let resolution_inputs = resolution_inputs_for_transform_style_sources(
        target_style_path,
        style_sources,
        package_manifests,
    );
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options_and_resolution_inputs(
        target_style_path,
        style_sources,
        target_query,
        context,
        target_options,
        &resolution_inputs,
    )
}

pub fn execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options_and_resolution_inputs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    target_query: &str,
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_options_additional_passes_and_resolution_inputs(
        target_style_path,
        style_sources,
        target_query,
        context,
        target_options,
        &[],
        resolution_inputs,
    )
}

pub fn execute_omena_query_consumer_build_style_sources_for_target_query_with_context_options_additional_passes_and_resolution_inputs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    target_query: &str,
    context: &TransformExecutionContextV0,
    target_options: OmenaQueryTargetTransformOptionsV0,
    additional_pass_ids: &[String],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    let build_options = OmenaQueryConsumerBuildOptionsV0::default();
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_build_inputs(
        target_style_path,
        style_sources,
        target_query,
        context,
        OmenaQueryTargetConsumerBuildInputsV0 {
            target_options,
            additional_pass_ids,
            resolution_inputs,
            build_options: &build_options,
        },
    )
}

#[derive(Debug, Clone, Copy)]
pub struct OmenaQueryTargetConsumerBuildInputsV0<'a> {
    pub target_options: OmenaQueryTargetTransformOptionsV0,
    pub additional_pass_ids: &'a [String],
    pub resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    pub build_options: &'a OmenaQueryConsumerBuildOptionsV0,
}

pub fn execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_build_inputs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    target_query: &str,
    context: &TransformExecutionContextV0,
    inputs: OmenaQueryTargetConsumerBuildInputsV0<'_>,
) -> Result<OmenaQueryConsumerBuildSummaryV0, String> {
    let OmenaQueryTargetConsumerBuildInputsV0 {
        target_options,
        additional_pass_ids,
        resolution_inputs,
        build_options,
    } = inputs;
    let Some(target_source) = find_target_style_source(target_style_path, style_sources) else {
        return Err(format!(
            "target style path {target_style_path:?} was not found in workspace style sources"
        ));
    };
    let context = merge_workspace_transform_context(
        target_style_path,
        style_sources,
        context,
        TransformResolutionContext::from_resolution_inputs(resolution_inputs),
    );
    let plan = summarize_omena_query_transform_plan_from_target_query_with_context(
        target_style_path,
        target_source,
        target_query,
        target_options,
        default_omena_query_transform_print_options(),
        &context,
    );
    let mut requested_pass_ids = plan
        .combined_pass_ids
        .iter()
        .map(|pass_id| (*pass_id).to_string())
        .collect::<Vec<_>>();
    extend_unique_pass_ids(&mut requested_pass_ids, additional_pass_ids);
    let mut execution_context = merge_target_options_transform_context(&context, target_options);
    execution_context.vendor_prefix_policy = plan
        .target_query
        .as_ref()
        .and_then(|target_query| target_query.vendor_prefix_policy);
    execution_context.supports_target_capability = plan
        .target_query
        .as_ref()
        .map(|target_query| supports_target_capability_from_feature_support(target_query.support));
    let execution_summary = execute_omena_query_consumer_build_style_sources_with_context_resolution_inputs_and_options(
            target_style_path,
            style_sources,
            &requested_pass_ids,
            &execution_context,
            resolution_inputs,
            build_options,
        )?;
    let ready_surfaces = extend_ready_surfaces(
        execution_summary.ready_surfaces.clone(),
        [
            "targetQueryBuildFacade",
            "multiSourceTransformContextProducer",
        ],
    );
    let ready_surfaces = consumer_build_ready_surfaces_with_open_world_snapshot(
        execution_summary.open_world_snapshot.as_ref(),
        ready_surfaces,
    );

    Ok(OmenaQueryConsumerBuildSummaryV0 {
        schema_version: "0",
        product: "omena-query.consumer-build-style-source",
        style_path: plan.style_path,
        dialect: plan.dialect,
        requested_pass_ids,
        effective_pass_ids: execution_summary.effective_pass_ids,
        target_query: plan.target_query,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
        bundle_emission_path: None,
        source_map_v3: None,
        open_world_snapshot: execution_summary.open_world_snapshot,
        ready_surfaces,
    })
}

fn extend_unique_pass_ids(target: &mut Vec<String>, additional: &[String]) {
    for pass_id in additional {
        if !target.contains(pass_id) {
            target.push(pass_id.clone());
        }
    }
}

fn supports_target_capability_from_feature_support(
    support: OmenaQueryTargetFeatureSupportV0,
) -> SupportsTargetCapabilityV0 {
    SupportsTargetCapabilityV0 {
        supports_light_dark: support.supports_light_dark,
        supports_color_mix: support.supports_color_mix,
        supports_oklch_oklab: support.supports_oklch_oklab,
        supports_color_function: support.supports_color_function,
        supports_relative_color: support.supports_relative_color,
        supports_logical_properties: support.supports_logical_properties,
        supports_css_nesting: support.supports_css_nesting,
        supports_css_scope: support.supports_css_scope,
        supports_cascade_layers: support.supports_cascade_layers,
    }
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

pub fn attach_omena_query_consumer_build_bundle_summary(
    summary: &mut OmenaQueryConsumerBuildSummaryV0,
    style_source: &str,
) {
    let bundle = summarize_omena_transform_bundle_from_source(
        &summary.style_path,
        style_source,
        omena_parser_dialect_for_style_path(&summary.style_path),
    );
    summary.bundle = Some(bundle);
    if !summary.ready_surfaces.contains(&"bundleAssetUrlResolution") {
        summary.ready_surfaces.push("bundleAssetUrlResolution");
    }
    if summary
        .bundle
        .as_ref()
        .is_some_and(|bundle| bundle.code_splitting_required)
        && !summary.ready_surfaces.contains(&"bundleCodeSplitPlan")
    {
        summary.ready_surfaces.push("bundleCodeSplitPlan");
    }
}

pub fn summarize_omena_query_bundle_code_split_workspace_plan(
    primary_entry_style_path: &str,
    bundle_entry_style_paths: &[String],
    style_sources: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Result<OmenaQueryBundleCodeSplitWorkspacePlanV0, String> {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let dependency_specifiers_by_path =
        collect_omena_query_bundle_code_split_dependency_specifiers(style_sources);
    let mut entry_style_paths = vec![primary_entry_style_path.to_string()];
    for configured_entry in bundle_entry_style_paths {
        if configured_entry != primary_entry_style_path
            && !entry_style_paths.contains(configured_entry)
        {
            entry_style_paths.push(configured_entry.clone());
        }
    }
    for entry_style_path in &entry_style_paths {
        if !available_style_paths.contains(entry_style_path.as_str()) {
            return Err(format!(
                "bundle entry source is not loaded: {entry_style_path}"
            ));
        }
    }

    let entry_style_path_set = entry_style_paths.iter().cloned().collect::<BTreeSet<_>>();
    let entry_reachability = collect_omena_query_bundle_code_split_entry_reachability(
        entry_style_paths.as_slice(),
        &dependency_specifiers_by_path,
        &available_style_paths,
        resolution_inputs,
    );

    let mut outputs = Vec::new();
    for (style_path, reachable_from_entries) in entry_reachability {
        let split_boundary = omena_query_bundle_code_split_boundary(
            style_path.as_str(),
            primary_entry_style_path,
            &entry_style_path_set,
            reachable_from_entries.len(),
        );
        outputs.push(OmenaQueryBundleCodeSplitWorkspacePlanOutputV0 {
            is_entry: entry_style_path_set.contains(style_path.as_str()),
            source_path: style_path,
            split_boundary,
            reachable_from_entries: reachable_from_entries.into_iter().collect(),
        });
    }
    let configured_entry_count = outputs
        .iter()
        .filter(|output| output.split_boundary == "entryConfig")
        .count();
    let shared_boundary_count = outputs
        .iter()
        .filter(|output| output.split_boundary == "shared")
        .count();
    let mut ready_surfaces = vec!["bundleCodeSplitPlan", "bundleCodeSplitBoundaryPlan"];
    if configured_entry_count > 0 {
        ready_surfaces.push("bundleCodeSplitEntryConfig");
    }
    if shared_boundary_count > 0 {
        ready_surfaces.push("bundleCodeSplitSharedChunkPlan");
    }

    Ok(OmenaQueryBundleCodeSplitWorkspacePlanV0 {
        schema_version: "0",
        product: "omena-query.bundle-code-split-workspace-plan",
        primary_entry_style_path: primary_entry_style_path.to_string(),
        configured_entry_count,
        output_count: outputs.len(),
        shared_boundary_count,
        outputs,
        ready_surfaces,
    })
}

fn collect_omena_query_bundle_code_split_dependency_specifiers(
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> BTreeMap<&str, Vec<String>> {
    let modules = style_sources_to_transform_bundle_modules(style_sources);
    let projection =
        project_omena_transform_bundle_linker_inputs_from_parsed_modules(&modules, &[]);
    let projection_path_by_source_path =
        projection_path_by_source_path(modules.as_slice(), style_sources);
    let dependency_specifiers_by_projection_path = projection
        .inputs()
        .iter()
        .map(|input| {
            let specifiers = input
                .dependency_edges
                .iter()
                .filter(|edge| bundle_edge_is_module_dependency(edge.kind))
                .map(|edge| edge.import_source.clone())
                .collect::<Vec<_>>();
            (input.source_path.as_str(), specifiers)
        })
        .collect::<BTreeMap<_, _>>();

    style_sources
        .iter()
        .map(|source| {
            let specifiers = projection_path_by_source_path
                .get(source.style_path.as_str())
                .and_then(|projection_path| {
                    dependency_specifiers_by_projection_path.get(projection_path.as_str())
                })
                .cloned()
                .unwrap_or_default();
            (source.style_path.as_str(), specifiers)
        })
        .collect()
}

fn collect_omena_query_bundle_code_split_entry_reachability(
    entry_style_paths: &[String],
    dependency_specifiers_by_path: &BTreeMap<&str, Vec<String>>,
    available_style_paths: &BTreeSet<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> BTreeMap<String, BTreeSet<String>> {
    let resolution_context = TransformResolutionContext::from_resolution_inputs(resolution_inputs);
    let mut reachability = BTreeMap::<String, BTreeSet<String>>::new();

    for entry_style_path in entry_style_paths {
        let mut visited = BTreeSet::new();
        let mut stack = vec![entry_style_path.clone()];

        while let Some(style_path) = stack.pop() {
            if !visited.insert(style_path.clone()) {
                continue;
            }
            let Some(import_sources) = dependency_specifiers_by_path.get(style_path.as_str())
            else {
                continue;
            };
            reachability
                .entry(style_path.clone())
                .or_default()
                .insert(entry_style_path.clone());
            for import_source in import_sources {
                let Some(target_path) = resolution_context.resolve_style_module_source(
                    style_path.as_str(),
                    import_source,
                    available_style_paths,
                ) else {
                    continue;
                };
                if dependency_specifiers_by_path.contains_key(target_path.as_str()) {
                    stack.push(target_path);
                }
            }
        }
    }

    reachability
}

fn omena_query_bundle_code_split_boundary(
    style_path: &str,
    primary_entry_style_path: &str,
    entry_style_paths: &BTreeSet<String>,
    reachable_entry_count: usize,
) -> &'static str {
    if style_path == primary_entry_style_path {
        return "entry";
    }
    if entry_style_paths.contains(style_path) {
        return "entryConfig";
    }
    if reachable_entry_count > 1 {
        return "shared";
    }
    "styleDependency"
}

pub fn attach_omena_query_consumer_build_source_map_v3(
    summary: &mut OmenaQueryConsumerBuildSummaryV0,
    style_source: &str,
) {
    let style_source = OmenaQueryStyleSourceInputV0 {
        style_path: summary.style_path.clone(),
        style_source: style_source.to_string(),
    };
    attach_omena_query_consumer_build_source_map_v3_with_sources(summary, &[style_source], &[]);
}

pub fn attach_omena_query_consumer_build_source_map_v3_with_sources(
    summary: &mut OmenaQueryConsumerBuildSummaryV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) {
    let resolution_inputs = resolution_inputs_for_transform_style_sources(
        summary.style_path.as_str(),
        style_sources,
        package_manifests,
    );
    attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs(
        summary,
        style_sources,
        &resolution_inputs,
    );
}

pub fn attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs(
    summary: &mut OmenaQueryConsumerBuildSummaryV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) {
    let source_map = summarize_omena_query_consumer_build_source_map_v3_with_resolution_inputs(
        &summary.style_path,
        style_sources,
        &summary.execution,
        resolution_inputs,
    );
    summary.source_map_v3 = Some(source_map);
    if !summary.ready_surfaces.contains(&"sourceMapV3Serializer") {
        summary.ready_surfaces.push("sourceMapV3Serializer");
    }
    if summary
        .source_map_v3
        .as_ref()
        .is_some_and(|source_map| source_map.sources.len() > 1)
        && !summary
            .ready_surfaces
            .contains(&"bundleSourceMapOriginChain")
    {
        summary.ready_surfaces.push("bundleSourceMapOriginChain");
    }
}

pub fn summarize_omena_query_consumer_build_source_map_v3(
    style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    execution: &TransformExecutionSummaryV0,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryTransformSourceMapV3V0 {
    let resolution_inputs =
        resolution_inputs_for_transform_style_sources(style_path, style_sources, package_manifests);
    summarize_omena_query_consumer_build_source_map_v3_with_resolution_inputs(
        style_path,
        style_sources,
        execution,
        &resolution_inputs,
    )
}

pub fn summarize_omena_query_consumer_build_source_map_v3_with_resolution_inputs(
    style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    execution: &TransformExecutionSummaryV0,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryTransformSourceMapV3V0 {
    let source_by_path = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let style_source = source_by_path.get(style_path).copied().unwrap_or_default();
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let artifact = print_transform_execution_artifact_with_dialect_and_source(
        style_path,
        style_source,
        dialect,
        format!(
            "omena-query-consumer-build-source-map-v3:{}:{}",
            style_path,
            style_source.len()
        ),
        &[TransformPassKind::PrintCss],
        default_omena_query_transform_print_options(),
        execution,
    );
    let available_style_paths = source_by_path.keys().copied().collect::<BTreeSet<_>>();
    let mut segments = artifact.source_map_segments.clone();
    segments.extend(import_inline_source_map_segments(
        style_path,
        execution,
        &source_by_path,
        &available_style_paths,
        TransformResolutionContext::from_resolution_inputs(resolution_inputs),
    ));
    let source_contents = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    serialize_transform_source_map_v3_with_source_contents(
        style_path,
        execution.output_css.as_str(),
        style_path,
        source_contents.as_slice(),
        segments.as_slice(),
    )
}

fn summarize_omena_query_linked_bundle_source_map_v3(
    style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    execution: &TransformExecutionSummaryV0,
    materialization: &LinkedEmissionArtifactV0,
    module_executions: &[LinkedModuleExecutionV0],
) -> Result<
    (
        OmenaQueryTransformSourceMapV3V0,
        Vec<OmenaQueryLinkedSourceMapDispositionV0>,
    ),
    String,
> {
    let (segments, dispositions) = linked_bundle_source_map_segments(
        style_sources,
        execution.output_css.as_str(),
        materialization,
        module_executions,
    )?;
    let source_contents = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    Ok((
        serialize_transform_source_map_v3_with_source_contents(
            style_path,
            execution.output_css.as_str(),
            style_path,
            source_contents.as_slice(),
            segments.as_slice(),
        ),
        dispositions,
    ))
}

fn linked_bundle_source_map_segments(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    generated_css: &str,
    materialization: &LinkedEmissionArtifactV0,
    module_executions: &[LinkedModuleExecutionV0],
) -> Result<
    (
        Vec<TransformSourceMapSegmentV0>,
        Vec<OmenaQueryLinkedSourceMapDispositionV0>,
    ),
    String,
> {
    let source_by_path = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let execution_by_instance = module_executions
        .iter()
        .map(|module| (&module.module_instance, &module.execution))
        .collect::<BTreeMap<_, _>>();
    let mut segments = Vec::new();
    let mut dispositions = Vec::new();
    for region in &materialization.module_regions {
        let source_path = region.module_instance.module().as_str();
        let source = source_by_path.get(source_path).copied().ok_or_else(|| {
            format!("linked source-map module {source_path:?} has no source document")
        })?;
        let module_execution = execution_by_instance
            .get(&region.module_instance)
            .copied()
            .ok_or_else(|| {
                format!(
                    "linked source-map module {:?} has no retained execution",
                    region.module_instance
                )
            })?;
        if region.generated_start > region.generated_end
            || region.generated_end > generated_css.len()
        {
            return Err(format!(
                "linked source-map region for {source_path:?} is outside generated CSS: {}..{} of {}",
                region.generated_start,
                region.generated_end,
                generated_css.len()
            ));
        }
        let (mut module_segments, granularity, fallback_reason) =
            if source == module_execution.output_css {
                let artifact = print_omena_query_transform_source_with_pretty_options(
                    source_path,
                    source,
                    transform_print_dialect_for_style_path(source_path),
                    format!("linked-module-source-map:{source_path}"),
                    &[],
                    default_omena_query_transform_print_options(),
                    OmenaQueryPrettyFormatOptionsV0 {
                        line_width: 100,
                        indent_width: 2,
                    },
                );
                (
                    artifact.source_map_segments,
                    OmenaQueryLinkedSourceMapGranularityV0::CstAnchors,
                    None,
                )
            } else {
                let (segment, fallback_reason) = linked_whole_module_fallback_segment(
                    source_path,
                    source,
                    module_execution.output_css.as_str(),
                );
                (
                    vec![segment],
                    OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback,
                    Some(fallback_reason),
                )
            };
        for segment in &module_segments {
            validate_linked_source_map_original_segment(
                source_path,
                source,
                module_execution.output_css.as_str(),
                segment,
                granularity,
                fallback_reason,
            )?;
        }
        let segment_start = segments.len();
        for segment in &mut module_segments {
            segment.generated_start += region.generated_start;
            segment.generated_end += region.generated_start;
            if segment.generated_start < region.generated_start
                || segment.generated_end > region.generated_end
            {
                return Err(format!(
                    "linked source-map segment for {source_path:?} is outside its materialized region: {}..{} not within {}..{}",
                    segment.generated_start,
                    segment.generated_end,
                    region.generated_start,
                    region.generated_end
                ));
            }
            segment.generated_start_point =
                transform_source_map_point(generated_css, segment.generated_start);
            segment.generated_end_point =
                transform_source_map_point(generated_css, segment.generated_end);
            segment.pass_id = "linked-order-emission";
        }
        segments.extend(module_segments);
        dispositions.push(OmenaQueryLinkedSourceMapDispositionV0 {
            module_instance: region.module_instance.clone(),
            granularity,
            fallback_reason,
            segment_count: segments.len() - segment_start,
        });
    }
    Ok((segments, dispositions))
}

pub(crate) const LINKED_FALLBACK_EXACT_TOKEN_REASON: &str =
    "module output differs; fallback anchors a unique surviving token sequence";
pub(crate) const LINKED_FALLBACK_AMBIGUOUS_TOKEN_REASON: &str = "module output differs; fallback uses source-start convention because the surviving token sequence is ambiguous";
pub(crate) const LINKED_FALLBACK_SOURCE_START_REASON: &str =
    "module output differs; fallback uses source-start convention without token correspondence";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkedFallbackSourceTokenRangeV0 {
    NoMatch,
    Unique { start: usize, end: usize },
    Ambiguous,
}

fn linked_whole_module_fallback_segment(
    source_path: &str,
    source: &str,
    generated_module_css: &str,
) -> (TransformSourceMapSegmentV0, &'static str) {
    let token_range =
        linked_fallback_exact_source_token_range(source_path, source, generated_module_css);
    let (original_start, original_end, reason) = match token_range {
        LinkedFallbackSourceTokenRangeV0::Unique { start, end } => {
            (start, end, LINKED_FALLBACK_EXACT_TOKEN_REASON)
        }
        LinkedFallbackSourceTokenRangeV0::Ambiguous => (
            linked_fallback_source_start(source_path, source),
            source.len(),
            LINKED_FALLBACK_AMBIGUOUS_TOKEN_REASON,
        ),
        LinkedFallbackSourceTokenRangeV0::NoMatch => (
            linked_fallback_source_start(source_path, source),
            source.len(),
            LINKED_FALLBACK_SOURCE_START_REASON,
        ),
    };
    (
        TransformSourceMapSegmentV0 {
            source_path: source_path.to_string(),
            original_start,
            original_end,
            generated_start: 0,
            generated_end: generated_module_css.len(),
            original_start_point: transform_source_map_point(source, original_start),
            original_end_point: transform_source_map_point(source, original_end),
            generated_start_point: transform_source_map_point(generated_module_css, 0),
            generated_end_point: transform_source_map_point(
                generated_module_css,
                generated_module_css.len(),
            ),
            pass_id: "linked-order-emission",
        },
        reason,
    )
}

fn linked_fallback_exact_source_token_range(
    source_path: &str,
    source: &str,
    generated_module_css: &str,
) -> LinkedFallbackSourceTokenRangeV0 {
    let dialect = omena_parser_dialect_for_style_path(source_path);
    let source_lexed = lex_omena_query_omena_parser_style_source(source, dialect);
    let generated_lexed = lex_omena_query_omena_parser_style_source(generated_module_css, dialect);
    if !source_lexed.errors().is_empty() || !generated_lexed.errors().is_empty() {
        return LinkedFallbackSourceTokenRangeV0::NoMatch;
    }
    let source_tokens = canonical_linked_fallback_tokens(source_lexed.tokens());
    let generated_tokens = canonical_linked_fallback_tokens(generated_lexed.tokens());
    if generated_tokens.is_empty() || source_tokens.len() < generated_tokens.len() {
        return LinkedFallbackSourceTokenRangeV0::NoMatch;
    }
    let mut matching_ranges = source_tokens
        .windows(generated_tokens.len())
        .filter(|window| {
            window
                .iter()
                .zip(&generated_tokens)
                .all(|(source_token, generated_token)| {
                    source_token.kind == generated_token.kind
                        && source_token.text == generated_token.text
                })
        })
        .filter_map(|window| {
            let first = window.first()?;
            let last = window.last()?;
            Some((
                u32::from(first.range.start()) as usize,
                u32::from(last.range.end()) as usize,
            ))
        });
    let Some((start, end)) = matching_ranges.next() else {
        return LinkedFallbackSourceTokenRangeV0::NoMatch;
    };
    if matching_ranges.next().is_some() {
        LinkedFallbackSourceTokenRangeV0::Ambiguous
    } else {
        LinkedFallbackSourceTokenRangeV0::Unique { start, end }
    }
}

fn linked_fallback_source_start(source_path: &str, source: &str) -> usize {
    let dialect = omena_parser_dialect_for_style_path(source_path);
    let lexed = lex_omena_query_omena_parser_style_source(source, dialect);
    if !lexed.errors().is_empty() {
        return source.len();
    }
    let tokens = lexed
        .tokens()
        .iter()
        .filter(|token| !token.kind.is_trivia())
        .collect::<Vec<_>>();
    let mut cursor = 0;
    while tokens.get(cursor).is_some_and(|token| {
        token.kind == omena_syntax::SyntaxKind::AtKeyword
            && token.text.eq_ignore_ascii_case("@import")
    }) {
        let Some(relative_end) = tokens[cursor..]
            .iter()
            .position(|token| token.kind == omena_syntax::SyntaxKind::Semicolon)
        else {
            return source.len();
        };
        cursor += relative_end + 1;
    }
    tokens.get(cursor).map_or(source.len(), |token| {
        u32::from(token.range.start()) as usize
    })
}

fn canonical_linked_fallback_tokens(
    tokens: &[omena_parser::LexedToken],
) -> Vec<&omena_parser::LexedToken> {
    let non_trivia = tokens
        .iter()
        .filter(|token| !token.kind.is_trivia())
        .collect::<Vec<_>>();
    non_trivia
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let optional_terminal_semicolon = token.kind == omena_syntax::SyntaxKind::Semicolon
                && non_trivia
                    .get(index + 1)
                    .is_some_and(|next| next.kind == omena_syntax::SyntaxKind::RightBrace);
            (!optional_terminal_semicolon).then_some(*token)
        })
        .collect()
}

fn validate_linked_source_map_original_segment(
    source_path: &str,
    source: &str,
    generated_module_css: &str,
    segment: &TransformSourceMapSegmentV0,
    granularity: OmenaQueryLinkedSourceMapGranularityV0,
    fallback_reason: Option<&str>,
) -> Result<(), String> {
    if segment.source_path != source_path
        || segment.original_start > segment.original_end
        || segment.original_end > source.len()
        || !source.is_char_boundary(segment.original_start)
        || !source.is_char_boundary(segment.original_end)
    {
        return Err(format!(
            "linked source-map segment for {source_path:?} has invalid original range {}..{} of {}",
            segment.original_start,
            segment.original_end,
            source.len()
        ));
    }
    let expected_start_point = transform_source_map_point(source, segment.original_start);
    let expected_end_point = transform_source_map_point(source, segment.original_end);
    if segment.original_start_point != expected_start_point
        || segment.original_end_point != expected_end_point
    {
        return Err(format!(
            "linked source-map segment for {source_path:?} has original points inconsistent with {}..{}",
            segment.original_start, segment.original_end
        ));
    }
    if granularity == OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback {
        let token_range =
            linked_fallback_exact_source_token_range(source_path, source, generated_module_css);
        let expected_source_start = linked_fallback_source_start(source_path, source);
        match fallback_reason {
            Some(LINKED_FALLBACK_EXACT_TOKEN_REASON)
                if token_range
                    != (LinkedFallbackSourceTokenRangeV0::Unique {
                        start: segment.original_start,
                        end: segment.original_end,
                    }) =>
            {
                return Err(format!(
                    "linked source-map fallback for {source_path:?} claims correspondence without one unique matching token window"
                ));
            }
            Some(LINKED_FALLBACK_AMBIGUOUS_TOKEN_REASON) => {
                if token_range != LinkedFallbackSourceTokenRangeV0::Ambiguous
                    || segment.original_start != expected_source_start
                    || segment.original_end != source.len()
                {
                    return Err(format!(
                        "linked source-map fallback for {source_path:?} has a dishonest ambiguous-token convention"
                    ));
                }
            }
            Some(LINKED_FALLBACK_SOURCE_START_REASON) => {
                if token_range != LinkedFallbackSourceTokenRangeV0::NoMatch
                    || segment.original_start != expected_source_start
                    || segment.original_end != source.len()
                {
                    return Err(format!(
                        "linked source-map fallback for {source_path:?} has a dishonest source-start convention"
                    ));
                }
            }
            Some(LINKED_FALLBACK_EXACT_TOKEN_REASON) => {}
            _ => {
                return Err(format!(
                    "linked source-map fallback for {source_path:?} has no recognized anchor disclosure"
                ));
            }
        }
    }
    Ok(())
}

fn transform_print_dialect_for_style_path(style_path: &str) -> OmenaQueryTransformStyleDialect {
    if style_path.ends_with(".sass") {
        OmenaQueryTransformStyleDialect::Sass
    } else if style_path.ends_with(".scss") {
        OmenaQueryTransformStyleDialect::Scss
    } else if style_path.ends_with(".less") {
        OmenaQueryTransformStyleDialect::Less
    } else {
        OmenaQueryTransformStyleDialect::Css
    }
}

pub fn summarize_omena_query_bundle_code_split_source_map_v3(
    output_file_name: &str,
    generated_css: &str,
    source_path: &str,
    source_content: &str,
) -> OmenaQueryTransformSourceMapV3V0 {
    let segment = TransformSourceMapSegmentV0 {
        source_path: source_path.to_string(),
        original_start: 0,
        original_end: source_content.len(),
        generated_start: 0,
        generated_end: generated_css.len(),
        original_start_point: transform_source_map_point(source_content, 0),
        original_end_point: transform_source_map_point(source_content, source_content.len()),
        generated_start_point: transform_source_map_point(generated_css, 0),
        generated_end_point: transform_source_map_point(generated_css, generated_css.len()),
        pass_id: "code-split-emission",
    };
    serialize_transform_source_map_v3_with_source_contents(
        output_file_name,
        generated_css,
        source_path,
        &[(source_path, source_content)],
        &[segment],
    )
}

fn import_inline_source_map_segments(
    style_path: &str,
    execution: &TransformExecutionSummaryV0,
    source_by_path: &BTreeMap<&str, &str>,
    available_style_paths: &BTreeSet<&str>,
    resolution_context: TransformResolutionContext<'_>,
) -> Vec<TransformSourceMapSegmentV0> {
    let mut segments = Vec::new();
    let mut seen_segments = BTreeSet::new();
    extend_import_graph_source_map_segments(
        &mut segments,
        &mut seen_segments,
        style_path,
        execution,
        source_by_path,
        available_style_paths,
        resolution_context,
    );
    let mut search_start = 0;
    for inline in &execution.css_import_inlines {
        if inline.replacement_css.is_empty() || search_start > execution.output_css.len() {
            continue;
        }
        let Some(resolved_style_path) = resolution_context.resolve_style_module_source(
            style_path,
            inline.import_source.as_str(),
            available_style_paths,
        ) else {
            continue;
        };
        let Some(imported_source) = source_by_path.get(resolved_style_path.as_str()).copied()
        else {
            continue;
        };
        let Some((generated_start, generated_end, _exact_match)) =
            find_import_origin_generated_range(
                execution.output_css.as_str(),
                search_start..execution.output_css.len(),
                &inline.replacement_css,
                resolved_style_path.as_str(),
                imported_source,
            )
        else {
            continue;
        };
        push_unique_import_origin_segment(
            &mut segments,
            &mut seen_segments,
            resolved_style_path,
            imported_source,
            execution.output_css.as_str(),
            generated_start,
            generated_end,
        );
        search_start = generated_end;
    }
    segments
}

fn extend_import_graph_source_map_segments(
    segments: &mut Vec<TransformSourceMapSegmentV0>,
    seen_segments: &mut BTreeSet<(String, usize, usize, &'static str)>,
    style_path: &str,
    execution: &TransformExecutionSummaryV0,
    source_by_path: &BTreeMap<&str, &str>,
    available_style_paths: &BTreeSet<&str>,
    resolution_context: TransformResolutionContext<'_>,
) {
    let style_sources = source_by_path
        .iter()
        .map(|(style_path, style_source)| (*style_path, *style_source))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_sources.as_slice());
    let entries_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let owned_source_by_path = source_by_path
        .iter()
        .map(|(style_path, style_source)| ((*style_path).to_string(), (*style_source).to_string()))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = BTreeSet::new();
    let context = ImportGraphSourceMapSegmentContext {
        output_css: execution.output_css.as_str(),
        entries_by_path: &entries_by_path,
        owned_source_by_path: &owned_source_by_path,
        source_by_path,
        available_style_paths,
        resolution_context,
    };
    collect_import_graph_source_map_segments(
        segments,
        seen_segments,
        style_path,
        0,
        execution.output_css.len(),
        &context,
        &mut visiting,
    );
}

struct ImportGraphSourceMapSegmentContext<'a> {
    output_css: &'a str,
    entries_by_path: &'a BTreeMap<&'a str, &'a OmenaQueryStyleFactEntry>,
    owned_source_by_path: &'a BTreeMap<String, String>,
    source_by_path: &'a BTreeMap<&'a str, &'a str>,
    available_style_paths: &'a BTreeSet<&'a str>,
    resolution_context: TransformResolutionContext<'a>,
}

fn collect_import_graph_source_map_segments(
    segments: &mut Vec<TransformSourceMapSegmentV0>,
    seen_segments: &mut BTreeSet<(String, usize, usize, &'static str)>,
    importer_style_path: &str,
    generated_start_bound: usize,
    generated_end_bound: usize,
    context: &ImportGraphSourceMapSegmentContext<'_>,
    visiting: &mut BTreeSet<String>,
) {
    if !visiting.insert(importer_style_path.to_string()) {
        return;
    }
    let Some(entry) = context.entries_by_path.get(importer_style_path) else {
        visiting.remove(importer_style_path);
        return;
    };

    for edge in entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassImport")
    {
        let Some(resolved_style_path) = context.resolution_context.resolve_style_module_source(
            importer_style_path,
            edge.source.as_str(),
            context.available_style_paths,
        ) else {
            continue;
        };
        let Some(imported_source) = context
            .source_by_path
            .get(resolved_style_path.as_str())
            .copied()
        else {
            continue;
        };
        let Some(replacement_css) = resolve_import_inline_replacement_for_transform_context(
            resolved_style_path.as_str(),
            context.entries_by_path,
            context.available_style_paths,
            context.owned_source_by_path,
            context.resolution_context,
            &mut BTreeSet::new(),
        ) else {
            continue;
        };
        if replacement_css.is_empty() || generated_start_bound > generated_end_bound {
            continue;
        }
        let Some((generated_start, generated_end, exact_match)) =
            find_import_origin_generated_range(
                context.output_css,
                generated_start_bound..generated_end_bound,
                replacement_css.as_str(),
                resolved_style_path.as_str(),
                imported_source,
            )
        else {
            continue;
        };
        push_unique_import_origin_segment(
            segments,
            seen_segments,
            resolved_style_path.clone(),
            imported_source,
            context.output_css,
            generated_start,
            generated_end,
        );
        collect_import_graph_source_map_segments(
            segments,
            seen_segments,
            resolved_style_path.as_str(),
            if exact_match {
                generated_start
            } else {
                generated_start_bound
            },
            if exact_match {
                generated_end
            } else {
                generated_end_bound
            },
            context,
            visiting,
        );
    }

    visiting.remove(importer_style_path);
}

fn find_import_origin_generated_range(
    output_css: &str,
    search_range: std::ops::Range<usize>,
    replacement_css: &str,
    source_path: &str,
    source: &str,
) -> Option<(usize, usize, bool)> {
    if search_range.start > search_range.end || search_range.end > output_css.len() {
        return None;
    }
    if let Some(relative_start) = output_css[search_range.clone()].find(replacement_css) {
        let generated_start = search_range.start + relative_start;
        return Some((
            generated_start,
            generated_start + replacement_css.len(),
            true,
        ));
    }

    let runtime_index =
        omena_semantic::summarize_style_runtime_index_facts_from_source(source_path, source);
    let mut candidate_needles = Vec::new();
    if let Some(runtime_index) = runtime_index {
        candidate_needles.extend(
            runtime_index
                .class_selector_names
                .iter()
                .map(|name| format!(".{name}")),
        );
        candidate_needles.extend(runtime_index.custom_property_names.iter().cloned());
        candidate_needles.extend(
            runtime_index
                .keyframe_names
                .iter()
                .map(|name| format!("@keyframes {name}")),
        );
    } else {
        let facts = summarize_omena_query_omena_parser_style_facts(
            source,
            omena_parser_dialect_for_style_path(source_path),
        );
        candidate_needles.extend(
            facts
                .class_selector_names
                .iter()
                .map(|name| format!(".{name}")),
        );
        candidate_needles.extend(facts.custom_property_names.iter().cloned());
        candidate_needles.extend(
            facts
                .keyframe_names
                .iter()
                .map(|name| format!("@keyframes {name}")),
        );
    }

    let mut generated_start = None;
    let mut generated_end = None;
    for needle in candidate_needles {
        if needle.is_empty() {
            continue;
        }
        let Some(relative_start) = output_css[search_range.clone()].find(needle.as_str()) else {
            continue;
        };
        let start = search_range.start + relative_start;
        let end = start + needle.len();
        generated_start = Some(generated_start.map_or(start, |current: usize| current.min(start)));
        generated_end = Some(generated_end.map_or(end, |current: usize| current.max(end)));
    }

    match (generated_start, generated_end) {
        (Some(start), Some(end)) if start < end => Some((start, end, false)),
        _ => None,
    }
}

fn push_unique_import_origin_segment(
    segments: &mut Vec<TransformSourceMapSegmentV0>,
    seen_segments: &mut BTreeSet<(String, usize, usize, &'static str)>,
    source_path: String,
    source: &str,
    output_css: &str,
    generated_start: usize,
    generated_end: usize,
) {
    let pass_id = TransformPassKind::ImportInline.id();
    if !seen_segments.insert((source_path.clone(), generated_start, generated_end, pass_id)) {
        return;
    }
    segments.push(TransformSourceMapSegmentV0 {
        source_path,
        original_start: 0,
        original_end: source.len(),
        generated_start,
        generated_end,
        original_start_point: transform_source_map_point(source, 0),
        original_end_point: transform_source_map_point(source, source.len()),
        generated_start_point: transform_source_map_point(output_css, generated_start),
        generated_end_point: transform_source_map_point(output_css, generated_end),
        pass_id,
    });
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

fn resolution_inputs_for_transform_style_sources(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryStyleResolutionInputsV0 {
    let workspace_uri = infer_transform_workspace_uri(target_style_path, style_sources);
    load_omena_query_workspace_style_resolution_inputs(workspace_uri.as_deref(), package_manifests)
}

fn infer_transform_workspace_uri(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> Option<String> {
    let target_path = path_from_transform_style_path(target_style_path);
    let target_parent = target_path.as_deref().and_then(Path::parent);
    if let Some(root) = target_parent.and_then(discover_transform_workspace_root) {
        return Some(transform_path_to_file_uri(root));
    }

    style_sources
        .iter()
        .filter_map(|source| path_from_transform_style_path(source.style_path.as_str()))
        .filter_map(|path| {
            path.parent()
                .and_then(discover_transform_workspace_root)
                .map(transform_path_to_file_uri)
        })
        .next()
}

fn path_from_transform_style_path(style_path: &str) -> Option<PathBuf> {
    if let Some(path) = style_path.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }
    if style_path.starts_with('/') {
        return Some(PathBuf::from(style_path));
    }
    None
}

fn discover_transform_workspace_root(path: &Path) -> Option<&Path> {
    path.ancestors().find(|candidate| {
        [
            "tsconfig.json",
            "tsconfig.base.json",
            "jsconfig.json",
            "package.json",
            "vite.config.ts",
            "vite.config.mts",
            "vite.config.cts",
            "vite.config.js",
            "vite.config.mjs",
            "vite.config.cjs",
            "webpack.config.ts",
            "webpack.config.mts",
            "webpack.config.cts",
            "webpack.config.js",
            "webpack.config.mjs",
            "webpack.config.cjs",
            "next.config.ts",
            "next.config.mts",
            "next.config.cts",
            "next.config.js",
            "next.config.mjs",
            "next.config.cjs",
        ]
        .iter()
        .any(|marker| candidate.join(marker).is_file())
    })
}

fn transform_path_to_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
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
    resolution_context: TransformResolutionContext<'_>,
) -> TransformExecutionContextV0 {
    let style_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let derived = summarize_omena_query_transform_context_from_sources_with_resolution_context(
        target_style_path,
        style_refs,
        resolution_context,
    )
    .context;
    merge_transform_context(derived, context)
}

pub fn list_omena_query_transform_pass_summaries() -> Vec<OmenaQueryTransformPassSummaryV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(|kind| OmenaQueryTransformPassSummaryV0 {
            id: kind.id(),
            title: kind.title(),
            reads_semantic_graph: kind.reads_semantic_graph(),
            reads_cascade_model: kind.reads_cascade_model(),
            explicit_opt_in_required: kind.explicit_opt_in_required(),
            dialect_restriction: kind.dialect_restriction(),
            spec_snapshot: kind.spec_snapshot(),
            opt_in_policy: kind.opt_in_policy(),
        })
        .collect()
}

pub fn execute_omena_query_transform_passes_from_source_with_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryTransformExecuteSummaryV0 {
    let context = merge_single_source_transform_context(style_path, style_source, context);
    if pass_ids_require_closed_world_bundle(requested_pass_ids)
        && let Some(closed_world_bundle) = build_closed_world_bundle_for_single_style_source_context(
            style_path,
            style_source,
            requested_pass_ids,
            &context,
        )
    {
        return execute_omena_query_transform_passes_from_source_with_context_and_closed_world_bundle(
            style_path,
            style_source,
            requested_pass_ids,
            &context,
            &closed_world_bundle,
            closed_world_bundle_reachability_precision(&context, &closed_world_bundle),
            &TransformExecutionPolicyV0::default(),
        );
    }

    execute_omena_query_transform_passes_from_source_with_open_world_context(
        style_path,
        style_source,
        requested_pass_ids,
        &context,
        &TransformExecutionPolicyV0::default(),
    )
}

fn execute_omena_query_transform_passes_from_source_with_open_world_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    execution_policy: &TransformExecutionPolicyV0,
) -> OmenaQueryTransformExecuteSummaryV0 {
    let (requested_passes, unknown_pass_ids) =
        requested_transform_passes_from_ids(requested_pass_ids);

    let (admitted_passes, preflight_refusals) = strict_query_preflight(
        requested_pass_ids,
        requested_passes,
        execution_policy,
        false,
    );
    let expected_decision_count = admitted_passes.len();

    let dialect = omena_parser_dialect_for_style_path(style_path);
    let mut execution = execute_transform_passes_on_source_with_dialect_context_and_policy(
        style_source,
        dialect,
        &admitted_passes,
        context,
        execution_policy,
    );
    merge_strict_preflight_refusals(&mut execution, preflight_refusals);
    enforce_strict_decision_coverage(&mut execution, execution_policy, expected_decision_count);
    let semantic_removal_count = execution.semantic_removals.len();
    let open_world_snapshot = open_world_snapshot_for_closed_world_passes(requested_pass_ids);
    let ready_surfaces = transform_execute_ready_surfaces_with_open_world_snapshot(
        open_world_snapshot.as_ref(),
        vec!["transformExecutionRuntime", "transformPassOutcomeContract"],
    );

    OmenaQueryTransformExecuteSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-execute",
        style_path: style_path.to_string(),
        requested_pass_ids: requested_pass_ids.to_vec(),
        unknown_pass_ids,
        execution,
        semantic_removal_count,
        open_world_snapshot,
        ready_surfaces,
    }
}

fn execute_omena_query_transform_passes_from_source_with_context_and_closed_world_bundle(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    closed_world_bundle: &ClosedWorldBundleV0,
    reachability_precision: FactPrecision,
    execution_policy: &TransformExecutionPolicyV0,
) -> OmenaQueryTransformExecuteSummaryV0 {
    let (requested_passes, unknown_pass_ids) =
        requested_transform_passes_from_ids(requested_pass_ids);

    let (admitted_passes, preflight_refusals) =
        strict_query_preflight(requested_pass_ids, requested_passes, execution_policy, true);
    let expected_decision_count = admitted_passes.len();

    let dialect = omena_parser_dialect_for_style_path(style_path);
    let mut execution = execute_transform_passes_on_source_with_dialect_context_closed_world_bundle_precision_and_policy(
            style_source,
            dialect,
            &admitted_passes,
            context,
            closed_world_bundle,
            reachability_precision,
            execution_policy,
        );
    merge_strict_preflight_refusals(&mut execution, preflight_refusals);
    enforce_strict_decision_coverage(&mut execution, execution_policy, expected_decision_count);
    let semantic_removal_count = execution.semantic_removals.len();

    OmenaQueryTransformExecuteSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-execute",
        style_path: style_path.to_string(),
        requested_pass_ids: requested_pass_ids.to_vec(),
        unknown_pass_ids,
        execution,
        semantic_removal_count,
        open_world_snapshot: None,
        ready_surfaces: vec![
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
            "closedWorldBundle",
        ],
    }
}

fn execute_omena_query_transform_passes_from_module_with_context_and_closed_world_bundle(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    execution_inputs: ModuleQualifiedExecutionInputsV0<'_>,
    execution_policy: &TransformExecutionPolicyV0,
) -> Result<OmenaQueryTransformExecuteSummaryV0, TransformModuleQualifiedExecutionErrorV0> {
    let (requested_passes, unknown_pass_ids) =
        requested_transform_passes_from_ids(requested_pass_ids);
    let (admitted_passes, preflight_refusals) =
        strict_query_preflight(requested_pass_ids, requested_passes, execution_policy, true);
    let expected_decision_count = admitted_passes.len();

    let dialect = omena_parser_dialect_for_style_path(style_path);
    let mut execution =
        execute_transform_passes_on_module_with_dialect_context_policy_and_closed_world_bundle_and_retained_class_names(
            style_source,
            dialect,
            &admitted_passes,
            context,
            execution_inputs.closed_world_bundle,
            execution_inputs.module_instance,
            execution_inputs.reachability_precision,
            execution_policy,
            execution_inputs.retained_class_names,
        )?;
    merge_strict_preflight_refusals(&mut execution, preflight_refusals);
    enforce_strict_decision_coverage(&mut execution, execution_policy, expected_decision_count);
    let semantic_removal_count = execution.semantic_removals.len();

    Ok(OmenaQueryTransformExecuteSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-execute",
        style_path: style_path.to_string(),
        requested_pass_ids: requested_pass_ids.to_vec(),
        unknown_pass_ids,
        execution,
        semantic_removal_count,
        open_world_snapshot: None,
        ready_surfaces: vec![
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
            "closedWorldBundle",
            "moduleQualifiedReachability",
        ],
    })
}

fn strict_query_preflight(
    requested_pass_ids: &[String],
    requested_passes: Vec<TransformPassKind>,
    execution_policy: &TransformExecutionPolicyV0,
    has_closed_world_bundle: bool,
) -> (Vec<TransformPassKind>, Vec<TransformStrictPolicyEventV0>) {
    let Some(policy) = execution_policy.strict_policy.as_ref() else {
        return (requested_passes, Vec::new());
    };
    let requirements = OmenaQueryBuildAdmissionRequirementsV0 {
        refuse_unknown_pass_ids: policy.refuse_unknown_pass_ids,
        require_closed_world_evidence: policy.require_closed_world_evidence,
        require_complete_decisions: policy.require_complete_decisions,
    };
    let refusals = summarize_omena_query_build_preflight_refusals(
        requested_pass_ids,
        has_closed_world_bundle,
        requirements,
    );
    let refused_pass_ids = refusals
        .iter()
        .map(|event| event.pass_id.as_str())
        .collect::<BTreeSet<_>>();
    let admitted_passes = requested_passes
        .into_iter()
        .filter(|pass| !refused_pass_ids.contains(pass.id()))
        .collect();
    (admitted_passes, refusals)
}

pub fn summarize_omena_query_build_preflight_refusals(
    pass_ids: &[String],
    has_closed_world_bundle: bool,
    requirements: OmenaQueryBuildAdmissionRequirementsV0,
) -> Vec<TransformStrictPolicyEventV0> {
    let mut seen = BTreeSet::new();
    pass_ids
        .iter()
        .filter(|pass_id| seen.insert(pass_id.as_str()))
        .filter_map(|pass_id| match transform_pass_kind_from_id(pass_id) {
            None if requirements.refuse_unknown_pass_ids => Some(TransformStrictPolicyEventV0 {
                pass_id: pass_id.clone(),
                reasons: vec![TransformStrictPolicyReasonV0::UnknownPass],
            }),
            Some(pass)
                if requirements.require_closed_world_evidence
                    && transform_pass_requires_closed_world_bundle(pass)
                    && !has_closed_world_bundle =>
            {
                Some(TransformStrictPolicyEventV0 {
                    pass_id: pass_id.clone(),
                    reasons: vec![TransformStrictPolicyReasonV0::ClosedWorldEvidenceUnavailable],
                })
            }
            _ => None,
        })
        .collect()
}

pub fn summarize_omena_query_build_decision_coverage_refusal(
    decision_coverage_complete: bool,
    requirements: OmenaQueryBuildAdmissionRequirementsV0,
) -> Option<TransformStrictPolicyEventV0> {
    (requirements.require_complete_decisions && !decision_coverage_complete).then(|| {
        TransformStrictPolicyEventV0 {
            pass_id: "execution-plan".to_string(),
            reasons: vec![TransformStrictPolicyReasonV0::DecisionCoverageIncomplete],
        }
    })
}

fn merge_strict_preflight_refusals(
    execution: &mut TransformExecutionSummaryV0,
    refusals: Vec<TransformStrictPolicyEventV0>,
) {
    for refusal in refusals {
        execution
            .strict_policy
            .record_refusal(refusal.pass_id, refusal.reasons);
    }
}

fn enforce_strict_decision_coverage(
    execution: &mut TransformExecutionSummaryV0,
    execution_policy: &TransformExecutionPolicyV0,
    expected_decision_count: usize,
) {
    let requirements = execution_policy
        .strict_policy
        .as_ref()
        .map(|policy| OmenaQueryBuildAdmissionRequirementsV0 {
            refuse_unknown_pass_ids: policy.refuse_unknown_pass_ids,
            require_closed_world_evidence: policy.require_closed_world_evidence,
            require_complete_decisions: policy.require_complete_decisions,
        })
        .unwrap_or_default();
    if let Some(refusal) = summarize_omena_query_build_decision_coverage_refusal(
        execution.decisions.len() == expected_decision_count,
        requirements,
    ) {
        execution
            .strict_policy
            .record_refusal(refusal.pass_id, refusal.reasons);
    }
}

#[cfg(feature = "lawvere-trace")]
pub fn execute_omena_query_transform_passes_from_source_with_lawvere_trace(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
) -> OmenaQueryLawvereTransformExecuteSummaryV0 {
    let execution = execute_omena_query_transform_passes_from_source(
        style_path,
        style_source,
        requested_pass_ids,
    );
    let requested_passes = requested_pass_ids
        .iter()
        .filter_map(|pass_id| transform_pass_kind_from_id(pass_id))
        .collect::<Vec<_>>();
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let (_traced_execution, lawvere_trace) =
        execute_transform_passes_on_source_with_lawvere_trace_and_dialect(
            style_source,
            dialect,
            requested_passes.as_slice(),
        );
    let parallel_plan = plan_transform_passes_parallel_lawvere_layers(requested_passes.as_slice());
    let mut reorderability_certificates = Vec::new();
    let mut differential_witnesses = Vec::new();

    if let Some((left, right)) = requested_passes.first().zip(requested_passes.get(1)) {
        let (certificate, witness) = evaluate_lawvere_reorderability_with_differential_corpus(
            *left,
            *right,
            &[style_source],
        );
        reorderability_certificates.push(certificate);
        differential_witnesses.push(witness);
    }

    OmenaQueryLawvereTransformExecuteSummaryV0 {
        schema_version: "0",
        product: "omena-query.transform-execute-lawvere-trace",
        product_scope: "explicitOptInLawvereTraceProductLane",
        default_product_mechanism: false,
        global_transform_theorem_claimed: false,
        execution,
        lawvere_trace,
        parallel_plan,
        reorderability_certificates,
        differential_witnesses,
        ready_surfaces: vec![
            "queryTransformExecutionHandoff",
            "lawvereModelTrace",
            "lawvereParallelPlanTrace",
            "lawvereDifferentialReorderabilityCertificate",
        ],
    }
}

pub fn summarize_omena_query_transform_context_from_sources<'a>(
    target_style_path: &str,
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryTransformContextFromSourcesSummaryV0 {
    let styles = styles.into_iter().collect::<Vec<_>>();
    let style_sources = styles
        .iter()
        .map(|(style_path, style_source)| OmenaQueryStyleSourceInputV0 {
            style_path: (*style_path).to_string(),
            style_source: (*style_source).to_string(),
        })
        .collect::<Vec<_>>();
    let resolution_inputs = resolution_inputs_for_transform_style_sources(
        target_style_path,
        style_sources.as_slice(),
        package_manifests,
    );
    summarize_omena_query_transform_context_from_sources_with_resolution_context(
        target_style_path,
        styles,
        TransformResolutionContext::from_resolution_inputs(&resolution_inputs),
    )
}

pub fn summarize_omena_query_transform_context_from_sources_with_resolution_inputs<'a>(
    target_style_path: &str,
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryTransformContextFromSourcesSummaryV0 {
    summarize_omena_query_transform_context_from_sources_with_resolution_context(
        target_style_path,
        styles,
        TransformResolutionContext::from_resolution_inputs(resolution_inputs),
    )
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

fn requested_transform_passes_from_ids(
    requested_pass_ids: &[String],
) -> (Vec<TransformPassKind>, Vec<String>) {
    let mut requested_passes = Vec::new();
    let mut unknown_pass_ids = Vec::new();

    for pass_id in requested_pass_ids {
        match transform_pass_kind_from_id(pass_id) {
            Some(pass) => requested_passes.push(pass),
            None => unknown_pass_ids.push(pass_id.clone()),
        }
    }

    (requested_passes, unknown_pass_ids)
}

fn pass_ids_require_closed_world_bundle(pass_ids: &[String]) -> bool {
    pass_ids
        .iter()
        .filter_map(|pass_id| transform_pass_kind_from_id(pass_id))
        .any(transform_pass_requires_closed_world_bundle)
}

fn open_world_snapshot_for_closed_world_passes(pass_ids: &[String]) -> Option<OpenWorldSnapshotV0> {
    if !pass_ids_require_closed_world_bundle(pass_ids) {
        return None;
    }

    Some(OpenWorldSnapshotV0::new(format!(
        "closed-world bundle unavailable for requested passes: {}",
        pass_ids.join(", ")
    )))
}

fn consumer_build_ready_surfaces_with_open_world_snapshot(
    snapshot: Option<&OpenWorldSnapshotV0>,
    mut ready_surfaces: Vec<&'static str>,
) -> Vec<&'static str> {
    if snapshot.is_some() && !ready_surfaces.contains(&"openWorldSnapshot") {
        ready_surfaces.push("openWorldSnapshot");
    }
    ready_surfaces
}

fn extend_ready_surfaces(
    mut ready_surfaces: Vec<&'static str>,
    additions: impl IntoIterator<Item = &'static str>,
) -> Vec<&'static str> {
    for surface in additions {
        if !ready_surfaces.contains(&surface) {
            ready_surfaces.push(surface);
        }
    }
    ready_surfaces
}

fn transform_execute_ready_surfaces_with_open_world_snapshot(
    snapshot: Option<&OpenWorldSnapshotV0>,
    ready_surfaces: Vec<&'static str>,
) -> Vec<&'static str> {
    consumer_build_ready_surfaces_with_open_world_snapshot(snapshot, ready_surfaces)
}

fn requested_pass_ids_include_tree_shake(requested_pass_ids: &[String]) -> bool {
    requested_pass_ids
        .iter()
        .filter_map(|pass_id| transform_pass_kind_from_id(pass_id))
        .any(|pass| {
            matches!(
                pass,
                TransformPassKind::TreeShakeClass
                    | TransformPassKind::TreeShakeKeyframes
                    | TransformPassKind::TreeShakeValue
                    | TransformPassKind::TreeShakeCustomProperty
            )
        })
}

#[derive(Clone, Copy)]
struct ClosedWorldStylesheetRequestV0<'a> {
    target_style_path: &'a str,
    style_sources: &'a [OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &'a [String],
    context: &'a TransformExecutionContextV0,
    reachability_context: &'a TransformExecutionContextV0,
    attribution_report: Option<&'a OmenaQueryModuleReachabilityAttributionReportV0>,
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
}

fn build_closed_world_outcome_for_style_sources(
    request: ClosedWorldStylesheetRequestV0<'_>,
) -> OmenaQueryClosedWorldOutcomeV0 {
    closed_world_outcome_from_link_result(
        link_closed_world_stylesheet_for_style_sources(
            request,
            TransformBundleLinkOptionsV0::default(),
        )
        .into_requested_policy_result()
        .map(|linked| linked.linked_stylesheet),
        request.requested_pass_ids,
    )
}

fn link_closed_world_stylesheet_for_style_sources(
    request: ClosedWorldStylesheetRequestV0<'_>,
    link_options: TransformBundleLinkOptionsV0,
) -> TransformBundleEmissionAdmissionV0 {
    let reachability_inputs = if requested_pass_ids_include_tree_shake(request.requested_pass_ids) {
        request
            .style_sources
            .iter()
            .filter_map(|source| {
                transform_bundle_semantic_reachability_input_from_context_and_attribution(
                    source.style_path.as_str(),
                    request.reachability_context,
                    request.attribution_report,
                )
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let prepared = prepare_transform_bundle_linker_projection(
        &[request.target_style_path],
        request.style_sources,
        reachability_inputs.as_slice(),
        TransformResolutionContext::from_resolution_inputs(request.resolution_inputs),
    );
    let module_metadata = style_sources_to_closed_world_metadata(
        &prepared.projection,
        request.context,
        request.external_sifs,
    );
    evaluate_omena_transform_bundle_projection_emission_admission_with_resolved_dependencies_and_options(
        &[request.target_style_path],
        &prepared.projection,
        &prepared.emission_item_projection,
        prepared.resolved_dependencies.as_slice(),
        &module_metadata,
        link_options,
    )
}

struct PreparedTransformBundleLinkerProjectionV0 {
    projection: TransformBundleLinkerProjectionV0,
    emission_item_projection: TransformBundleEmissionItemProjectionV0,
    resolved_dependencies: Vec<TransformBundleResolvedDependencyV0>,
}

struct TransformBundleDependencyResolutionTemplateV0 {
    source_path: String,
    edge_kind: TransformBundleEdgeKind,
    import_source: String,
    import_ordinal: Option<u32>,
    policy_step_keys: Vec<&'static str>,
    resolution_kind: &'static str,
    candidate_count: usize,
    target_source_path: Option<String>,
    target_configuration: omena_parser::ConfigurationHashV0,
}

fn prepare_transform_bundle_linker_projection(
    entrypoint_paths: &[&str],
    style_sources: &[OmenaQueryStyleSourceInputV0],
    reachability_inputs: &[TransformBundleSemanticReachabilityInputV0],
    resolution_context: TransformResolutionContext<'_>,
) -> PreparedTransformBundleLinkerProjectionV0 {
    let mut modules = style_sources_to_transform_bundle_modules(style_sources);
    let provisional_projection =
        project_omena_transform_bundle_linker_inputs_from_parsed_modules(&modules, &[]);
    let (templates, mut configurations_by_source_path) =
        resolve_transform_bundle_projection_dependency_templates(
            &provisional_projection,
            modules.as_slice(),
            style_sources,
            resolution_context,
        );
    let projection_path_by_source_path =
        projection_path_by_source_path(modules.as_slice(), style_sources);
    for entrypoint_path in entrypoint_paths {
        if let Some(projection_path) = projection_path_by_source_path.get(*entrypoint_path) {
            configurations_by_source_path
                .entry(projection_path.clone())
                .or_default()
                .insert(omena_parser::ConfigurationHashV0::none());
        }
    }
    for configurations in configurations_by_source_path.values_mut() {
        if configurations.is_empty() {
            configurations.insert(omena_parser::ConfigurationHashV0::none());
        }
    }

    modules = modules
        .into_iter()
        .map(|module| {
            let projection_path = module
                .module_instance_keys()
                .into_iter()
                .next()
                .map(|instance| instance.module().as_str().to_string())
                .unwrap_or_else(|| module.source_path().to_string());
            let configurations = configurations_by_source_path
                .remove(&projection_path)
                .unwrap_or_else(|| BTreeSet::from([omena_parser::ConfigurationHashV0::none()]))
                .into_iter()
                .collect();
            module.with_configuration_hashes(configurations)
        })
        .collect();

    let projections = project_omena_transform_bundle_linker_and_emission_items_from_parsed_modules(
        modules.as_slice(),
        reachability_inputs,
    );
    let projection = projections.linker_projection().clone();
    let resolved_dependencies =
        materialize_transform_bundle_resolved_dependencies(&projection, templates);
    PreparedTransformBundleLinkerProjectionV0 {
        projection,
        emission_item_projection: projections.emission_item_projection().clone(),
        resolved_dependencies,
    }
}

fn resolve_transform_bundle_projection_dependency_templates(
    projection: &TransformBundleLinkerProjectionV0,
    modules: &[TransformBundleParsedModuleInputV0],
    style_sources: &[OmenaQueryStyleSourceInputV0],
    resolution_context: TransformResolutionContext<'_>,
) -> (
    Vec<TransformBundleDependencyResolutionTemplateV0>,
    BTreeMap<String, BTreeSet<omena_parser::ConfigurationHashV0>>,
) {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let projection_path_by_source_path = projection_path_by_source_path(modules, style_sources);
    let source_by_projection_path = modules
        .iter()
        .zip(style_sources)
        .filter_map(|(module, source)| {
            module
                .module_instance_keys()
                .into_iter()
                .next()
                .map(|instance| {
                    (
                        instance.module().as_str().to_string(),
                        source.style_source.as_str(),
                    )
                })
        })
        .collect::<BTreeMap<_, _>>();
    let mut configurations_by_source_path = projection
        .inputs()
        .iter()
        .map(|input| (input.source_path.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let policy_step_keys = summarize_omena_query_style_resolution_policy_v0()
        .steps
        .into_iter()
        .map(|step| step.key)
        .collect::<Vec<_>>();
    let mut templates = Vec::new();
    for input in projection.inputs() {
        let source = source_by_projection_path
            .get(input.source_path.as_str())
            .copied()
            .unwrap_or_default();
        let mut sass_use_ordinal = 0usize;
        let mut sass_forward_ordinal = 0usize;
        for edge in &input.dependency_edges {
            let target_configuration = match edge.kind {
                TransformBundleEdgeKind::SassUse => {
                    let overrides =
                        omena_semantic::derive_sass_module_rule_variable_overrides_at_ordinal(
                            source,
                            "@use",
                            sass_use_ordinal,
                        );
                    sass_use_ordinal += 1;
                    omena_parser::ConfigurationHashV0::new(
                        omena_semantic::summarize_sass_module_configuration_signature(&overrides),
                    )
                }
                TransformBundleEdgeKind::SassForward => {
                    let overrides =
                        omena_semantic::derive_sass_module_forward_variable_override_values_at_ordinal(
                            source,
                            sass_forward_ordinal,
                        );
                    sass_forward_ordinal += 1;
                    omena_parser::ConfigurationHashV0::new(
                        omena_semantic::summarize_sass_module_configuration_signature(&overrides),
                    )
                }
                _ => omena_parser::ConfigurationHashV0::none(),
            };
            let resolution = resolution_context.resolve_style_module(
                input.source_path.as_str(),
                edge.import_source.as_str(),
                &available_style_paths,
            );
            let target_source_path = resolution
                .resolved_style_path
                .as_deref()
                .and_then(|path| projection_path_by_source_path.get(path))
                .cloned();
            if let Some(target_source_path) = target_source_path.as_ref() {
                configurations_by_source_path
                    .entry(target_source_path.clone())
                    .or_default()
                    .insert(target_configuration.clone());
            }
            templates.push(TransformBundleDependencyResolutionTemplateV0 {
                source_path: input.source_path.clone(),
                edge_kind: edge.kind,
                import_source: edge.import_source.clone(),
                import_ordinal: edge.import_ordinal,
                policy_step_keys: policy_step_keys.clone(),
                resolution_kind: resolution.resolution_kind,
                candidate_count: resolution.candidate_count,
                target_source_path,
                target_configuration,
            });
        }
    }
    (templates, configurations_by_source_path)
}

fn projection_path_by_source_path(
    modules: &[TransformBundleParsedModuleInputV0],
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> BTreeMap<String, String> {
    let mut projection_path_by_source_path = BTreeMap::new();
    for (module, source) in modules.iter().zip(style_sources) {
        let Some(instance) = module.module_instance_keys().into_iter().next() else {
            continue;
        };
        let projection_path = instance.module().as_str().to_string();
        projection_path_by_source_path.insert(source.style_path.clone(), projection_path.clone());
        projection_path_by_source_path.insert(projection_path.clone(), projection_path);
    }
    projection_path_by_source_path
}

fn materialize_transform_bundle_resolved_dependencies(
    projection: &TransformBundleLinkerProjectionV0,
    templates: Vec<TransformBundleDependencyResolutionTemplateV0>,
) -> Vec<TransformBundleResolvedDependencyV0> {
    let instance_by_path_and_configuration = projection
        .inputs()
        .iter()
        .map(|input| {
            (
                (
                    input.source_path.as_str(),
                    input.instance.configuration().as_str(),
                ),
                input.instance.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let source_instances_by_path = projection.inputs().iter().fold(
        BTreeMap::<&str, Vec<omena_parser::ModuleInstanceKeyV0>>::new(),
        |mut by_path, input| {
            by_path
                .entry(input.source_path.as_str())
                .or_default()
                .push(input.instance.clone());
            by_path
        },
    );
    let mut resolved_dependencies = Vec::new();
    for template in templates {
        let target_instance = template.target_source_path.as_deref().and_then(|path| {
            instance_by_path_and_configuration
                .get(&(path, template.target_configuration.as_str()))
                .cloned()
        });
        let Some(source_instances) = source_instances_by_path.get(template.source_path.as_str())
        else {
            continue;
        };
        for source_instance in source_instances {
            resolved_dependencies.push(TransformBundleResolvedDependencyV0::new(
                source_instance.clone(),
                template.edge_kind,
                template.import_source.as_str(),
                template.import_ordinal,
                TransformBundleDependencyResolutionV0::attempted(
                    template.policy_step_keys.clone(),
                    template.resolution_kind,
                    template.candidate_count,
                    target_instance.clone(),
                ),
            ));
        }
    }
    resolved_dependencies
}

fn execute_linked_bundle_modules(
    linked: &LinkedStylesheetWithEmissionItemsV0,
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    effective_pass_ids: &[String],
    base_context: &TransformExecutionContextV0,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    options: &OmenaQueryConsumerBuildOptionsV0,
) -> Result<LinkedBundleExecutionV0, String> {
    let linked_stylesheet = &linked.linked_stylesheet;
    let pass_set = consumer_build_pass_set(effective_pass_ids);
    let resolution_context = TransformResolutionContext::from_resolution_inputs(resolution_inputs);
    let target_instance = linked_stylesheet
        .entrypoints
        .first()
        .ok_or_else(|| format!("linked bundle has no entrypoint for {target_style_path:?}"))?;
    let mut transformed_modules = Vec::with_capacity(linked_stylesheet.module_instances.len());
    let mut module_executions = Vec::with_capacity(linked_stylesheet.module_instances.len());

    let mut module_inputs = Vec::with_capacity(linked_stylesheet.module_instances.len());
    for module_instance in &linked_stylesheet.module_instances {
        let style_path = module_instance.module().as_str();
        let Some(style_source) = find_target_style_source(style_path, style_sources) else {
            return Err(format!(
                "linked module {style_path:?} was not found in workspace style sources"
            ));
        };
        let mut module_context = merge_workspace_transform_context(
            style_path,
            style_sources,
            base_context,
            resolution_context,
        );
        for inline in &mut module_context.import_inlines {
            inline.replacement_css.clear();
        }
        module_inputs.push(LinkedModuleExecutionInputV0 {
            module_instance,
            style_source,
            context: module_context,
        });
    }
    let retained_class_names_by_module = retained_class_names_for_live_linked_emission_tokens(
        &linked_stylesheet.closed_world_bundle,
        module_inputs.as_slice(),
    );

    for module_input in module_inputs {
        let module_instance = module_input.module_instance;
        let style_path = module_instance.module().as_str();
        let retained_class_names = retained_class_names_by_module
            .get(module_instance)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let summary =
            execute_omena_query_consumer_build_style_module_with_context_and_closed_world_bundle(
                style_path,
                module_input.style_source,
                &pass_set,
                &module_input.context,
                ModuleQualifiedExecutionInputsV0 {
                    closed_world_bundle: &linked_stylesheet.closed_world_bundle,
                    module_instance,
                    reachability_precision: closed_world_bundle_reachability_precision(
                        &module_input.context,
                        &linked_stylesheet.closed_world_bundle,
                    ),
                    retained_class_names,
                },
                options,
            )?;
        let execution = summary.execution;
        let non_empty_import_replacement_count = execution
            .css_import_inlines
            .iter()
            .filter(|inline| !inline.replacement_css.is_empty())
            .count();
        transformed_modules.push(
            TransformBundleTransformedModuleV0::new(
                module_instance.clone(),
                execution.output_css.clone(),
            )
            .with_non_empty_import_replacement_count(non_empty_import_replacement_count),
        );
        module_executions.push(LinkedModuleExecutionV0 {
            module_instance: module_instance.clone(),
            execution,
        });
    }

    let materialized = materialize_omena_transform_bundle_linked_stylesheet_with_emission_items(
        linked,
        &transformed_modules,
    )
    .map_err(|error| format!("linked bundle materialization failed: {error:?}"))?;
    let Some(entry_execution) = module_executions
        .iter()
        .find(|module| &module.module_instance == target_instance)
        .map(|module| module.execution.clone())
    else {
        return Err(format!(
            "linked entrypoint {target_style_path:?} was not transformed"
        ));
    };
    let execution =
        project_linked_bundle_execution(entry_execution, materialized.output_css.as_str());
    Ok(LinkedBundleExecutionV0 {
        execution,
        entry_module_instance: target_instance.clone(),
        module_executions,
        materialization: materialized,
    })
}

fn project_linked_bundle_execution(
    mut execution: TransformExecutionSummaryV0,
    materialized_output_css: &str,
) -> TransformExecutionSummaryV0 {
    execution.output_byte_len = materialized_output_css.len();
    execution.output_css = materialized_output_css.to_string();
    execution
}

struct LinkedModuleExecutionV0 {
    module_instance: omena_parser::ModuleInstanceKeyV0,
    execution: TransformExecutionSummaryV0,
}

struct LinkedModuleExecutionInputV0<'a> {
    module_instance: &'a omena_parser::ModuleInstanceKeyV0,
    style_source: &'a str,
    context: TransformExecutionContextV0,
}

fn retained_class_names_for_live_linked_emission_tokens(
    closed_world_bundle: &ClosedWorldBundleV0,
    module_inputs: &[LinkedModuleExecutionInputV0<'_>],
) -> BTreeMap<omena_parser::ModuleInstanceKeyV0, Vec<String>> {
    let mut live_emitted_tokens = BTreeSet::new();
    for module_input in module_inputs {
        let Some(symbols) = closed_world_bundle
            .reachability()
            .symbols_for_module(module_input.module_instance)
        else {
            continue;
        };
        for class_name in symbols.class_names() {
            let emitted_token = module_input
                .context
                .class_name_rewrites
                .iter()
                .find(|rewrite| {
                    css_identifier_names_match(rewrite.original_name.as_str(), class_name)
                })
                .map_or(class_name.as_str(), |rewrite| {
                    rewrite.rewritten_name.as_str()
                });
            live_emitted_tokens.insert(emitted_token.to_string());
        }
    }

    module_inputs
        .iter()
        .map(|module_input| {
            let own_reachable = closed_world_bundle
                .reachability()
                .symbols_for_module(module_input.module_instance)
                .map(|symbols| symbols.class_names())
                .unwrap_or_default();
            let retained = if css_modules::style_path_is_css_module_path(
                module_input.module_instance.module().as_str(),
            ) {
                module_input
                    .context
                    .class_name_rewrites
                    .iter()
                    .filter(|rewrite| live_emitted_tokens.contains(&rewrite.rewritten_name))
                    .map(|rewrite| rewrite.original_name.clone())
                    .collect::<BTreeSet<_>>()
            } else {
                live_emitted_tokens.clone()
            }
            .into_iter()
            .filter(|name| {
                !own_reachable
                    .iter()
                    .any(|own| css_identifier_names_match(own, name))
            })
            .collect::<Vec<_>>();
            (module_input.module_instance.clone(), retained)
        })
        .collect()
}

struct LinkedBundleExecutionV0 {
    execution: TransformExecutionSummaryV0,
    entry_module_instance: omena_parser::ModuleInstanceKeyV0,
    module_executions: Vec<LinkedModuleExecutionV0>,
    materialization: LinkedEmissionArtifactV0,
}

fn summarize_linked_bundle_execution_scope(
    linked: &LinkedBundleExecutionV0,
) -> Result<OmenaQueryBundleExecutionScopeEvidenceV0, String> {
    let mut module_executions = Vec::with_capacity(linked.module_executions.len());
    for module in &linked.module_executions {
        let region = linked
            .materialization
            .module_regions
            .iter()
            .find(|region| region.module_instance == module.module_instance)
            .ok_or_else(|| {
                format!(
                    "linked execution evidence has no materialized region for {:?}",
                    module.module_instance
                )
            })?;
        let generated_len = region.generated_end.saturating_sub(region.generated_start);
        if generated_len != module.execution.output_byte_len {
            return Err(format!(
                "linked execution evidence byte mismatch for {:?}: execution={}, materialized={generated_len}",
                module.module_instance, module.execution.output_byte_len
            ));
        }
        module_executions.push(OmenaQueryBundleModuleExecutionByteFactsV0 {
            module_instance: module.module_instance.clone(),
            input_byte_len: module.execution.input_byte_len,
            output_byte_len: module.execution.output_byte_len,
            generated_start: region.generated_start,
            generated_end: region.generated_end,
        });
    }

    if module_executions.len() != linked.materialization.module_regions.len() {
        return Err(format!(
            "linked execution evidence cardinality mismatch: executions={}, regions={}",
            module_executions.len(),
            linked.materialization.module_regions.len()
        ));
    }
    let summed_module_input_byte_len = module_executions
        .iter()
        .map(|module| module.input_byte_len)
        .sum();
    let summed_module_output_byte_len = module_executions
        .iter()
        .map(|module| module.output_byte_len)
        .sum::<usize>();
    let materialized_output_byte_len = linked.materialization.output_css.len();
    let inter_module_separator_byte_len =
        linked_materialization_separator_byte_len(&linked.materialization)?;
    if summed_module_output_byte_len + inter_module_separator_byte_len
        != materialized_output_byte_len
    {
        return Err(
            "linked execution evidence could not account for bundle output bytes".to_string(),
        );
    }

    Ok(OmenaQueryBundleExecutionScopeEvidenceV0 {
        schema_version: "0",
        product: "omena-query.bundle-execution-scope",
        entry_module_instance: linked.entry_module_instance.clone(),
        field_scopes: bundle_execution_field_scopes(),
        bundle_composite: OmenaQueryBundleCompositeExecutionByteFactsV0 {
            module_count: module_executions.len(),
            summed_module_input_byte_len,
            summed_module_output_byte_len,
            inter_module_separator_byte_len,
            materialized_output_byte_len,
        },
        module_executions,
        source_map_dispositions: Vec::new(),
    })
}

fn linked_materialization_separator_byte_len(
    materialization: &LinkedEmissionArtifactV0,
) -> Result<usize, String> {
    let mut cursor = 0usize;
    let mut separator_byte_len = 0usize;
    for region in &materialization.module_regions {
        if region.generated_start < cursor
            || region.generated_start > region.generated_end
            || region.generated_end > materialization.output_css.len()
        {
            return Err(format!(
                "linked execution evidence has invalid materialized region {}..{} after {cursor}",
                region.generated_start, region.generated_end
            ));
        }
        separator_byte_len += region.generated_start - cursor;
        cursor = region.generated_end;
    }
    separator_byte_len += materialization.output_css.len() - cursor;
    Ok(separator_byte_len)
}

fn bundle_execution_field_scopes() -> Vec<OmenaQueryExecutionFieldScopeV0> {
    use OmenaQueryExecutionEvidenceScopeV0::{Bundle, Entry};

    vec![
        execution_field_scope("schemaVersion", Entry, "retained entry execution schema"),
        execution_field_scope("product", Entry, "retained entry execution product"),
        execution_field_scope("inputByteLen", Entry, "retained entry source byte length"),
        execution_field_scope(
            "outputByteLen",
            Bundle,
            "materialized linked bundle output byte length",
        ),
        execution_field_scope(
            "requestedPassIds",
            Entry,
            "retained entry requested pass identifiers",
        ),
        execution_field_scope(
            "orderedPassIds",
            Entry,
            "retained entry ordered pass identifiers",
        ),
        execution_field_scope(
            "executedPassIds",
            Entry,
            "retained entry executed pass identifiers",
        ),
        execution_field_scope(
            "plannedOnlyPassIds",
            Entry,
            "retained entry planned-only pass identifiers",
        ),
        execution_field_scope("mutationCount", Entry, "retained entry mutation count"),
        execution_field_scope(
            "provenancePreserved",
            Entry,
            "retained entry provenance status",
        ),
        execution_field_scope("outputCss", Bundle, "materialized linked bundle CSS"),
        execution_field_scope(
            "cssModuleEvaluation",
            Entry,
            "retained entry CSS module evaluation",
        ),
        execution_field_scope(
            "cssImportInlines",
            Entry,
            "retained entry import-inline outcomes",
        ),
        execution_field_scope(
            "cssModuleComposesExports",
            Entry,
            "retained entry composes exports",
        ),
        execution_field_scope(
            "designTokenRoutes",
            Entry,
            "retained entry design-token routes",
        ),
        execution_field_scope(
            "semanticRemovals",
            Entry,
            "retained entry semantic removals",
        ),
        execution_field_scope(
            "moduleQualifiedShake",
            Entry,
            "retained entry module-qualified shake summary",
        ),
        execution_field_scope(
            "cascadeProofObligations",
            Entry,
            "retained entry cascade proof obligations",
        ),
        execution_field_scope(
            "winnerEqualityObligations",
            Entry,
            "retained entry winner-equality obligations",
        ),
        execution_field_scope(
            "provenanceDerivationForest",
            Entry,
            "retained entry provenance derivation forest",
        ),
        execution_field_scope(
            "structuralIrTransactionTelemetry",
            Entry,
            "retained entry structural transaction telemetry",
        ),
        execution_field_scope(
            "semanticPreservationTelemetry",
            Entry,
            "retained entry semantic preservation telemetry",
        ),
        execution_field_scope(
            "dischargeLedgerTelemetry",
            Entry,
            "retained entry discharge ledger telemetry",
        ),
        execution_field_scope(
            "strictPolicy",
            Entry,
            "retained entry strict-policy summary",
        ),
        execution_field_scope("decisions", Entry, "retained entry transform decisions"),
        execution_field_scope("outcomes", Entry, "retained entry pass outcomes"),
        execution_field_scope("passPlan", Entry, "retained entry transform pass plan"),
    ]
}

const fn execution_field_scope(
    field_name: &'static str,
    scope: OmenaQueryExecutionEvidenceScopeV0,
    derivation: &'static str,
) -> OmenaQueryExecutionFieldScopeV0 {
    OmenaQueryExecutionFieldScopeV0 {
        field_name,
        scope,
        derivation,
    }
}

pub(crate) fn build_closed_world_bundle_for_single_style_source_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> Option<ClosedWorldBundleV0> {
    build_closed_world_outcome_for_single_style_source_context(
        style_path,
        style_source,
        requested_pass_ids,
        context,
    )
    .bundle()
    .cloned()
}

pub fn summarize_omena_query_closed_world_outcome_for_style_source(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryClosedWorldOutcomeV0 {
    let context = merge_single_source_transform_context(style_path, style_source, context);
    build_closed_world_outcome_for_single_style_source_context(
        style_path,
        style_source,
        requested_pass_ids,
        &context,
    )
}

fn build_closed_world_outcome_for_single_style_source_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryClosedWorldOutcomeV0 {
    let source = OmenaQueryStyleSourceInputV0 {
        style_path: style_path.to_string(),
        style_source: style_source.to_string(),
    };
    let sources = std::slice::from_ref(&source);
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let reachability_input =
        transform_bundle_semantic_reachability_input_from_context(style_path, context);
    let reachability_inputs = reachability_input.as_slice();
    let prepared = prepare_transform_bundle_linker_projection(
        &[style_path],
        sources,
        reachability_inputs,
        TransformResolutionContext::from_resolution_inputs(&resolution_inputs),
    );
    let module_metadata =
        style_sources_to_closed_world_metadata(&prepared.projection, context, &[]);
    if reachability_input.is_none() {
        if requested_pass_ids_include_tree_shake(requested_pass_ids) {
            return OmenaQueryClosedWorldOutcomeV0::Open {
                blockers: vec![OmenaQueryClosedWorldBlockerV0::ClosedWorldPassUnavailable {
                    requested_pass_ids: requested_pass_ids.to_vec(),
                }],
            };
        }
        return closed_world_outcome_from_link_result(
            link_omena_transform_bundle_projection_with_resolved_dependencies_and_options(
                &[style_path],
                &prepared.projection,
                prepared.resolved_dependencies.as_slice(),
                &module_metadata,
                TransformBundleLinkOptionsV0::default(),
            ),
            requested_pass_ids,
        );
    }

    closed_world_outcome_from_link_result(
        link_omena_transform_bundle_projection_with_resolved_dependencies_and_options(
            &[style_path],
            &prepared.projection,
            prepared.resolved_dependencies.as_slice(),
            &module_metadata,
            TransformBundleLinkOptionsV0::default(),
        ),
        requested_pass_ids,
    )
}

fn style_sources_to_transform_bundle_modules(
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> Vec<TransformBundleParsedModuleInputV0> {
    style_sources
        .iter()
        .map(|source| {
            let dialect = omena_parser_dialect_for_style_path(source.style_path.as_str());
            let parsed =
                parse_omena_query_omena_parser_style_source(source.style_source.as_str(), dialect);
            TransformBundleParsedModuleInputV0::new(
                source.style_path.as_str(),
                dialect,
                omena_parser::facts_from_cst(source.style_source.as_str(), &parsed),
            )
            .with_emission_selectors(
                omena_parser::collect_emission_selector_facts_from_cst(
                    source.style_source.as_str(),
                    &parsed,
                ),
            )
        })
        .collect()
}

fn style_sources_to_closed_world_metadata(
    projection: &TransformBundleLinkerProjectionV0,
    context: &TransformExecutionContextV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Vec<ClosedWorldModuleMetadataV0> {
    let source_precision = closed_world_source_precision_summary(context);
    projection
        .inputs()
        .iter()
        .map(|input| {
            let mut metadata = ClosedWorldModuleMetadataV0::new(input.instance.clone())
                .with_interface_hash(linker_input_interface_hash(
                    input.source_path.as_str(),
                    [
                        input.class_names.as_slice(),
                        input.keyframe_names.as_slice(),
                        input.value_names.as_slice(),
                        input.custom_property_names.as_slice(),
                    ],
                ))
                .with_source_precision(source_precision);
            if let Some(interface_hash) = external_sifs.iter().find_map(|external_sif| {
                sif_matches_style_path(external_sif, input.source_path.as_str()).then(|| {
                    external_sif
                        .sif
                        .fingerprints
                        .interface_hash
                        .as_str()
                        .to_string()
                })
            }) {
                metadata = metadata.with_interface_hash(interface_hash);
            }
            metadata
        })
        .collect()
}

fn linker_input_interface_hash(source_path: &str, symbol_domains: [&[String]; 4]) -> String {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in source_path.as_bytes().iter().copied().chain([0]) {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    for domain in symbol_domains {
        for value in domain {
            for byte in value.as_bytes().iter().copied().chain([0]) {
                digest ^= u64::from(byte);
                digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        digest ^= 0xff;
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("local-interface-fnv1a64:{digest:016x}")
}

fn closed_world_source_precision_summary(
    context: &TransformExecutionContextV0,
) -> ClosedWorldSourcePrecisionSummaryV0 {
    let precision = if context.reachable_class_names.is_empty()
        && context.reachable_keyframe_names.is_empty()
        && context.reachable_value_names.is_empty()
        && context.reachable_custom_property_names.is_empty()
    {
        FactPrecision::Unknown
    } else {
        FactPrecision::Conservative
    };
    let mut summary = ClosedWorldSourcePrecisionSummaryV0::default();
    match precision {
        FactPrecision::Exact => summary.exact_source_count = 1,
        FactPrecision::Conservative => summary.conservative_source_count = 1,
        FactPrecision::Heuristic => summary.heuristic_source_count = 1,
        FactPrecision::Unknown => summary.unknown_source_count = 1,
    }
    summary
}

fn closed_world_bundle_reachability_precision(
    context: &TransformExecutionContextV0,
    bundle: &ClosedWorldBundleV0,
) -> FactPrecision {
    let precision_ceiling = bundle.source_precision().and_then(|precision| {
        if precision.unknown_source_count > 0 {
            Some(FactPrecision::Unknown)
        } else if precision.heuristic_source_count > 0 {
            Some(FactPrecision::Heuristic)
        } else if precision.conservative_source_count > 0 {
            Some(FactPrecision::Conservative)
        } else if precision.exact_source_count > 0 {
            Some(FactPrecision::Exact)
        } else {
            None
        }
    });
    classify_transform_reachability_precision(context, true, precision_ceiling)
}

fn sif_matches_style_path(external_sif: &OmenaQueryExternalSifInputV0, style_path: &str) -> bool {
    let style_path = normalize_bundle_sif_location(style_path);
    [
        external_sif.canonical_url.as_str(),
        external_sif.sif.canonical_url.as_str(),
    ]
    .into_iter()
    .map(normalize_bundle_sif_location)
    .any(|candidate| candidate == style_path)
}

fn normalize_bundle_sif_location(location: &str) -> String {
    location
        .strip_prefix("file://")
        .unwrap_or(location)
        .replace('\\', "/")
}

fn closed_world_outcome_from_link_result(
    result: Result<omena_query_transform_runner::LinkedStylesheetV0, TransformBundleLinkErrorV0>,
    requested_pass_ids: &[String],
) -> OmenaQueryClosedWorldOutcomeV0 {
    match result {
        Ok(linked) => OmenaQueryClosedWorldOutcomeV0::Closed {
            bundle: Box::new(linked.closed_world_bundle),
        },
        Err(error) => OmenaQueryClosedWorldOutcomeV0::Open {
            blockers: vec![closed_world_blocker_from_link_error(
                error,
                requested_pass_ids,
            )],
        },
    }
}

fn closed_world_blocker_from_link_error(
    error: TransformBundleLinkErrorV0,
    requested_pass_ids: &[String],
) -> OmenaQueryClosedWorldBlockerV0 {
    match error {
        TransformBundleLinkErrorV0::MissingEntrypoint { source_path } => {
            OmenaQueryClosedWorldBlockerV0::MissingEntrypoint { source_path }
        }
        TransformBundleLinkErrorV0::AmbiguousModulePath { source_path } => {
            OmenaQueryClosedWorldBlockerV0::AmbiguousModulePath { source_path }
        }
        TransformBundleLinkErrorV0::MissingDependency {
            source_path,
            import_source,
        } => OmenaQueryClosedWorldBlockerV0::MissingDependency {
            source_path,
            import_source,
        },
        TransformBundleLinkErrorV0::ClosedWorldBundle { error } => match error {
            ClosedWorldBundleBuildErrorV0::EmptyEntrypoints => {
                OmenaQueryClosedWorldBlockerV0::EmptyEntrypoints
            }
            ClosedWorldBundleBuildErrorV0::MissingEntrypoint { module } => {
                OmenaQueryClosedWorldBlockerV0::MissingModuleInstance { module }
            }
            ClosedWorldBundleBuildErrorV0::MissingDependency { module, dependency } => {
                OmenaQueryClosedWorldBlockerV0::MissingModuleDependency { module, dependency }
            }
        },
        TransformBundleLinkErrorV0::InvalidEmissionPlan { .. }
        | TransformBundleLinkErrorV0::UnsupportedEmissionCycle { .. } => {
            OmenaQueryClosedWorldBlockerV0::ClosedWorldPassUnavailable {
                requested_pass_ids: requested_pass_ids.to_vec(),
            }
        }
    }
}

fn transform_bundle_semantic_reachability_input_from_context(
    style_path: &str,
    context: &TransformExecutionContextV0,
) -> Option<TransformBundleSemanticReachabilityInputV0> {
    transform_bundle_semantic_reachability_input_from_context_and_attribution(
        style_path, context, None,
    )
}

fn transform_bundle_semantic_reachability_input_from_context_and_attribution(
    style_path: &str,
    context: &TransformExecutionContextV0,
    attribution_report: Option<&OmenaQueryModuleReachabilityAttributionReportV0>,
) -> Option<TransformBundleSemanticReachabilityInputV0> {
    let mut class_names = context.reachable_class_names.clone();
    if let Some(attribution) =
        attribution_report.and_then(|report| report.entry_for_style_path(style_path))
    {
        class_names.extend(attribution.class_names().iter().cloned());
    }
    class_names.sort();
    class_names.dedup();
    let input = TransformBundleSemanticReachabilityInputV0 {
        source_path: style_path.to_string(),
        class_names,
        keyframe_names: context.reachable_keyframe_names.clone(),
        value_names: context.reachable_value_names.clone(),
        custom_property_names: context.reachable_custom_property_names.clone(),
    };
    input.has_reachable_symbols().then_some(input)
}

fn transform_pass_kind_from_id(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|candidate| candidate.id() == pass_id)
}

#[cfg(test)]
mod linked_source_map_tests {
    use super::*;

    #[derive(Debug)]
    struct DecodedSourceMapSegment {
        generated_line: usize,
        generated_column: usize,
        source_index: usize,
        original_line: usize,
        original_column: usize,
    }

    fn decode_source_map_mappings(mappings: &str) -> Result<Vec<DecodedSourceMapSegment>, String> {
        let mut decoded = Vec::new();
        let mut previous_source_index = 0isize;
        let mut previous_original_line = 0isize;
        let mut previous_original_column = 0isize;
        for (generated_line, line) in mappings.split(';').enumerate() {
            let mut previous_generated_column = 0isize;
            for segment in line.split(',').filter(|segment| !segment.is_empty()) {
                let values = decode_source_map_vlq_values(segment)?;
                if values.len() < 4 {
                    return Err(format!(
                        "source-map segment has too few fields: {segment:?}"
                    ));
                }
                previous_generated_column += values[0];
                previous_source_index += values[1];
                previous_original_line += values[2];
                previous_original_column += values[3];
                if previous_generated_column < 0
                    || previous_source_index < 0
                    || previous_original_line < 0
                    || previous_original_column < 0
                {
                    return Err(format!("source-map segment underflowed: {segment:?}"));
                }
                decoded.push(DecodedSourceMapSegment {
                    generated_line,
                    generated_column: previous_generated_column as usize,
                    source_index: previous_source_index as usize,
                    original_line: previous_original_line as usize,
                    original_column: previous_original_column as usize,
                });
            }
        }
        Ok(decoded)
    }

    fn decode_source_map_vlq_values(segment: &str) -> Result<Vec<isize>, String> {
        const BASE64: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut values = Vec::new();
        let mut value = 0usize;
        let mut shift = 0usize;
        for character in segment.chars() {
            let digit = BASE64
                .find(character)
                .ok_or_else(|| format!("invalid source-map digit {character:?}"))?;
            value |= (digit & 31) << shift;
            if digit & 32 == 0 {
                let magnitude = (value >> 1) as isize;
                values.push(if value & 1 == 0 {
                    magnitude
                } else {
                    -magnitude
                });
                value = 0;
                shift = 0;
            } else {
                shift += 5;
            }
        }
        if shift != 0 {
            return Err("unterminated source-map VLQ value".to_string());
        }
        Ok(values)
    }

    #[test]
    fn linked_bundle_retains_each_module_execution_before_bundle_projection() -> Result<(), String>
    {
        let style_sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/app.css".to_string(),
                style_source: "@import \"./tokens.css\";\n.app { color: green; }\n".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/tokens.css".to_string(),
                style_source: "@import \"./base.css\";\n.token { color: blue; }\n".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/base.css".to_string(),
                style_source: ".base { color: red; }\n".to_string(),
            },
        ];
        let pass_ids = vec!["import-inline".to_string(), "print-css".to_string()];
        let context = OmenaQueryTransformExecutionContextV0::default();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let link_options = TransformBundleLinkOptionsV0 {
            emission_ordering_policy: EmissionOrderingPolicyV0::ImportOrderPreserving,
        };
        let admission = link_closed_world_stylesheet_for_style_sources(
            ClosedWorldStylesheetRequestV0 {
                target_style_path: "src/app.css",
                style_sources: &style_sources,
                requested_pass_ids: &pass_ids,
                context: &context,
                reachability_context: &context,
                attribution_report: None,
                resolution_inputs: &resolution_inputs,
                external_sifs: &[],
            },
            link_options,
        );
        let linked = admission
            .into_requested_policy_result()
            .map_err(|error| format!("retention fixture should link: {error:?}"))?;
        let execution = execute_linked_bundle_modules(
            &linked,
            "src/app.css",
            &style_sources,
            &pass_ids,
            &context,
            &resolution_inputs,
            &OmenaQueryConsumerBuildOptionsV0 {
                bundle_emission_path: OmenaQueryBundleEmissionPathV0::LinkedOrder,
                ..OmenaQueryConsumerBuildOptionsV0::default()
            },
        )?;

        assert_eq!(
            execution.module_executions.len(),
            linked.linked_stylesheet.module_instances.len()
        );
        let retained_keys = execution
            .module_executions
            .iter()
            .map(|module| module.module_instance.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(retained_keys.len(), execution.module_executions.len());

        let target_instance = linked
            .linked_stylesheet
            .entrypoints
            .first()
            .ok_or_else(|| "retention fixture should have an entrypoint".to_string())?;
        let retained_entry = execution
            .module_executions
            .iter()
            .find(|module| &module.module_instance == target_instance)
            .ok_or_else(|| "entry execution should be retained".to_string())?;
        let scope_evidence = summarize_linked_bundle_execution_scope(&execution)?;
        assert_eq!(scope_evidence.field_scopes.len(), 27);
        assert_eq!(scope_evidence.module_executions.len(), 3);
        assert_eq!(
            scope_evidence.bundle_composite.module_count,
            scope_evidence.module_executions.len()
        );
        assert_eq!(
            scope_evidence
                .module_executions
                .iter()
                .map(|module| module.input_byte_len)
                .sum::<usize>(),
            scope_evidence.bundle_composite.summed_module_input_byte_len
        );
        assert_eq!(
            scope_evidence
                .module_executions
                .iter()
                .map(|module| module.output_byte_len)
                .sum::<usize>(),
            scope_evidence
                .bundle_composite
                .summed_module_output_byte_len
        );
        assert_eq!(
            scope_evidence
                .bundle_composite
                .summed_module_output_byte_len
                + scope_evidence
                    .bundle_composite
                    .inter_module_separator_byte_len,
            scope_evidence.bundle_composite.materialized_output_byte_len
        );

        let retained_json =
            serde_json::to_value(&retained_entry.execution).map_err(|error| error.to_string())?;
        let projected_json =
            serde_json::to_value(&execution.execution).map_err(|error| error.to_string())?;
        let conditionally_serialized_fields = scope_evidence
            .field_scopes
            .iter()
            .filter(|field| {
                retained_json.get(field.field_name).is_none()
                    && projected_json.get(field.field_name).is_none()
            })
            .map(|field| field.field_name)
            .collect::<BTreeSet<_>>();
        for field in &scope_evidence.field_scopes {
            let projected_value = projected_json.get(field.field_name);
            match field.scope {
                OmenaQueryExecutionEvidenceScopeV0::Entry => {
                    let retained_value = retained_json.get(field.field_name);
                    if !conditionally_serialized_fields.contains(field.field_name) {
                        // A required entry key can be made absent by a serde rename,
                        // and this production serializer emits every such key here.
                        assert!(
                            projected_value.is_some() && retained_value.is_some(),
                            "required entry-scoped field {} must be present on both executions",
                            field.field_name
                        );
                    }
                    assert_eq!(
                        projected_value.is_some(),
                        retained_value.is_some(),
                        "entry-scoped field {} must have symmetric presence",
                        field.field_name
                    );
                    assert_eq!(
                        projected_value, retained_value,
                        "entry-scoped field {}",
                        field.field_name
                    );
                }
                OmenaQueryExecutionEvidenceScopeV0::Bundle => match field.field_name {
                    "outputByteLen" => assert_eq!(
                        projected_value,
                        Some(&serde_json::json!(
                            execution.materialization.output_css.len()
                        ))
                    ),
                    "outputCss" => assert_eq!(
                        projected_value,
                        Some(&serde_json::json!(execution.materialization.output_css))
                    ),
                    field_name => {
                        return Err(format!("field {field_name} has no bundle-scope derivation"));
                    }
                },
            }
        }

        let mut expected_projected_json = retained_json;
        let expected_object = expected_projected_json
            .as_object_mut()
            .ok_or_else(|| "retained execution should serialize as an object".to_string())?;
        expected_object.insert(
            "outputByteLen".to_string(),
            serde_json::json!(execution.materialization.output_css.len()),
        );
        expected_object.insert(
            "outputCss".to_string(),
            serde_json::json!(execution.materialization.output_css),
        );
        assert_eq!(expected_projected_json, projected_json);
        Ok(())
    }

    #[test]
    fn bundle_execution_scope_wire_matches_typescript_fixture() -> Result<(), String> {
        let module_instance = omena_parser::ModuleInstanceKeyV0::unconfigured(
            omena_parser::ModuleIdV0::new("src/app.css"),
        );
        let evidence = OmenaQueryBundleExecutionScopeEvidenceV0 {
            schema_version: "0",
            product: "omena-query.bundle-execution-scope",
            entry_module_instance: module_instance.clone(),
            field_scopes: vec![
                OmenaQueryExecutionFieldScopeV0 {
                    field_name: "outcomes",
                    scope: OmenaQueryExecutionEvidenceScopeV0::Entry,
                    derivation: "retained entry outcomes",
                },
                OmenaQueryExecutionFieldScopeV0 {
                    field_name: "outputCss",
                    scope: OmenaQueryExecutionEvidenceScopeV0::Bundle,
                    derivation: "materialized bundle css",
                },
            ],
            module_executions: vec![OmenaQueryBundleModuleExecutionByteFactsV0 {
                module_instance: module_instance.clone(),
                input_byte_len: 23,
                output_byte_len: 17,
                generated_start: 2,
                generated_end: 19,
            }],
            bundle_composite: OmenaQueryBundleCompositeExecutionByteFactsV0 {
                module_count: 1,
                summed_module_input_byte_len: 23,
                summed_module_output_byte_len: 17,
                inter_module_separator_byte_len: 2,
                materialized_output_byte_len: 19,
            },
            source_map_dispositions: vec![
                OmenaQueryLinkedSourceMapDispositionV0 {
                    module_instance: module_instance.clone(),
                    granularity: OmenaQueryLinkedSourceMapGranularityV0::CstAnchors,
                    fallback_reason: None,
                    segment_count: 3,
                },
                OmenaQueryLinkedSourceMapDispositionV0 {
                    module_instance,
                    granularity: OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback,
                    fallback_reason: Some(LINKED_FALLBACK_SOURCE_START_REASON),
                    segment_count: 1,
                },
            ],
        };
        let actual = serde_json::to_value(evidence).map_err(|error| error.to_string())?;
        let expected_fixture =
            include_str!("../../tests/fixtures/bundle-execution-scope-wire.json");
        let expected: serde_json::Value =
            serde_json::from_str(expected_fixture).map_err(|error| error.to_string())?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn linked_bundle_source_map_uses_materialized_module_offsets() -> Result<(), String> {
        let style_sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/app.css".to_string(),
                style_source: "@import \"./tokens.css\";\n.linked-map-app-a { color: red; }\n.linked-map-app-b { color: green; }"
                    .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/tokens.css".to_string(),
                style_source: ".linked-map-token-a { color: blue; }\n.linked-map-token-b { color: cyan; }\n.linked-map-token-c { color: navy; }"
                    .to_string(),
            },
        ];
        let modules = style_sources
            .iter()
            .map(|source| {
                TransformBundleModuleInputV0::new(
                    source.style_path.clone(),
                    source.style_source.clone(),
                    omena_parser::StyleDialect::Css,
                )
            })
            .collect::<Vec<_>>();
        let linked = link_omena_transform_bundle_modules(&["src/app.css"], &modules)
            .map_err(|error| format!("source-map fixture should link: {error:?}"))?;
        let transformed = linked
            .module_instances
            .iter()
            .map(|module_instance| {
                let source = style_sources
                    .iter()
                    .find(|source| source.style_path == module_instance.module().as_str())
                    .ok_or_else(|| {
                        format!(
                            "source-map fixture has no source for {:?}",
                            module_instance.module()
                        )
                    })?;
                Ok(TransformBundleTransformedModuleV0::new(
                    module_instance.clone(),
                    source.style_source.clone(),
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let module_executions = linked
            .module_instances
            .iter()
            .map(|module_instance| {
                let source = style_sources
                    .iter()
                    .find(|source| source.style_path == module_instance.module().as_str())
                    .ok_or_else(|| {
                        format!(
                            "source-map fixture has no execution source for {:?}",
                            module_instance.module()
                        )
                    })?;
                let summary = execute_omena_query_consumer_build_style_source_with_context(
                    source.style_path.as_str(),
                    source.style_source.as_str(),
                    &["print-css".to_string()],
                    &TransformExecutionContextV0::default(),
                );
                Ok(LinkedModuleExecutionV0 {
                    module_instance: module_instance.clone(),
                    execution: summary.execution,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let materialization =
            materialize_omena_transform_bundle_linked_stylesheet(&linked, &transformed)
                .map_err(|error| format!("source-map fixture should materialize: {error:?}"))?;
        let (segments, dispositions) = linked_bundle_source_map_segments(
            &style_sources,
            &materialization.output_css,
            &materialization,
            &module_executions,
        )?;

        assert!(
            segments.len() > transformed.len(),
            "segments={}, modules={}, dispositions={dispositions:?}",
            segments.len(),
            transformed.len()
        );
        assert_eq!(dispositions.len(), transformed.len());
        assert!(dispositions.iter().all(|disposition| {
            disposition.granularity == OmenaQueryLinkedSourceMapGranularityV0::CstAnchors
                && disposition.fallback_reason.is_none()
        }));
        for segment in &segments {
            let region = materialization
                .module_regions
                .iter()
                .find(|region| region.module_instance.module().as_str() == segment.source_path)
                .ok_or_else(|| {
                    format!(
                        "source-map segment has no region for {:?}",
                        segment.source_path
                    )
                })?;
            assert!(segment.generated_start >= region.generated_start);
            assert!(segment.generated_end <= region.generated_end);
            assert!(segment.generated_end > segment.generated_start);
            assert_eq!(
                segment.generated_start_point.byte_offset,
                segment.generated_start
            );
            assert_eq!(segment.pass_id, "linked-order-emission");
        }
        for region in &materialization.module_regions {
            assert!(segments.iter().any(|segment| {
                segment.source_path == region.module_instance.module().as_str()
                    && segment.original_start > 0
            }));
        }
        let token_lines = segments
            .iter()
            .filter(|segment| segment.source_path == "src/tokens.css")
            .map(|segment| segment.original_start_point.line)
            .collect::<BTreeSet<_>>();
        assert!(token_lines.len() >= 3);
        let third_rule = segments
            .iter()
            .filter(|segment| segment.source_path == "src/tokens.css")
            .find(|segment| segment.original_start_point.line == 2)
            .ok_or_else(|| "third token rule should map to its source line".to_string())?;
        assert_eq!(third_rule.original_start_point.line, 2);

        let entry_instance = linked
            .entrypoints
            .first()
            .ok_or_else(|| "source-map fixture should have an entrypoint".to_string())?;
        let mut bundle_execution = module_executions
            .iter()
            .find(|module| &module.module_instance == entry_instance)
            .ok_or_else(|| "source-map fixture should retain its entry execution".to_string())?
            .execution
            .clone();
        bundle_execution.output_byte_len = materialization.output_css.len();
        bundle_execution
            .output_css
            .clone_from(&materialization.output_css);
        let (source_map, _) = summarize_omena_query_linked_bundle_source_map_v3(
            "src/app.css",
            &style_sources,
            &bundle_execution,
            &materialization,
            &module_executions,
        )?;
        assert_eq!(
            &materialization.output_css[third_rule.generated_start..third_rule.generated_end],
            "linked-map-token-c"
        );
        let generated_point = &third_rule.generated_start_point;
        let decoded = decode_source_map_mappings(&source_map.mappings)?;
        let decoded_third_rule = decoded
            .iter()
            .filter(|segment| {
                segment.generated_line < generated_point.line
                    || (segment.generated_line == generated_point.line
                        && segment.generated_column <= generated_point.utf8_column)
            })
            .max_by_key(|segment| (segment.generated_line, segment.generated_column))
            .ok_or_else(|| "third token rule should have a decoded mapping".to_string())?;
        assert_eq!(
            source_map.sources[decoded_third_rule.source_index],
            "src/tokens.css"
        );
        assert_eq!(decoded_third_rule.original_line, 2);
        Ok(())
    }

    #[test]
    fn linked_bundle_source_map_falls_back_when_module_output_changes() -> Result<(), String> {
        let source = "\n  .app { color: red; }";
        let style_sources = vec![OmenaQueryStyleSourceInputV0 {
            style_path: "src/app.css".to_string(),
            style_source: source.to_string(),
        }];
        let modules = vec![TransformBundleModuleInputV0::new(
            "src/app.css",
            source,
            omena_parser::StyleDialect::Css,
        )];
        let linked = link_omena_transform_bundle_modules(&["src/app.css"], &modules)
            .map_err(|error| format!("fallback fixture should link: {error:?}"))?;
        let module_instance = linked
            .module_instances
            .first()
            .ok_or_else(|| "fallback fixture should contain one module".to_string())?
            .clone();
        let transformed = vec![TransformBundleTransformedModuleV0::new(
            module_instance.clone(),
            ".app{color:red}",
        )];
        let materialization =
            materialize_omena_transform_bundle_linked_stylesheet(&linked, &transformed)
                .map_err(|error| format!("fallback fixture should materialize: {error:?}"))?;
        let mut execution = execute_omena_query_consumer_build_style_source_with_context(
            "src/app.css",
            source,
            &[],
            &TransformExecutionContextV0::default(),
        )
        .execution;
        execution.output_css = ".app{color:red}".to_string();
        execution.output_byte_len = execution.output_css.len();
        let module_executions = vec![LinkedModuleExecutionV0 {
            module_instance,
            execution,
        }];
        let (segments, dispositions) = linked_bundle_source_map_segments(
            &style_sources,
            &materialization.output_css,
            &materialization,
            &module_executions,
        )?;

        assert_eq!(segments.len(), 1);
        assert_eq!(dispositions.len(), 1);
        assert_eq!(
            dispositions[0].granularity,
            OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback
        );
        assert_eq!(
            dispositions[0].fallback_reason,
            Some(LINKED_FALLBACK_EXACT_TOKEN_REASON)
        );
        assert_eq!(segments[0].original_start, 3);
        assert_eq!(segments[0].original_end, source.len());
        assert_eq!(segments[0].original_start_point.byte_offset, 3);
        assert_eq!(segments[0].original_start_point.line, 1);
        assert_eq!(segments[0].original_start_point.utf8_column, 2);
        let bundle_execution = project_linked_bundle_execution(
            module_executions[0].execution.clone(),
            materialization.output_css.as_str(),
        );
        let (source_map, _) = summarize_omena_query_linked_bundle_source_map_v3(
            "src/app.css",
            &style_sources,
            &bundle_execution,
            &materialization,
            &module_executions,
        )?;
        let decoded = decode_source_map_mappings(source_map.mappings.as_str())?;
        let first_mapping = decoded
            .first()
            .ok_or_else(|| "fallback should emit a serialized mapping".to_string())?;
        assert_eq!(
            source_map.sources[first_mapping.source_index],
            "src/app.css"
        );
        assert_eq!(first_mapping.original_line, 1);
        assert_eq!(first_mapping.original_column, 2);
        Ok(())
    }

    #[test]
    fn linked_bundle_source_map_fallback_anchors_surviving_tokens_after_removed_import()
    -> Result<(), String> {
        let source = "@import \"./tokens.css\";\n.app { color: red; }";
        let generated = ".app{color:red}";
        let (segment, reason) =
            linked_whole_module_fallback_segment("src/app.css", source, generated);
        assert_eq!(reason, LINKED_FALLBACK_EXACT_TOKEN_REASON);
        assert_eq!(
            &source[segment.original_start..segment.original_end],
            ".app { color: red; }"
        );
        validate_linked_source_map_original_segment(
            "src/app.css",
            source,
            generated,
            &segment,
            OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback,
            Some(reason),
        )
    }

    #[test]
    fn linked_bundle_source_map_fallback_discloses_source_start_without_correspondence()
    -> Result<(), String> {
        let source = "@import \"./tokens.css\";\n  .app { color: red; }";
        let generated = "._app_0{color:blue}";
        let (segment, reason) =
            linked_whole_module_fallback_segment("src/app.css", source, generated);
        assert_eq!(reason, LINKED_FALLBACK_SOURCE_START_REASON);
        assert_eq!(&source[segment.original_start..], ".app { color: red; }");
        assert_eq!(segment.original_end, source.len());
        validate_linked_source_map_original_segment(
            "src/app.css",
            source,
            generated,
            &segment,
            OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback,
            Some(reason),
        )
    }

    #[test]
    fn linked_bundle_source_map_fallback_discloses_ambiguous_surviving_tokens() -> Result<(), String>
    {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/linked-source-map-fallback-ambiguity.json"
        ))
        .map_err(|error| error.to_string())?;
        let source_path = fixture["sourcePath"]
            .as_str()
            .ok_or_else(|| "ambiguity fixture has no sourcePath".to_string())?;
        let source = fixture["source"]
            .as_str()
            .ok_or_else(|| "ambiguity fixture has no source".to_string())?;
        let generated = fixture["generated"]
            .as_str()
            .ok_or_else(|| "ambiguity fixture has no generated output".to_string())?;
        let source_tokens = canonical_linked_fallback_tokens(
            lex_omena_query_omena_parser_style_source(source, omena_parser::StyleDialect::Css)
                .tokens(),
        )
        .iter()
        .map(|token| format!("{:?}:{}", token.kind, token.text))
        .collect::<Vec<_>>();
        let generated_tokens = canonical_linked_fallback_tokens(
            lex_omena_query_omena_parser_style_source(generated, omena_parser::StyleDialect::Css)
                .tokens(),
        )
        .iter()
        .map(|token| format!("{:?}:{}", token.kind, token.text))
        .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(&source_tokens).map_err(|error| error.to_string())?,
            fixture["sourceTokens"]
        );
        assert_eq!(
            serde_json::to_value(&generated_tokens).map_err(|error| error.to_string())?,
            fixture["generatedTokens"]
        );
        let matching_window_starts = source_tokens
            .windows(generated_tokens.len())
            .enumerate()
            .filter_map(|(index, window)| (window == generated_tokens).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::to_value(&matching_window_starts).map_err(|error| error.to_string())?,
            fixture["matchingWindowStarts"]
        );

        let style_sources = vec![OmenaQueryStyleSourceInputV0 {
            style_path: source_path.to_string(),
            style_source: source.to_string(),
        }];
        let modules = vec![TransformBundleModuleInputV0::new(
            source_path,
            source,
            omena_parser::StyleDialect::Css,
        )];
        let linked = link_omena_transform_bundle_modules(&[source_path], &modules)
            .map_err(|error| format!("ambiguity fixture should link: {error:?}"))?;
        let module_instance = linked
            .module_instances
            .first()
            .ok_or_else(|| "ambiguity fixture should contain one module".to_string())?
            .clone();
        let transformed = vec![TransformBundleTransformedModuleV0::new(
            module_instance.clone(),
            generated,
        )];
        let materialization =
            materialize_omena_transform_bundle_linked_stylesheet(&linked, &transformed)
                .map_err(|error| format!("ambiguity fixture should materialize: {error:?}"))?;
        let mut execution = execute_omena_query_consumer_build_style_source_with_context(
            source_path,
            source,
            &[],
            &TransformExecutionContextV0::default(),
        )
        .execution;
        execution.output_css = generated.to_string();
        execution.output_byte_len = generated.len();
        let module_executions = vec![LinkedModuleExecutionV0 {
            module_instance,
            execution,
        }];
        let (segments, dispositions) = linked_bundle_source_map_segments(
            &style_sources,
            &materialization.output_css,
            &materialization,
            &module_executions,
        )?;
        assert_eq!(segments.len(), 1);
        assert_eq!(dispositions.len(), 1);
        let segment = &segments[0];
        let (segment, reason) = (
            segment,
            dispositions[0]
                .fallback_reason
                .ok_or_else(|| "ambiguity fixture should disclose a fallback reason".to_string())?,
        );
        assert_eq!(reason, LINKED_FALLBACK_AMBIGUOUS_TOKEN_REASON);
        assert_eq!(
            serde_json::json!({
                "originalStart": segment.original_start,
                "originalEnd": segment.original_end,
                "generatedStart": segment.generated_start,
                "generatedEnd": segment.generated_end,
            }),
            fixture["expectedSegment"]
        );
        validate_linked_source_map_original_segment(
            source_path,
            source,
            generated,
            segment,
            OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback,
            Some(reason),
        )
    }

    #[test]
    fn linked_bundle_source_map_validator_rejects_exact_claim_for_ambiguous_tokens() {
        let source = ".app { color: red; }\n.app { color: red; }";
        let generated = ".app{color:red}";
        let segment = TransformSourceMapSegmentV0 {
            source_path: "src/app.css".to_string(),
            original_start: 0,
            original_end: 20,
            generated_start: 0,
            generated_end: generated.len(),
            original_start_point: transform_source_map_point(source, 0),
            original_end_point: transform_source_map_point(source, 20),
            generated_start_point: transform_source_map_point(generated, 0),
            generated_end_point: transform_source_map_point(generated, generated.len()),
            pass_id: "linked-order-emission",
        };
        let result = validate_linked_source_map_original_segment(
            "src/app.css",
            source,
            generated,
            &segment,
            OmenaQueryLinkedSourceMapGranularityV0::WholeModuleFallback,
            Some(LINKED_FALLBACK_EXACT_TOKEN_REASON),
        );
        assert!(result.is_err());
        let error = result.err().unwrap_or_default();
        assert!(error.contains("without one unique matching token window"));
    }
}

#[cfg(test)]
mod dependency_resolution_tests {
    use super::*;

    fn configured_sass_sources() -> Vec<OmenaQueryStyleSourceInputV0> {
        vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/blue.scss".to_string(),
                style_source:
                    r#"@use "./theme" with ($brand: blue); .blue { color: theme.$brand; }"#
                        .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/red.scss".to_string(),
                style_source: r#"@use "./theme" with ($brand: red); .red { color: theme.$brand; }"#
                    .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/theme.scss".to_string(),
                style_source:
                    "$brand: black !default; .kept { color: $brand; } .dead { color: gray; }"
                        .to_string(),
            },
        ]
    }

    #[test]
    fn linker_projection_records_resolver_attempt_provenance() {
        let sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/app.css".to_string(),
                style_source: r#"@import "@acme/theme/tokens.css"; .app { color: green; }"#
                    .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "node_modules/@acme/theme/dist/tokens.css".to_string(),
                style_source: ".token { color: rebeccapurple; }".to_string(),
            },
        ];
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            package_manifests: vec![OmenaQueryStylePackageManifestV0 {
                package_json_path: "node_modules/@acme/theme/package.json".to_string(),
                package_json_source:
                    r#"{"name":"@acme/theme","exports":{"./tokens.css":"./dist/tokens.css"}}"#
                        .to_string(),
            }],
            ..OmenaQueryStyleResolutionInputsV0::default()
        };
        let prepared = prepare_transform_bundle_linker_projection(
            &["src/app.css"],
            &sources,
            &[],
            TransformResolutionContext::from_resolution_inputs(&resolution_inputs),
        );
        let resolved = prepared.resolved_dependencies;

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].resolution.attempt_state, "attempted");
        assert_eq!(
            resolved[0].resolution.resolution_kind,
            Some("packageStyleModule")
        );
        assert_eq!(
            resolved[0].resolution.policy_step_keys,
            vec![
                "externalUrlBoundary",
                "bundlerPathMapping",
                "tsconfigPathMapping",
                "sassPkgImporter",
                "fileRelativeOrAbsolute",
                "packageManifestSubpath",
                "nodePackageFallback",
                "sassLoadPathRoot",
            ]
        );
        assert_eq!(
            resolved[0].resolution.target_instance,
            prepared
                .projection
                .inputs()
                .iter()
                .find(|input| { input.source_path == "node_modules/@acme/theme/dist/tokens.css" })
                .map(|input| input.instance.clone())
        );
    }

    #[test]
    fn configured_sass_edges_select_distinct_instances_without_reparsing() -> Result<(), String> {
        let sources = configured_sass_sources();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let mut reachability = TransformBundleSemanticReachabilityInputV0::new("src/theme.scss");
        reachability.class_names.push("kept".to_string());
        let (prepared, parser_snapshot) =
            omena_parser::with_omena_parser_parse_instrumentation(|| {
                prepare_transform_bundle_linker_projection(
                    &["src/blue.scss", "src/red.scss"],
                    &sources,
                    std::slice::from_ref(&reachability),
                    TransformResolutionContext::from_resolution_inputs(&resolution_inputs),
                )
            });

        assert_eq!(parser_snapshot.parse_invocation_count, 3);
        let theme_inputs = prepared
            .projection
            .inputs()
            .iter()
            .filter(|input| input.source_path == "src/theme.scss")
            .collect::<Vec<_>>();
        assert_eq!(theme_inputs.len(), 2);
        assert_eq!(
            theme_inputs
                .iter()
                .map(|input| input.instance.configuration().as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["with|5:brand=3:red", "with|5:brand=4:blue"])
        );
        assert!(theme_inputs.iter().all(|input| {
            input.class_names == ["kept".to_string()]
                && !input.class_names.contains(&"dead".to_string())
        }));

        let target_by_source = prepared
            .resolved_dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.source_instance.module().as_str(),
                    dependency
                        .resolution
                        .target_instance
                        .as_ref()
                        .map(|instance| instance.configuration().as_str()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            target_by_source.get("src/blue.scss").copied().flatten(),
            Some("with|5:brand=4:blue")
        );
        assert_eq!(
            target_by_source.get("src/red.scss").copied().flatten(),
            Some("with|5:brand=3:red")
        );

        let linked = link_omena_transform_bundle_projection_with_resolved_dependencies_and_options(
            &["src/blue.scss", "src/red.scss"],
            &prepared.projection,
            prepared.resolved_dependencies.as_slice(),
            &[],
            TransformBundleLinkOptionsV0::default(),
        )
        .map_err(|error| format!("configured Sass workspace should link: {error:?}"))?;
        assert_eq!(
            linked
                .module_instances
                .iter()
                .filter(|instance| instance.module().as_str() == "src/theme.scss")
                .count(),
            2
        );
        Ok(())
    }

    #[test]
    fn configured_module_path_can_also_be_an_unconfigured_entrypoint() -> Result<(), String> {
        let sources = configured_sass_sources();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let prepared = prepare_transform_bundle_linker_projection(
            &["src/blue.scss", "src/red.scss", "src/theme.scss"],
            &sources,
            &[],
            TransformResolutionContext::from_resolution_inputs(&resolution_inputs),
        );
        let linked = link_omena_transform_bundle_projection_with_resolved_dependencies_and_options(
            &["src/blue.scss", "src/red.scss", "src/theme.scss"],
            &prepared.projection,
            prepared.resolved_dependencies.as_slice(),
            &[],
            TransformBundleLinkOptionsV0::default(),
        )
        .map_err(|error| format!("configured module entrypoint should link: {error:?}"))?;

        let theme_entrypoint = linked
            .entrypoints
            .iter()
            .find(|entrypoint| entrypoint.module().as_str() == "src/theme.scss")
            .ok_or_else(|| "theme entrypoint was not selected".to_string())?;
        assert_eq!(
            theme_entrypoint.configuration(),
            &omena_parser::ConfigurationHashV0::none()
        );
        Ok(())
    }

    #[test]
    fn unconfigured_projection_preserves_closure_identity() -> Result<(), String> {
        let sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/app.css".to_string(),
                style_source: r#"@import "./theme.css"; .app { color: green; }"#.to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "src/theme.css".to_string(),
                style_source: ".theme { color: rebeccapurple; }".to_string(),
            },
        ];
        let legacy_modules = sources
            .iter()
            .map(|source| {
                TransformBundleModuleInputV0::new(
                    source.style_path.as_str(),
                    source.style_source.as_str(),
                    omena_parser::StyleDialect::Css,
                )
            })
            .collect::<Vec<_>>();
        let legacy = link_omena_transform_bundle_modules(&["src/app.css"], &legacy_modules)
            .map_err(|error| format!("legacy unconfigured fixture should link: {error:?}"))?;
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let prepared = prepare_transform_bundle_linker_projection(
            &["src/app.css"],
            &sources,
            &[],
            TransformResolutionContext::from_resolution_inputs(&resolution_inputs),
        );
        let current =
            link_omena_transform_bundle_projection_with_resolved_dependencies_and_options(
                &["src/app.css"],
                &prepared.projection,
                prepared.resolved_dependencies.as_slice(),
                &[],
                TransformBundleLinkOptionsV0::default(),
            )
            .map_err(|error| format!("prepared unconfigured fixture should link: {error:?}"))?;

        assert_eq!(current.module_instances, legacy.module_instances);
        assert_eq!(
            current.closed_world_bundle.closure_hash(),
            legacy.closed_world_bundle.closure_hash()
        );
        Ok(())
    }
}

#[cfg(test)]
mod closed_world_link_error_tests {
    use super::closed_world_blocker_from_link_error;
    use crate::OmenaQueryClosedWorldBlockerV0;
    use omena_query_transform_runner::{TransformBundleEdgeKind, TransformBundleLinkErrorV0};

    #[test]
    fn engine_only_emission_failures_preserve_the_sdk_blocker_contract() {
        let requested_pass_ids = vec!["tree-shake".to_string()];
        let expected = OmenaQueryClosedWorldBlockerV0::ClosedWorldPassUnavailable {
            requested_pass_ids: requested_pass_ids.clone(),
        };

        for error in [
            TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: "duplicate order key".to_string(),
            },
            TransformBundleLinkErrorV0::UnsupportedEmissionCycle {
                edge_kind: TransformBundleEdgeKind::SassUse,
            },
        ] {
            assert_eq!(
                closed_world_blocker_from_link_error(error, &requested_pass_ids),
                expected
            );
        }
    }
}

#[cfg(test)]
mod closed_set_precision_tests {
    use super::*;

    #[test]
    fn sealed_bundle_content_binds_finite_reachability_precision() -> Result<(), String> {
        let style_path = "Workspace.module.css";
        let style_source = ".card {} .panel {} .toolbar {} .dead {}";
        let reachable_class_names = vec![
            "card".to_string(),
            "panel".to_string(),
            "toolbar".to_string(),
        ];
        let context = TransformExecutionContextV0 {
            reachable_class_names: reachable_class_names.clone(),
            ..TransformExecutionContextV0::default()
        };
        let requested_pass_ids = vec!["tree-shake-class".to_string()];
        let bundle = build_closed_world_bundle_for_single_style_source_context(
            style_path,
            style_source,
            &requested_pass_ids,
            &context,
        )
        .ok_or_else(|| {
            "the finite reachability fixture should produce a sealed bundle".to_string()
        })?;
        let finite_value = AbstractClassValueV0::FiniteSet {
            values: reachable_class_names,
        };
        let open_world_precision = fact_precision_from_class_value(&finite_value);
        let closed_world_precision = closed_world_bound_reachability_precision(
            &context,
            &bundle,
            Some(open_world_precision),
            true,
        );
        let non_enumerated_precision = closed_world_bound_reachability_precision(
            &context,
            &bundle,
            Some(open_world_precision),
            false,
        );
        let missing_member_context = TransformExecutionContextV0 {
            reachable_class_names: vec!["card".to_string(), "outside-bundle".to_string()],
            ..TransformExecutionContextV0::default()
        };
        let missing_member_precision = closed_world_bound_reachability_precision(
            &missing_member_context,
            &bundle,
            Some(open_world_precision),
            true,
        );

        assert_eq!(open_world_precision, FactPrecision::Conservative);
        assert_eq!(closed_world_precision, FactPrecision::Exact);
        assert_eq!(non_enumerated_precision, FactPrecision::Conservative);
        assert_eq!(missing_member_precision, FactPrecision::Conservative);

        let calibration_report: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../omena-precision-calibration-report.json"
        ))
        .map_err(|error| format!("precision calibration report should be valid JSON: {error}"))?;
        assert_eq!(
            calibration_report["cases"][1],
            serde_json::json!({
                "caseId": "closedSetFiniteReachability",
                "inputClassCount": 3,
                "representation": "finiteSet",
                "witnessDirection": "supersetOfProducible",
                "witnessBasis": "closedSetEnumeration",
                "authority": "closedWorldBundleClosureHash",
                "openWorldPrecision": open_world_precision,
                "closedWorldPrecision": closed_world_precision,
                "nonEnumeratedPrecision": non_enumerated_precision,
                "missingMemberPrecision": missing_member_precision,
            })
        );
        Ok(())
    }
}
