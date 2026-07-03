use std::collections::{BTreeMap, BTreeSet};

use super::shared::*;

pub(super) fn attach_omena_query_module_graph_property_value_narrowing_for_workspace(
    target_style_path: &str,
    summary: &mut OmenaQueryStyleDiagnosticsForFileV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) {
    // Substrate RES-A slot: plain resolution with (package_manifests, bundler, tsconfig).
    let resolution = &substrate.sass_resolution;
    let reachable_style_paths =
        collect_sass_module_graph_reachable_style_paths(target_style_path, resolution);
    if reachable_style_paths.len() <= 1 {
        return;
    }

    let graph_candidates = style_sources
        .iter()
        .filter(|source| reachable_style_paths.contains(source.style_path.as_str()))
        .flat_map(|source| {
            super::super::cascade_checker::collect_query_checker_cascade_declarations(
                source.style_source.as_str(),
            )
            .into_iter()
            .map(|declaration| {
                (
                    declaration.input.selector.as_str().to_string(),
                    AbstractPropertyValueCandidateV0 {
                        property_name: declaration.input.property,
                        value: declaration.input.value,
                        pseudo_state: None,
                        condition_context: declaration.input.condition_context,
                        layer_name: declaration.input.layer_name,
                        layer_order: declaration.input.layer_order,
                        source_order: Some(declaration.input.source_order),
                        important: declaration.input.important,
                        same_selector_ordering: false,
                    },
                )
            })
        })
        .collect::<Vec<_>>();
    if graph_candidates.is_empty() {
        return;
    }

    for diagnostic in &mut summary.diagnostics {
        let Some(cascade_narrowing) = diagnostic.cascade_narrowing.as_mut() else {
            continue;
        };
        let property_value_narrowing = &cascade_narrowing.property_value_narrowing;
        let mut static_reachability_by_context = BTreeMap::new();
        let property_candidates = graph_candidates
            .iter()
            .filter(|(selector, candidate)| {
                selector == &cascade_narrowing.selector
                    && candidate.property_name == cascade_narrowing.property_name
                    && *static_reachability_by_context
                        .entry(candidate.condition_context.clone())
                        .or_insert_with(|| {
                            super::super::cascade_checker::query_condition_context_static_supports_pruning_evidence(
                                candidate.condition_context.as_slice(),
                                Some(
                                    property_value_narrowing
                                        .requested_condition_context
                                        .as_slice(),
                                ),
                            )
                            .is_none_or(|evidence| !evidence.pruned)
                        })
            })
            .map(|(_, candidate)| candidate.clone())
            .collect::<Vec<_>>();
        if property_candidates.is_empty() {
            continue;
        }
        let mut narrowed = narrow_abstract_property_value_for_cascade_branch(
            cascade_narrowing.property_name.as_str(),
            property_value_narrowing.requested_pseudo_state.as_deref(),
            property_value_narrowing
                .requested_condition_context
                .as_slice(),
            property_value_narrowing.requested_layer_name.as_deref(),
            property_value_narrowing.requested_layer_order,
            property_value_narrowing.requested_layer_scope == "exactLayer",
            property_candidates.as_slice(),
        );
        narrowed.stylesheet_scope = "moduleGraph";
        cascade_narrowing.property_value_narrowing = narrowed;
    }
}

pub(super) fn attach_omena_query_runtime_state_inline_overrides_for_workspace(
    target_style_path: &str,
    summary: &mut OmenaQueryStyleDiagnosticsForFileV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) {
    let inline_overrides = collect_omena_query_inline_style_runtime_overrides_for_style(
        target_style_path,
        style_sources,
        source_documents,
        resolution_inputs,
        resolver_identity_index,
    );
    attach_omena_query_runtime_state_inline_overrides_from_overrides(summary, inline_overrides);
}

/// The shared-pass arm: the per-wave bucket map replaces the per-target
/// source sweep; the attach body is shared verbatim below.
pub(in crate::style) fn attach_omena_query_runtime_state_inline_overrides_with_shared(
    target_style_path: &str,
    summary: &mut OmenaQueryStyleDiagnosticsForFileV0,
    overrides_by_style: &BTreeMap<String, Vec<OmenaQueryInlineStyleRuntimeOverrideV0>>,
) {
    let inline_overrides = overrides_by_style
        .get(target_style_path)
        .cloned()
        .unwrap_or_default();
    attach_omena_query_runtime_state_inline_overrides_from_overrides(summary, inline_overrides);
}

fn attach_omena_query_runtime_state_inline_overrides_from_overrides(
    summary: &mut OmenaQueryStyleDiagnosticsForFileV0,
    inline_overrides: Vec<OmenaQueryInlineStyleRuntimeOverrideV0>,
) {
    if inline_overrides.is_empty() {
        return;
    }

    for diagnostic in &mut summary.diagnostics {
        let Some(runtime_state) = diagnostic
            .cascade_narrowing
            .as_mut()
            .and_then(|narrowing| narrowing.runtime_state.as_mut())
        else {
            continue;
        };
        runtime_state.inline_style_overrides = inline_overrides.clone();
        let property_name = runtime_state.property_name.clone();
        runtime_state.scenarios.extend(
            inline_overrides
                .iter()
                .filter(|override_fact| override_fact.property_name == property_name)
                .map(|override_fact| {
                    omena_query_inline_style_runtime_override_scenario(
                        property_name.as_str(),
                        override_fact,
                    )
                }),
        );
        for driver in &mut runtime_state.driver_summaries {
            if driver.driver == "inlineStyleHighestSpecificityTier" {
                driver.status = "sourceFactsJoined";
                driver.scenario_count = runtime_state
                    .inline_style_overrides
                    .iter()
                    .filter(|override_fact| override_fact.property_name == property_name)
                    .count();
            }
        }
        runtime_state.confidence_tier = query_runtime_state_confidence_tier(
            runtime_state.scenarios.as_slice(),
            runtime_state.inline_style_overrides.as_slice(),
        );
    }
}

fn omena_query_inline_style_runtime_override_scenario(
    property_name: &str,
    override_fact: &OmenaQueryInlineStyleRuntimeOverrideV0,
) -> OmenaQueryRuntimeStateScenarioV0 {
    let value = override_fact
        .value
        .clone()
        .unwrap_or_else(|| "<dynamic>".to_string());
    let property_value_narrowing = narrow_abstract_property_value_for_pseudo_state(
        property_name,
        None,
        &[AbstractPropertyValueCandidateV0 {
            property_name: property_name.to_string(),
            value: value.clone(),
            pseudo_state: None,
            condition_context: Vec::new(),
            layer_name: None,
            layer_order: None,
            source_order: Some(0),
            important: true,
            same_selector_ordering: true,
        }],
    );

    OmenaQueryRuntimeStateScenarioV0 {
        scenario_kind: "inlineStyleOverride",
        pseudo_state: None,
        condition_context: Vec::new(),
        declaration_ids: vec![format!(
            "inline-style:{}:{}:{}",
            override_fact.source_path,
            override_fact.range.start.line,
            override_fact.range.start.character
        )],
        winner_declaration_id: Some("inline-style-author-tier".to_string()),
        winner_value: Some(value),
        property_value_narrowing,
    }
}

fn collect_omena_query_inline_style_runtime_overrides_for_style(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> Vec<OmenaQueryInlineStyleRuntimeOverrideV0> {
    collect_omena_query_inline_style_runtime_overrides_by_style(
        style_sources,
        source_documents,
        resolution_inputs,
        resolver_identity_index,
    )
    .remove(target_style_path)
    .unwrap_or_default()
}

/// Target-INDEPENDENT core of the inline-override attach (rfcs#111 C1 slice
/// 2): every source document is parsed, its imports resolved, and its inline
/// style declarations attributed ONCE — bucketed by owning style path — so a
/// wave shares one pass over the sources instead of one per target. The
/// per-target consumer is a map lookup; per-bucket ordering matches the
/// single-target arm exactly (same collection order, same sort, same dedup).
pub(in crate::style) fn collect_omena_query_inline_style_runtime_overrides_by_style(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    resolver_identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> BTreeMap<String, Vec<OmenaQueryInlineStyleRuntimeOverrideV0>> {
    let available_style_paths = style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut overrides_by_style = BTreeMap::<String, Vec<_>>::new();

    for document in source_documents {
        let imports = summarize_omena_query_source_import_declarations_for_source_language(
            document.source_path.as_str(),
            &document.source_source,
            None,
        );
        let mut imported_style_bindings = Vec::new();
        let mut classnames_bind_bindings = Vec::new();
        for import in imports.imports {
            if import.specifier == "classnames/bind" {
                classnames_bind_bindings.push(import.binding);
                continue;
            }
            let Some(style_path) =
                resolve_style_module_source_with_path_mappings_and_identity_index(
                    &document.source_path,
                    &import.specifier,
                    &available_style_paths,
                    resolution_inputs.package_manifests.as_slice(),
                    resolution_inputs.bundler_path_mappings.as_slice(),
                    resolution_inputs.tsconfig_path_mappings.as_slice(),
                    resolution_inputs.disk_style_path_identities.as_slice(),
                    resolver_identity_index,
                )
            else {
                continue;
            };
            imported_style_bindings.push(OmenaQuerySourceImportedStyleBindingV0 {
                binding: import.binding,
                style_uri: style_path,
            });
        }
        if imported_style_bindings.is_empty() {
            continue;
        }

        let index = summarize_omena_query_source_syntax_index_for_source_language(
            document.source_path.as_str(),
            &document.source_source,
            None,
            imported_style_bindings,
            classnames_bind_bindings,
        );
        for declaration in index.inline_style_declarations {
            let Some(target_style_uri) = declaration.target_style_uri.as_deref() else {
                continue;
            };
            overrides_by_style
                .entry(target_style_uri.to_string())
                .or_default()
                .push(OmenaQueryInlineStyleRuntimeOverrideV0 {
                    source_path: document.source_path.clone(),
                    range: parser_range_for_byte_span(
                        &document.source_source,
                        declaration.byte_span,
                    ),
                    property_name: declaration.property_name,
                    value: declaration.value,
                    cascade_tier: declaration.cascade_tier,
                    static_value: declaration.static_value,
                });
        }
    }

    for overrides in overrides_by_style.values_mut() {
        overrides.sort_by(|left, right| {
            left.source_path
                .cmp(&right.source_path)
                .then_with(|| left.range.start.line.cmp(&right.range.start.line))
                .then_with(|| left.range.start.character.cmp(&right.range.start.character))
                .then_with(|| left.property_name.cmp(&right.property_name))
        });
        overrides.dedup();
    }
    overrides_by_style
}
