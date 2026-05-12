use crate::{EngineInputV2, QueryPlanSummaryV0};

pub fn summarize_query_plan_input(input: &EngineInputV2) -> QueryPlanSummaryV0 {
    let expression_ids: Vec<String> = input
        .sources
        .iter()
        .flat_map(|source| source.document.class_expressions.iter())
        .map(|expression| expression.id.clone())
        .collect();
    let selector_usage_ids: Vec<String> = input
        .styles
        .iter()
        .flat_map(|style| style.document.selectors.iter())
        .filter(|selector| selector.view_kind == "canonical")
        .filter_map(|selector| selector.canonical_name.as_ref())
        .cloned()
        .collect();

    QueryPlanSummaryV0 {
        schema_version: "0",
        input_version: input.version.clone(),
        total_query_count: expression_ids.len() * 2 + selector_usage_ids.len(),
        expression_semantics_ids: expression_ids.clone(),
        source_expression_resolution_ids: expression_ids,
        selector_usage_ids,
    }
}

#[cfg(test)]
mod tests {
    use super::summarize_query_plan_input;
    use crate::test_support::sample_input;

    #[test]
    fn summarizes_expected_query_plan() {
        let summary = summarize_query_plan_input(&sample_input());

        assert_eq!(
            summary.expression_semantics_ids,
            vec!["expr-1".to_string(), "expr-2".to_string()]
        );
        assert_eq!(
            summary.source_expression_resolution_ids,
            vec!["expr-1".to_string(), "expr-2".to_string()]
        );
        assert_eq!(
            summary.selector_usage_ids,
            vec!["btn-active".to_string(), "card-header".to_string()]
        );
        assert_eq!(summary.total_query_count, 6);
    }
}
