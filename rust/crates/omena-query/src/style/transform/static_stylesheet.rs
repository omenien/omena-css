use super::super::stylesheet_evaluation::derive_static_stylesheet_module_evaluation;
use super::*;
use omena_query_transform_runner::{
    TransformImportInlineV0, TransformModuleEvaluationOracleV0, TransformModuleEvaluationV0,
    restore_less_inline_literal_placeholders,
};
use std::collections::{BTreeMap, BTreeSet};

mod evaluation_source;
mod scss_forwarding;
mod scss_module_context;
mod scss_module_rules;
mod scss_use_inlining;
mod scss_variable_overrides;

use evaluation_source::{
    derive_import_aware_static_stylesheet_module_evaluation_source,
    static_stylesheet_module_system_evaluator_label,
};
pub(super) use scss_forwarding::{
    derive_static_scss_module_forward_effective_variable_override_values_for_resolution_at_ordinal,
    derive_static_scss_module_forward_variable_override_values_at_ordinal,
};
pub(super) use scss_module_context::derive_static_scss_module_configurable_variable_names_for_transform_context;
use scss_module_context::{
    StaticScssModuleContextRequest, derive_static_scss_module_context_for_transform_context,
};
pub(super) use scss_module_rules::derive_static_scss_module_rule_variable_overrides_at_ordinal;
pub(super) use scss_use_inlining::derive_scss_use_aware_static_stylesheet_module_evaluation_source;

pub(super) fn derive_static_stylesheet_module_evaluation_for_transform_context(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
    import_inlines: &[TransformImportInlineV0],
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> Option<TransformModuleEvaluationV0> {
    let import_aware_source = derive_import_aware_static_stylesheet_module_evaluation_source(
        style_source,
        dialect,
        import_inlines,
    );
    let evaluation_source = derive_scss_use_aware_static_stylesheet_module_evaluation_source(
        import_aware_source.source.as_ref(),
        dialect,
        scss_module_uses,
    );
    if let Some(evaluation) =
        derive_static_stylesheet_module_evaluation(evaluation_source.as_ref(), dialect)
    {
        let native_edit_output = evaluation.native_edit_output.as_deref().map(|output| {
            restore_less_inline_literal_placeholders(
                output,
                &import_aware_source.less_inline_literal_placeholders,
            )
        });
        let oracle = evaluation.oracle;
        return Some(TransformModuleEvaluationV0 {
            evaluator: evaluation.evaluator,
            product_output_source: evaluation.product_output_source,
            evaluated_css: restore_less_inline_literal_placeholders(
                evaluation.evaluated_css.as_str(),
                &import_aware_source.less_inline_literal_placeholders,
            ),
            native_edit_output,
            native_replacements: evaluation.native_replacements,
            native_edits: evaluation.native_edits,
            oracle,
        });
    }
    (evaluation_source.as_ref() != style_source).then(|| {
        let output = restore_less_inline_literal_placeholders(
            evaluation_source.as_ref(),
            &import_aware_source.less_inline_literal_placeholders,
        );
        TransformModuleEvaluationV0 {
            evaluator: static_stylesheet_module_system_evaluator_label(dialect).to_string(),
            product_output_source: Some("nativeEditOutput".to_string()),
            evaluated_css: output.clone(),
            native_edit_output: Some(output),
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
            oracle: Some(static_stylesheet_module_system_zero_divergence_oracle()),
        }
    })
}

fn static_stylesheet_module_system_zero_divergence_oracle() -> TransformModuleEvaluationOracleV0 {
    TransformModuleEvaluationOracleV0 {
        mode: "oracleOnly".to_string(),
        product_output_source: "legacyEvaluatedCss".to_string(),
        divergence_count: 0,
        all_legacy_declaration_values_preserved: true,
        ..TransformModuleEvaluationOracleV0::default()
    }
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssModuleUseEvaluation {
    source: String,
    use_rule_ordinal: usize,
    module_identity_key: String,
    namespace_kind: Option<&'static str>,
    namespace: Option<String>,
    module_output_css: String,
    variable_exports: BTreeMap<String, String>,
}

pub(super) fn derive_static_scss_module_use_evaluations_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
) -> Vec<StaticScssModuleUseEvaluation> {
    if !matches!(
        omena_parser_dialect_for_style_path(entry.style_path.as_str()),
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let mut emitted_module_identity_keys = BTreeSet::new();
    let mut loaded_module_overrides_by_path = BTreeMap::new();
    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassUse")
        .enumerate()
        .filter(|(_, edge)| {
            matches!(
                edge.namespace_kind,
                Some("alias") | Some("default") | Some("wildcard")
            )
        })
        .filter_map(|(use_rule_ordinal, edge)| {
            let resolved = resolution_context.resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
            )?;
            let source = source_by_path.get(resolved.as_str())?;
            let variable_overrides = derive_static_scss_module_rule_variable_overrides_at_ordinal(
                entry.style_source.as_str(),
                "@use",
                use_rule_ordinal,
            );
            let configurable_variable_names =
                derive_static_scss_module_configurable_variable_names_for_transform_context(
                    resolved.as_str(),
                    source,
                    available_style_paths,
                    source_by_path,
                    resolution_context,
                );
            if !omena_semantic::sass_module_configuration_variables_are_valid(
                &variable_overrides,
                &configurable_variable_names,
            ) {
                return None;
            }
            let variable_overrides =
                omena_semantic::resolve_sass_module_effective_variable_overrides(
                    resolved.as_str(),
                    &variable_overrides,
                    &mut loaded_module_overrides_by_path,
                )?;
            let module_identity_key = omena_semantic::summarize_sass_module_instance_identity_key(
                resolved.as_str(),
                &variable_overrides,
            );
            let module_context = derive_static_scss_module_context_for_transform_context(
                StaticScssModuleContextRequest {
                    style_path: resolved.as_str(),
                    style_source: source,
                    variable_overrides: &variable_overrides,
                    available_style_paths,
                    source_by_path,
                    resolution_context,
                },
                &mut emitted_module_identity_keys,
                &mut loaded_module_overrides_by_path,
            )?;
            let module_output_css =
                if emitted_module_identity_keys.insert(module_identity_key.clone()) {
                    module_context.module_output_css
                } else {
                    String::new()
                };
            Some(StaticScssModuleUseEvaluation {
                source: edge.source.clone(),
                use_rule_ordinal,
                module_identity_key,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                module_output_css,
                variable_exports: module_context.variable_exports,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::evaluation_source::static_stylesheet_module_output_css_from_evaluation;
    use super::*;
    use omena_query_transform_runner::{
        TransformModuleEvaluationNativeEditV0, TransformModuleEvaluationOracleV0,
    };

    #[test]
    fn static_module_output_rejects_blind_legacy_css_for_native_product_source() {
        let evaluation = test_transform_module_evaluation(Some("nativeEditOutput"), None, None);

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_declared_legacy_product_source_without_oracle() {
        let evaluation = test_transform_module_evaluation(Some("evaluatedCss"), None, None);

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_preserved_oracle_legacy_output() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            None,
            Some(
                omena_query_transform_runner::TransformModuleEvaluationOracleV0 {
                    mode: "oracleOnly".to_string(),
                    product_output_source: "legacyEvaluatedCss".to_string(),
                    all_legacy_declaration_values_preserved: true,
                    ..omena_query_transform_runner::TransformModuleEvaluationOracleV0::default()
                },
            ),
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_divergent_oracle_legacy_output() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            None,
            Some(TransformModuleEvaluationOracleV0 {
                mode: "oracleOnly".to_string(),
                product_output_source: "legacyEvaluatedCss".to_string(),
                divergence_count: 1,
                all_legacy_declaration_values_preserved: true,
                ..TransformModuleEvaluationOracleV0::default()
            }),
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_native_output_when_oracle_diverges() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            Some(".native { color: red; }".to_string()),
            Some(
                omena_query_transform_runner::TransformModuleEvaluationOracleV0 {
                    mode: "oracleOnly".to_string(),
                    product_output_source: "legacyEvaluatedCss".to_string(),
                    divergence_count: 1,
                    all_legacy_declaration_values_preserved: true,
                    ..omena_query_transform_runner::TransformModuleEvaluationOracleV0::default()
                },
            ),
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_accepts_oracle_backed_matching_native_edit_output() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            Some(".legacy { color: red; }".to_string()),
            Some(TransformModuleEvaluationOracleV0 {
                mode: "oracleOnly".to_string(),
                product_output_source: "legacyEvaluatedCss".to_string(),
                all_legacy_declaration_values_preserved: true,
                ..TransformModuleEvaluationOracleV0::default()
            }),
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            Some(".legacy { color: red; }".to_string())
        );
    }

    #[test]
    fn static_module_output_rejects_oracle_backed_mismatched_native_edit_output() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            Some(".native { color: red; }".to_string()),
            Some(TransformModuleEvaluationOracleV0 {
                mode: "oracleOnly".to_string(),
                product_output_source: "legacyEvaluatedCss".to_string(),
                all_legacy_declaration_values_preserved: true,
                ..TransformModuleEvaluationOracleV0::default()
            }),
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_native_edit_output_without_oracle() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            Some(".native { color: red; }".to_string()),
            None,
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_native_edit_output_without_native_marker() {
        let evaluation = test_transform_module_evaluation(
            Some("legacyEvaluatedCss"),
            Some(".native { color: red; }".to_string()),
            None,
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_materializes_matching_native_edits() {
        let input_css = ".button { color: red; }";
        let start = ".button { color: ".len();
        let end = start + "red".len();
        let mut evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            None,
            Some(oracle_allowing_native_output()),
        );
        evaluation.evaluated_css = ".button { color: blue; }".to_string();
        evaluation
            .native_edits
            .push(TransformModuleEvaluationNativeEditV0 {
                start,
                end,
                replacement: "blue".to_string(),
                edit_kind: "value".to_string(),
                abstract_value: None,
                abstract_value_kind: None,
            });

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation(input_css, evaluation),
            Some(".button { color: blue; }".to_string())
        );
    }

    #[test]
    fn static_module_output_rejects_matching_native_edits_without_native_marker() {
        let input_css = ".button { color: red; }";
        let start = ".button { color: ".len();
        let end = start + "red".len();
        let mut evaluation =
            test_transform_module_evaluation(Some("legacyEvaluatedCss"), None, None);
        evaluation.evaluated_css = ".button { color: blue; }".to_string();
        evaluation
            .native_edits
            .push(TransformModuleEvaluationNativeEditV0 {
                start,
                end,
                replacement: "blue".to_string(),
                edit_kind: "value".to_string(),
                abstract_value: None,
                abstract_value_kind: None,
            });

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation(input_css, evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_mismatched_native_edits_without_oracle() {
        let input_css = ".button { color: red; }";
        let start = ".button { color: ".len();
        let end = start + "red".len();
        let mut evaluation = test_transform_module_evaluation(Some("nativeEditOutput"), None, None);
        evaluation.evaluated_css = ".button { color: green; }".to_string();
        evaluation
            .native_edits
            .push(TransformModuleEvaluationNativeEditV0 {
                start,
                end,
                replacement: "blue".to_string(),
                edit_kind: "value".to_string(),
                abstract_value: None,
                abstract_value_kind: None,
            });

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation(input_css, evaluation),
            None
        );
    }

    fn test_transform_module_evaluation(
        product_output_source: Option<&str>,
        native_edit_output: Option<String>,
        oracle: Option<TransformModuleEvaluationOracleV0>,
    ) -> TransformModuleEvaluationV0 {
        TransformModuleEvaluationV0 {
            evaluator: "test".to_string(),
            product_output_source: product_output_source.map(str::to_string),
            evaluated_css: ".legacy { color: red; }".to_string(),
            native_edit_output,
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
            oracle,
        }
    }

    fn oracle_allowing_native_output() -> TransformModuleEvaluationOracleV0 {
        TransformModuleEvaluationOracleV0 {
            mode: "oracleOnly".to_string(),
            product_output_source: "legacyEvaluatedCss".to_string(),
            divergence_count: 0,
            all_legacy_declaration_values_preserved: true,
            ..TransformModuleEvaluationOracleV0::default()
        }
    }
}
