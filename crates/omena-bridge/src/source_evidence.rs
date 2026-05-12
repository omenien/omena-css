use std::collections::{BTreeMap, BTreeSet};

use engine_input_producers::{
    EngineInputV2, ExpressionSemanticsEvaluatorCandidatePayloadV0,
    summarize_expression_semantics_evaluator_candidates_input,
    summarize_selector_usage_evaluator_candidates_input,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInputPromotionEvidenceSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub reference_site_identity: ReferenceSiteIdentityEvidenceV0,
    pub certainty_reason: CertaintyReasonEvidenceV0,
    pub binding_origin: BindingOriginEvidenceV0,
    pub style_module_edge: StyleModuleEdgeEvidenceV0,
    pub value_domain_explanation: ValueDomainExplanationEvidenceV0,
    pub blocking_gaps: Vec<&'static str>,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceSiteIdentityEvidenceV0 {
    pub status: &'static str,
    pub selector_count: usize,
    pub reference_site_count: usize,
    pub direct_reference_site_count: usize,
    pub expanded_reference_site_count: usize,
    pub style_dependency_reference_site_count: usize,
    pub editable_direct_site_count: usize,
    pub reference_kind_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertaintyReasonEvidenceV0 {
    pub status: &'static str,
    pub expression_count: usize,
    pub exact_count: usize,
    pub inferred_count: usize,
    pub possible_count: usize,
    pub missing_reason_count: usize,
    pub reason_counts: BTreeMap<String, usize>,
    pub shape_kind_counts: BTreeMap<String, usize>,
    pub shape_label_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingOriginEvidenceV0 {
    pub status: &'static str,
    pub expression_count: usize,
    pub direct_class_name_count: usize,
    pub root_binding_count: usize,
    pub access_path_count: usize,
    pub access_path_segment_count: usize,
    pub expression_kind_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleModuleEdgeEvidenceV0 {
    pub status: &'static str,
    pub source_style_edge_count: usize,
    pub distinct_style_module_count: usize,
    pub missing_style_document_edge_count: usize,
    pub composed_edge_count: usize,
    pub imported_composed_edge_count: usize,
    pub global_composed_edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueDomainExplanationEvidenceV0 {
    pub status: &'static str,
    pub expression_count: usize,
    pub exact_expression_count: usize,
    pub finite_value_expression_count: usize,
    pub constrained_expression_count: usize,
    pub unknown_expression_count: usize,
    pub finite_value_count: usize,
    pub derivation_count: usize,
    pub derivation_step_count: usize,
    pub value_domain_kind_counts: BTreeMap<String, usize>,
    pub constraint_kind_counts: BTreeMap<String, usize>,
    pub derivation_product_counts: BTreeMap<String, usize>,
    pub derivation_reduced_kind_counts: BTreeMap<String, usize>,
    pub derivation_operation_counts: BTreeMap<String, usize>,
}

pub fn summarize_omena_bridge_source_input_evidence(
    input: &EngineInputV2,
) -> SourceInputPromotionEvidenceSummaryV0 {
    let reference_site_identity = summarize_reference_site_identity(input);
    let certainty_reason = summarize_certainty_reason(input);
    let binding_origin = summarize_binding_origin(input);
    let style_module_edge = summarize_style_module_edge(input);
    let value_domain_explanation = summarize_value_domain_explanation(input);
    let mut blocking_gaps = Vec::new();

    if reference_site_identity.status == "gap" {
        blocking_gaps.push("referenceSiteIdentity");
    }
    if certainty_reason.status == "gap" {
        blocking_gaps.push("certaintyReason");
    }
    if binding_origin.status == "gap" {
        blocking_gaps.push("bindingOrigin");
    }
    if style_module_edge.status == "gap" {
        blocking_gaps.push("styleModuleEdge");
    }
    if value_domain_explanation.status == "gap" {
        blocking_gaps.push("valueDomainExplanation");
    }

    SourceInputPromotionEvidenceSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.source-input-evidence",
        input_version: input.version.clone(),
        reference_site_identity,
        certainty_reason,
        binding_origin,
        style_module_edge,
        value_domain_explanation,
        blocking_gaps,
        next_priorities: Vec::new(),
    }
}

fn summarize_reference_site_identity(input: &EngineInputV2) -> ReferenceSiteIdentityEvidenceV0 {
    let selector_usage = summarize_selector_usage_evaluator_candidates_input(input);
    let selector_count = selector_usage.results.len();
    let mut reference_site_count = 0usize;
    let mut direct_reference_site_count = 0usize;
    let mut expanded_reference_site_count = 0usize;
    let mut style_dependency_reference_site_count = 0usize;
    let mut editable_direct_site_count = 0usize;
    let mut reference_kind_counts = BTreeMap::new();

    for result in selector_usage.results {
        editable_direct_site_count += result.payload.editable_direct_sites.len();
        for site in result.payload.all_sites {
            reference_site_count += 1;
            if site.expansion == "direct" {
                direct_reference_site_count += 1;
            } else {
                expanded_reference_site_count += 1;
            }
            if site.reference_kind == "styleDependency" {
                style_dependency_reference_site_count += 1;
            }
            *reference_kind_counts
                .entry(site.reference_kind)
                .or_insert(0) += 1;
        }
    }

    ReferenceSiteIdentityEvidenceV0 {
        status: if reference_site_count > 0 {
            "ready"
        } else {
            "gap"
        },
        selector_count,
        reference_site_count,
        direct_reference_site_count,
        expanded_reference_site_count,
        style_dependency_reference_site_count,
        editable_direct_site_count,
        reference_kind_counts,
    }
}

fn summarize_certainty_reason(input: &EngineInputV2) -> CertaintyReasonEvidenceV0 {
    let expression_semantics = summarize_expression_semantics_evaluator_candidates_input(input);
    let mut expression_count = 0usize;
    let mut exact_count = 0usize;
    let mut inferred_count = 0usize;
    let mut possible_count = 0usize;
    let mut missing_reason_count = 0usize;
    let mut reason_counts = BTreeMap::new();
    let mut shape_kind_counts = BTreeMap::new();
    let mut shape_label_counts = BTreeMap::new();

    for result in expression_semantics.results {
        expression_count += 1;
        let payload = result.payload;
        match payload.selector_certainty.as_str() {
            "exact" => exact_count += 1,
            "inferred" => inferred_count += 1,
            "possible" => possible_count += 1,
            _ => {}
        }
        *shape_kind_counts
            .entry(payload.selector_certainty_shape_kind.clone())
            .or_insert(0) += 1;
        *shape_label_counts
            .entry(payload.selector_certainty_shape_label.clone())
            .or_insert(0) += 1;

        if let Some(reason) = selector_certainty_reason(&payload) {
            *reason_counts.entry(reason).or_insert(0) += 1;
        } else {
            missing_reason_count += 1;
        }
    }

    CertaintyReasonEvidenceV0 {
        status: if expression_count == 0 {
            "gap"
        } else if missing_reason_count == 0 {
            "ready"
        } else {
            "partial"
        },
        expression_count,
        exact_count,
        inferred_count,
        possible_count,
        missing_reason_count,
        reason_counts,
        shape_kind_counts,
        shape_label_counts,
    }
}

fn summarize_binding_origin(input: &EngineInputV2) -> BindingOriginEvidenceV0 {
    let mut expression_count = 0usize;
    let mut direct_class_name_count = 0usize;
    let mut root_binding_count = 0usize;
    let mut access_path_count = 0usize;
    let mut access_path_segment_count = 0usize;
    let mut expression_kind_counts = BTreeMap::new();

    for source in &input.sources {
        for expression in &source.document.class_expressions {
            expression_count += 1;
            *expression_kind_counts
                .entry(expression.kind.clone())
                .or_insert(0) += 1;
            if expression.class_name.is_some() {
                direct_class_name_count += 1;
            }
            if expression.root_binding_decl_id.is_some() {
                root_binding_count += 1;
            }
            if let Some(access_path) = &expression.access_path {
                access_path_count += 1;
                access_path_segment_count += access_path.len();
            }
        }
    }

    BindingOriginEvidenceV0 {
        status: if expression_count == 0 {
            "gap"
        } else if direct_class_name_count + root_binding_count + access_path_count > 0 {
            "ready"
        } else {
            "partial"
        },
        expression_count,
        direct_class_name_count,
        root_binding_count,
        access_path_count,
        access_path_segment_count,
        expression_kind_counts,
    }
}

fn summarize_style_module_edge(input: &EngineInputV2) -> StyleModuleEdgeEvidenceV0 {
    let style_paths = input
        .styles
        .iter()
        .map(|style| style.file_path.clone())
        .collect::<BTreeSet<_>>();
    let mut referenced_style_paths = BTreeSet::new();
    let mut source_style_edge_count = 0usize;
    let mut missing_style_document_edge_count = 0usize;
    let mut composed_edge_count = 0usize;
    let mut imported_composed_edge_count = 0usize;
    let mut global_composed_edge_count = 0usize;

    for source in &input.sources {
        for expression in &source.document.class_expressions {
            source_style_edge_count += 1;
            referenced_style_paths.insert(expression.scss_module_path.clone());
            if !style_paths.contains(&expression.scss_module_path) {
                missing_style_document_edge_count += 1;
            }
        }
    }

    for style in &input.styles {
        for selector in &style.document.selectors {
            let Some(composes) = &selector.composes else {
                continue;
            };
            composed_edge_count += composes.len();
            for compose in composes {
                if compose
                    .get("fromGlobal")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false)
                {
                    global_composed_edge_count += 1;
                } else if compose
                    .get("from")
                    .and_then(|value| value.as_str())
                    .is_some()
                {
                    imported_composed_edge_count += 1;
                }
            }
        }
    }

    StyleModuleEdgeEvidenceV0 {
        status: if source_style_edge_count == 0 {
            "gap"
        } else if missing_style_document_edge_count == 0 {
            "ready"
        } else {
            "partial"
        },
        source_style_edge_count,
        distinct_style_module_count: referenced_style_paths.len(),
        missing_style_document_edge_count,
        composed_edge_count,
        imported_composed_edge_count,
        global_composed_edge_count,
    }
}

fn summarize_value_domain_explanation(input: &EngineInputV2) -> ValueDomainExplanationEvidenceV0 {
    let expression_semantics = summarize_expression_semantics_evaluator_candidates_input(input);
    let mut expression_count = 0usize;
    let mut exact_expression_count = 0usize;
    let mut finite_value_expression_count = 0usize;
    let mut constrained_expression_count = 0usize;
    let mut unknown_expression_count = 0usize;
    let mut finite_value_count = 0usize;
    let mut derivation_count = 0usize;
    let mut derivation_step_count = 0usize;
    let mut value_domain_kind_counts = BTreeMap::new();
    let mut constraint_kind_counts = BTreeMap::new();
    let mut derivation_product_counts = BTreeMap::new();
    let mut derivation_reduced_kind_counts = BTreeMap::new();
    let mut derivation_operation_counts = BTreeMap::new();

    for result in expression_semantics.results {
        expression_count += 1;
        let payload = result.payload;
        *value_domain_kind_counts
            .entry(payload.value_domain_kind.clone())
            .or_insert(0) += 1;

        match payload.value_domain_kind.as_str() {
            "exact" => exact_expression_count += 1,
            "finiteSet" => finite_value_expression_count += 1,
            "constrained" => constrained_expression_count += 1,
            "none" | "unknown" | "top" => unknown_expression_count += 1,
            _ => {}
        }

        if let Some(values) = &payload.finite_values {
            finite_value_count += values.len();
        }
        if let Some(kind) = &payload.value_constraint_kind {
            *constraint_kind_counts.entry(kind.clone()).or_insert(0) += 1;
        }

        derivation_count += 1;
        let derivation = payload.value_domain_derivation;
        *derivation_product_counts
            .entry(derivation.product.to_string())
            .or_insert(0) += 1;
        *derivation_reduced_kind_counts
            .entry(derivation.reduced_kind.to_string())
            .or_insert(0) += 1;
        for step in derivation.steps {
            derivation_step_count += 1;
            *derivation_operation_counts
                .entry(step.operation.to_string())
                .or_insert(0) += 1;
        }
    }

    ValueDomainExplanationEvidenceV0 {
        status: if expression_count == 0 {
            "gap"
        } else if exact_expression_count
            + finite_value_expression_count
            + constrained_expression_count
            > 0
            && derivation_count == expression_count
        {
            "ready"
        } else {
            "partial"
        },
        expression_count,
        exact_expression_count,
        finite_value_expression_count,
        constrained_expression_count,
        unknown_expression_count,
        finite_value_count,
        derivation_count,
        derivation_step_count,
        value_domain_kind_counts,
        constraint_kind_counts,
        derivation_product_counts,
        derivation_reduced_kind_counts,
        derivation_operation_counts,
    }
}

fn selector_certainty_reason(
    payload: &ExpressionSemanticsEvaluatorCandidatePayloadV0,
) -> Option<String> {
    match payload.selector_certainty.as_str() {
        "exact" => {
            if payload.selector_names.len() == 1 {
                Some("single selector matched".to_string())
            } else {
                Some("selector set exactly matched the proven value domain".to_string())
            }
        }
        "inferred" => match payload.selector_constraint_kind.as_deref() {
            Some("prefix" | "suffix" | "prefixSuffix" | "charInclusion" | "composite") => {
                Some("constrained runtime shape matched a bounded selector set".to_string())
            }
            _ => Some("finite candidate values matched a bounded selector set".to_string()),
        },
        "possible" => {
            if payload.selector_names.is_empty() {
                Some("no selector could be proven for this value".to_string())
            } else {
                Some("analysis could not prove an exact selector set".to_string())
            }
        }
        _ => None,
    }
}
