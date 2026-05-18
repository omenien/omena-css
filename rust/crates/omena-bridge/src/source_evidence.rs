use engine_input_producers::EngineInputV2;
pub use omena_semantic::{
    BindingOriginEvidenceV0, CertaintyReasonEvidenceV0, ReferenceSiteIdentityEvidenceV0,
    SourceInputPromotionEvidenceSummaryV0, StyleModuleEdgeEvidenceV0,
    ValueDomainExplanationEvidenceV0,
};

pub fn summarize_omena_bridge_source_input_evidence(
    input: &EngineInputV2,
) -> SourceInputPromotionEvidenceSummaryV0 {
    omena_semantic::summarize_source_input_evidence(input)
}
