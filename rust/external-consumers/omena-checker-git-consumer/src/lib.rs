use omena_checker::{
    list_omena_checker_rule_code_names, summarize_omena_checker_boundary,
};

pub fn consume_checker_boundary_product() -> &'static str {
    summarize_omena_checker_boundary().product
}

pub fn consume_checker_rule_names() -> Vec<&'static str> {
    list_omena_checker_rule_code_names()
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_abstract_value::{
        CompositeClassValueInputV0, composite_class_value, finite_set_class_value,
    };
    use omena_checker::{
        OmenaCheckerDynamicClassDomainInputV0, OmenaCheckerDynamicClassDomainOutcomeV0,
        OmenaCheckerRuleCodeV0, evaluate_omena_checker_dynamic_class_domain,
        list_omena_checker_code_bundles,
    };
    use serde_json::json;

    #[test]
    fn consumes_remote_checker_boundary_via_git_dependency() {
        let summary = summarize_omena_checker_boundary();

        assert_eq!(consume_checker_boundary_product(), "omena-checker.boundary");
        assert_eq!(summary.owner_crate, "omena-checker");
        assert_eq!(summary.rule_count, 13);
        assert_eq!(summary.source_rule_count, 5);
        assert_eq!(summary.style_rule_count, 8);
        assert!(
            summary
                .bridge_policy
                .contains(&"rustOwnsRuleAndBundleMetadataBeforeRuntimeMigration")
        );
    }

    #[test]
    fn consumes_remote_checker_registry_and_bundles() {
        let names = consume_checker_rule_names();
        let bundles = list_omena_checker_code_bundles();

        assert!(names.contains(&"missing-resolved-class-values"));
        assert!(names.contains(&"missing-sass-symbol"));
        assert_eq!(bundles.len(), 4);
        assert!(
            bundles
                .iter()
                .any(|bundle| bundle.bundle_name == "source-missing")
        );
    }

    #[test]
    fn consumes_remote_dynamic_class_domain_evaluator() -> Result<(), serde_json::Error> {
        let evaluation =
            evaluate_omena_checker_dynamic_class_domain(OmenaCheckerDynamicClassDomainInputV0 {
                abstract_value: finite_set_class_value(["btn-primary", "btn-missing"]),
                selector_universe: vec!["btn-primary".to_string(), "card".to_string()],
            });

        assert_eq!(
            evaluation.outcome,
            OmenaCheckerDynamicClassDomainOutcomeV0::MissingResolvedClassValues
        );
        assert_eq!(
            evaluation.rule_code,
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassValues)
        );
        assert_eq!(evaluation.missing_values, vec!["btn-missing"]);
        let rule_code_name = serde_json::to_value(evaluation.rule_code_name)?;

        assert_eq!(rule_code_name, json!("missing-resolved-class-values"));
        Ok(())
    }

    #[test]
    fn consumes_remote_dynamic_class_domain_projection_policy() {
        let evaluation =
            evaluate_omena_checker_dynamic_class_domain(OmenaCheckerDynamicClassDomainInputV0 {
                abstract_value: composite_class_value(CompositeClassValueInputV0 {
                    prefix: Some("btn-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_length: Some(16),
                    must_chars: "-abceintv".to_string(),
                    may_chars: "-abceinprtv".to_string(),
                    may_include_other_chars: false,
                    provenance: None,
                }),
                selector_universe: vec!["btn-primary".to_string(), "card".to_string()],
            });

        assert_eq!(
            evaluation.outcome,
            OmenaCheckerDynamicClassDomainOutcomeV0::MissingResolvedClassDomain
        );
        assert_eq!(
            evaluation.rule_code,
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassDomain)
        );
        assert!(evaluation.selector_names.is_empty());
    }
}
