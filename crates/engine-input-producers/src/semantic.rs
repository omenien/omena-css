use crate::{
    EngineInputV2, SemanticCanonicalCandidateBundleV0, SemanticCanonicalProducerSignalV0,
    SemanticEvaluatorCandidatesV0, summarize_expression_domain_canonical_candidate_bundle_input,
    summarize_expression_domain_evaluator_candidates_input,
    summarize_source_side_canonical_candidate_bundle_input,
    summarize_source_side_evaluator_candidates_input,
};

pub fn summarize_semantic_canonical_candidate_bundle_input(
    input: &EngineInputV2,
) -> SemanticCanonicalCandidateBundleV0 {
    SemanticCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        source_side: summarize_source_side_canonical_candidate_bundle_input(input),
        expression_domain: summarize_expression_domain_canonical_candidate_bundle_input(input),
    }
}

pub fn summarize_semantic_evaluator_candidates_input(
    input: &EngineInputV2,
) -> SemanticEvaluatorCandidatesV0 {
    SemanticEvaluatorCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        source_side: summarize_source_side_evaluator_candidates_input(input),
        expression_domain: summarize_expression_domain_evaluator_candidates_input(input),
    }
}

pub fn summarize_semantic_canonical_producer_signal_input(
    input: &EngineInputV2,
) -> SemanticCanonicalProducerSignalV0 {
    SemanticCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        canonical_bundle: summarize_semantic_canonical_candidate_bundle_input(input),
        evaluator_candidates: summarize_semantic_evaluator_candidates_input(input),
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic::{
        summarize_semantic_canonical_candidate_bundle_input,
        summarize_semantic_canonical_producer_signal_input,
        summarize_semantic_evaluator_candidates_input,
    };
    use crate::test_support::sample_input;

    #[test]
    fn builds_semantic_canonical_candidate_bundle() {
        let summary = summarize_semantic_canonical_candidate_bundle_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.source_side.expression_semantics.candidates.len(), 2);
        assert_eq!(summary.source_side.source_resolution.candidates.len(), 2);
        assert_eq!(summary.expression_domain.candidates.len(), 2);
    }

    #[test]
    fn builds_semantic_evaluator_candidates() {
        let summary = summarize_semantic_evaluator_candidates_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.source_side.expression_semantics.results.len(), 2);
        assert_eq!(summary.source_side.source_resolution.results.len(), 2);
        assert_eq!(summary.expression_domain.results.len(), 2);
    }

    #[test]
    fn builds_semantic_canonical_producer_signal() {
        let summary = summarize_semantic_canonical_producer_signal_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(
            summary
                .canonical_bundle
                .source_side
                .expression_semantics
                .candidates
                .len(),
            2
        );
        assert_eq!(
            summary
                .canonical_bundle
                .source_side
                .source_resolution
                .candidates
                .len(),
            2
        );
        assert_eq!(
            summary.canonical_bundle.expression_domain.candidates.len(),
            2
        );
        assert_eq!(
            summary
                .evaluator_candidates
                .source_side
                .expression_semantics
                .results
                .len(),
            2
        );
        assert_eq!(
            summary
                .evaluator_candidates
                .source_side
                .source_resolution
                .results
                .len(),
            2
        );
        assert_eq!(
            summary.evaluator_candidates.expression_domain.results.len(),
            2
        );
    }
}
