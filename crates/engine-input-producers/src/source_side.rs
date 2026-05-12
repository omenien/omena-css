use crate::{
    EngineInputV2, SourceSideCanonicalCandidateBundleV0, SourceSideCanonicalProducerSignalV0,
    SourceSideEvaluatorCandidatesV0,
    summarize_expression_semantics_canonical_candidate_bundle_input,
    summarize_expression_semantics_evaluator_candidates_input,
    summarize_source_resolution_canonical_candidate_bundle_input,
    summarize_source_resolution_evaluator_candidates_input,
};

pub fn summarize_source_side_canonical_candidate_bundle_input(
    input: &EngineInputV2,
) -> SourceSideCanonicalCandidateBundleV0 {
    SourceSideCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        expression_semantics: summarize_expression_semantics_canonical_candidate_bundle_input(
            input,
        ),
        source_resolution: summarize_source_resolution_canonical_candidate_bundle_input(input),
    }
}

pub fn summarize_source_side_evaluator_candidates_input(
    input: &EngineInputV2,
) -> SourceSideEvaluatorCandidatesV0 {
    SourceSideEvaluatorCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        expression_semantics: summarize_expression_semantics_evaluator_candidates_input(input),
        source_resolution: summarize_source_resolution_evaluator_candidates_input(input),
    }
}

pub fn summarize_source_side_canonical_producer_signal_input(
    input: &EngineInputV2,
) -> SourceSideCanonicalProducerSignalV0 {
    SourceSideCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        canonical_bundle: summarize_source_side_canonical_candidate_bundle_input(input),
        evaluator_candidates: summarize_source_side_evaluator_candidates_input(input),
    }
}

#[cfg(test)]
mod tests {
    use crate::source_side::{
        summarize_source_side_canonical_candidate_bundle_input,
        summarize_source_side_canonical_producer_signal_input,
        summarize_source_side_evaluator_candidates_input,
    };
    use crate::test_support::sample_input;

    #[test]
    fn builds_source_side_canonical_candidate_bundle() {
        let summary = summarize_source_side_canonical_candidate_bundle_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.expression_semantics.candidates.len(), 2);
        assert_eq!(summary.source_resolution.candidates.len(), 2);
    }

    #[test]
    fn builds_source_side_evaluator_candidates() {
        let summary = summarize_source_side_evaluator_candidates_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.expression_semantics.results.len(), 2);
        assert_eq!(summary.source_resolution.results.len(), 2);
    }

    #[test]
    fn builds_source_side_canonical_producer_signal() {
        let summary = summarize_source_side_canonical_producer_signal_input(&sample_input());

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.input_version, "2");
        assert_eq!(
            summary
                .canonical_bundle
                .expression_semantics
                .candidates
                .len(),
            2
        );
        assert_eq!(
            summary
                .evaluator_candidates
                .expression_semantics
                .results
                .len(),
            2
        );
        assert_eq!(
            summary.canonical_bundle.source_resolution.candidates.len(),
            2
        );
        assert_eq!(
            summary.evaluator_candidates.source_resolution.results.len(),
            2
        );
    }
}
