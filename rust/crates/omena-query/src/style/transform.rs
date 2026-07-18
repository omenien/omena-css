use super::*;
use omena_cascade::SupportsTargetCapabilityV0;
use omena_parser::{
    ClosedWorldBundleBuildErrorV0, ClosedWorldBundleV0, ClosedWorldModuleMetadataV0,
    ClosedWorldSourcePrecisionSummaryV0, OpenWorldSnapshotV0,
};
use omena_query_transform_runner::{
    TransformBundleLinkErrorV0, TransformBundleModuleInputV0,
    TransformBundleSemanticReachabilityInputV0, classify_transform_reachability_precision,
    execute_transform_passes_on_source_with_dialect_context_closed_world_bundle_and_precision,
    link_omena_transform_bundle_modules,
    link_omena_transform_bundle_modules_with_semantic_reachability,
    link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata,
};
use omena_query_transform_runner::{
    transform_pass_requires_closed_world_bundle, transform_pass_sort_ordinal,
};
use std::path::{Path, PathBuf};

use super::parser_facade::parse_omena_query_omena_parser_style_source;

mod context;
mod css_modules;
pub(super) use css_modules::derive_class_name_rewrites_for_transform_context;
mod design_tokens;
mod imports;
mod static_stylesheet;

use context::TransformResolutionContext;
pub use context::summarize_omena_query_transform_context_from_engine_input;

use context::{
    derive_omena_query_transform_context_from_engine_input, find_target_style_source,
    merge_target_options_transform_context, merge_transform_context,
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
    let context = merge_workspace_transform_context(
        target_style_path,
        style_sources,
        context,
        TransformResolutionContext::from_resolution_inputs(resolution_inputs),
    );
    let summary =
        execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
            target_style_path,
            style_sources,
            requested_pass_ids,
            &context,
            resolution_inputs,
        )?;
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
    let source_map_v3 = summarize_omena_query_consumer_build_source_map_v3_with_resolution_inputs(
        target_style_path,
        source_map_sources,
        &summary.execution,
        resolution_inputs,
    );
    let code_split_outputs = summarize_omena_query_bundle_code_split_workspace_plan(
        target_style_path,
        bundle_entry_style_paths,
        style_sources,
        resolution_inputs,
    )?
    .outputs;
    let closed_world_outcome = build_closed_world_outcome_for_style_sources(
        target_style_path,
        style_sources,
        requested_pass_ids,
        &context,
        external_sifs,
    );
    let legacy_open_decision = legacy_bundle_open_decision(
        target_style_path,
        style_sources,
        requested_pass_ids,
        &context,
    );
    let closed_world_decision_parity = OmenaQueryClosedWorldDecisionParityV0 {
        legacy_open_decision,
        typed_outcome_open: closed_world_outcome.is_open(),
        equivalent: legacy_open_decision == closed_world_outcome.is_open(),
    };
    validate_omena_query_closed_world_decision_parity(&closed_world_decision_parity)?;

    let artifact = OmenaQueryBundleArtifactV0 {
        schema_version: "0",
        product: "omena-query.bundle-artifact",
        style_path: target_style_path.to_string(),
        output_css: summary.execution.output_css.clone(),
        bundle,
        source_map_v3,
        code_split_outputs,
        asset_rewrites,
        per_pass_provenance: summary.execution.outcomes.clone(),
        execution: summary.execution,
        ready_surfaces: vec![
            "bundleOperationFacade",
            "transformBundlePlan",
            "transformExecutionRuntime",
            "sourceMapV3Serializer",
            "bundleCodeSplitPlan",
            "transformPassOutcomeContract",
        ],
    };
    Ok(OmenaQueryBundleResultV0 {
        artifact,
        closed_world_outcome,
        closed_world_decision_parity,
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
    execute_omena_query_consumer_build_style_source_with_context_and_reachability_precision(
        style_path,
        style_source,
        requested_pass_ids,
        context,
        None,
        false,
    )
}

fn execute_omena_query_consumer_build_style_source_with_context_and_reachability_precision(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    reachability_precision: Option<FactPrecision>,
    closed_set_enumeration_candidate: bool,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let context = merge_single_source_transform_context(style_path, style_source, context);
    let closed_world_outcome = requested_pass_ids_require_closed_world_bundle(requested_pass_ids)
        .then(|| {
            build_closed_world_outcome_for_single_style_source_context(
                style_path,
                style_source,
                requested_pass_ids,
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
            requested_pass_ids,
            &context,
            closed_world_bundle,
            reachability_precision,
        );
    }

    execute_omena_query_consumer_build_style_source_with_open_world_context(
        style_path,
        style_source,
        requested_pass_ids,
        &context,
    )
}

fn execute_omena_query_consumer_build_style_source_with_open_world_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let pass_ids = if requested_pass_ids.is_empty() {
        default_consumer_build_transform_pass_ids()
    } else {
        requested_pass_ids.to_vec()
    };
    let execution_summary =
        execute_omena_query_transform_passes_from_source_with_open_world_context(
            style_path,
            style_source,
            &pass_ids,
            context,
        );
    let open_world_snapshot =
        open_world_snapshot_for_requested_closed_world_passes(requested_pass_ids);
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
        requested_pass_ids: requested_pass_ids.to_vec(),
        target_query: None,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
        source_map_v3: None,
        open_world_snapshot,
        ready_surfaces,
    }
}

fn execute_omena_query_consumer_build_style_source_with_context_and_closed_world_bundle(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    closed_world_bundle: &ClosedWorldBundleV0,
    reachability_precision: FactPrecision,
) -> OmenaQueryConsumerBuildSummaryV0 {
    let pass_ids = if requested_pass_ids.is_empty() {
        default_consumer_build_transform_pass_ids()
    } else {
        requested_pass_ids.to_vec()
    };
    let context = merge_single_source_transform_context(style_path, style_source, context);
    let execution_summary =
        execute_omena_query_transform_passes_from_source_with_context_and_closed_world_bundle(
            style_path,
            style_source,
            &pass_ids,
            &context,
            closed_world_bundle,
            reachability_precision,
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
        bundle: None,
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

fn default_consumer_build_transform_pass_ids() -> Vec<String> {
    all_transform_pass_kinds()
        .into_iter()
        .filter(|pass| *pass != TransformPassKind::NativeCssStaticEval)
        .map(|pass| pass.id().to_string())
        .collect()
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
            &context_derivation.summary.context,
            context_derivation.reachability_precision,
            context_derivation.closed_set_enumeration_candidate,
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
    let resolution_inputs = resolution_inputs_for_transform_style_sources(
        target_style_path,
        style_sources,
        package_manifests,
    );
    execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
        target_style_path,
        style_sources,
        requested_pass_ids,
        context,
        &resolution_inputs,
    )
}

pub fn execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
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
    let closed_world_outcome = requested_pass_ids_require_closed_world_bundle(requested_pass_ids)
        .then(|| {
            build_closed_world_outcome_for_style_sources(
                target_style_path,
                style_sources,
                requested_pass_ids,
                &context,
                &[],
            )
        });
    let mut summary = if let Some(closed_world_bundle) = closed_world_outcome
        .as_ref()
        .and_then(OmenaQueryClosedWorldOutcomeV0::bundle)
    {
        execute_omena_query_consumer_build_style_source_with_context_and_closed_world_bundle(
            target_style_path,
            target_source,
            requested_pass_ids,
            &context,
            closed_world_bundle,
            FactPrecision::Conservative,
        )
    } else {
        execute_omena_query_consumer_build_style_source_with_open_world_context(
            target_style_path,
            target_source,
            requested_pass_ids,
            &context,
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
    let execution_summary = execute_omena_query_consumer_build_style_source_with_context(
        style_path,
        style_source,
        &requested_pass_ids,
        &execution_context,
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
        target_query: plan.target_query,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
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
    let execution_summary =
        execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
            target_style_path,
            style_sources,
            &requested_pass_ids,
            &execution_context,
            resolution_inputs,
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
        target_query: plan.target_query,
        unknown_pass_ids: execution_summary.unknown_pass_ids,
        semantic_removal_count: execution_summary.semantic_removal_count,
        execution: execution_summary.execution,
        bundle: None,
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
    let source_by_path = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut entry_style_paths = vec![primary_entry_style_path.to_string()];
    for configured_entry in bundle_entry_style_paths {
        if configured_entry != primary_entry_style_path
            && !entry_style_paths.contains(configured_entry)
        {
            entry_style_paths.push(configured_entry.clone());
        }
    }
    for entry_style_path in &entry_style_paths {
        if !source_by_path.contains_key(entry_style_path.as_str()) {
            return Err(format!(
                "bundle entry source is not loaded: {entry_style_path}"
            ));
        }
    }

    let entry_style_path_set = entry_style_paths.iter().cloned().collect::<BTreeSet<_>>();
    let entry_reachability = collect_omena_query_bundle_code_split_entry_reachability(
        entry_style_paths.as_slice(),
        &source_by_path,
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

fn collect_omena_query_bundle_code_split_entry_reachability(
    entry_style_paths: &[String],
    source_by_path: &BTreeMap<&str, &str>,
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
            let Some(source) = source_by_path.get(style_path.as_str()) else {
                continue;
            };
            reachability
                .entry(style_path.clone())
                .or_default()
                .insert(entry_style_path.clone());
            let bundle = summarize_omena_transform_bundle_from_source(
                style_path.as_str(),
                source,
                omena_parser_dialect_for_style_path(style_path.as_str()),
            );
            for edge in bundle.bundle_edges {
                if !matches!(
                    edge.kind,
                    TransformBundleEdgeKind::CssImport | TransformBundleEdgeKind::LessImport
                ) {
                    continue;
                }
                let Some(import_source) = edge.import_source.as_deref() else {
                    continue;
                };
                let Some(target_path) = resolution_context.resolve_style_module_source(
                    style_path.as_str(),
                    import_source,
                    available_style_paths,
                ) else {
                    continue;
                };
                if source_by_path.contains_key(target_path.as_str()) {
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
    if requested_pass_ids_require_closed_world_bundle(requested_pass_ids)
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
            FactPrecision::Conservative,
        );
    }

    execute_omena_query_transform_passes_from_source_with_open_world_context(
        style_path,
        style_source,
        requested_pass_ids,
        &context,
    )
}

fn execute_omena_query_transform_passes_from_source_with_open_world_context(
    style_path: &str,
    style_source: &str,
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> OmenaQueryTransformExecuteSummaryV0 {
    let (requested_passes, unknown_pass_ids) =
        requested_transform_passes_from_ids(requested_pass_ids);

    let dialect = omena_parser_dialect_for_style_path(style_path);
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        style_source,
        dialect,
        &requested_passes,
        context,
    );
    let semantic_removal_count = execution.semantic_removals.len();
    let open_world_snapshot =
        open_world_snapshot_for_requested_closed_world_passes(requested_pass_ids);
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
) -> OmenaQueryTransformExecuteSummaryV0 {
    let (requested_passes, unknown_pass_ids) =
        requested_transform_passes_from_ids(requested_pass_ids);

    let dialect = omena_parser_dialect_for_style_path(style_path);
    let execution =
        execute_transform_passes_on_source_with_dialect_context_closed_world_bundle_and_precision(
            style_source,
            dialect,
            &requested_passes,
            context,
            closed_world_bundle,
            reachability_precision,
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
        open_world_snapshot: None,
        ready_surfaces: vec![
            "transformExecutionRuntime",
            "transformPassOutcomeContract",
            "closedWorldBundle",
        ],
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

fn requested_pass_ids_require_closed_world_bundle(requested_pass_ids: &[String]) -> bool {
    requested_pass_ids
        .iter()
        .filter_map(|pass_id| transform_pass_kind_from_id(pass_id))
        .any(transform_pass_requires_closed_world_bundle)
}

fn open_world_snapshot_for_requested_closed_world_passes(
    requested_pass_ids: &[String],
) -> Option<OpenWorldSnapshotV0> {
    if !requested_pass_ids_require_closed_world_bundle(requested_pass_ids) {
        return None;
    }

    Some(OpenWorldSnapshotV0::new(format!(
        "closed-world bundle unavailable for requested passes: {}",
        requested_pass_ids.join(", ")
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

fn build_closed_world_outcome_for_style_sources(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> OmenaQueryClosedWorldOutcomeV0 {
    let reachability_inputs = if requested_pass_ids_include_tree_shake(requested_pass_ids) {
        style_sources
            .iter()
            .filter_map(|source| {
                transform_bundle_semantic_reachability_input_from_context(
                    source.style_path.as_str(),
                    context,
                )
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let modules = style_sources_to_transform_bundle_modules(style_sources);
    let module_metadata =
        style_sources_to_closed_world_metadata(style_sources, &modules, context, external_sifs);
    let linked = link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata(
        &[target_style_path],
        &modules,
        reachability_inputs.as_slice(),
        &module_metadata,
    );
    closed_world_outcome_from_link_result(linked, requested_pass_ids)
}

fn legacy_bundle_open_decision(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    requested_pass_ids: &[String],
    context: &TransformExecutionContextV0,
) -> bool {
    let modules = style_sources
        .iter()
        .map(|source| {
            TransformBundleModuleInputV0::new(
                source.style_path.as_str(),
                source.style_source.as_str(),
                omena_parser_dialect_for_style_path(source.style_path.as_str()),
            )
        })
        .collect::<Vec<_>>();
    let reachability_inputs = if requested_pass_ids_include_tree_shake(requested_pass_ids) {
        style_sources
            .iter()
            .filter_map(|source| {
                transform_bundle_semantic_reachability_input_from_context(
                    source.style_path.as_str(),
                    context,
                )
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    if reachability_inputs.is_empty() {
        link_omena_transform_bundle_modules(&[target_style_path], &modules).is_err()
    } else {
        link_omena_transform_bundle_modules_with_semantic_reachability(
            &[target_style_path],
            &modules,
            reachability_inputs.as_slice(),
        )
        .is_err()
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
    let modules = style_sources_to_transform_bundle_modules(sources);
    let module_metadata = style_sources_to_closed_world_metadata(sources, &modules, context, &[]);
    let Some(reachability_input) =
        transform_bundle_semantic_reachability_input_from_context(style_path, context)
    else {
        if requested_pass_ids_include_tree_shake(requested_pass_ids) {
            return OmenaQueryClosedWorldOutcomeV0::Open {
                blockers: vec![OmenaQueryClosedWorldBlockerV0::ClosedWorldPassUnavailable {
                    requested_pass_ids: requested_pass_ids.to_vec(),
                }],
            };
        }
        return closed_world_outcome_from_link_result(
            link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata(
                &[style_path],
                &modules,
                &[],
                &module_metadata,
            ),
            requested_pass_ids,
        );
    };

    closed_world_outcome_from_link_result(
        link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata(
            &[style_path],
            &modules,
            std::slice::from_ref(&reachability_input),
            &module_metadata,
        ),
        requested_pass_ids,
    )
}

fn style_sources_to_transform_bundle_modules(
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> Vec<TransformBundleModuleInputV0> {
    style_sources
        .iter()
        .map(|source| {
            TransformBundleModuleInputV0::new(
                source.style_path.as_str(),
                source.style_source.as_str(),
                omena_parser_dialect_for_style_path(source.style_path.as_str()),
            )
        })
        .collect()
}

fn style_sources_to_closed_world_metadata(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    modules: &[TransformBundleModuleInputV0],
    context: &TransformExecutionContextV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Vec<ClosedWorldModuleMetadataV0> {
    let source_precision = closed_world_source_precision_summary(context);
    style_sources
        .iter()
        .zip(modules)
        .map(|(source, module)| {
            let mut metadata = ClosedWorldModuleMetadataV0::new(module.module_instance_key())
                .with_source_precision(source_precision);
            if let Some(interface_hash) = external_sifs.iter().find_map(|external_sif| {
                sif_matches_style_path(external_sif, source.style_path.as_str()).then(|| {
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

fn closed_world_source_precision_summary(
    context: &TransformExecutionContextV0,
) -> ClosedWorldSourcePrecisionSummaryV0 {
    let precision = classify_transform_reachability_precision(context, true, None);
    let mut summary = ClosedWorldSourcePrecisionSummaryV0::default();
    match precision {
        FactPrecision::Exact => summary.exact_source_count = 1,
        FactPrecision::Conservative => summary.conservative_source_count = 1,
        FactPrecision::Heuristic => summary.heuristic_source_count = 1,
        FactPrecision::Unknown => summary.unknown_source_count = 1,
    }
    summary
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
    let input = TransformBundleSemanticReachabilityInputV0 {
        source_path: style_path.to_string(),
        class_names: context.reachable_class_names.clone(),
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
    fn sealed_bundle_content_binds_finite_reachability_precision() {
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
        .expect("the finite reachability fixture should produce a sealed bundle");
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
        .expect("precision calibration report should be valid JSON");
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
    }
}
