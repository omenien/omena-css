use omena_query_checker_orchestrator::run_omena_query_checker_cascade_gate_with_standard_property_value_verdicts_v0;
#[cfg(test)]
use omena_query_checker_orchestrator::{CanonicalSelector, OmenaCheckerCascadeDeclarationInputV0};
use omena_syntax::ident::PropertyNameV0;
mod confidence;
mod custom_property_registration;
mod declaration_facts;
#[cfg(test)]
mod declaration_facts_differential;
mod diagnostic_render;
mod input;
mod replica_ensemble;
mod runtime_state;
mod smt;
#[cfg(test)]
mod source_scanner;
mod theory_hints;
mod value_references;
use confidence::summarize_query_cascade_confidence_for_evaluation;
pub(super) use custom_property_registration::collect_query_checker_custom_property_registrations;
use diagnostic_render::{
    query_cascade_checker_code, query_cascade_checker_diagnostic_severity,
    query_cascade_checker_diagnostic_tags, summarize_query_cascade_narrowing_for_evaluation,
};
#[cfg(test)]
pub(crate) use input::cascade_declarations_collect_probe;
#[cfg(feature = "salsa-memo")]
pub(super) use input::collect_query_checker_cascade_declarations_from_syntax_and_context;
pub(in crate::style) use input::collect_query_checker_cascade_declarations_with_dialect;
pub(super) use input::{
    QueryCheckerCascadeDeclaration, collect_query_checker_cascade_declarations,
};
use input::{QueryCheckerCascadeInputCollection, collect_query_checker_cascade_input};
pub(super) use replica_ensemble::collect_query_replica_ensemble_site_outcomes;
pub(crate) use runtime_state::query_condition_context_static_supports_pruning_evidence;
pub(super) use runtime_state::query_runtime_cascade_declaration_from_input;
#[cfg(test)]
use runtime_state::query_runtime_selector_matches_anchor_classes;
pub(super) use runtime_state::query_runtime_state_confidence_tier;
pub(in crate::style) use runtime_state::query_runtime_state_scenario_with_inline_override;
pub(super) use smt::query_smt_box_shorthand_longhand_quartets;
use smt::summarize_query_smt_cascade_obligation_diagnostics;
#[cfg(test)]
use smt::{QuerySmtCascadeObligation, query_smt_layer_inversion_obligations};
pub(super) use theory_hints::query_exercised_cascade_primitive_role_pairs_from_source;
use theory_hints::{
    deduplicate_query_theory_hints_against_circular_var,
    summarize_query_categorical_cascade_evidence_diagnostics,
    summarize_query_multiscale_complexity_heuristic_coupling_diagnostics,
};

use super::{
    OmenaQueryStyleDiagnosticV0, ParserByteSpanV0, ParserRangeV0,
    omena_parser_dialect_for_style_path, parser_range_for_byte_span,
};

/// Cascade diagnostics with an optional theory-hint pass. Hints overlapping
/// `circularVar` fold into its provenance to avoid duplicate diagnostics.
pub(super) fn summarize_query_cascade_checker_diagnostics_with_deep_analysis(
    style_uri: &str,
    source: &str,
    deep_analysis: bool,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let QueryCheckerCascadeInputCollection {
        checker_input,
        declaration_ranges,
        custom_property_ranges,
        topology_incomplete_unresolved_count,
        standard_property_value_verdicts,
    } = collect_query_checker_cascade_input(style_uri, source);
    let mut diagnostics = Vec::new();

    // Keep whole-file theory hints off the default LSP/CLI surface.
    let (multiscale_complexity_heuristic_diagnostics, categorical_diagnostics, smt_diagnostics) =
        if deep_analysis {
            (
                summarize_query_multiscale_complexity_heuristic_coupling_diagnostics(
                    source,
                    &checker_input.custom_properties,
                ),
                summarize_query_categorical_cascade_evidence_diagnostics(
                    source,
                    &checker_input.custom_properties,
                ),
                summarize_query_smt_cascade_obligation_diagnostics(
                    source,
                    &checker_input.declarations,
                    &declaration_ranges,
                ),
            )
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

    let gate = run_omena_query_checker_cascade_gate_with_standard_property_value_verdicts_v0(
        checker_input.clone(),
        &standard_property_value_verdicts,
    );
    if !gate.enforcement_passed {
        return vec![OmenaQueryStyleDiagnosticV0 {
            code: "checkerDiagnosticGateFailed",
            severity: "warning",
            provenance: omena_query_evidence_graph_provenance![
                "omena-query-checker-orchestrator.cascade-gate",
                "omena-query.cascade-checker",
            ],
            range: parser_range_for_byte_span(
                source,
                ParserByteSpanV0 {
                    start: 0,
                    end: source.len(),
                },
            ),
            message: "Checker diagnostic gate rejected unregistered rule output.".to_string(),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        }];
    }

    // Build the product cascade gate diagnostics first so the `circularVar`
    // ranges are known before the theory hints are deduplicated against them.
    for evaluation in gate.evaluations {
        if topology_incomplete_unresolved_count.is_some()
            && matches!(
                evaluation.rule_code_name,
                "dead-cascade-layer" | "unspecified-cascade-tie"
            )
        {
            continue;
        }
        if evaluation.rule_code_name == "iacvt-prone"
            && evaluation.custom_property_names.iter().all(|name| {
                !custom_property_ranges.contains_key(&PropertyNameV0::canonical_custom_key(name))
            })
        {
            continue;
        }
        let range = evaluation
            .declaration_ids
            .iter()
            .find_map(|declaration_id| declaration_ranges.get(declaration_id).copied())
            .or_else(|| {
                evaluation.custom_property_names.iter().find_map(|name| {
                    custom_property_ranges
                        .get(&PropertyNameV0::canonical_custom_key(name))
                        .copied()
                })
            })
            .unwrap_or_else(|| {
                parser_range_for_byte_span(
                    source,
                    ParserByteSpanV0 {
                        start: 0,
                        end: source.len(),
                    },
                )
            });
        let mut provenance = vec![
            "omena-query-checker-orchestrator.cascade-gate",
            "omena-checker.cascade-rules",
            "omena-query.cascade-checker",
        ];
        provenance.extend(evaluation.mechanism_products.iter().copied());
        let cascade_narrowing = summarize_query_cascade_narrowing_for_evaluation(
            &evaluation,
            checker_input.declarations.as_slice(),
            topology_incomplete_unresolved_count,
        );
        if cascade_narrowing.is_some() {
            provenance.extend([
                "omena-query.cascade-narrowing",
                "omena-abstract-value.property-value-narrowing",
                "omena-abstract-value.reduced-product-iteration",
            ]);
        }
        let cascade_confidence = summarize_query_cascade_confidence_for_evaluation(
            &evaluation,
            checker_input.declarations.as_slice(),
        );
        if cascade_confidence.is_some() {
            provenance.extend(["omena-cascade.margin", "omena-query.cascade-confidence"]);
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: query_cascade_checker_code(evaluation.rule_code_name),
            severity: query_cascade_checker_diagnostic_severity(evaluation.rule_code_name),
            provenance,
            range,
            message: evaluation.message,
            tags: query_cascade_checker_diagnostic_tags(evaluation.rule_code_name),
            create_custom_property: None,
            cascade_narrowing,
            cascade_confidence,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }

    if deep_analysis {
        deduplicate_query_theory_hints_against_circular_var(
            &mut diagnostics,
            multiscale_complexity_heuristic_diagnostics,
            categorical_diagnostics,
        );
        // The SMT cascade-violation diagnostics are anchored on the specific
        // longhand declaration that breaks the combination obligation (not the
        // whole-file span the multiscale-complexity-heuristic/categorical hints use), so they are a
        // distinct, actionable diagnostic and are appended directly rather than
        // deduplicated against `circularVar`.
        diagnostics.extend(smt_diagnostics);
    }

    diagnostics
}

#[cfg(test)]
mod reach_tests;

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    fn recorded(source: &str) -> Vec<(String, String, String)> {
        collect_query_checker_cascade_declarations(source)
            .into_iter()
            .map(|declaration| {
                (
                    declaration.input.selector.into_string(),
                    declaration.input.property,
                    declaration.input.value,
                )
            })
            .collect()
    }

    fn diagnostic_codes(source: &str) -> Vec<&'static str> {
        summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            "file:///tmp/test.scss",
            source,
            false,
        )
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
    }

    #[test]
    fn standard_valid_value_fixture_has_no_invalid_property_diagnostics() {
        let source = include_str!("../../tests/fixtures/standard-value-grammar.css");
        let codes = diagnostic_codes(source);
        assert!(
            !codes.contains(&"invalidPropertyValue"),
            "valid declarations produced invalid-property-value diagnostics: {codes:?}"
        );
    }

    fn diagnostic_codes_with_deep_analysis(source: &str, deep_analysis: bool) -> Vec<&'static str> {
        summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            "file:///tmp/test.scss",
            source,
            deep_analysis,
        )
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
    }

    fn cascade_codes(source: &str) -> Vec<&'static str> {
        diagnostic_codes(source)
            .into_iter()
            .filter(|code| matches!(*code, "unreachableDeclaration" | "unspecifiedCascadeTie"))
            .collect()
    }

    fn layered_declaration(
        declaration_id: &str,
        selector: &str,
        property: &str,
        source_order: u32,
        layer_order: i32,
    ) -> OmenaCheckerCascadeDeclarationInputV0 {
        OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id: declaration_id.to_string(),
            selector: CanonicalSelector::from_canonical(selector),
            property: property.to_string(),
            value: "red".to_string(),
            source_order,
            condition_context: Vec::new(),
            layer_name: Some(format!("layer-{layer_order}")),
            layer_order: Some(layer_order),
            origin: omena_cascade::CascadeOriginV0::Author,
            important: false,
            var_references: Vec::new(),
        }
    }

    // ---- B1: comment poisoning ----------------------------------------

    #[test]
    fn b1_block_comment_before_property_does_not_drop_declaration() {
        let recorded = recorded(".a { /* primary */ color: red; color: blue; }");
        let properties: Vec<_> = recorded.iter().map(|(_, property, _)| property).collect();
        assert_eq!(properties, vec!["color", "color"], "{recorded:?}");
    }

    #[test]
    fn b1_block_comment_repro_fires_tie_and_unreachable() {
        let cascade = cascade_codes(".a { /* primary */ color: red; color: blue; }");
        assert!(
            cascade.contains(&"unreachableDeclaration")
                && cascade.contains(&"unspecifiedCascadeTie"),
            "expected both cascade diagnostics, got {cascade:?}"
        );
    }

    #[test]
    fn b1_line_comment_before_declarations_does_not_drop_them() {
        let cascade = cascade_codes(".a { // primary\ncolor: red; color: blue; }");
        assert!(
            cascade.contains(&"unreachableDeclaration")
                && cascade.contains(&"unspecifiedCascadeTie"),
            "expected both cascade diagnostics, got {cascade:?}"
        );
    }

    #[test]
    fn b1_value_comment_is_stripped_but_property_survives() {
        let recorded = recorded(".a { color /* c */ : red /* d */; }");
        assert_eq!(
            recorded,
            vec![(".a".to_string(), "color".to_string(), "red".to_string())],
            "comment-laden declaration should still record cleanly"
        );
    }

    // ---- B1 over-correction: commented-out declarations stay inert -----

    #[test]
    fn b1_line_commented_out_declaration_is_not_analyzed_as_live() {
        // The override is commented out, so there is no live duplicate / tie.
        let cascade = cascade_codes(".a { color: red; // color: blue;\n}");
        assert!(
            cascade.is_empty(),
            "commented-out decl must not tie: {cascade:?}"
        );
    }

    #[test]
    fn b1_block_commented_out_declaration_is_not_analyzed_as_live() {
        let cascade = cascade_codes(".a { color: red; /* color: blue; */ }");
        assert!(
            cascade.is_empty(),
            "commented-out decl must not tie: {cascade:?}"
        );
    }

    #[test]
    fn b1_url_with_double_slash_value_is_preserved_and_later_tie_still_fires() {
        // The `//` inside `url(http://…)` must not be treated as a line comment,
        // and the genuine `color` duplicate that follows must still tie.
        let source = ".a { background: url(http://example.com/a.png); color: red; color: blue; }";
        let recorded = recorded(source);
        assert!(
            recorded
                .iter()
                .any(|(_, property, value)| property == "background"
                    && value == "url(http://example.com/a.png)"),
            "url value should survive intact: {recorded:?}"
        );
        let cascade = cascade_codes(source);
        assert!(
            cascade.contains(&"unspecifiedCascadeTie"),
            "later real tie should still fire: {cascade:?}"
        );
    }

    // ---- B2: selector-list cross-rule tie -----------------------------

    #[test]
    fn b2_selector_list_member_records_separately() {
        let recorded = recorded(".a, .b { color: red; }");
        let selectors: Vec<_> = recorded.iter().map(|(selector, ..)| selector).collect();
        assert_eq!(selectors, vec![".a", ".b"], "{recorded:?}");
    }

    #[test]
    fn b2_selector_list_member_ties_with_sibling_rule() {
        let cascade = cascade_codes(".a, .b { color: red; }\n.a { color: blue; }");
        assert!(
            cascade.contains(&"unreachableDeclaration")
                && cascade.contains(&"unspecifiedCascadeTie"),
            "list member .a should tie with .a sibling: {cascade:?}"
        );
    }

    // ---- B2 over-correction: no spurious ties -------------------------

    #[test]
    fn b2_distinct_list_member_does_not_tie_with_unrelated_rule() {
        // `.a, .b` vs `.c` share no selector, so no tie may be reported.
        let cascade = cascade_codes(".a, .b { color: red; }\n.c { color: blue; }");
        assert!(
            cascade.is_empty(),
            "unrelated rule must not tie: {cascade:?}"
        );
    }

    #[test]
    fn b2_duplicate_member_in_one_prelude_is_deduplicated() {
        // `.a, .a` is a single rule; the duplicated member must not self-tie.
        let recorded = recorded(".a, .a { color: red; }");
        assert_eq!(
            recorded.len(),
            1,
            "identical members must be de-duplicated: {recorded:?}"
        );
        let cascade = cascade_codes(".a, .a { color: red; }");
        assert!(
            cascade.is_empty(),
            "deduped member must not self-tie: {cascade:?}"
        );
    }

    #[test]
    fn b2_comma_inside_functional_pseudo_is_not_split() {
        // The comma inside `:is(.a, .b)` is paren-protected, so the rule records
        // as a single opaque-compound selector rather than two bogus members.
        let recorded = recorded(":is(.a, .b) { color: red; }");
        let selectors: Vec<_> = recorded.iter().map(|(selector, ..)| selector).collect();
        assert_eq!(selectors, vec![":is(.a, .b)"], "{recorded:?}");
    }

    #[test]
    fn runtime_selector_filter_uses_conservative_co_match_axes() {
        assert!(query_runtime_selector_matches_anchor_classes(
            ".btn",
            "button.btn"
        ));
        assert!(query_runtime_selector_matches_anchor_classes(
            ".btn",
            ".btn.active"
        ));
        assert!(query_runtime_selector_matches_anchor_classes(
            ".btn:is(.active)",
            ".btn .icon"
        ));
        assert!(!query_runtime_selector_matches_anchor_classes(
            "div.btn", "span.btn"
        ));
        assert!(!query_runtime_selector_matches_anchor_classes(
            "#save", "#cancel"
        ));
    }

    #[test]
    fn layer_inversion_obligations_group_property_equal_co_matching_selectors_pairwise() {
        let declarations = vec![
            layered_declaration("base", ".btn", "color", 20, 0),
            layered_declaration("theme", "button.btn", "color", 10, 1),
            layered_declaration("other-property", "button.btn", "background", 30, 2),
        ];

        let obligations = query_smt_layer_inversion_obligations(&declarations);

        assert_eq!(obligations.len(), 1, "{obligations:?}");
        let layer_obligations = obligations
            .iter()
            .filter_map(|(obligation, _)| match obligation {
                QuerySmtCascadeObligation::LayerInversion(obligation) => Some(obligation),
                QuerySmtCascadeObligation::BoxShorthand(_) => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            layer_obligations.len(),
            1,
            "expected layer inversion obligation: {obligations:?}"
        );
        let obligation = layer_obligations[0];
        assert_eq!(
            obligation
                .declarations
                .iter()
                .map(|declaration| declaration.declaration_id.as_str())
                .collect::<Vec<_>>(),
            vec!["theme", "base"]
        );
    }

    #[test]
    fn layer_inversion_obligations_skip_disjoint_single_valued_axes() {
        let declarations = vec![
            layered_declaration("button", "button.btn", "color", 10, 0),
            layered_declaration("anchor", "a.btn", "color", 20, 1),
        ];

        let obligations = query_smt_layer_inversion_obligations(&declarations);

        assert!(
            obligations.is_empty(),
            "conflicting required tags must not compete: {obligations:?}"
        );
    }

    #[test]
    fn layer_inversion_obligations_keep_maybe_co_matches_competing() {
        let declarations = vec![
            layered_declaration("base", ".btn .icon", "color", 20, 0),
            layered_declaration("theme", ".btn:is(.active)", "color", 10, 1),
        ];

        let obligations = query_smt_layer_inversion_obligations(&declarations);

        assert_eq!(
            obligations.len(),
            1,
            "unsupported selector structure must stay possibly competing: {obligations:?}"
        );
    }

    // ---- WP7-b: de-noise multiscale-complexity-heuristic + categorical theory hints -----------

    /// A two-property custom-property reference cycle that the product chain
    /// flags as `circularVar`.
    const VAR_CYCLE_SOURCE: &str = ":root { --a: var(--b); --b: var(--a); }";

    #[test]
    fn wp7b_var_cycle_still_fires_circular_var_warning() {
        // Over-correction guard: the product `circularVar` warning must keep
        // firing on a real custom-property reference cycle regardless of the
        // deep-analysis flag — the dedup removes only the theory hints.
        for deep_analysis in [false, true] {
            let codes = diagnostic_codes_with_deep_analysis(VAR_CYCLE_SOURCE, deep_analysis);
            assert!(
                codes.contains(&"circularVar"),
                "circularVar must still fire on a real var cycle (deep_analysis={deep_analysis}): {codes:?}"
            );
        }
    }

    #[test]
    fn wp7b_default_surface_var_cycle_emits_only_circular_var() {
        // Default surface (deep-analysis OFF): a lone var cycle yields exactly the
        // `circularVar` warning and no whole-file-ranged theory hints.
        let codes = diagnostic_codes(VAR_CYCLE_SOURCE);
        assert!(
            codes.contains(&"circularVar"),
            "circularVar must fire on the default surface: {codes:?}"
        );
        assert!(
            !codes.contains(
                &crate::OMENA_QUERY_MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_DIAGNOSTIC_CODE_V0,
            ),
            "multiscale-complexity-heuristic theory hint must be OFF by default: {codes:?}"
        );
        assert!(
            !codes.contains(&"categoricalCascadeEvidenceInconsistency"),
            "categorical theory hint must be OFF by default: {codes:?}"
        );
        // No theory triple-fire: the cycle yields the product `circularVar`
        // warning (and any other product cascade diagnostics) but neither of the
        // two redundant, whole-file-ranged theory hints.
        assert!(
            codes.iter().all(|code| !matches!(
                *code,
                crate::OMENA_QUERY_MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_DIAGNOSTIC_CODE_V0
                    | "categoricalCascadeEvidenceInconsistency"
            )),
            "default surface must surface no theory hints for a lone var cycle: {codes:?}"
        );
    }

    #[test]
    fn wp7b_deep_analysis_dedups_theory_hints_into_circular_var() -> Result<(), &'static str> {
        // Deep-analysis ON: the multiscale-complexity-heuristic + categorical hints key off the same
        // reference-cycle predicate as `circularVar`, so they are deduplicated
        // (folded into `circularVar`'s provenance) rather than triple-firing.
        let codes = diagnostic_codes_with_deep_analysis(VAR_CYCLE_SOURCE, true);
        assert!(
            codes.contains(&"circularVar"),
            "circularVar must fire with deep analysis ON: {codes:?}"
        );
        assert!(
            !codes.contains(
                &crate::OMENA_QUERY_MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_DIAGNOSTIC_CODE_V0,
            ),
            "multiscale-complexity-heuristic hint must be deduplicated against circularVar: {codes:?}"
        );
        assert!(
            !codes.contains(&"categoricalCascadeEvidenceInconsistency"),
            "categorical hint must be deduplicated against circularVar: {codes:?}"
        );

        // The suppressed theory mechanisms' provenance is merged into the
        // surviving `circularVar` diagnostic so the audit trail is preserved.
        let diagnostics = summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            "file:///tmp/test.scss",
            VAR_CYCLE_SOURCE,
            true,
        );
        let circular_var = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "circularVar")
            .ok_or("circularVar diagnostic must exist")?;
        assert!(
            crate::OMENA_QUERY_MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_PROVENANCE_V0
                .iter()
                .all(|label| circular_var.provenance.contains(label)),
            "the existing query path must fold its exact compatibility provenance into circularVar: {:?}",
            circular_var.provenance
        );
        assert!(
            circular_var
                .provenance
                .iter()
                .any(|label| label.contains("categorical")),
            "categorical provenance should be folded into circularVar: {:?}",
            circular_var.provenance
        );
        Ok(())
    }

    #[test]
    fn wp7b_deep_analysis_reaches_theory_gate_on_cyclic_input() {
        // With deep-analysis ON the theory producers are reachable (the gate runs)
        // even though their output is deduplicated here: the multiscale-complexity-heuristic coupling and
        // categorical mapping are both populated for a cyclic stylesheet, so the
        // underlying mechanisms still execute (proving the opt-in path is live).
        let collection =
            collect_query_checker_cascade_input("file:///tmp/test.scss", VAR_CYCLE_SOURCE);
        let multiscale_complexity_heuristic =
            summarize_query_multiscale_complexity_heuristic_coupling_diagnostics(
                VAR_CYCLE_SOURCE,
                &collection.checker_input.custom_properties,
            );
        let categorical = summarize_query_categorical_cascade_evidence_diagnostics(
            VAR_CYCLE_SOURCE,
            &collection.checker_input.custom_properties,
        );
        assert!(
            !multiscale_complexity_heuristic.is_empty(),
            "multiscale-complexity-heuristic theory gate should fire on a cyclic stylesheet when reached"
        );
        assert!(
            !categorical.is_empty(),
            "categorical theory gate should fire on a cyclic stylesheet when reached"
        );
    }

    #[test]
    fn wp7b_acyclic_stylesheet_emits_no_theory_hints_even_with_deep_analysis() {
        // Over-correction guard (the other direction): an acyclic custom-property
        // graph must not spuriously surface a theory hint under deep analysis.
        let acyclic = ":root { --a: 1px; --b: var(--a); }";
        let codes = diagnostic_codes_with_deep_analysis(acyclic, true);
        assert!(
            !codes.contains(
                &crate::OMENA_QUERY_MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_DIAGNOSTIC_CODE_V0,
            )
                && !codes.contains(&"categoricalCascadeEvidenceInconsistency"),
            "acyclic stylesheet must not surface theory hints: {codes:?}"
        );
    }

    #[test]
    fn wp7b_acyclic_high_gain_hub_surfaces_standalone_multiscale_complexity_heuristic_hint() {
        let high_gain = r#"
:root {
  --seed: 1px;
  --a: var(--seed);
  --b: var(--seed);
  --c: var(--seed);
  --d: var(--seed);
}
"#;

        let default_codes = diagnostic_codes_with_deep_analysis(high_gain, false);
        assert!(
            !default_codes.contains(
                &crate::OMENA_QUERY_MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_DIAGNOSTIC_CODE_V0,
            ),
            "multiscale-complexity-heuristic theory hint must stay off on the default surface: {default_codes:?}"
        );

        let deep_codes = diagnostic_codes_with_deep_analysis(high_gain, true);
        assert!(
            deep_codes.contains(
                &crate::OMENA_QUERY_MULTISCALE_COMPLEXITY_HEURISTIC_COMPATIBILITY_DIAGNOSTIC_CODE_V0,
            ),
            "acyclic high-gain hub should surface a standalone multiscale-complexity-heuristic hint: {deep_codes:?}"
        );
        assert!(
            !deep_codes.contains(&"circularVar"),
            "standalone multiscale-complexity-heuristic hint must not depend on circularVar: {deep_codes:?}"
        );
    }
}
