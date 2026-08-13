use std::collections::{BTreeMap, BTreeSet};

use crate::{
    EngineInputV2, SourceResolutionCandidateV0, SourceResolutionCandidatesV0,
    SourceResolutionCanonicalCandidateBundleV0, SourceResolutionCanonicalProducerSignalV0,
    SourceResolutionEvaluatorCandidatePayloadV0, SourceResolutionEvaluatorCandidateV0,
    SourceResolutionEvaluatorCandidatesV0, SourceResolutionFragmentV0, SourceResolutionFragmentsV0,
    SourceResolutionMatchFragmentV0, SourceResolutionMatchFragmentsV0,
    SourceResolutionPlanSummaryV0, SourceResolutionQueryFragmentV0,
    SourceResolutionQueryFragmentsV0, canonical_selector_count,
    expression_domain::{
        ExpressionDomainSelectorCertaintyFlowHedgesV0,
        collect_expression_domain_selector_certainty_flow_hedges,
    },
    finite_values_for_facts, map_selector_certainty_projection, map_value_certainty,
    map_value_certainty_shape_kind, map_value_certainty_shape_label, resolve_selector_names,
};

struct SourceResolutionInputRows {
    query_fragments: Vec<SourceResolutionQueryFragmentV0>,
    fragments: Vec<SourceResolutionFragmentV0>,
    match_fragments: Vec<SourceResolutionMatchFragmentV0>,
    candidates: Vec<SourceResolutionCandidateV0>,
    evaluator_candidates: Vec<SourceResolutionEvaluatorCandidateV0>,
}

fn collect_source_resolution_input_rows(input: &EngineInputV2) -> SourceResolutionInputRows {
    let selector_certainty_flow_hedges =
        collect_expression_domain_selector_certainty_flow_hedges(input);
    collect_source_resolution_input_rows_with_flow_hedges(input, &selector_certainty_flow_hedges)
}

fn collect_source_resolution_input_rows_with_flow_hedges(
    input: &EngineInputV2,
    selector_certainty_flow_hedges: &ExpressionDomainSelectorCertaintyFlowHedgesV0,
) -> SourceResolutionInputRows {
    let mut expression_index = BTreeMap::new();
    let mut style_index = BTreeMap::new();
    let mut query_fragments = Vec::new();

    for source in &input.sources {
        for expression in &source.document.class_expressions {
            expression_index.insert(expression.id.clone(), expression);
            query_fragments.push(SourceResolutionQueryFragmentV0 {
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
        let selector_certainty_projection = map_selector_certainty_projection(
            &entry.facts,
            selector_names.len(),
            canonical_selector_count(style),
            selector_certainty_flow_hedges
                .get(&(entry.file_path.clone(), entry.expression_id.clone())),
        );
        let selector_certainty = selector_certainty_projection.certainty;
        let selector_certainty_shape_label = selector_certainty_projection.shape_label;
        let selector_certainty_shape_kind = selector_certainty_projection.shape_kind;
        let value_certainty = map_value_certainty(&entry.facts);
        let value_certainty_shape_kind = map_value_certainty_shape_kind(&entry.facts);
        let value_certainty_shape_label = map_value_certainty_shape_label(&entry.facts);

        fragments.push(SourceResolutionFragmentV0 {
            query_id: entry.expression_id.clone(),
            expression_id: entry.expression_id.clone(),
            style_file_path: expression.scss_module_path.clone(),
            value_certainty_shape_kind: value_certainty_shape_kind.clone(),
            value_certainty_constraint_kind: entry.facts.constraint_kind.clone(),
            value_prefix: entry.facts.prefix.clone(),
            value_suffix: entry.facts.suffix.clone(),
            value_min_len: entry.facts.min_len,
            value_max_len: entry.facts.max_len,
            value_char_must: entry.facts.char_must.clone(),
            value_char_may: entry.facts.char_may.clone(),
            value_may_include_other_chars: entry.facts.may_include_other_chars,
        });

        match_fragments.push(SourceResolutionMatchFragmentV0 {
            query_id: entry.expression_id.clone(),
            expression_id: entry.expression_id.clone(),
            style_file_path: expression.scss_module_path.clone(),
            selector_names: selector_names.clone(),
            finite_values: finite_values.clone(),
        });

        let candidate = SourceResolutionCandidateV0 {
            query_id: entry.expression_id.clone(),
            expression_id: entry.expression_id.clone(),
            style_file_path: expression.scss_module_path.clone(),
            selector_names,
            finite_values,
            selector_certainty,
            value_certainty,
            selector_certainty_shape_kind,
            selector_certainty_shape_label,
            value_certainty_shape_kind,
            value_certainty_shape_label,
            selector_constraint_kind: entry.facts.constraint_kind.clone(),
            value_certainty_constraint_kind: entry.facts.constraint_kind.clone(),
            value_prefix: entry.facts.prefix.clone(),
            value_suffix: entry.facts.suffix.clone(),
            value_min_len: entry.facts.min_len,
            value_max_len: entry.facts.max_len,
            value_char_must: entry.facts.char_must.clone(),
            value_char_may: entry.facts.char_may.clone(),
            value_may_include_other_chars: entry.facts.may_include_other_chars,
        };

        candidates.push(candidate.clone());
        evaluator_candidates.push(SourceResolutionEvaluatorCandidateV0 {
            kind: "source-expression-resolution",
            file_path: entry.file_path.clone(),
            query_id: entry.expression_id.clone(),
            payload: SourceResolutionEvaluatorCandidatePayloadV0 {
                expression_id: entry.expression_id.clone(),
                style_file_path: candidate.style_file_path.clone(),
                selector_names: candidate.selector_names.clone(),
                finite_values: candidate.finite_values.clone(),
                selector_certainty: candidate.selector_certainty.clone(),
                value_certainty: candidate.value_certainty.clone(),
                selector_certainty_shape_kind: candidate.selector_certainty_shape_kind.clone(),
                selector_certainty_shape_label: candidate.selector_certainty_shape_label.clone(),
                value_certainty_shape_kind: candidate.value_certainty_shape_kind.clone(),
                value_certainty_shape_label: candidate.value_certainty_shape_label.clone(),
                selector_constraint_kind: candidate.selector_constraint_kind.clone(),
                value_certainty_constraint_kind: candidate.value_certainty_constraint_kind.clone(),
                value_prefix: candidate.value_prefix.clone(),
                value_suffix: candidate.value_suffix.clone(),
                value_min_len: candidate.value_min_len,
                value_max_len: candidate.value_max_len,
                value_char_must: candidate.value_char_must.clone(),
                value_char_may: candidate.value_char_may.clone(),
                value_may_include_other_chars: candidate.value_may_include_other_chars,
            },
        });
    }

    query_fragments.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    fragments.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    match_fragments.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    candidates.sort_by(|a, b| a.query_id.cmp(&b.query_id));
    evaluator_candidates.sort_by(|a, b| a.query_id.cmp(&b.query_id));

    SourceResolutionInputRows {
        query_fragments,
        fragments,
        match_fragments,
        candidates,
        evaluator_candidates,
    }
}

pub fn summarize_source_resolution_candidates_input(
    input: &EngineInputV2,
) -> SourceResolutionCandidatesV0 {
    let rows = collect_source_resolution_input_rows(input);

    SourceResolutionCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        candidates: rows.candidates,
    }
}

pub fn summarize_source_resolution_evaluator_candidates_input(
    input: &EngineInputV2,
) -> SourceResolutionEvaluatorCandidatesV0 {
    let rows = collect_source_resolution_input_rows(input);

    SourceResolutionEvaluatorCandidatesV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        results: rows.evaluator_candidates,
    }
}

pub fn summarize_source_resolution_canonical_candidate_bundle_input(
    input: &EngineInputV2,
) -> SourceResolutionCanonicalCandidateBundleV0 {
    let rows = collect_source_resolution_input_rows(input);

    SourceResolutionCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        query_fragments: rows.query_fragments,
        fragments: rows.fragments,
        match_fragments: rows.match_fragments,
        candidates: rows.candidates,
    }
}

pub fn summarize_source_resolution_canonical_producer_signal_input(
    input: &EngineInputV2,
) -> SourceResolutionCanonicalProducerSignalV0 {
    let rows = collect_source_resolution_input_rows(input);
    let input_version = input.version.clone();

    SourceResolutionCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: input_version.clone(),
        canonical_bundle: SourceResolutionCanonicalCandidateBundleV0 {
            schema_version: "0",
            input_version: input_version.clone(),
            query_fragments: rows.query_fragments.clone(),
            fragments: rows.fragments.clone(),
            match_fragments: rows.match_fragments.clone(),
            candidates: rows.candidates.clone(),
        },
        evaluator_candidates: SourceResolutionEvaluatorCandidatesV0 {
            schema_version: "0",
            input_version,
            results: rows.evaluator_candidates,
        },
    }
}

pub fn summarize_source_resolution_plan_input(
    input: &EngineInputV2,
) -> SourceResolutionPlanSummaryV0 {
    let mut planned_expression_ids = Vec::new();
    let mut expression_kind_counts = BTreeMap::new();
    let mut distinct_style_file_paths = BTreeSet::new();
    let mut symbol_ref_with_binding_count = 0usize;
    let mut style_access_count = 0usize;
    let mut style_access_path_depth_sum = 0usize;

    for source in &input.sources {
        for expression in &source.document.class_expressions {
            planned_expression_ids.push(expression.id.clone());
            distinct_style_file_paths.insert(expression.scss_module_path.clone());
            *expression_kind_counts
                .entry(expression.kind.clone())
                .or_insert(0) += 1;

            if expression.kind == "symbolRef" && expression.root_binding_decl_id.is_some() {
                symbol_ref_with_binding_count += 1;
            }

            if expression.kind == "styleAccess" {
                style_access_count += 1;
                style_access_path_depth_sum += expression.access_path.as_ref().map_or(0, Vec::len);
            }
        }
    }

    SourceResolutionPlanSummaryV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        planned_expression_ids,
        expression_kind_counts,
        distinct_style_file_paths: distinct_style_file_paths.into_iter().collect(),
        symbol_ref_with_binding_count,
        style_access_count,
        style_access_path_depth_sum,
    }
}

pub fn summarize_source_resolution_fragments_input(
    input: &EngineInputV2,
) -> SourceResolutionFragmentsV0 {
    let rows = collect_source_resolution_input_rows(input);

    SourceResolutionFragmentsV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        fragments: rows.fragments,
    }
}

pub fn summarize_source_resolution_query_fragments_input(
    input: &EngineInputV2,
) -> SourceResolutionQueryFragmentsV0 {
    let rows = collect_source_resolution_input_rows(input);

    SourceResolutionQueryFragmentsV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        fragments: rows.query_fragments,
    }
}

pub fn summarize_source_resolution_match_fragments_input(
    input: &EngineInputV2,
) -> SourceResolutionMatchFragmentsV0 {
    let rows = collect_source_resolution_input_rows(input);

    SourceResolutionMatchFragmentsV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        fragments: rows.match_fragments,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        summarize_source_resolution_candidates_input,
        summarize_source_resolution_canonical_candidate_bundle_input,
        summarize_source_resolution_canonical_producer_signal_input,
        summarize_source_resolution_evaluator_candidates_input,
        summarize_source_resolution_fragments_input,
        summarize_source_resolution_match_fragments_input, summarize_source_resolution_plan_input,
        summarize_source_resolution_query_fragments_input,
    };
    use crate::{
        PositionV2, RangeV2, StyleSelectorV2, configure_nonconvergent_selector_certainty_fixture,
        test_support::sample_input,
    };

    #[test]
    fn builds_source_resolution_fragment_from_type_fact() {
        let summary = summarize_source_resolution_fragments_input(&sample_input());

        assert_eq!(summary.fragments.len(), 2);
        let first = &summary.fragments[0];
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.style_file_path, "/tmp/App.module.scss");
        assert_eq!(first.value_certainty_shape_kind, "constrained");
        assert_eq!(
            first.value_certainty_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );

        let second = &summary.fragments[1];
        assert_eq!(second.query_id, "expr-2");
        assert_eq!(second.expression_id, "expr-2");
        assert_eq!(second.style_file_path, "/tmp/Card.module.scss");
        assert_eq!(second.value_certainty_shape_kind, "boundedFinite");
        assert!(second.value_certainty_constraint_kind.is_none());
    }

    #[test]
    fn builds_source_resolution_plan_from_input() {
        let summary = summarize_source_resolution_plan_input(&sample_input());

        assert_eq!(
            summary.planned_expression_ids,
            vec!["expr-1".to_string(), "expr-2".to_string()]
        );
        assert_eq!(
            summary.distinct_style_file_paths,
            vec![
                "/tmp/App.module.scss".to_string(),
                "/tmp/Card.module.scss".to_string()
            ]
        );
        assert_eq!(summary.symbol_ref_with_binding_count, 1);
        assert_eq!(summary.style_access_count, 1);
        assert_eq!(summary.style_access_path_depth_sum, 2);
    }

    #[test]
    fn builds_source_resolution_query_fragments_from_input() {
        let summary = summarize_source_resolution_query_fragments_input(&sample_input());

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
    fn builds_source_resolution_match_fragments_from_input() {
        let summary = summarize_source_resolution_match_fragments_input(&sample_input());

        assert_eq!(summary.fragments.len(), 2);
        let first = &summary.fragments[0];
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.expression_id, "expr-1");
        assert_eq!(first.style_file_path, "/tmp/App.module.scss");
        assert_eq!(first.selector_names, vec!["btn-active".to_string()]);
        assert!(first.finite_values.is_none());

        let second = &summary.fragments[1];
        assert_eq!(second.query_id, "expr-2");
        assert_eq!(second.style_file_path, "/tmp/Card.module.scss");
        assert_eq!(second.selector_names, vec!["card-header".to_string()]);
        assert_eq!(
            second.finite_values,
            Some(vec!["card-header".to_string(), "card-body".to_string()])
        );
    }

    #[test]
    fn builds_source_resolution_candidates_from_input() {
        let summary = summarize_source_resolution_candidates_input(&sample_input());

        assert_eq!(summary.candidates.len(), 2);
        let first = &summary.candidates[0];
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.expression_id, "expr-1");
        assert_eq!(first.style_file_path, "/tmp/App.module.scss");
        assert_eq!(first.selector_names, vec!["btn-active".to_string()]);
        assert_eq!(first.selector_certainty, "inferred");
        assert_eq!(first.selector_certainty_shape_kind, "constrained");
        assert_eq!(
            first.selector_certainty_shape_label,
            "constrained edge selector set (1)"
        );
        assert_eq!(
            first.selector_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );
        assert_eq!(first.value_certainty.as_deref(), Some("inferred"));
        assert_eq!(first.value_certainty_shape_kind, "constrained");
        assert_eq!(
            first.value_certainty_shape_label,
            "constrained prefix `btn-` + suffix `-active`"
        );
        assert_eq!(
            first.value_certainty_constraint_kind.as_deref(),
            Some("prefixSuffix")
        );

        let second = &summary.candidates[1];
        assert_eq!(second.query_id, "expr-2");
        assert_eq!(second.selector_names, vec!["card-header".to_string()]);
        assert_eq!(second.selector_certainty, "inferred");
        assert_eq!(second.selector_certainty_shape_kind, "boundedFinite");
        assert_eq!(
            second.selector_certainty_shape_label,
            "bounded selector set (1)"
        );
        assert_eq!(second.value_certainty.as_deref(), Some("inferred"));
        assert_eq!(second.value_certainty_shape_kind, "boundedFinite");
        assert_eq!(second.value_certainty_shape_label, "bounded finite (2)");
        assert_eq!(
            second.finite_values,
            Some(vec!["card-header".to_string(), "card-body".to_string()])
        );
    }

    #[test]
    fn source_resolution_applies_utf16_min_and_max_bounds_to_non_ascii_selectors() {
        let original_e8_korean = unicode_length_input(
            "카드-",
            "-활성",
            &["카드-활성", "카드-큰-활성", "카드-x-활성"],
            Some(10),
            None,
        );
        let original_e8_ascii = unicode_length_input(
            "ab-",
            "-cd",
            &["ab-cd", "ab-x-cd", "ab-long-cd"],
            Some(10),
            None,
        );
        assert!(
            summarize_source_resolution_candidates_input(&original_e8_korean).candidates[0]
                .selector_names
                .is_empty(),
            "the original Korean E8 candidates are five, seven, and seven UTF-16 units"
        );
        assert_eq!(
            summarize_source_resolution_candidates_input(&original_e8_ascii).candidates[0]
                .selector_names,
            vec!["ab-long-cd".to_string()],
            "the original E8 ASCII control has a ten-unit third member and is not a structural twin"
        );

        let korean = unicode_length_input(
            "카드-",
            "-활성",
            &["카드-활성", "카드-x-활성", "카드-1234-활성"],
            Some(10),
            None,
        );
        let ascii = unicode_length_input(
            "ab-",
            "-cd",
            &["ab-cd", "ab-x-cd", "ab-1234-cd"],
            Some(10),
            None,
        );

        assert_eq!(
            summarize_source_resolution_candidates_input(&korean).candidates[0].selector_names,
            vec!["카드-1234-활성".to_string()]
        );
        assert_eq!(
            summarize_source_resolution_candidates_input(&ascii).candidates[0].selector_names,
            vec!["ab-1234-cd".to_string()]
        );

        let max_five = unicode_length_input(
            "카드-",
            "-활성",
            &["카드-활성", "카드-x-활성", "카드-1234-활성"],
            None,
            Some(5),
        );
        assert_eq!(
            summarize_source_resolution_candidates_input(&max_five).candidates[0].selector_names,
            vec!["카드-활성".to_string()],
            "the genuine five-code-unit member must pass while longer names are excluded"
        );

        let max_four = unicode_length_input(
            "카드-",
            "-활성",
            &["카드-활성", "카드-x-활성", "카드-1234-활성"],
            None,
            Some(4),
        );
        assert!(
            summarize_source_resolution_candidates_input(&max_four).candidates[0]
                .selector_names
                .is_empty(),
            "a five-code-unit member must not pass a four-code-unit maximum"
        );
    }

    #[test]
    fn source_resolution_applies_utf16_bounds_to_exact_and_finite_values() {
        let mut exact = unicode_length_input("카드-", "-활성", &["카드-활성"], Some(5), Some(5));
        let exact_facts = &mut exact.type_facts[0].facts;
        exact_facts.kind = "exact".to_string();
        exact_facts.constraint_kind = None;
        exact_facts.values = Some(vec!["카드-활성".to_string()]);
        assert_eq!(
            summarize_source_resolution_candidates_input(&exact).candidates[0].selector_names,
            vec!["카드-활성".to_string()]
        );

        exact.type_facts[0].facts.max_len = Some(4);
        assert!(
            summarize_source_resolution_candidates_input(&exact).candidates[0]
                .selector_names
                .is_empty(),
            "an exact five-code-unit value must not pass a four-code-unit maximum"
        );

        exact.type_facts[0].facts.max_len = Some(5);
        exact.type_facts[0].facts.min_len = Some(6);
        assert!(
            summarize_source_resolution_candidates_input(&exact).candidates[0]
                .selector_names
                .is_empty(),
            "an exact five-code-unit value must not pass a six-code-unit minimum"
        );

        let mut finite = unicode_length_input(
            "카드-",
            "-활성",
            &["카드-활성", "카드-x-활성"],
            Some(6),
            Some(7),
        );
        let finite_facts = &mut finite.type_facts[0].facts;
        finite_facts.kind = "finiteSet".to_string();
        finite_facts.constraint_kind = None;
        finite_facts.values = Some(vec!["카드-활성".to_string(), "카드-x-활성".to_string()]);
        assert_eq!(
            summarize_source_resolution_candidates_input(&finite).candidates[0].selector_names,
            vec!["카드-x-활성".to_string()],
            "the minimum excludes the five-unit value while the maximum admits the seven-unit value"
        );

        finite.type_facts[0].facts.max_len = Some(6);
        assert!(
            summarize_source_resolution_candidates_input(&finite).candidates[0]
                .selector_names
                .is_empty(),
            "the finite seven-unit value must be excluded once the maximum drops below it"
        );
    }

    #[test]
    fn nonconverged_flow_hedge_demotes_source_resolution_product() {
        let mut input = sample_input();
        input.styles[0].document.selectors[0].name = "x".to_string();
        input.styles[0].document.selectors[0].canonical_name = Some("x".to_string());
        configure_nonconvergent_selector_certainty_fixture(&mut input.type_facts[0], "x");
        let product = summarize_source_resolution_canonical_producer_signal_input(&input);
        let candidate = &product.canonical_bundle.candidates[0];

        assert_eq!(candidate.query_id, "expr-1");
        assert_eq!(candidate.selector_certainty, "possible");
        assert_eq!(candidate.selector_certainty_shape_kind, "unknown");
        assert_eq!(candidate.selector_certainty_shape_label, "unknown");
        assert_eq!(
            product.evaluator_candidates.results[0]
                .payload
                .selector_certainty,
            "possible"
        );
    }

    #[test]
    fn builds_source_resolution_evaluator_candidates() {
        let summary = summarize_source_resolution_evaluator_candidates_input(&sample_input());

        assert_eq!(summary.results.len(), 2);
        let first = &summary.results[0];
        assert_eq!(first.kind, "source-expression-resolution");
        assert_eq!(first.file_path, "/tmp/App.tsx");
        assert_eq!(first.query_id, "expr-1");
        assert_eq!(first.payload.style_file_path, "/tmp/App.module.scss");
        assert_eq!(first.payload.selector_certainty_shape_kind, "constrained");
    }

    #[test]
    fn builds_source_resolution_canonical_candidate_bundle() {
        let summary = summarize_source_resolution_canonical_candidate_bundle_input(&sample_input());

        assert_eq!(summary.query_fragments.len(), 2);
        assert_eq!(summary.fragments.len(), 2);
        assert_eq!(summary.match_fragments.len(), 2);
        assert_eq!(summary.candidates.len(), 2);
    }

    #[test]
    fn builds_source_resolution_canonical_producer_signal() {
        let summary = summarize_source_resolution_canonical_producer_signal_input(&sample_input());

        assert_eq!(summary.canonical_bundle.candidates.len(), 2);
        assert_eq!(summary.evaluator_candidates.results.len(), 2);
        assert_eq!(summary.evaluator_candidates.results[0].query_id, "expr-1");
    }

    fn unicode_length_input(
        prefix: &str,
        suffix: &str,
        selector_names: &[&str],
        min_len: Option<crate::Utf16CodeUnitLengthV2>,
        max_len: Option<crate::Utf16CodeUnitLengthV2>,
    ) -> crate::EngineInputV2 {
        let mut input = sample_input();
        input.styles.truncate(1);
        input.styles[0].document.selectors = selector_names
            .iter()
            .enumerate()
            .map(|(index, name)| StyleSelectorV2 {
                name: (*name).to_string(),
                view_kind: "canonical".to_string(),
                canonical_name: Some((*name).to_string()),
                range: RangeV2 {
                    start: PositionV2 {
                        line: index,
                        character: 0,
                    },
                    end: PositionV2 {
                        line: index,
                        character: name.encode_utf16().count(),
                    },
                },
                nested_safety: Some("safe".to_string()),
                composes: None,
                bem_suffix: None,
            })
            .collect();
        input.type_facts.truncate(1);
        let facts = &mut input.type_facts[0].facts;
        facts.prefix = Some(prefix.to_string());
        facts.suffix = Some(suffix.to_string());
        facts.min_len = min_len;
        facts.max_len = max_len;
        input
    }
}
