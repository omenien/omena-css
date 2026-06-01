use std::collections::BTreeMap;

use crate::{
    EngineInputV2, ExpressionSemanticsCandidateV0, ExpressionSemanticsCandidatesV0,
    ExpressionSemanticsCanonicalCandidateBundleV0, ExpressionSemanticsCanonicalProducerSignalV0,
    ExpressionSemanticsEvaluatorCandidatePayloadV0, ExpressionSemanticsEvaluatorCandidateV0,
    ExpressionSemanticsEvaluatorCandidatesV0, ExpressionSemanticsFragmentV0,
    ExpressionSemanticsFragmentsV0, ExpressionSemanticsMatchFragmentV0,
    ExpressionSemanticsMatchFragmentsV0, ExpressionSemanticsQueryFragmentV0,
    ExpressionSemanticsQueryFragmentsV0, canonical_selector_count, finite_values_for_facts,
    map_expression_value_domain_kind, map_reduced_expression_value_domain_derivation,
    map_reduced_expression_value_domain_provenance_tree, map_selector_certainty,
    map_selector_certainty_shape_kind, map_selector_certainty_shape_label, map_value_certainty,
    map_value_certainty_shape_kind, map_value_certainty_shape_label, resolve_selector_names,
};

struct ExpressionSemanticsInputRows {
    query_fragments: Vec<ExpressionSemanticsQueryFragmentV0>,
    fragments: Vec<ExpressionSemanticsFragmentV0>,
    match_fragments: Vec<ExpressionSemanticsMatchFragmentV0>,
    candidates: Vec<ExpressionSemanticsCandidateV0>,
    evaluator_candidates: Vec<ExpressionSemanticsEvaluatorCandidateV0>,
}

fn collect_expression_semantics_input_rows(input: &EngineInputV2) -> ExpressionSemanticsInputRows {
    let mut expression_index = BTreeMap::new();
    let mut style_index = BTreeMap::new();
    let mut query_fragments = Vec::new();

    for source in &input.sources {
        for expression in &source.document.class_expressions {
            expression_index.insert(expression.id.clone(), expression);
            query_fragments.push(ExpressionSemanticsQueryFragmentV0 {
                query_id: expression.id.clone(),
                expression_id: expression.id.clone(),
                expression_kind: expression.kind.clone(),
                style_file_path: expression.scss_module_path.clone(),
            });
        }
    }

    for style in &input.styles {
        style_index.insert(style.file_path.clone(), style);
    }

    let mut fragments = Vec::new();
    let mut match_fragments = Vec::new();
    let mut candidates = Vec::new();
    let mut evaluator_candidates = Vec::new();

    for entry in &input.type_facts {
        let Some(expression) = expression_index.get(&entry.expression_id) else {
            continue;
        };
        let Some(style) = style_index.get(&expression.scss_module_path) else {
            continue;
        };

        let selector_names = resolve_selector_names(style, &entry.facts);
        let finite_values = finite_values_for_facts(&entry.facts);
        let candidate_names = finite_values
            .clone()
            .unwrap_or_else(|| selector_names.clone());
        let selector_certainty = map_selector_certainty(
            &entry.facts,
            selector_names.len(),
            canonical_selector_count(style),
        );
        let value_certainty = map_value_certainty(&entry.facts);
        let selector_certainty_shape_label = map_selector_certainty_shape_label(
            &entry.facts,
            selector_names.len(),
            canonical_selector_count(style),
        );
        let selector_certainty_shape_kind = map_selector_certainty_shape_kind(
            &entry.facts,
            selector_names.len(),
            canonical_selector_count(style),
        );
        let value_certainty_shape_kind = map_value_certainty_shape_kind(&entry.facts);
        let value_certainty_shape_label = map_value_certainty_shape_label(&entry.facts);

        fragments.push(ExpressionSemanticsFragmentV0 {
            query_id: entry.expression_id.clone(),
            expression_id: entry.expression_id.clone(),
            expression_kind: expression.kind.clone(),
            style_file_path: expression.scss_module_path.clone(),
            value_domain_kind: map_expression_value_domain_kind(&entry.facts),
            value_constraint_kind: entry.facts.constraint_kind.clone(),
            value_prefix: entry.facts.prefix.clone(),
            value_suffix: entry.facts.suffix.clone(),
            value_min_len: entry.facts.min_len,
            value_max_len: entry.facts.max_len,
            value_char_must: entry.facts.char_must.clone(),
            value_char_may: entry.facts.char_may.clone(),
            value_may_include_other_chars: entry.facts.may_include_other_chars,
        });

        match_fragments.push(ExpressionSemanticsMatchFragmentV0 {
            query_id: entry.expression_id.clone(),
            expression_id: entry.expression_id.clone(),
            style_file_path: expression.scss_module_path.clone(),
            selector_names: selector_names.clone(),
            candidate_names: candidate_names.clone(),
            finite_values: finite_values.clone(),
        });

        let candidate = ExpressionSemanticsCandidateV0 {
            query_id: entry.expression_id.clone(),
            expression_id: entry.expression_id.clone(),
            expression_kind: expression.kind.clone(),
            style_file_path: expression.scss_module_path.clone(),
            selector_names,
            candidate_names,
            finite_values,
            value_domain_kind: map_expression_value_domain_kind(&entry.facts),
            selector_certainty,
            value_certainty,
            selector_certainty_shape_kind,
            selector_certainty_shape_label,
            value_certainty_shape_kind,
            value_certainty_shape_label,
            selector_constraint_kind: entry.facts.constraint_kind.clone(),
            value_certainty_constraint_kind: entry.facts.constraint_kind.clone(),
            value_constraint_kind: entry.facts.constraint_kind.clone(),
            value_prefix: entry.facts.prefix.clone(),
            value_suffix: entry.facts.suffix.clone(),
            value_min_len: entry.facts.min_len,
            value_max_len: entry.facts.max_len,
            value_char_must: entry.facts.char_must.clone(),
            value_char_may: entry.facts.char_may.clone(),
            value_may_include_other_chars: entry.facts.may_include_other_chars,
        };

        candidates.push(candidate.clone());

        evaluator_candidates.push(ExpressionSemanticsEvaluatorCandidateV0 {
            kind: "expression-semantics",
            file_path: entry.file_path.clone(),
            query_id: entry.expression_id.clone(),
            payload: ExpressionSemanticsEvaluatorCandidatePayloadV0 {
                expression_id: entry.expression_id.clone(),
                expression_kind: candidate.expression_kind.clone(),
                style_file_path: candidate.style_file_path.clone(),
                selector_names: candidate.selector_names.clone(),
                candidate_names: candidate.candidate_names.clone(),
                finite_values: candidate.finite_values.clone(),
                value_domain_kind: candidate.value_domain_kind.clone(),
                selector_certainty: candidate.selector_certainty.clone(),
                value_certainty: candidate.value_certainty.clone(),
                selector_certainty_shape_kind: candidate.selector_certainty_shape_kind.clone(),
                selector_certainty_shape_label: candidate.selector_certainty_shape_label.clone(),
                value_certainty_shape_kind: candidate.value_certainty_shape_kind.clone(),
                value_certainty_shape_label: candidate.value_certainty_shape_label.clone(),
                selector_constraint_kind: candidate.selector_constraint_kind.clone(),
                value_certainty_constraint_kind: candidate.value_certainty_constraint_kind.clone(),
                value_constraint_kind: candidate.value_constraint_kind.clone(),
                value_prefix: candidate.value_prefix.clone(),
                value_suffix: candidate.value_suffix.clone(),
                value_min_len: candidate.value_min_len,
                value_max_len: candidate.value_max_len,
                value_char_must: candidate.value_char_must.clone(),
                value_char_may: candidate.value_char_may.clone(),
                value_may_include_other_chars: candidate.value_may_include_other_chars,
                value_domain_derivation: map_reduced_expression_value_domain_derivation(
                    &entry.facts,
                ),
                value_domain_provenance_tree: map_reduced_expression_value_domain_provenance_tree(
                    &entry.facts,
                ),
            },
        });
    }

    query_fragments.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    fragments.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    match_fragments.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    candidates.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    evaluator_candidates.sort_by(|a, b| a.query_id.cmp(&b.query_id));

    ExpressionSemanticsInputRows {
        query_fragments,
        fragments,
        match_fragments,
        candidates,
        evaluator_candidates,
    }
}

pub fn summarize_expression_semantics_canonical_candidate_bundle_input(
    input: &EngineInputV2,
) -> ExpressionSemanticsCanonicalCandidateBundleV0 {
    let rows = collect_expression_semantics_input_rows(input);

    ExpressionSemanticsCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        query_fragments: rows.query_fragments,
        fragments: rows.fragments,
        match_fragments: rows.match_fragments,
        candidates: rows.candidates,
    }
}

pub fn summarize_expression_semantics_candidates_input(
    input: &EngineInputV2,
) -> ExpressionSemanticsCandidatesV0 {
    let rows = collect_expression_semantics_input_rows(input);

    ExpressionSemanticsCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        candidates: rows.candidates,
    }
}

pub fn summarize_expression_semantics_evaluator_candidates_input(
    input: &EngineInputV2,
) -> ExpressionSemanticsEvaluatorCandidatesV0 {
    let rows = collect_expression_semantics_input_rows(input);

    ExpressionSemanticsEvaluatorCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        results: rows.evaluator_candidates,
    }
}

pub fn summarize_expression_semantics_canonical_producer_signal_input(
    input: &EngineInputV2,
) -> ExpressionSemanticsCanonicalProducerSignalV0 {
    let rows = collect_expression_semantics_input_rows(input);
    let input_version = input.version.clone();

    ExpressionSemanticsCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: input_version.clone(),
        canonical_bundle: ExpressionSemanticsCanonicalCandidateBundleV0 {
            schema_version: "0",
            input_version: input_version.clone(),
            query_fragments: rows.query_fragments.clone(),
            fragments: rows.fragments.clone(),
            match_fragments: rows.match_fragments.clone(),
            candidates: rows.candidates.clone(),
        },
        evaluator_candidates: ExpressionSemanticsEvaluatorCandidatesV0 {
            schema_version: "0",
            input_version,
            results: rows.evaluator_candidates,
        },
    }
}

pub fn summarize_expression_semantics_fragments_input(
    input: &EngineInputV2,
) -> ExpressionSemanticsFragmentsV0 {
    let rows = collect_expression_semantics_input_rows(input);

    ExpressionSemanticsFragmentsV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        fragments: rows.fragments,
    }
}

pub fn summarize_expression_semantics_query_fragments_input(
    input: &EngineInputV2,
) -> ExpressionSemanticsQueryFragmentsV0 {
    let rows = collect_expression_semantics_input_rows(input);

    ExpressionSemanticsQueryFragmentsV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        fragments: rows.query_fragments,
    }
}

pub fn summarize_expression_semantics_match_fragments_input(
    input: &EngineInputV2,
) -> ExpressionSemanticsMatchFragmentsV0 {
    let rows = collect_expression_semantics_input_rows(input);

    ExpressionSemanticsMatchFragmentsV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        fragments: rows.match_fragments,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        summarize_expression_semantics_candidates_input,
        summarize_expression_semantics_canonical_candidate_bundle_input,
        summarize_expression_semantics_canonical_producer_signal_input,
        summarize_expression_semantics_evaluator_candidates_input,
        summarize_expression_semantics_fragments_input,
        summarize_expression_semantics_match_fragments_input,
        summarize_expression_semantics_query_fragments_input,
    };
    use crate::test_support::sample_input;

    #[test]
    fn builds_expression_semantics_fragment_from_type_fact() {
        let summary = summarize_expression_semantics_fragments_input(&sample_input());

        assert_eq!(summary.fragments.len(), 2);
        let fragment = &summary.fragments[0];
        assert_eq!(fragment.query_id, "expr-1");
        assert_eq!(fragment.expression_id, "expr-1");
        assert_eq!(fragment.expression_kind, "symbolRef");
        assert_eq!(fragment.style_file_path, "/tmp/App.module.scss");
        assert_eq!(fragment.value_domain_kind, "constrained");
        assert_eq!(
            fragment.value_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(fragment.value_prefix.as_deref(), Some("btn-"));
        assert_eq!(fragment.value_suffix.as_deref(), Some("-active"));
        assert_eq!(fragment.value_min_len, Some(10));

        let second = &summary.fragments[1];
        assert_eq!(second.query_id, "expr-2");
        assert_eq!(second.value_domain_kind, "finiteSet");
    }

    #[test]
    fn builds_expression_semantics_query_fragments_from_input() {
        let summary = summarize_expression_semantics_query_fragments_input(&sample_input());

        assert_eq!(summary.fragments.len(), 2);
        let first = &summary.fragments[0];
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.expression_id, "expr-1");
        assert_eq!(first.expression_kind, "symbolRef");
        assert_eq!(first.style_file_path, "/tmp/App.module.scss");

        let second = &summary.fragments[1];
        assert_eq!(second.query_id, "expr-2");
        assert_eq!(second.expression_kind, "styleAccess");
        assert_eq!(second.style_file_path, "/tmp/Card.module.scss");
    }

    #[test]
    fn builds_expression_semantics_match_fragments_from_input() {
        let summary = summarize_expression_semantics_match_fragments_input(&sample_input());

        assert_eq!(summary.fragments.len(), 2);
        let first = &summary.fragments[0];
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.expression_id, "expr-1");
        assert_eq!(first.style_file_path, "/tmp/App.module.scss");
        assert_eq!(first.selector_names, vec!["btn-active".to_string()]);
        assert_eq!(first.candidate_names, vec!["btn-active".to_string()]);
        assert!(first.finite_values.is_none());

        let second = &summary.fragments[1];
        assert_eq!(second.query_id, "expr-2");
        assert_eq!(second.selector_names, vec!["card-header".to_string()]);
        assert_eq!(
            second.candidate_names,
            vec!["card-header".to_string(), "card-body".to_string()]
        );
        assert_eq!(
            second.finite_values,
            Some(vec!["card-header".to_string(), "card-body".to_string()])
        );
    }

    #[test]
    fn builds_expression_semantics_candidates_from_input() {
        let summary = summarize_expression_semantics_candidates_input(&sample_input());

        assert_eq!(summary.candidates.len(), 2);
        let first = &summary.candidates[0];
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.expression_kind, "symbolRef");
        assert_eq!(first.style_file_path, "/tmp/App.module.scss");
        assert_eq!(first.selector_names, vec!["btn-active".to_string()]);
        assert_eq!(first.candidate_names, vec!["btn-active".to_string()]);
        assert_eq!(first.value_domain_kind, "constrained");
        assert_eq!(first.selector_certainty, "inferred");
        assert_eq!(first.value_certainty.as_deref(), Some("inferred"));
        assert_eq!(first.selector_certainty_shape_kind, "constrained");
        assert_eq!(
            first.selector_certainty_shape_label,
            "constrained edge selector set (1)"
        );
        assert_eq!(first.value_certainty_shape_kind, "constrained");
        assert_eq!(
            first.value_certainty_shape_label,
            "constrained prefix `btn-` + suffix `-active`"
        );
        assert_eq!(
            first.selector_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(
            first.value_certainty_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(first.value_constraint_kind.as_deref(), Some("prefixSuffix"));
        assert_eq!(first.value_prefix.as_deref(), Some("btn-"));
        assert_eq!(first.value_suffix.as_deref(), Some("-active"));

        let second = &summary.candidates[1];
        assert_eq!(second.query_id, "expr-2");
        assert_eq!(second.expression_kind, "styleAccess");
        assert_eq!(second.selector_names, vec!["card-header".to_string()]);
        assert_eq!(
            second.finite_values,
            Some(vec!["card-header".to_string(), "card-body".to_string()])
        );
        assert_eq!(second.value_domain_kind, "finiteSet");
        assert_eq!(second.selector_certainty, "inferred");
        assert_eq!(second.value_certainty.as_deref(), Some("inferred"));
        assert_eq!(second.selector_certainty_shape_kind, "boundedFinite");
        assert_eq!(
            second.selector_certainty_shape_label,
            "bounded selector set (1)"
        );
        assert_eq!(second.value_certainty_shape_kind, "boundedFinite");
        assert_eq!(second.value_certainty_shape_label, "bounded finite (2)");
    }

    #[test]
    fn builds_expression_semantics_canonical_candidate_bundle() {
        let bundle =
            summarize_expression_semantics_canonical_candidate_bundle_input(&sample_input());

        assert_eq!(bundle.query_fragments.len(), 2);
        assert_eq!(bundle.fragments.len(), 2);
        assert_eq!(bundle.match_fragments.len(), 2);
        assert_eq!(bundle.candidates.len(), 2);
        assert_eq!(bundle.query_fragments[0].query_id, "expr-1");
        assert_eq!(bundle.candidates[0].query_id, "expr-1");
    }

    #[test]
    fn builds_expression_semantics_evaluator_candidates() {
        let summary = summarize_expression_semantics_evaluator_candidates_input(&sample_input());

        assert_eq!(summary.results.len(), 2);
        let first = &summary.results[0];
        assert_eq!(first.kind, "expression-semantics");
        assert_eq!(first.file_path, "/tmp/App.tsx");
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.payload.expression_kind, "symbolRef");
        assert_eq!(first.payload.value_domain_kind, "constrained");
        assert_eq!(
            first.payload.value_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(
            first.payload.value_domain_derivation.product,
            "omena-abstract-value.reduced-class-value-derivation"
        );
        assert_eq!(
            first.payload.value_domain_derivation.reduced_kind,
            "prefixSuffix"
        );
        assert_eq!(
            first.payload.value_domain_provenance_tree.product,
            "omena-abstract-value.provenance-tree"
        );
        assert_eq!(
            first.payload.value_domain_provenance_tree.root.operation,
            "constraintDomain"
        );
    }

    #[test]
    fn builds_expression_semantics_canonical_producer_signal() {
        let summary =
            summarize_expression_semantics_canonical_producer_signal_input(&sample_input());

        assert_eq!(summary.canonical_bundle.candidates.len(), 2);
        assert_eq!(summary.evaluator_candidates.results.len(), 2);
        assert_eq!(summary.evaluator_candidates.results[0].query_id, "expr-1");
    }
}
