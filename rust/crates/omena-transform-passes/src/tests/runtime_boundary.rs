use std::collections::BTreeSet;

use crate::{
    TransformExecutionContextV0, TransformModuleQualifiedExecutionErrorV0,
    TransformPassDispatchKindV0, default_transform_pass_registry,
    execute_transform_passes_incremental_with_database,
    execute_transform_passes_on_module_with_dialect_context_and_closed_world_bundle,
    execute_transform_passes_on_source,
    execute_transform_passes_on_source_with_dialect_and_context,
    execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle,
    plan_transform_passes, plan_transform_passes_checked,
    registry::{
        flatten_css_layers_in_ir, tree_shake_css_class_rules_in_ir,
        tree_shake_css_custom_properties_in_ir, tree_shake_css_keyframes_in_ir,
        tree_shake_css_modules_values_in_ir,
    },
    run_transform_fuzz_seed_corpus, summarize_omena_transform_passes_boundary,
    summarize_structural_ir_shadow_equivalence_v0, transform_pass_incremental_graph_input,
};
use omena_incremental::{IncrementalRevisionV0, OmenaIncrementalDatabaseV0};
use omena_parser::{
    ClosedWorldBundleV0, ClosedWorldLinkedModuleV0, ConfigurationHashV0, ModuleIdV0,
    ModuleInstanceKeyV0, StyleDialect,
};
use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformPassClassV0, TransformPassKind,
    default_transform_pass_contracts, default_transform_pass_descriptors,
    lower_transform_ir_from_source, print_transform_ir_css,
};

fn expected_structural_transform_pass_ids() -> Vec<&'static str> {
    let mut pass_ids = default_transform_pass_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Structural)
        .map(|descriptor| descriptor.id)
        .collect::<Vec<_>>();
    pass_ids.sort_unstable();
    pass_ids
}

#[test]
fn registry_covers_full_transform_catalog() {
    let boundary = summarize_omena_transform_passes_boundary();

    assert_eq!(boundary.schema_version, "0");
    assert_eq!(boundary.product, "omena-transform-passes.boundary");
    assert_eq!(boundary.pass_count, TRANSFORM_PASS_CATALOG_LEN);
    assert!(boundary.full_catalog_registered);
    assert_eq!(boundary.semantic_aware_pass_count, 14);
    assert!(boundary.cascade_aware_pass_count >= 9);
    assert_eq!(boundary.structural_pass_count, 21);
    assert_eq!(boundary.text_local_pass_count, 20);
    assert_eq!(boundary.module_evaluation_pass_count, 2);
    assert!(boundary.planner_enforces_dag_edges);
    assert!(boundary.planner_uses_pass_descriptors);
    assert!(!boundary.ordinal_has_execution_semantics);
    assert!(boundary.execution_runtime_ready);
    assert!(boundary.incremental_execution_runtime_ready);
    assert_eq!(
        boundary.module_evaluation_native_output_marker,
        "nativeEditOutput"
    );
    assert!(boundary.module_evaluation_requires_native_product_output);
    assert!(boundary.module_evaluation_requires_oracle_readiness);
    assert!(boundary.module_evaluation_legacy_output_is_oracle_only);
    assert!(boundary.module_evaluation_preserves_source_without_native_output);
    let mutation_pass_ids = default_transform_pass_contracts()
        .into_iter()
        .filter(|contract| contract.executes_mutation)
        .map(|contract| contract.id)
        .collect::<Vec<_>>();
    assert_eq!(boundary.implemented_mutation_pass_ids, mutation_pass_ids);
    assert_eq!(
        boundary.implemented_mutation_pass_ids.len(),
        TRANSFORM_PASS_CATALOG_LEN
    );
    assert!(boundary.registry_entries.iter().any(|entry| {
        entry.contract.kind == TransformPassKind::TreeShakeClass
            && entry.descriptor.kind == TransformPassKind::TreeShakeClass
            && entry.descriptor.pass_class == TransformPassClassV0::Structural
            && entry.module_family == "semantic-reachability"
    }));
    assert!(boundary.registry_entries.iter().any(|entry| {
        entry.contract.kind == TransformPassKind::StalePrefixRemoval
            && entry.descriptor.kind == TransformPassKind::StalePrefixRemoval
            && entry.descriptor.pass_class == TransformPassClassV0::TextLocal
            && entry.module_family == "target-lowering"
            && entry.contract.read_model == omena_transform_cst::TransformPassReadModel::TargetData
    }));
    assert!(
        !boundary
            .next_surfaces
            .contains(&"transformContextProducers")
    );
    assert!(
        !boundary
            .next_surfaces
            .contains(&"provenanceSourceSpanMapping")
    );
    assert!(!boundary.next_surfaces.contains(&"transformSalsaQueries"));
    assert!(!boundary.next_surfaces.contains(&"sourceMapSpanPrecision"));
}

#[test]
fn pass_registry_subsumes_contracts_and_descriptors() {
    let registry = default_transform_pass_registry();
    let boundary = summarize_omena_transform_passes_boundary();

    assert_eq!(registry.schema_version, "0");
    assert_eq!(registry.product, "omena-transform-passes.pass-registry");
    assert_eq!(registry.entries.len(), TRANSFORM_PASS_CATALOG_LEN);
    assert_eq!(boundary.registry_entries, registry.entries);
    assert!(registry.entries.iter().all(|entry| {
        entry.contract.kind == entry.descriptor.kind && entry.contract.id == entry.descriptor.id
    }));
    assert!(registry.entries.iter().all(|entry| {
        let expected_dispatch_kind = match entry.descriptor.pass_class {
            TransformPassClassV0::TextLocal => TransformPassDispatchKindV0::TextLocalSliceRewrite,
            TransformPassClassV0::Structural => {
                TransformPassDispatchKindV0::StructuralIrTransaction
            }
            TransformPassClassV0::ModuleEvaluation => {
                TransformPassDispatchKindV0::ModuleEvaluationHandler
            }
            TransformPassClassV0::Emission => TransformPassDispatchKindV0::EmissionBoundary,
        };
        entry.dispatch_kind == expected_dispatch_kind
    }));
    assert!(
        registry
            .entries
            .iter()
            .filter(|entry| entry.descriptor.pass_class == TransformPassClassV0::Emission)
            .all(|entry| entry.contract.kind == TransformPassKind::PrintCss)
    );
}

#[test]
fn structural_ir_shadow_report_covers_structural_ir_paths() {
    let report = summarize_structural_ir_shadow_equivalence_v0();
    let expected_pass_ids = expected_structural_transform_pass_ids();

    assert_eq!(
        report.product,
        "omena-transform-passes.structural-ir-shadow-equivalence"
    );
    assert_eq!(report.compared_pass_ids, expected_pass_ids);
    assert_eq!(
        report.compared_fields,
        vec![
            "canonicalCssBytes",
            "selectorSet",
            "declarationSet",
            "cascadeOutcome",
            "mutationSpanRanges",
            "mutationCount",
            "semanticRemovals",
            "cssImportInlines",
            "cssModuleComposesExports",
            "cssModuleEvaluation",
            "designTokenRoutes",
            "irTransactionCommitCount"
        ]
    );
    assert_eq!(
        report
            .reports
            .iter()
            .map(|fixture| fixture.pass_id)
            .collect::<BTreeSet<_>>(),
        report
            .compared_pass_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(report.fixture_count, 28);
    assert!(report.all_fields_match, "{report:#?}");
    assert!(report.all_typed_path_fields_match, "{report:#?}");
    assert!(report.typed_payload_projections_consumed > 0);
    assert!(report.typed_payload_memo_hits > 0);
    assert!(report.reports.iter().all(|fixture| {
        fixture.all_fields_match
            && fixture.string_path_mutation_count == fixture.ir_path_mutation_count
            && fixture.ir_path_mutation_count == fixture.typed_path_mutation_count
            && fixture.ir_path_transaction_commit_count.is_some()
    }));
    assert!(report.reports.iter().any(|fixture| {
        fixture.pass_id == "nesting-unwrap"
            && fixture.typed_payload_projections_consumed > 0
            && fixture.typed_payload_memo_hits > 0
    }));
}

#[test]
fn pass_registry_exposes_nested_color_lowering_conflict_from_shared_value_rewrites() {
    let descriptors = default_transform_pass_descriptors();
    let color_mix_conflicts = descriptors
        .iter()
        .filter(|descriptor| descriptor.kind == TransformPassKind::ColorMixLowering)
        .flat_map(|descriptor| descriptor.conflicts_with.iter().copied())
        .collect::<Vec<_>>();
    let color_function_conflicts = descriptors
        .iter()
        .filter(|descriptor| descriptor.kind == TransformPassKind::ColorFunctionLowering)
        .flat_map(|descriptor| descriptor.conflicts_with.iter().copied())
        .collect::<Vec<_>>();

    assert_eq!(
        color_mix_conflicts,
        vec![TransformPassKind::ColorFunctionLowering.id()]
    );
    assert_eq!(
        color_function_conflicts,
        vec![TransformPassKind::ColorMixLowering.id()]
    );
}

#[test]
fn planner_rejects_unordered_color_lowering_conflict_without_reordering_other_sets() {
    let conflict_plan = plan_transform_passes(&[
        TransformPassKind::ColorMixLowering,
        TransformPassKind::ColorFunctionLowering,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(
        conflict_plan.ordered_pass_ids,
        vec!["color-mix-lowering", "color-function-lowering", "print-css"]
    );
    assert_eq!(conflict_plan.conflicting_unordered_pass_pairs.len(), 1);
    assert_eq!(
        conflict_plan.conflicting_unordered_pass_pairs[0].pass_a,
        "color-mix-lowering"
    );
    assert_eq!(
        conflict_plan.conflicting_unordered_pass_pairs[0].pass_b,
        "color-function-lowering"
    );
    assert_eq!(
        plan_transform_passes_checked(&[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::ColorFunctionLowering,
            TransformPassKind::PrintCss,
        ]),
        Err(conflict_plan.conflicting_unordered_pass_pairs[0].clone())
    );

    let accepted_plan = plan_transform_passes(&[
        TransformPassKind::ColorMixLowering,
        TransformPassKind::LightDarkLowering,
        TransformPassKind::PrintCss,
    ]);
    assert!(accepted_plan.conflicting_unordered_pass_pairs.is_empty());
    assert_eq!(
        accepted_plan.ordered_pass_ids,
        vec!["light-dark-lowering", "color-mix-lowering", "print-css"]
    );
    assert_eq!(
        plan_transform_passes_checked(&[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::LightDarkLowering,
            TransformPassKind::PrintCss,
        ])
        .map(|plan| plan.ordered_pass_ids),
        Ok(accepted_plan.ordered_pass_ids)
    );
}

#[test]
fn nested_color_lowering_conflict_has_swapped_order_output_divergence() {
    fn execute_pair(source: &str, left: TransformPassKind, right: TransformPassKind) -> String {
        let left_first = execute_transform_passes_on_source(source, &[left]);
        execute_transform_passes_on_source(&left_first.output_css, &[right]).output_css
    }

    let source = ".card { color: color-mix(in srgb, color(srgb 1 0 0), blue); }";
    let mix_then_function = execute_pair(
        source,
        TransformPassKind::ColorMixLowering,
        TransformPassKind::ColorFunctionLowering,
    );
    let function_then_mix = execute_pair(
        source,
        TransformPassKind::ColorFunctionLowering,
        TransformPassKind::ColorMixLowering,
    );

    assert_ne!(mix_then_function, function_then_mix);
    assert_eq!(
        mix_then_function,
        ".card { color: color-mix(in srgb, rgb(255 0 0), blue); }"
    );
    assert_eq!(function_then_mix, ".card { color: rgb(128 0 128); }");
}

#[test]
fn executor_loop_no_longer_threads_output_css_as_interpass_currency() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("executor.rs"),
    )
    .map_err(|err| format!("executor source should be readable: {err:?}"))?;
    let loop_anchor = source
        .find("fn execute_transform_passes_on_source_with_active_lex_cache")
        .ok_or_else(|| "executor loop should exist".to_string())?;
    let loop_body = &source[loop_anchor..];

    assert!(!loop_body.contains("let mut output_css"));
    assert!(!loop_body.contains("pass_input_css = output_css"));
    assert!(!loop_body.contains("output_css = next_css"));
    assert!(loop_body.contains("TransformExecutionDocumentV0::new(source, dialect)"));
    assert!(loop_body.contains("document.current_ir_mut()"));
    assert!(loop_body.contains("document.output_css()"));
    Ok(())
}

#[test]
fn structural_ir_transaction_helper_has_no_fallback_currency() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("helpers")
            .join("ir_transaction.rs"),
    )
    .map_err(|err| format!("IR transaction helper source should be readable: {err:?}"))?;
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .ok_or_else(|| "IR transaction helper production body should exist".to_string())?;

    assert!(!production_source.contains("apply_source_range_replacements_to_ir"));
    assert!(!production_source.contains("apply_ir_source_replacements_to_ir"));
    assert!(!production_source.contains("validate_source_range_replacements"));
    assert!(!production_source.contains("record_source_range_rewrite_fallback"));
    assert!(!production_source.contains("record_print_relower_fallback"));
    assert!(!production_source.contains("build_stable_transform_ir_from_source"));
    assert!(!production_source.contains("StableTransformIrV0"));
    assert!(!production_source.contains("StableTransformIrNodeKindV0"));
    assert!(!production_source.contains("stable_fact"));
    assert!(!production_source.contains("print_transform_ir_css"));
    assert!(!production_source.contains("let source = ir.source_text().to_string();"));

    for entry in [
        "delete_ir_nodes_in_ir",
        "replace_ir_nodes_in_ir",
        "replace_ir_node_spans_in_ir",
        "replace_ir_node_with_inserted_nodes_in_ir",
        "replace_ir_nodes_with_inserted_ir_roots_in_ir",
        "commit_ir_replacement_targets",
    ] {
        let anchor = production_source
            .find(&format!("fn {entry}("))
            .or_else(|| production_source.find(&format!("pub(crate) fn {entry}(")))
            .ok_or_else(|| format!("{entry} should exist"))?;
        let signature_end = production_source[anchor..]
            .find('{')
            .map(|offset| anchor + offset)
            .ok_or_else(|| format!("{entry} should have a function body"))?;
        let signature = &production_source[anchor..signature_end];

        assert!(
            !signature.contains("Result<(String"),
            "{entry} should not expose rendered CSS as transaction currency"
        );
    }
    Ok(())
}

#[test]
fn structural_registry_wrappers_do_not_export_rendered_css() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("registry.rs"),
    )
    .map_err(|err| format!("registry source should be readable: {err:?}"))?;
    let structural_entries = [
        "remove_empty_css_rules_in_ir",
        "dedupe_exact_css_rules_in_ir",
        "merge_adjacent_same_selector_css_rules_in_ir",
        "merge_adjacent_same_block_css_selectors_in_ir",
        "unwrap_css_nesting_in_ir",
        "flatten_css_scopes_in_ir",
        "flatten_css_layers_in_ir",
        "evaluate_static_supports_rules_in_ir",
        "evaluate_static_media_rules_in_ir",
        "evaluate_static_container_rules_in_ir",
        "evaluate_native_css_static_values_in_ir",
        "evaluate_dead_media_branch_rules_in_ir",
        "inline_css_imports_in_ir",
        "resolve_css_module_composes_in_ir",
        "route_design_token_values_in_ir",
        "tree_shake_css_class_rules_in_ir",
        "tree_shake_css_keyframes_in_ir",
        "tree_shake_css_modules_values_in_ir",
        "tree_shake_css_custom_properties_in_ir",
        "rewrite_css_module_class_names_in_ir",
    ];

    for entry in structural_entries {
        let anchor = source
            .find(&format!("pub(crate) fn {entry}("))
            .ok_or_else(|| format!("{entry} should exist"))?;
        let tail = &source[anchor + 1..];
        let next_pub = tail.find("\npub").unwrap_or(tail.len());
        let body = &source[anchor..anchor + 1 + next_pub];

        assert!(
            !body.contains("Result<(String"),
            "{entry} should not expose rendered CSS as structural dispatch currency"
        );
    }
    Ok(())
}

#[test]
fn structural_domain_ir_entrypoints_do_not_return_rendered_css() -> Result<(), String> {
    let domains_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("domains");
    for entry in std::fs::read_dir(domains_dir)
        .map_err(|err| format!("domains dir should be readable: {err:?}"))?
    {
        let entry = entry.map_err(|err| format!("domain entry should be readable: {err:?}"))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path)
            .map_err(|err| format!("domain source should be readable: {err:?}"))?;
        let mut search_start = 0usize;
        while let Some(relative_anchor) = source[search_start..].find("with_ir_transaction_on_ir(")
        {
            let anchor = search_start + relative_anchor;
            let signature_end = source[anchor..]
                .find('{')
                .map(|offset| anchor + offset)
                .ok_or_else(|| format!("{} has an unterminated on-IR signature", path.display()))?;
            let signature = &source[anchor..signature_end];
            if signature.contains("Result<(String") {
                return Err(format!(
                    "{} exposes rendered CSS from an on-IR structural entrypoint: {signature}",
                    path.display()
                ));
            }
            search_start = signature_end;
        }
    }
    Ok(())
}

#[test]
fn structural_cascade_proof_obligations_match_source_and_ir_collectors() -> Result<(), String> {
    let source = "@scope (.card) { .item { color: red; } }\
        @layer reset, theme;\
        @layer reset { .item { color: blue; } }\
        @layer theme { .item { color: red !important; } }\
        @supports (display: grid) { .grid { display: grid; } }";
    let instance = ModuleInstanceKeyV0::new(
        ModuleIdV0::new("runtime-boundary.css"),
        ConfigurationHashV0::none(),
    );
    let closed_world_bundle = ClosedWorldBundleV0::try_from_linked_modules(
        vec![instance.clone()],
        vec![
            ClosedWorldLinkedModuleV0::new(instance)
                .with_class_name("item")
                .with_class_name("grid"),
        ],
    )
    .map_err(|err| format!("closed-world test bundle should be constructible: {err:?}"))?;
    let ir = lower_transform_ir_from_source(
        source,
        StyleDialect::Css,
        "omena-transform-passes.test.structural-cascade-proof-ir",
    );
    let context = TransformExecutionContextV0::default();

    for pass in [
        TransformPassKind::ScopeFlatten,
        TransformPassKind::LayerFlatten,
        TransformPassKind::SupportsStaticEval,
    ] {
        let source_obligations =
            crate::runtime::cascade_proof::collect_cascade_proof_obligations_for_pass_input(
                pass.id(),
                Some(pass),
                source,
                StyleDialect::Css,
                &context,
                Some(&closed_world_bundle),
            );
        let ir_obligations =
            crate::runtime::cascade_proof::collect_cascade_proof_obligations_for_ir_pass_input(
                pass.id(),
                Some(pass),
                &ir,
                &context,
                Some(&closed_world_bundle),
            );

        assert_eq!(
            ir_obligations.len(),
            source_obligations.len(),
            "obligation count drift for {}",
            pass.id()
        );
        assert_eq!(
            ir_obligations
                .iter()
                .map(|obligation| obligation.proof_product)
                .collect::<Vec<_>>(),
            source_obligations
                .iter()
                .map(|obligation| obligation.proof_product)
                .collect::<Vec<_>>(),
            "proof product drift for {}",
            pass.id()
        );
        assert_eq!(
            ir_obligations
                .iter()
                .map(|obligation| obligation.accepted)
                .collect::<Vec<_>>(),
            source_obligations
                .iter()
                .map(|obligation| obligation.accepted)
                .collect::<Vec<_>>(),
            "acceptance drift for {}",
            pass.id()
        );
    }
    Ok(())
}

#[test]
fn structural_cascade_proof_runtime_path_uses_ir_collectors() -> Result<(), String> {
    let executor_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("executor.rs"),
    )
    .map_err(|err| format!("executor source should be readable: {err:?}"))?;
    let loop_anchor = executor_source
        .find("fn execute_transform_passes_on_source_with_active_lex_cache")
        .ok_or_else(|| "executor loop should exist".to_string())?;
    let next_section_anchor = executor_source[loop_anchor..]
        .find("fn transform_pass_may_consume_lex_cache")
        .ok_or_else(|| "lex cache classifier should delimit executor loop".to_string())?;
    let loop_body = &executor_source[loop_anchor..loop_anchor + next_section_anchor];

    assert!(loop_body.contains("collect_cascade_proof_obligations_for_ir_pass_input("));
    assert!(loop_body.contains("pass_cascade_proof_obligations"));
    assert!(loop_body.contains("&document.current_ir"));

    let cascade_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("cascade_proof.rs"),
    )
    .map_err(|err| format!("cascade proof source should be readable: {err:?}"))?;
    let ir_anchor = cascade_source
        .find("pub(crate) fn collect_cascade_proof_obligations_for_ir_pass_input")
        .ok_or_else(|| "IR cascade proof collector should exist".to_string())?;
    let summary_anchor = cascade_source[ir_anchor..]
        .find("pub(crate) fn summarize_cascade_proof_obligations")
        .ok_or_else(|| "cascade proof summary should delimit IR collector".to_string())?;
    let ir_body = &cascade_source[ir_anchor..ir_anchor + summary_anchor];

    assert!(ir_body.contains("collect_scope_flatten_proof_candidates_from_ir(ir)"));
    assert!(ir_body.contains("collect_layer_flatten_proof_candidates_from_ir(ir"));
    assert!(ir_body.contains("collect_layer_inversion_declarations_from_ir(ir)"));
    assert!(ir_body.contains("collect_static_supports_proof_candidates_from_ir("));
    assert!(!ir_body.contains("_with_lexer("));
    Ok(())
}

#[test]
fn closed_world_structural_gates_read_bundle_witness() -> Result<(), String> {
    let executor_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("executor.rs"),
    )
    .map_err(|err| format!("executor source should be readable: {err:?}"))?;

    for function_name in [
        "fn run_layer_flatten_structural",
        "fn run_tree_shake_class_structural",
        "fn run_tree_shake_keyframes_structural",
        "fn run_tree_shake_value_structural",
        "fn run_tree_shake_custom_property_structural",
    ] {
        let anchor = executor_source
            .find(function_name)
            .ok_or_else(|| format!("{function_name} should exist"))?;
        let next_function = executor_source[anchor + function_name.len()..]
            .find("\nfn ")
            .ok_or_else(|| format!("{function_name} should be delimited by the next function"))?;
        let body = &executor_source[anchor..anchor + function_name.len() + next_function];
        assert!(body.contains("input.closed_world_bundle()"));
        let retired_context_gate = format!("input.context.{}{}", "closed_style", "_world");
        assert!(!body.contains(&retired_context_gate));
    }
    Ok(())
}

#[test]
fn closed_world_bundle_authority_drives_reachability_transform_families() -> Result<(), String> {
    let instance = ModuleInstanceKeyV0::new(
        ModuleIdV0::new("bundle-authority.module.css"),
        ConfigurationHashV0::none(),
    );
    let bundle = ClosedWorldBundleV0::try_from_linked_modules(
        vec![instance.clone()],
        vec![
            ClosedWorldLinkedModuleV0::new(instance)
                .with_class_name("used")
                .with_keyframe_name("live")
                .with_value_name("usedValue")
                .with_custom_property_name("--explicit"),
        ],
    )
    .map_err(|err| format!("closed-world bundle should be constructible: {err:?}"))?;
    let misleading_context = TransformExecutionContextV0 {
        reachable_class_names: vec!["dead".to_string()],
        reachable_keyframe_names: vec!["ghost".to_string()],
        reachable_value_names: vec!["deadValue".to_string()],
        reachable_custom_property_names: vec!["--dead".to_string()],
        ..TransformExecutionContextV0::default()
    };

    let cases = [
        (
            TransformPassKind::TreeShakeClass,
            ".used { color: red; } .dead { color: blue; }",
            ".dead { color: blue; }",
        ),
        (
            TransformPassKind::TreeShakeKeyframes,
            ".used { animation: live 1s; } .dead { animation: ghost 1s; } @keyframes live { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } }",
            "@keyframes ghost",
        ),
        (
            TransformPassKind::TreeShakeValue,
            "@value usedValue: red; @value deadValue: blue; .used { color: usedValue; } .dead { color: deadValue; }",
            "@value deadValue:",
        ),
        (
            TransformPassKind::TreeShakeCustomProperty,
            ":root { --used: red; --dead: blue; --explicit: green; } .used { color: var(--used); border-color: var(--explicit); } .dead { color: var(--dead); }",
            "--dead:",
        ),
        (
            TransformPassKind::LayerFlatten,
            "@layer theme { .used { color: red; } }",
            "@layer theme",
        ),
    ];

    for (pass, source, removed_fragment) in cases {
        let requested = [pass, TransformPassKind::PrintCss];
        let without_bundle = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &requested,
            &misleading_context,
        );
        assert_eq!(without_bundle.output_css, source);
        assert_eq!(without_bundle.mutation_count, 0);
        assert!(
            without_bundle.planned_only_pass_ids.contains(&pass.id()),
            "{} should stay planned-only without a closed-world bundle",
            pass.id()
        );

        let with_bundle =
            execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle(
                source,
                StyleDialect::Css,
                &requested,
                &misleading_context,
                &bundle,
            );
        assert!(
            with_bundle.executed_pass_ids.contains(&pass.id()),
            "{} should execute with an explicit closed-world bundle",
            pass.id()
        );
        assert!(
            with_bundle.mutation_count > 0,
            "{} should mutate when the bundle supplies the closed-world authority",
            pass.id()
        );
        assert!(
            !with_bundle.output_css.contains(removed_fragment),
            "{} should ignore caller reachability context and use bundle reachability",
            pass.id()
        );
    }

    Ok(())
}

#[test]
fn tree_shake_bundle_driven_matches_reachability_projection_byte_identical() -> Result<(), String> {
    let instance = ModuleInstanceKeyV0::new(
        ModuleIdV0::new("reachability-projection.module.css"),
        ConfigurationHashV0::none(),
    );
    let bundle = ClosedWorldBundleV0::try_from_linked_modules(
        vec![instance.clone()],
        vec![
            ClosedWorldLinkedModuleV0::new(instance)
                .with_class_name("used")
                .with_keyframe_name("live")
                .with_value_name("usedValue")
                .with_custom_property_name("--explicit"),
        ],
    )
    .map_err(|err| format!("closed-world bundle should be constructible: {err:?}"))?;
    let cases = [
        (
            TransformPassKind::TreeShakeClass,
            ".used { color: red; } .dead { color: blue; }",
        ),
        (
            TransformPassKind::TreeShakeKeyframes,
            ".used { animation: live 1s; } .dead { animation: ghost 1s; } @keyframes live { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } }",
        ),
        (
            TransformPassKind::TreeShakeValue,
            "@value usedValue: red; @value deadValue: blue; .used { color: usedValue; } .dead { color: deadValue; }",
        ),
        (
            TransformPassKind::TreeShakeCustomProperty,
            ":root { --used: red; --dead: blue; --explicit: green; } .used { color: var(--used); border-color: var(--explicit); } .dead { color: var(--dead); }",
        ),
        (
            TransformPassKind::LayerFlatten,
            "@layer theme { .used { color: red; } }",
        ),
    ];

    for (pass, source) in cases {
        let mut expected_ir = lower_transform_ir_from_source(
            source,
            StyleDialect::Css,
            "omena-transform-passes.test.reachability-projection",
        );
        let expected_mutation_count =
            apply_direct_reachability_projection(pass, &mut expected_ir, &bundle)?;
        let expected_css =
            print_transform_ir_css(&expected_ir).map_err(|err| format!("{err:?}"))?;
        let execution =
            execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle(
                source,
                StyleDialect::Css,
                &[pass, TransformPassKind::PrintCss],
                &TransformExecutionContextV0::default(),
                &bundle,
            );

        assert_eq!(
            execution.output_css.as_bytes(),
            expected_css.as_bytes(),
            "{} should render the same bytes as direct bundle reachability projection",
            pass.id()
        );
        assert_eq!(
            execution.mutation_count,
            expected_mutation_count,
            "{} should preserve direct bundle reachability mutation count",
            pass.id()
        );
    }

    Ok(())
}

#[test]
fn module_qualified_tree_shake_distinguishes_same_name_owners() -> Result<(), String> {
    let app = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("app.module.css"));
    let detached = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("detached.module.css"));
    let unknown = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("unknown.module.css"));
    let shared_names = ["shared", "shared-secondary"];
    let app_module = ClosedWorldLinkedModuleV0::new(app.clone())
        .with_class_name(shared_names[0])
        .with_class_name(shared_names[1]);
    let detached_module = ClosedWorldLinkedModuleV0::new(detached.clone())
        .with_class_name(shared_names[0])
        .with_class_name(shared_names[1]);
    let bundle = ClosedWorldBundleV0::try_from_linked_modules(
        vec![app.clone()],
        vec![app_module.clone(), detached_module.clone()],
    )
    .map_err(|error| format!("closed-world bundle should be constructible: {error:?}"))?;
    let source = ".shared { color: red; } .shared-secondary { color: blue; }";
    let requested = [
        TransformPassKind::TreeShakeClass,
        TransformPassKind::PrintCss,
    ];

    assert!(shared_names.iter().all(|name| {
        bundle
            .reachability()
            .class_names()
            .iter()
            .any(|item| item == name)
    }));
    let detached_symbols = bundle
        .reachability()
        .symbols_for_module(&detached)
        .ok_or_else(|| "known detached module should have a qualified bucket".to_string())?;
    assert!(!detached_symbols.is_reachable());
    assert!(detached_symbols.class_names().is_empty());

    let default_execution =
        execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle(
            source,
            StyleDialect::Css,
            &requested,
            &TransformExecutionContextV0::default(),
            &bundle,
        );
    assert!(
        shared_names
            .iter()
            .all(|name| default_execution.output_css.contains(name))
    );
    assert!(default_execution.module_qualified_shake.is_none());
    let default_json = serde_json::to_value(&default_execution)
        .map_err(|error| format!("default execution should serialize: {error}"))?;
    assert!(default_json.get("moduleQualifiedShake").is_none());

    let qualified_execution =
        execute_transform_passes_on_module_with_dialect_context_and_closed_world_bundle(
            source,
            StyleDialect::Css,
            &requested,
            &TransformExecutionContextV0::default(),
            &bundle,
            &detached,
        )
        .map_err(|error| format!("known module execution should be accepted: {error:?}"))?;
    assert!(
        shared_names
            .iter()
            .all(|name| !qualified_execution.output_css.contains(name)),
        "qualified execution should remove detached owners: {qualified_execution:#?}"
    );
    let qualified_shake = qualified_execution
        .module_qualified_shake
        .as_ref()
        .ok_or_else(|| "qualified execution should report removals".to_string())?;
    assert_eq!(qualified_shake.module_instance, detached);
    assert_eq!(qualified_shake.removed_count, 2);

    assert_eq!(
        execute_transform_passes_on_module_with_dialect_context_and_closed_world_bundle(
            source,
            StyleDialect::Css,
            &requested,
            &TransformExecutionContextV0::default(),
            &bundle,
            &unknown,
        ),
        Err(
            TransformModuleQualifiedExecutionErrorV0::UnknownModuleInstance {
                module_instance: unknown,
            }
        )
    );

    let both_reachable_bundle = ClosedWorldBundleV0::try_from_linked_modules(
        vec![app.clone()],
        vec![
            app_module.with_dependency(detached.clone()),
            detached_module,
        ],
    )
    .map_err(|error| format!("connected bundle should be constructible: {error:?}"))?;
    let both_reachable_execution =
        execute_transform_passes_on_module_with_dialect_context_and_closed_world_bundle(
            source,
            StyleDialect::Css,
            &requested,
            &TransformExecutionContextV0::default(),
            &both_reachable_bundle,
            &detached,
        )
        .map_err(|error| format!("reachable module execution should be accepted: {error:?}"))?;
    assert!(
        shared_names
            .iter()
            .all(|name| both_reachable_execution.output_css.contains(name))
    );
    assert_eq!(
        both_reachable_execution
            .module_qualified_shake
            .as_ref()
            .map(|summary| summary.removed_count),
        Some(0)
    );

    Ok(())
}

fn apply_direct_reachability_projection(
    pass: TransformPassKind,
    ir: &mut omena_transform_cst::TransformIrV0,
    bundle: &ClosedWorldBundleV0,
) -> Result<usize, String> {
    let reachability = bundle.reachability();
    match pass {
        TransformPassKind::TreeShakeClass => {
            tree_shake_css_class_rules_in_ir(ir, StyleDialect::Css, reachability.class_names())
                .map(|removals| removals.len())
                .map_err(|err| format!("{err:?}"))
        }
        TransformPassKind::TreeShakeKeyframes => tree_shake_css_keyframes_in_ir(
            ir,
            StyleDialect::Css,
            reachability.keyframe_names(),
            reachability.class_names(),
        )
        .map(|removals| removals.len())
        .map_err(|err| format!("{err:?}")),
        TransformPassKind::TreeShakeValue => tree_shake_css_modules_values_in_ir(
            ir,
            StyleDialect::Css,
            reachability.value_names(),
            reachability.keyframe_names(),
            reachability.class_names(),
        )
        .map(|removals| removals.len())
        .map_err(|err| format!("{err:?}")),
        TransformPassKind::TreeShakeCustomProperty => tree_shake_css_custom_properties_in_ir(
            ir,
            StyleDialect::Css,
            reachability.custom_property_names(),
            reachability.keyframe_names(),
            reachability.class_names(),
        )
        .map(|removals| removals.len())
        .map_err(|err| format!("{err:?}")),
        TransformPassKind::LayerFlatten => {
            flatten_css_layers_in_ir(ir, StyleDialect::Css, true).map_err(|err| format!("{err:?}"))
        }
        _ => Err(format!(
            "unsupported reachability projection pass: {}",
            pass.id()
        )),
    }
}

#[test]
fn static_at_rule_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("static_eval.rs"),
    )
    .map_err(|err| format!("static eval source should be readable: {err:?}"))?;
    let static_ir_anchor = source
        .find("fn apply_static_ir_replacements_until_stable")
        .ok_or_else(|| "static IR replacement helper should exist".to_string())?;
    let next_section_anchor = source[static_ir_anchor..]
        .find("fn normalize_simple_media_range_features")
        .ok_or_else(|| "media normalization section should delimit static IR helper".to_string())?;
    let static_ir_body = &source[static_ir_anchor..static_ir_anchor + next_section_anchor];

    assert!(static_ir_body.contains("collect: impl Fn(&TransformIrV0)"));
    assert!(static_ir_body.contains("collect(ir)"));
    assert!(!static_ir_body.contains("collect(ir.source_text()"));
    assert!(!static_ir_body.contains("lex("));
    Ok(())
}

#[test]
fn native_css_static_eval_structural_path_uses_transform_ir_plan() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("registry.rs"),
    )
    .map_err(|err| format!("registry source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn evaluate_native_css_static_values_in_ir")
        .ok_or_else(|| "native CSS static IR entrypoint should exist".to_string())?;
    let next_entry_anchor = source[entry_anchor..]
        .find("pub(crate) fn evaluate_dead_media_branch_rules_in_ir")
        .ok_or_else(|| {
            "dead media entrypoint should delimit native CSS static IR entry".to_string()
        })?;
    let entry_body = &source[entry_anchor..entry_anchor + next_entry_anchor];

    assert!(entry_body.contains("summarize_native_css_static_edit_plan_from_transform_ir(ir"));
    assert!(!entry_body.contains("summarize_native_css_static_edit_plan(source.as_str()"));
    assert!(!entry_body.contains("let source = ir.source_text().to_string();"));
    Ok(())
}

#[test]
fn nesting_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("nesting.rs"),
    )
    .map_err(|err| format!("nesting source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn unwrap_css_nesting_with_ir_transaction_on_ir")
        .ok_or_else(|| "nesting IR transaction entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn collect_nesting_unwrap_replacements(")
        .ok_or_else(|| "legacy nesting collector should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_nesting_unwrap_rule_sets_from_ir")
        .ok_or_else(|| "nesting IR collector should exist".to_string())?;
    let next_legacy_section_anchor = source[ir_collector_anchor..]
        .find("fn unwrap_nested_rule_body(")
        .ok_or_else(|| "legacy nesting body collector should delimit IR collector".to_string())?;
    let ir_collector_body =
        &source[ir_collector_anchor..ir_collector_anchor + next_legacy_section_anchor];

    assert!(entry_body.contains("collect_nesting_unwrap_rule_sets_from_ir(ir)"));
    assert!(!entry_body.contains("collect_nesting_unwrap_replacements(ir.source_text()"));
    assert!(!ir_collector_body.contains("collect_nesting_unwrap_replacements("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn cascade_flatten_structural_ir_paths_use_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("cascade_flatten.rs"),
    )
    .map_err(|err| format!("cascade flatten source should be readable: {err:?}"))?;
    let scope_entry_anchor = source
        .find("pub(crate) fn flatten_css_scopes_with_ir_transaction_on_ir")
        .ok_or_else(|| "scope flatten IR entrypoint should exist".to_string())?;
    let scope_legacy_anchor = source[scope_entry_anchor..]
        .find("fn collect_scope_flatten_replacements(")
        .ok_or_else(|| "legacy scope collector should delimit entrypoint".to_string())?;
    let scope_entry_body = &source[scope_entry_anchor..scope_entry_anchor + scope_legacy_anchor];
    let layer_entry_anchor = source
        .find("pub(crate) fn flatten_css_layers_with_ir_transaction_on_ir")
        .ok_or_else(|| "layer flatten IR entrypoint should exist".to_string())?;
    let layer_legacy_anchor = source[layer_entry_anchor..]
        .find("fn collect_layer_flatten_replacements(")
        .ok_or_else(|| "legacy layer collector should delimit entrypoint".to_string())?;
    let layer_entry_body = &source[layer_entry_anchor..layer_entry_anchor + layer_legacy_anchor];
    let ir_collector_anchor = source
        .find("fn collect_scope_flatten_replacements_from_ir")
        .ok_or_else(|| "scope flatten IR collector should exist".to_string())?;
    let legacy_proof_anchor = source[ir_collector_anchor..]
        .find("pub(crate) fn collect_scope_flatten_proof_candidates_with_lexer")
        .ok_or_else(|| {
            "proof candidate collector should delimit cascade flatten IR section".to_string()
        })?;
    let ir_collector_body = &source[ir_collector_anchor..ir_collector_anchor + legacy_proof_anchor];

    assert!(scope_entry_body.contains("collect_scope_flatten_replacements_from_ir(ir)"));
    assert!(layer_entry_body.contains("collect_layer_flatten_replacements_from_ir(ir"));
    assert!(!scope_entry_body.contains("collect_scope_flatten_replacements(ir.source_text()"));
    assert!(!layer_entry_body.contains("collect_layer_flatten_replacements(ir.source_text()"));
    assert!(!ir_collector_body.contains("collect_scope_flatten_replacements("));
    assert!(!ir_collector_body.contains("collect_layer_flatten_replacements("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn rule_dedup_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("rule_cleanup.rs"),
    )
    .map_err(|err| format!("rule cleanup source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn dedupe_exact_css_rules_with_ir_transaction_on_ir")
        .ok_or_else(|| "rule dedup IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn remove_overridden_same_property_declarations_with_lexer")
        .ok_or_else(|| "legacy rule dedup section should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_overridden_same_property_declaration_replacements_from_ir")
        .ok_or_else(|| "rule dedup IR declaration collector should exist".to_string())?;
    let next_runtime_section_anchor = source[ir_collector_anchor..]
        .find("fn rule_dedup_deletion_node_ids")
        .ok_or_else(|| "rule dedup deletion section should delimit IR collectors".to_string())?;
    let ir_collector_body =
        &source[ir_collector_anchor..ir_collector_anchor + next_runtime_section_anchor];

    assert!(
        entry_body
            .contains("collect_overridden_same_property_declaration_replacements_from_ir(ir)")
    );
    assert!(entry_body.contains("collect_duplicate_ordinary_rule_replacements_from_ir(ir)"));
    assert!(
        !entry_body
            .contains("collect_overridden_same_property_declaration_replacements(ir.source_text()")
    );
    assert!(!entry_body.contains("collect_duplicate_ordinary_rule_replacements(ir.source_text()"));
    assert!(!ir_collector_body.contains("collect_declaration_ordinary_rule_slices("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn rule_merge_structural_ir_paths_use_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("rule_merge.rs"),
    )
    .map_err(|err| format!("rule merge source should be readable: {err:?}"))?;
    let selector_entry_anchor = source
        .find("pub(crate) fn merge_adjacent_same_block_css_selectors_with_ir_transaction_on_ir")
        .ok_or_else(|| "selector merge IR entrypoint should exist".to_string())?;
    let selector_legacy_anchor = source[selector_entry_anchor..]
        .find("fn collect_adjacent_same_block_selector_replacements(")
        .ok_or_else(|| "legacy selector merge collector should delimit entrypoint".to_string())?;
    let selector_entry_body =
        &source[selector_entry_anchor..selector_entry_anchor + selector_legacy_anchor];
    let rule_entry_anchor = source
        .find("pub(crate) fn merge_adjacent_same_selector_css_rules_with_ir_transaction_on_ir")
        .ok_or_else(|| "rule merge IR entrypoint should exist".to_string())?;
    let rule_legacy_anchor = source[rule_entry_anchor..]
        .find("fn merge_adjacent_same_selector_ordinary_css_rules_with_lexer")
        .ok_or_else(|| "legacy rule merge section should delimit entrypoint".to_string())?;
    let rule_entry_body = &source[rule_entry_anchor..rule_entry_anchor + rule_legacy_anchor];
    let selector_ir_anchor = source
        .find("fn collect_adjacent_same_block_selector_replacements_from_ir")
        .ok_or_else(|| "selector merge IR collector should exist".to_string())?;
    let selector_ir_end = source[selector_ir_anchor..]
        .find("fn normalized_same_block_merge_value")
        .ok_or_else(|| "selector merge normalization should delimit IR collector".to_string())?;
    let selector_ir_body = &source[selector_ir_anchor..selector_ir_anchor + selector_ir_end];
    let rule_ir_anchor = source
        .find("fn collect_adjacent_same_selector_ordinary_rule_replacements_from_ir")
        .ok_or_else(|| "ordinary rule merge IR collector should exist".to_string())?;
    let rule_ir_end = source[rule_ir_anchor..]
        .find("fn join_rule_blocks_for_merge")
        .ok_or_else(|| "rule merge join helper should delimit IR collector".to_string())?;
    let rule_ir_body = &source[rule_ir_anchor..rule_ir_anchor + rule_ir_end];
    let at_rule_ir_anchor = source
        .find("fn collect_adjacent_same_conditional_at_rule_block_replacements_from_ir")
        .ok_or_else(|| "conditional at-rule merge IR collector should exist".to_string())?;
    let at_rule_ir_end = source[at_rule_ir_anchor..]
        .find("fn source_replacement_ranges")
        .ok_or_else(|| {
            "source replacement helper should delimit conditional IR collector".to_string()
        })?;
    let at_rule_ir_body = &source[at_rule_ir_anchor..at_rule_ir_anchor + at_rule_ir_end];

    assert!(
        selector_entry_body
            .contains("collect_adjacent_same_block_selector_replacements_from_ir(ir)")
    );
    assert!(
        rule_entry_body
            .contains("collect_adjacent_same_selector_ordinary_rule_replacements_from_ir(ir)")
    );
    assert!(
        rule_entry_body
            .contains("collect_adjacent_same_conditional_at_rule_block_replacements_from_ir(ir)")
    );
    assert!(
        !selector_entry_body
            .contains("collect_adjacent_same_block_selector_replacements(ir.source_text()")
    );
    assert!(
        !rule_entry_body
            .contains("collect_adjacent_same_selector_ordinary_rule_replacements(ir.source_text()")
    );
    assert!(
        !rule_entry_body.contains(
            "collect_adjacent_same_conditional_at_rule_block_replacements(ir.source_text()"
        )
    );
    assert!(!selector_ir_body.contains("lex("));
    assert!(!rule_ir_body.contains("lex("));
    assert!(!at_rule_ir_body.contains("lex("));
    Ok(())
}

#[test]
fn design_token_routing_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("design_token.rs"),
    )
    .map_err(|err| format!("design token source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn route_design_token_values_with_ir_transaction_on_ir")
        .ok_or_else(|| "design token routing IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn collect_design_token_route_replacements(")
        .ok_or_else(|| "legacy design token collector should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_design_token_route_replacements_from_ir")
        .ok_or_else(|| "design token IR collector should exist".to_string())?;
    let ir_collector_end = source[ir_collector_anchor..]
        .find("fn at_rule_prelude_can_route_design_tokens")
        .ok_or_else(|| {
            "design token value routing helpers should delimit IR collector".to_string()
        })?;
    let ir_collector_body = &source[ir_collector_anchor..ir_collector_anchor + ir_collector_end];

    assert!(entry_body.contains("collect_design_token_route_replacements_from_ir(ir"));
    assert!(!entry_body.contains("collect_design_token_route_replacements(ir.source_text()"));
    assert!(!ir_collector_body.contains("collect_design_token_route_replacements("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn import_inline_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("import_inline.rs"),
    )
    .map_err(|err| format!("import inline source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn inline_css_imports_with_ir_transaction_on_ir")
        .ok_or_else(|| "import inline IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn inline_css_imports_with_lexer_mode")
        .ok_or_else(|| "legacy import inline collector should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_inline_css_import_replacements_from_ir")
        .ok_or_else(|| "import inline IR collector should exist".to_string())?;
    let ir_collector_end = source[ir_collector_anchor..]
        .find("fn import_inline_deletion_node_ids")
        .ok_or_else(|| "import inline deletion section should delimit IR collector".to_string())?;
    let ir_collector_body = &source[ir_collector_anchor..ir_collector_anchor + ir_collector_end];

    assert!(entry_body.contains("collect_inline_css_import_replacements_from_ir(ir"));
    assert!(!entry_body.contains("collect_inline_css_import_replacements(ir.source_text()"));
    assert!(!entry_body.contains("materialize_transform_ir_printed_source"));
    assert!(!entry_body.contains("lower_transform_ir_from_source("));
    assert!(!ir_collector_body.contains("collect_inline_css_import_replacements("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn public_import_inline_wrapper_routes_through_ir_transaction() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("registry.rs"),
    )
    .map_err(|err| format!("registry source should be readable: {err:?}"))?;
    let wrapper_anchor = source
        .find("pub fn inline_css_imports(")
        .ok_or_else(|| "public import-inline wrapper should exist".to_string())?;
    let next_entry_anchor = source[wrapper_anchor..]
        .find("pub(crate) fn inline_css_imports_in_ir")
        .ok_or_else(|| "IR import-inline entrypoint should delimit public wrapper".to_string())?;
    let wrapper_body = &source[wrapper_anchor..wrapper_anchor + next_entry_anchor];

    assert!(wrapper_body.contains("inline_css_imports_with_ir_transaction("));
    assert!(!wrapper_body.contains("inline_css_imports_with_lexer("));
    Ok(())
}

#[test]
fn class_tree_shake_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("css_modules_classes.rs"),
    )
    .map_err(|err| format!("css modules classes source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn tree_shake_css_class_rules_with_ir_transaction_on_ir")
        .ok_or_else(|| "class tree-shake IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn collect_tree_shake_css_class_rule_replacements(")
        .ok_or_else(|| "legacy class tree-shake collector should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_tree_shake_css_class_rule_replacements_from_ir")
        .ok_or_else(|| "class tree-shake IR collector should exist".to_string())?;
    let ir_collector_end = source[ir_collector_anchor..]
        .find("fn non_overlapping_class_rule_replacements")
        .ok_or_else(|| {
            "class tree-shake replacement helper should delimit IR collector".to_string()
        })?;
    let ir_collector_body = &source[ir_collector_anchor..ir_collector_anchor + ir_collector_end];

    assert!(entry_body.contains("collect_tree_shake_css_class_rule_replacements_from_ir("));
    assert!(
        !entry_body.contains("collect_tree_shake_css_class_rule_replacements(ir.source_text()")
    );
    assert!(!ir_collector_body.contains("collect_tree_shake_css_class_rule_replacements("));
    assert!(!ir_collector_body.contains("collect_declaration_ordinary_rule_slices("));
    assert!(!ir_collector_body.contains("collect_css_module_scope_blocks("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn composes_resolution_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("css_modules_classes.rs"),
    )
    .map_err(|err| format!("css modules classes source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn strip_resolved_css_module_composes_with_ir_transaction_on_ir")
        .ok_or_else(|| "composes-resolution IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn collect_resolved_css_module_composes_replacements(")
        .ok_or_else(|| {
            "legacy composes-resolution collector should delimit entrypoint".to_string()
        })?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_resolved_css_module_composes_replacements_from_ir")
        .ok_or_else(|| "composes-resolution IR collector should exist".to_string())?;
    let ir_collector_end = source[ir_collector_anchor..]
        .find("fn composable_declaration_node_ids")
        .ok_or_else(|| {
            "composes-resolution deletion helper should delimit IR collector".to_string()
        })?;
    let ir_collector_body = &source[ir_collector_anchor..ir_collector_anchor + ir_collector_end];

    assert!(entry_body.contains("collect_resolved_css_module_composes_replacements_from_ir("));
    assert!(
        !entry_body.contains("collect_resolved_css_module_composes_replacements(ir.source_text()")
    );
    assert!(!ir_collector_body.contains("collect_resolved_css_module_composes_replacements("));
    assert!(!ir_collector_body.contains("collect_declaration_ordinary_rule_slices("));
    assert!(!ir_collector_body.contains("collect_css_module_scope_blocks("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn composes_resolution_structural_precompute_uses_ir_collectors() -> Result<(), String> {
    let registry_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("registry.rs"),
    )
    .map_err(|err| format!("registry source should be readable: {err:?}"))?;
    let resolution_anchor = registry_source
        .find("pub(crate) fn css_module_composes_resolutions_for_ir")
        .ok_or_else(|| "composes-resolution IR precompute wrapper should exist".to_string())?;
    let routing_anchor = registry_source[resolution_anchor..]
        .find("pub(crate) fn route_design_token_values_in_ir")
        .ok_or_else(|| {
            "design-token routing entrypoint should delimit precompute wrapper".to_string()
        })?;
    let resolution_body = &registry_source[resolution_anchor..resolution_anchor + routing_anchor];
    assert!(resolution_body.contains("local_css_module_composes_resolutions_from_ir(ir)"));
    assert!(!resolution_body.contains("css_module_composes_resolutions_for_source("));
    assert!(!resolution_body.contains("local_css_module_composes_resolutions_with_lexer("));
    assert!(!resolution_body.contains("ir.source_text()"));

    let domain_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("css_modules_classes.rs"),
    )
    .map_err(|err| format!("css modules classes source should be readable: {err:?}"))?;
    let edge_anchor = domain_source
        .find("fn collect_local_css_module_composes_edges_from_ir")
        .ok_or_else(|| "local composes IR edge collector should exist".to_string())?;
    let next_section_anchor = domain_source[edge_anchor..]
        .find("fn rewritten_class_name_for")
        .ok_or_else(|| {
            "class name rewrite helper should delimit local composes IR collector".to_string()
        })?;
    let edge_body = &domain_source[edge_anchor..edge_anchor + next_section_anchor];

    assert!(edge_body.contains("collect_declaration_ordinary_rule_slices_from_ir(ir)"));
    assert!(edge_body.contains("collect_css_module_scope_blocks_from_ir(ir)"));
    assert!(edge_body.contains("collect_simple_declarations_from_ir(ir, rule)"));
    assert!(!edge_body.contains("collect_local_css_module_composes_edges("));
    assert!(!edge_body.contains("collect_declaration_ordinary_rule_slices(source"));
    assert!(!edge_body.contains("collect_simple_declarations_in_block("));
    assert!(!edge_body.contains("lex("));
    Ok(())
}

#[test]
fn class_hashing_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("css_modules_classes.rs"),
    )
    .map_err(|err| format!("css modules classes source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn rewrite_css_module_class_names_with_ir_transaction_on_ir")
        .ok_or_else(|| "class-hashing IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn collect_css_module_class_name_rewrite_replacements(")
        .ok_or_else(|| "legacy class-hashing collector should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_css_module_class_name_rewrite_replacements_from_ir")
        .ok_or_else(|| "class-hashing IR collector should exist".to_string())?;
    let ir_collector_end = source[ir_collector_anchor..]
        .find("pub(crate) fn local_css_module_composes_resolutions_with_lexer")
        .ok_or_else(|| "composes-resolution helper should delimit IR collector".to_string())?;
    let ir_collector_body = &source[ir_collector_anchor..ir_collector_anchor + ir_collector_end];

    assert!(entry_body.contains("collect_css_module_class_name_rewrite_replacements_from_ir("));
    assert!(
        !entry_body.contains("collect_css_module_class_name_rewrite_replacements(ir.source_text()")
    );
    assert!(!ir_collector_body.contains("collect_css_module_class_name_rewrite_replacements("));
    assert!(!ir_collector_body.contains("collect_declaration_ordinary_rule_slices("));
    assert!(!ir_collector_body.contains("collect_css_module_scope_blocks("));
    assert!(!ir_collector_body.contains("collect_simple_declarations_in_block("));
    assert!(!ir_collector_body.contains("lex("));
    Ok(())
}

#[test]
fn keyframes_tree_shake_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("keyframes.rs"),
    )
    .map_err(|err| format!("keyframes source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn tree_shake_css_keyframes_with_ir_transaction_on_ir")
        .ok_or_else(|| "keyframes IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("fn collect_tree_shake_css_keyframe_removals(")
        .ok_or_else(|| "legacy keyframes collector should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_tree_shake_css_keyframe_removals_from_ir")
        .ok_or_else(|| "keyframes IR collector should exist".to_string())?;
    let ir_collector_end = source[ir_collector_anchor..]
        .find("fn keyframe_removal_replacements")
        .ok_or_else(|| "keyframes replacement helper should delimit IR collector".to_string())?;
    let ir_collector_body = &source[ir_collector_anchor..ir_collector_anchor + ir_collector_end];
    let referenced_ir_anchor = source
        .find("fn collect_referenced_keyframe_names_from_ir")
        .ok_or_else(|| "keyframes referenced-name IR collector should exist".to_string())?;
    let referenced_ir_end = source[referenced_ir_anchor..]
        .find("pub(crate) fn keyframe_name_is_reachable")
        .ok_or_else(|| {
            "keyframes name reachability helper should delimit IR collector".to_string()
        })?;
    let referenced_ir_body =
        &source[referenced_ir_anchor..referenced_ir_anchor + referenced_ir_end];

    assert!(entry_body.contains("collect_tree_shake_css_keyframe_removals_from_ir("));
    assert!(!entry_body.contains("collect_tree_shake_css_keyframe_removals(ir.source_text()"));
    assert!(!ir_collector_body.contains("collect_tree_shake_css_keyframe_removals("));
    assert!(!ir_collector_body.contains("lex("));
    assert!(!referenced_ir_body.contains("collect_referenced_keyframe_names("));
    assert!(!referenced_ir_body.contains("collect_declaration_ordinary_rule_slices("));
    assert!(!referenced_ir_body.contains("lex("));
    Ok(())
}

#[test]
fn css_modules_value_tree_shake_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("css_modules_values.rs"),
    )
    .map_err(|err| format!("css modules values source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn tree_shake_css_modules_values_with_ir_transaction_on_ir")
        .ok_or_else(|| "CSS Modules value tree-shake IR entrypoint should exist".to_string())?;
    let ir_collector_anchor = source[entry_anchor..]
        .find("fn collect_tree_shake_css_modules_value_replacements_from_ir")
        .ok_or_else(|| "CSS Modules value tree-shake IR collector should exist".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + ir_collector_anchor];
    let ir_collector_anchor = entry_anchor + ir_collector_anchor;
    let legacy_collector_anchor = source[ir_collector_anchor..]
        .find("fn collect_tree_shake_css_modules_value_replacements(")
        .ok_or_else(|| {
            "legacy CSS Modules value collector should delimit IR collector".to_string()
        })?;
    let ir_collector_body =
        &source[ir_collector_anchor..ir_collector_anchor + legacy_collector_anchor];

    assert!(entry_body.contains("collect_tree_shake_css_modules_value_replacements_from_ir("));
    assert!(
        !entry_body.contains("collect_tree_shake_css_modules_value_replacements(ir.source_text()")
    );
    assert!(!ir_collector_body.contains("collect_tree_shake_css_modules_value_replacements("));
    assert!(!ir_collector_body.contains("collect_static_css_modules_icss_export_rules("));
    assert!(!ir_collector_body.contains("collect_declaration_ordinary_rule_slices("));
    assert!(ir_collector_body.contains("collect_declaration_ordinary_rule_slices_from_ir("));
    assert!(ir_collector_body.contains("collect_static_css_modules_icss_export_rules_from_ir("));
    Ok(())
}

#[test]
fn custom_property_tree_shake_structural_ir_path_uses_ir_node_collectors() -> Result<(), String> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("domains")
            .join("custom_property.rs"),
    )
    .map_err(|err| format!("custom property source should be readable: {err:?}"))?;
    let entry_anchor = source
        .find("pub(crate) fn tree_shake_css_custom_properties_with_ir_transaction_on_ir")
        .ok_or_else(|| "custom-property tree-shake IR entrypoint should exist".to_string())?;
    let legacy_collector_anchor = source[entry_anchor..]
        .find("\nfn collect_tree_shake_css_custom_property_replacements(")
        .ok_or_else(|| "legacy custom-property collector should delimit entrypoint".to_string())?;
    let entry_body = &source[entry_anchor..entry_anchor + legacy_collector_anchor];
    let ir_collector_anchor = source
        .find("fn collect_tree_shake_css_custom_property_replacements_from_ir")
        .ok_or_else(|| "custom-property tree-shake IR collector should exist".to_string())?;
    let removal_helper_anchor = source[ir_collector_anchor..]
        .find("fn push_custom_property_rule_removals_from_declarations")
        .ok_or_else(|| "custom-property removal helper should delimit IR collector".to_string())?;
    let ir_collector_body =
        &source[ir_collector_anchor..ir_collector_anchor + removal_helper_anchor];

    assert!(entry_body.contains("collect_tree_shake_css_custom_property_replacements_from_ir("));
    assert!(
        !entry_body
            .contains("collect_tree_shake_css_custom_property_replacements(ir.source_text()")
    );
    assert!(!ir_collector_body.contains("collect_tree_shake_css_custom_property_replacements("));
    assert!(!ir_collector_body.contains("collect_static_custom_property_icss_export_rules("));
    assert!(!ir_collector_body.contains("collect_declaration_ordinary_rule_slices("));
    assert!(
        ir_collector_body.contains("collect_static_custom_property_icss_export_rules_from_ir(")
    );
    assert!(ir_collector_body.contains("collect_declaration_ordinary_rule_slices_from_ir("));
    assert!(ir_collector_body.contains("collect_keyframe_declaration_rule_slices_from_ir("));
    Ok(())
}

#[test]
fn planner_uses_descriptor_order_without_pass_ordinals() -> Result<(), String> {
    let planner_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("planner.rs"),
    )
    .map_err(|err| format!("planner source should be readable: {err:?}"))?;

    assert!(
        !planner_source.contains(".ordinal()"),
        "execution planner must not use TransformPassKind::ordinal as an ordering tiebreak"
    );
    assert!(planner_source.contains("descriptor.phase"));
    assert!(planner_source.contains("descriptor.phase_order"));
    assert!(planner_source.contains("descriptor.depends_on"));

    let plan = plan_transform_passes(&[
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::PrintCss,
        TransformPassKind::CalcReduction,
        TransformPassKind::StaticVarSubstitution,
    ]);
    assert_eq!(
        plan.build_profile.pass_ids,
        vec![
            "custom-property-static-resolve",
            "calc-reduction",
            "whitespace-strip",
            "print-css"
        ]
    );
    assert_eq!(plan.build_profile.pass_ids, plan.ordered_pass_ids);
    Ok(())
}

#[test]
fn contract_execution_phases_preserve_near_full_catalog_ordering() -> Result<(), String> {
    let mut requested = default_transform_pass_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.kind != TransformPassKind::ColorFunctionLowering)
        .map(|descriptor| descriptor.kind)
        .collect::<Vec<_>>();
    assert_eq!(requested.len(), TRANSFORM_PASS_CATALOG_LEN - 1);
    requested.reverse();
    let plan = plan_transform_passes_checked(requested.as_slice()).map_err(|conflict| {
        format!("near-full catalog unexpectedly contains an unordered conflict: {conflict:?}")
    })?;

    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "import-inline",
            "scss-module-evaluate",
            "less-module-evaluate",
            "composes-resolution",
            "value-resolution",
            "custom-property-static-resolve",
            "tree-shake-class",
            "tree-shake-keyframes",
            "tree-shake-value",
            "tree-shake-custom-property",
            "dead-media-branch-removal",
            "dead-supports-branch-removal",
            "design-token-routing",
            "light-dark-lowering",
            "color-mix-lowering",
            "oklch-oklab-lowering",
            "logical-to-physical",
            "nesting-unwrap",
            "css-modules-class-hashing",
            "scope-flatten",
            "layer-flatten",
            "supports-static-eval",
            "media-static-eval",
            "relative-color-lowering",
            "container-static-eval",
            "native-css-static-eval",
            "vendor-prefixing",
            "stale-prefix-removal",
            "selector-is-where-compression",
            "shorthand-combining",
            "rule-deduplication",
            "rule-merging",
            "calc-reduction",
            "comment-strip",
            "empty-rule-removal",
            "number-compression",
            "unit-normalization",
            "color-compression",
            "url-quote-strip",
            "string-quote-normalize",
            "selector-merging",
            "whitespace-strip",
            "print-css",
        ]
    );
    Ok(())
}

#[test]
fn mutation_contracts_have_executor_outcomes() {
    let context = TransformExecutionContextV0::default();
    let source = ".used { color: #ffffff; margin: 0px; user-select: none; }";

    for contract in default_transform_pass_contracts()
        .into_iter()
        .filter(|contract| contract.executes_mutation)
    {
        let summary = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[contract.kind],
            &context,
        );
        let outcome = summary
            .outcomes
            .iter()
            .find(|outcome| outcome.pass_id == contract.id);

        assert!(
            outcome.is_some(),
            "missing executor outcome for {}",
            contract.id
        );
        assert_ne!(
            outcome.map(|outcome| outcome.detail),
            Some("unknown pass id in execution plan"),
            "executor fell through to unknown-pass outcome for {}",
            contract.id
        );
    }
}

#[test]
fn incremental_transform_graph_tracks_source_context_plan_and_pass_dependencies() {
    let context = TransformExecutionContextV0 {
        reachable_class_names: vec!["used".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let graph = transform_pass_incremental_graph_input(
        ".used { color: red; }",
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::PrintCss,
        ],
        &context,
        IncrementalRevisionV0 { value: 1 },
    );

    assert_eq!(graph.revision.value, 1);
    assert!(graph.nodes.iter().any(|node| node.id == "transform:source"));
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.id == "transform:context")
    );
    assert!(graph.nodes.iter().any(|node| node.id == "transform:plan"));
    let execution_node = graph
        .nodes
        .iter()
        .find(|node| node.id == "transform:execution");
    assert!(execution_node.is_some());
    if let Some(execution_node) = execution_node {
        assert!(
            execution_node
                .dependency_ids
                .contains(&"transform:pass:print-css".to_string())
        );
    }
}

#[test]
fn incremental_transform_execution_reuses_clean_salsa_database_plan() {
    let mut incremental_database = OmenaIncrementalDatabaseV0::default();
    let context = TransformExecutionContextV0::default();
    let requested = [TransformPassKind::CommentStrip, TransformPassKind::PrintCss];
    let first = execute_transform_passes_incremental_with_database(
        ".button { /* keep no comment */ color: red; }",
        StyleDialect::Css,
        &requested,
        &context,
        &mut incremental_database,
        None,
        IncrementalRevisionV0 { value: 1 },
    );

    assert_eq!(
        first.product,
        "omena-transform-passes.incremental-execution"
    );
    assert_eq!(first.incremental_engine, "omena-incremental");
    assert!(!first.reused_previous_execution);
    assert!(first.incremental_plan.dirty_node_count > 0);
    assert!(first.ready_surfaces.contains(&"transformSalsaQueries"));

    let reused = execute_transform_passes_incremental_with_database(
        ".button { /* keep no comment */ color: red; }",
        StyleDialect::Css,
        &requested,
        &context,
        &mut incremental_database,
        Some(&first.execution),
        IncrementalRevisionV0 { value: 2 },
    );

    assert!(reused.reused_previous_execution);
    assert_eq!(reused.incremental_plan.dirty_node_count, 0);
    assert_eq!(reused.execution.output_css, first.execution.output_css);

    let changed = execute_transform_passes_incremental_with_database(
        ".button { /* changed */ color: blue; }",
        StyleDialect::Css,
        &requested,
        &context,
        &mut incremental_database,
        Some(&reused.execution),
        IncrementalRevisionV0 { value: 3 },
    );

    assert!(!changed.reused_previous_execution);
    assert!(changed.incremental_plan.changed_input_count >= 1);
    assert!(changed.execution.output_css.contains("blue"));
}

#[test]
fn fuzz_seed_corpus_preserves_transform_cascade_safe_invariants() {
    let report = run_transform_fuzz_seed_corpus();

    assert_eq!(report.product, "omena-transform-passes.fuzz-seed-corpus");
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.passed_count, report.case_count);
    assert!(
        report
            .results
            .iter()
            .all(|result| result.output_error_count == 0)
    );
    assert!(
        report
            .results
            .iter()
            .any(|result| !result.executed_pass_ids.is_empty())
    );
}

#[test]
fn planner_respects_var_before_calc_before_print_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::PrintCss,
        TransformPassKind::CalcReduction,
        TransformPassKind::StaticVarSubstitution,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert!(plan.all_requested_registered);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "custom-property-static-resolve",
            "calc-reduction",
            "print-css"
        ]
    );
}

#[test]
fn planner_respects_value_and_var_resolution_before_static_branch_evaluation() {
    let plan = plan_transform_passes(&[
        TransformPassKind::MediaStaticEval,
        TransformPassKind::SupportsStaticEval,
        TransformPassKind::StaticVarSubstitution,
        TransformPassKind::ValueResolution,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert!(plan.all_requested_registered);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "value-resolution",
            "custom-property-static-resolve",
            "supports-static-eval",
            "media-static-eval",
            "print-css"
        ]
    );
}

#[test]
fn planner_respects_composes_before_hash_before_selector_merge_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::SelectorMerging,
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::ResolveCssModulesComposes,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "composes-resolution",
            "css-modules-class-hashing",
            "selector-merging"
        ]
    );
}

#[test]
fn planner_respects_block_canonicalization_before_selector_merge_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::SelectorMerging,
        TransformPassKind::ColorCompression,
        TransformPassKind::UnitNormalization,
        TransformPassKind::CalcReduction,
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "calc-reduction",
            "unit-normalization",
            "color-compression",
            "selector-merging",
            "whitespace-strip",
            "print-css"
        ]
    );
}

#[test]
fn planner_respects_nesting_before_hash_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::NestingUnwrap,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn planner_respects_class_tree_shake_before_hash_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::TreeShakeClass,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec!["tree-shake-class", "css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn planner_respects_comment_strip_before_empty_rule_removal_edge() {
    let plan = plan_transform_passes(&[
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::CommentStrip,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec!["comment-strip", "empty-rule-removal", "print-css"]
    );
}

#[test]
fn planner_respects_minify_structural_ordering_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::PrintCss,
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::SelectorMerging,
        TransformPassKind::RuleMerging,
        TransformPassKind::RuleDeduplication,
        TransformPassKind::ShorthandCombining,
        TransformPassKind::ColorCompression,
        TransformPassKind::NumberCompression,
        TransformPassKind::CommentStrip,
        TransformPassKind::CalcReduction,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_before(
        &plan.ordered_pass_ids,
        "shorthand-combining",
        "rule-merging",
    );
    assert_before(
        &plan.ordered_pass_ids,
        "shorthand-combining",
        "selector-merging",
    );
    assert_before(
        &plan.ordered_pass_ids,
        "comment-strip",
        "empty-rule-removal",
    );
    assert_before(
        &plan.ordered_pass_ids,
        "selector-merging",
        "whitespace-strip",
    );
    assert_eq!(plan.ordered_pass_ids.last(), Some(&"print-css"));
}

#[test]
fn planner_respects_semantic_tree_shaking_before_empty_rule_removal_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::TreeShakeCustomProperty,
        TransformPassKind::TreeShakeValue,
        TransformPassKind::TreeShakeKeyframes,
        TransformPassKind::TreeShakeClass,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "tree-shake-class",
            "tree-shake-keyframes",
            "tree-shake-value",
            "tree-shake-custom-property",
            "empty-rule-removal",
            "print-css"
        ]
    );
}

fn assert_before(pass_ids: &[&'static str], before: &'static str, after: &'static str) {
    let before_index = pass_ids.iter().position(|pass_id| *pass_id == before);
    let after_index = pass_ids.iter().position(|pass_id| *pass_id == after);
    assert!(
        before_index
            .zip(after_index)
            .is_some_and(|(before_index, after_index)| before_index < after_index),
        "expected {before} before {after}, got {pass_ids:?}"
    );
}
