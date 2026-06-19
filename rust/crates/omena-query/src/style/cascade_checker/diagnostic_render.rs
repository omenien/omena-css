use omena_query_checker_orchestrator::{
    OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeEvaluationV0,
};
use omena_query_core::{
    AbstractPropertyValueCandidateV0, iterate_reduced_class_value_product_constraints,
    narrow_abstract_property_value_for_cascade_branch,
};

use super::super::OmenaQueryCascadeNarrowingEvidenceV0;
use super::runtime_state::{
    query_element_class_signature_constraints, query_selector_class_names,
    summarize_query_runtime_state_for_evaluation,
};

const LSP_DIAGNOSTIC_TAG_UNNECESSARY: u8 = 1;

pub(super) fn query_cascade_checker_code(code: &'static str) -> &'static str {
    match code {
        "unreachable-declaration" => "unreachableDeclaration",
        "dead-cascade-layer" => "deadCascadeLayer",
        "iacvt-prone" => "iacvtProne",
        "circular-var" => "circularVar",
        "registered-property-type-mismatch" => "registeredPropertyTypeMismatch",
        "unspecified-cascade-tie" => "unspecifiedCascadeTie",
        "designer-intent-inconsistency" => "designerIntentInconsistency",
        _ => "cascadeAware",
    }
}

pub(super) fn query_cascade_checker_diagnostic_severity(code: &'static str) -> &'static str {
    match code {
        "unreachable-declaration" | "dead-cascade-layer" | "designer-intent-inconsistency" => {
            "hint"
        }
        _ => "warning",
    }
}

pub(super) fn query_cascade_checker_diagnostic_tags(code: &'static str) -> Vec<u8> {
    match code {
        "unreachable-declaration" | "dead-cascade-layer" => {
            vec![LSP_DIAGNOSTIC_TAG_UNNECESSARY]
        }
        _ => Vec::new(),
    }
}

pub(super) fn summarize_query_cascade_narrowing_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Option<OmenaQueryCascadeNarrowingEvidenceV0> {
    let anchor_id = evaluation.declaration_ids.first()?;
    let anchor = declarations
        .iter()
        .find(|declaration| declaration.declaration_id == *anchor_id)?;
    let site_declarations = declarations
        .iter()
        .filter(|declaration| {
            declaration.selector == anchor.selector
                && declaration.property == anchor.property
                && declaration.condition_context == anchor.condition_context
        })
        .collect::<Vec<_>>();
    if site_declarations.is_empty() {
        return None;
    }

    let property_candidates = site_declarations
        .iter()
        .map(|declaration| AbstractPropertyValueCandidateV0 {
            property_name: declaration.property.clone(),
            value: declaration.value.clone(),
            pseudo_state: None,
            condition_context: declaration.condition_context.clone(),
            layer_name: declaration.layer_name.clone(),
            layer_order: declaration.layer_order,
            source_order: Some(declaration.source_order),
            important: declaration.important,
            same_selector_ordering: true,
        })
        .collect::<Vec<_>>();
    let property_value_narrowing = narrow_abstract_property_value_for_cascade_branch(
        anchor.property.as_str(),
        None,
        anchor.condition_context.as_slice(),
        anchor.layer_name.as_deref(),
        anchor.layer_order,
        true,
        property_candidates.as_slice(),
    );

    let selector_class_names = query_selector_class_names(anchor.selector.as_str());
    let element_class_constraints =
        query_element_class_signature_constraints(selector_class_names.as_slice());
    let element_class_iteration =
        iterate_reduced_class_value_product_constraints(element_class_constraints.as_slice());

    Some(OmenaQueryCascadeNarrowingEvidenceV0 {
        schema_version: "0",
        product: "omena-query.cascade-narrowing-evidence",
        selector: anchor.selector.as_str().to_string(),
        selector_class_names,
        property_name: anchor.property.clone(),
        condition_context: anchor.condition_context.clone(),
        declaration_ids: site_declarations
            .into_iter()
            .map(|declaration| declaration.declaration_id.clone())
            .collect(),
        element_class_iteration,
        property_value_narrowing,
        runtime_state: summarize_query_runtime_state_for_evaluation(evaluation, declarations),
    })
}
