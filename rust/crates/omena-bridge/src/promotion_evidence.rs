use engine_input_producers::EngineInputV2;
use omena_semantic::{ParserBoundarySyntaxFactsV0, StyleSemanticFactsV0};
pub use omena_semantic::{SemanticPromotionEvidenceItemV0, SemanticPromotionEvidenceSummaryV0};

pub fn summarize_omena_bridge_semantic_promotion_evidence(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    semantic_facts: &StyleSemanticFactsV0,
) -> SemanticPromotionEvidenceSummaryV0 {
    omena_semantic::summarize_semantic_promotion_evidence(parser_facts, semantic_facts)
}

pub fn summarize_omena_bridge_promotion_evidence_with_source_input(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    semantic_facts: &StyleSemanticFactsV0,
    input: &EngineInputV2,
) -> SemanticPromotionEvidenceSummaryV0 {
    omena_semantic::summarize_semantic_promotion_evidence_with_source_input(
        parser_facts,
        semantic_facts,
        input,
    )
}
