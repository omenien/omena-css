use std::collections::BTreeSet;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticFrameFootprintV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub diagnostic_code: String,
    pub diagnostic_instance_id: String,
    pub evidence_module_ids: Vec<String>,
    pub resolver_evidence: Vec<ResolverEvidenceV0>,
    pub cascade_evidence: Vec<CascadeEvidenceV0>,
    pub custom_property_evidence: Vec<CustomPropertyEvidenceV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_conjunction_witness: Option<OutcomeConjunctionWitnessV0>,
    pub conservative: bool,
    pub layer_marker: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeEvidenceV0 {
    pub selector: String,
    pub property: String,
    pub declaration_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomPropertyEvidenceV0 {
    pub custom_property_name: String,
    pub dependency_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverEvidenceV0 {
    pub specifier: String,
    pub resolved_module_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutcomeConjunctionWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub partition_id: String,
    pub outcome_key_count: usize,
    pub conservative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleFootprintV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub module_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecheckSelectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selected_diagnostic_instance_ids: Vec<String>,
    pub skipped_diagnostic_instance_ids: Vec<String>,
    pub conservative: bool,
}

pub fn derive_frame_for_diagnostic(
    diagnostic_code: impl Into<String>,
    diagnostic_instance_id: impl Into<String>,
    evidence_module_ids: Vec<String>,
) -> DiagnosticFrameFootprintV0 {
    let evidence_module_ids = canonicalize_module_ids(evidence_module_ids);
    DiagnosticFrameFootprintV0 {
        schema_version: "0",
        product: "omena-cascade.diagnostic-frame-footprint",
        diagnostic_code: diagnostic_code.into(),
        diagnostic_instance_id: diagnostic_instance_id.into(),
        resolver_evidence: evidence_module_ids
            .iter()
            .map(|module_id| ResolverEvidenceV0 {
                specifier: module_id.clone(),
                resolved_module_id: module_id.clone(),
            })
            .collect(),
        cascade_evidence: Vec::new(),
        custom_property_evidence: Vec::new(),
        outcome_conjunction_witness: Some(outcome_conjunction_witness(&evidence_module_ids)),
        evidence_module_ids,
        conservative: true,
        layer_marker: "frame-rule",
    }
}

pub fn derive_frames_for_diagnostic_set(
    diagnostics: Vec<(String, String, Vec<String>)>,
) -> Vec<DiagnosticFrameFootprintV0> {
    diagnostics
        .into_iter()
        .map(|(code, instance_id, module_ids)| {
            derive_frame_for_diagnostic(code, instance_id, module_ids)
        })
        .collect()
}

pub fn compute_edit_footprint(module_ids: Vec<String>) -> ModuleFootprintV0 {
    ModuleFootprintV0 {
        schema_version: "0",
        product: "omena-cascade.module-footprint",
        module_ids: canonicalize_module_ids(module_ids),
    }
}

pub fn select_recheck_set(
    frames: &[DiagnosticFrameFootprintV0],
    edit_footprint: &ModuleFootprintV0,
) -> RecheckSelectionV0 {
    let edit_modules = edit_footprint
        .module_ids
        .iter()
        .collect::<BTreeSet<&String>>();
    let mut selected = Vec::new();
    let mut skipped = Vec::new();

    for frame in frames {
        if frame
            .evidence_module_ids
            .iter()
            .any(|module_id| edit_modules.contains(module_id))
        {
            selected.push(frame.diagnostic_instance_id.clone());
        } else {
            skipped.push(frame.diagnostic_instance_id.clone());
        }
    }

    RecheckSelectionV0 {
        schema_version: "0",
        product: "omena-cascade.recheck-selection",
        selected_diagnostic_instance_ids: selected,
        skipped_diagnostic_instance_ids: skipped,
        conservative: true,
    }
}

pub fn intersect_frame_with_footprint(
    frame: &DiagnosticFrameFootprintV0,
    footprint: &ModuleFootprintV0,
) -> bool {
    let module_ids = footprint.module_ids.iter().collect::<BTreeSet<&String>>();
    frame
        .evidence_module_ids
        .iter()
        .any(|module_id| module_ids.contains(module_id))
}

pub fn outcome_conjunction_witness(module_ids: &[String]) -> OutcomeConjunctionWitnessV0 {
    OutcomeConjunctionWitnessV0 {
        schema_version: "0",
        product: "omena-cascade.outcome-conjunction-witness",
        partition_id: module_ids.join("+"),
        outcome_key_count: module_ids.len(),
        conservative: true,
    }
}

pub fn partition_into_outcome_conjunction_classes(
    frames: &[DiagnosticFrameFootprintV0],
) -> Vec<OutcomeConjunctionWitnessV0> {
    frames
        .iter()
        .filter_map(|frame| frame.outcome_conjunction_witness.clone())
        .collect()
}

fn canonicalize_module_ids(module_ids: Vec<String>) -> Vec<String> {
    module_ids
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_selection_is_sorted_deduped_and_conservative() {
        let frame = derive_frame_for_diagnostic(
            "missing-static-class",
            "d1",
            vec!["b".into(), "a".into(), "a".into()],
        );
        let footprint = compute_edit_footprint(vec!["a".into()]);
        let selection = select_recheck_set(std::slice::from_ref(&frame), &footprint);

        assert_eq!(frame.evidence_module_ids, vec!["a", "b"]);
        assert!(frame.conservative);
        assert!(intersect_frame_with_footprint(&frame, &footprint));
        assert_eq!(selection.selected_diagnostic_instance_ids, vec!["d1"]);
    }
}
