use omena_benchmarks::{bundler_productization_corpus, style_corpus};
use omena_transform_cst::all_transform_pass_kinds;
use omena_transform_passes::{
    TransformCascadeEnvironmentV0, TransformExecutionContextV0,
    TransformWinnerEqualityObligationV0, TransformWinnerEqualityObservationV0,
    TransformWinnerEqualityWitnessV0, execute_transform_passes_on_source_with_dialect_and_context,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// A transform observation where the selected cascade winner changed.
pub struct TransformWinnerEqualityAuditFindingV0 {
    pub sample_name: String,
    pub pass_id: &'static str,
    pub property: omena_syntax::ident::AuthoredPropertyTextV0,
    pub input: TransformWinnerEqualityWitnessV0,
    pub output: TransformWinnerEqualityWitnessV0,
}

impl PartialEq for TransformWinnerEqualityAuditFindingV0 {
    fn eq(&self, other: &Self) -> bool {
        self.sample_name == other.sample_name
            && self.pass_id == other.pass_id
            && self
                .property
                .to_property_name()
                .same_as(&other.property.to_property_name())
            && self.input == other.input
            && self.output == other.output
    }
}

impl Eq for TransformWinnerEqualityAuditFindingV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Aggregate winner-equality observations from the shared transform corpus.
pub struct TransformWinnerEqualityAuditReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub sample_count: usize,
    pub obligation_count: usize,
    pub observed_equal_count: usize,
    pub observed_different_count: usize,
    pub typed_absence_count: usize,
    pub findings: Vec<TransformWinnerEqualityAuditFindingV0>,
}

/// Audits transform-pass winner-equality obligations over the shared corpus.
pub fn summarize_transform_winner_equality_audit_v0() -> TransformWinnerEqualityAuditReportV0 {
    let samples = style_corpus()
        .into_iter()
        .chain(bundler_productization_corpus())
        .collect::<Vec<_>>();
    let requested = all_transform_pass_kinds();
    let context = TransformExecutionContextV0 {
        cascade_environment: Some(TransformCascadeEnvironmentV0::default()),
        ..TransformExecutionContextV0::default()
    };
    let mut report = empty_report(
        "omena-diff-test.transform-winner-equality-audit",
        samples.len(),
    );

    for sample in samples {
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            sample.source.as_str(),
            sample.dialect,
            requested.as_slice(),
            &context,
        );
        record_obligations(
            &mut report,
            sample.name,
            execution.winner_equality_obligations.as_slice(),
        );
    }
    report
}

fn empty_report(
    product: &'static str,
    sample_count: usize,
) -> TransformWinnerEqualityAuditReportV0 {
    TransformWinnerEqualityAuditReportV0 {
        schema_version: "0",
        product,
        sample_count,
        obligation_count: 0,
        observed_equal_count: 0,
        observed_different_count: 0,
        typed_absence_count: 0,
        findings: Vec::new(),
    }
}

fn record_obligations(
    report: &mut TransformWinnerEqualityAuditReportV0,
    sample_name: &str,
    obligations: &[TransformWinnerEqualityObligationV0],
) {
    report.obligation_count += obligations.len();
    for obligation in obligations {
        match &obligation.observation {
            TransformWinnerEqualityObservationV0::ObservedEqual { .. }
            | TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. } => {
                report.observed_equal_count += 1;
            }
            TransformWinnerEqualityObservationV0::ObservedDifferent { input, output, .. } => {
                report.observed_different_count += 1;
                report.findings.push(TransformWinnerEqualityAuditFindingV0 {
                    sample_name: sample_name.to_string(),
                    pass_id: obligation.pass_id,
                    property: obligation.affected_pair.property.clone(),
                    input: input.clone(),
                    output: output.clone(),
                });
            }
            TransformWinnerEqualityObservationV0::Absent { .. } => {
                report.typed_absence_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use omena_cascade::CascadeValue;
    use omena_parser::{StyleDialect, collect_parser_declaration_syntax_facts};
    use omena_query::{
        summarize_omena_query_style_diagnostics_for_file,
        summarize_omena_query_style_hover_candidates,
    };
    use omena_syntax::ident::PropertyNameV0;
    use omena_transform_cst::TransformPassKind;
    use omena_transform_passes::{
        compare_transform_winner_equality_for_conformance_v0,
        execute_transform_passes_on_source_with_dialect,
    };

    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct CascadeAgreementFixtureV0 {
        name: &'static str,
        path: &'static str,
        source: &'static str,
        dialect: StyleDialect,
        property: &'static str,
    }

    fn emitted_css(fixture: CascadeAgreementFixtureV0) -> String {
        execute_transform_passes_on_source_with_dialect(
            fixture.source,
            fixture.dialect,
            &[TransformPassKind::RuleMerging],
        )
        .output_css
    }

    fn diagnostic_plane_answers(
        fixture: CascadeAgreementFixtureV0,
    ) -> Result<Vec<Option<String>>, String> {
        let candidates = summarize_omena_query_style_hover_candidates(fixture.path, fixture.source)
            .ok_or_else(|| format!("{} has no hover candidate carrier", fixture.name))?;
        let diagnostics = summarize_omena_query_style_diagnostics_for_file(
            fixture.path,
            fixture.source,
            candidates.candidates.as_slice(),
        );
        let answers = diagnostics
            .diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.cascade_narrowing.as_ref())
            .filter(|narrowing| {
                narrowing.property_key
                    == PropertyNameV0::from_authored(fixture.property).canonical_key()
            })
            .filter_map(|narrowing| narrowing.runtime_state.as_ref())
            .flat_map(|runtime| runtime.scenarios.iter())
            .map(|scenario| scenario.winner_value.clone())
            .collect::<Vec<_>>();
        if answers.is_empty() {
            let observed = diagnostics
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    (
                        diagnostic.code,
                        diagnostic.cascade_narrowing.as_ref().map(|narrowing| {
                            let mut property = String::new();
                            let _ = omena_syntax::ident::render_authored(
                                &narrowing.property_name,
                                &mut property,
                            );
                            property
                        }),
                    )
                })
                .collect::<Vec<_>>();
            return Err(format!(
                "{} has no diagnostics-plane cascade answer: {observed:?}",
                fixture.name,
            ));
        }
        Ok(answers)
    }

    fn normalized_witness_value(
        source: &str,
        dialect: StyleDialect,
        property: &str,
        winner: &TransformWinnerEqualityWitnessV0,
    ) -> Result<String, String> {
        let CascadeValue::Literal(raw_value) = &winner.winner.value else {
            return Err(format!("{property} has a non-literal winner"));
        };
        collect_parser_declaration_syntax_facts(source, dialect)
            .into_iter()
            .find(|fact| {
                fact.property_name.to_property_name().same_as(
                    &omena_syntax::ident::PropertyNameV0::from_authored(property),
                ) && source
                    .get(fact.byte_span.start..fact.byte_span.end)
                    .is_some_and(|declaration| declaration.contains(raw_value))
            })
            .map(|fact| fact.value_text)
            .ok_or_else(|| {
                format!(
                    "parser facts cannot map authority winner {raw_value:?} for property {property}"
                )
            })
    }

    fn assert_reconciled_winner_agreement(
        fixture: CascadeAgreementFixtureV0,
        seeded_diagnostic_answer: Option<&str>,
    ) -> Result<(), String> {
        let emission = emitted_css(fixture);
        let obligations = compare_transform_winner_equality_for_conformance_v0(
            fixture.source,
            emission.as_str(),
            fixture.dialect,
            TransformPassKind::RuleMerging,
        );
        let mut diagnostic_answers = diagnostic_plane_answers(fixture)?;
        if let Some(seed) = seeded_diagnostic_answer {
            diagnostic_answers = vec![Some(seed.to_string())];
        }
        let observation = obligations
            .iter()
            .find(|obligation| {
                obligation
                    .affected_pair
                    .property
                    .to_property_name()
                    .same_as(&omena_syntax::ident::PropertyNameV0::from_authored(
                        fixture.property,
                    ))
            })
            .map(|obligation| &obligation.observation)
            .ok_or_else(|| format!("{} has no emission-plane obligation", fixture.name))?;
        if fixture.name == "guarded-winner-reconciliation"
            && !matches!(
                observation,
                TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. }
            )
        {
            return Err(format!(
                "{} did not consume the guarded-plane reconciliation authority: {observation:?}",
                fixture.name
            ));
        }
        if fixture.name == "ranked-set-refusal"
            && !matches!(
                observation,
                TransformWinnerEqualityObservationV0::Absent { .. }
            )
        {
            return Err(format!(
                "{} must preserve the typed refusal instead of claiming agreement: {observation:?}",
                fixture.name
            ));
        }
        match observation {
            TransformWinnerEqualityObservationV0::ObservedEqual { input, output, .. }
            | TransformWinnerEqualityObservationV0::ObservedGuardedEqual {
                input, output, ..
            } => {
                let input_value = normalized_witness_value(
                    fixture.source,
                    fixture.dialect,
                    fixture.property,
                    input,
                )?;
                let output_value = normalized_witness_value(
                    emission.as_str(),
                    fixture.dialect,
                    fixture.property,
                    output,
                )?;
                if input_value != output_value {
                    return Err(format!(
                        "{} emission planes disagree: {input_value:?} != {output_value:?}",
                        fixture.name
                    ));
                }
                if !diagnostic_answers
                    .iter()
                    .flatten()
                    .any(|answer| answer == &input_value)
                {
                    return Err(format!(
                        "{} diagnostics answers {diagnostic_answers:?} disagree with parser-normalized authority winner {input_value:?}",
                        fixture.name,
                    ));
                }
            }
            TransformWinnerEqualityObservationV0::Absent { .. } => {
                if diagnostic_answers.iter().any(Option::is_some) {
                    return Err(format!(
                        "{} emission authority refused while diagnostics claimed {diagnostic_answers:?}",
                        fixture.name
                    ));
                }
            }
            TransformWinnerEqualityObservationV0::ObservedDifferent { input, output, .. } => {
                return Err(format!(
                    "{} emission planes disagree: input={input:?} output={output:?}",
                    fixture.name
                ));
            }
        }
        Ok(())
    }

    fn cascade_input_authority_fixtures() -> [CascadeAgreementFixtureV0; 6] {
        [
            CascadeAgreementFixtureV0 {
                name: "url-semicolon",
                path: "url-semicolon.css",
                source: include_str!(
                    "../../omena-query/src/tests/fixtures/cascade-input-authority/url-semicolon.css"
                ),
                dialect: StyleDialect::Css,
                property: "background-image",
            },
            CascadeAgreementFixtureV0 {
                name: "url-brace",
                path: "url-brace.css",
                source: include_str!(
                    "../../omena-query/src/tests/fixtures/cascade-input-authority/url-brace.css"
                ),
                dialect: StyleDialect::Css,
                property: "background-image",
            },
            CascadeAgreementFixtureV0 {
                name: "comment-adjacent",
                path: "comment-adjacent.scss",
                source: include_str!(
                    "../../omena-query/src/tests/fixtures/cascade-input-authority/comment-adjacent.scss"
                ),
                dialect: StyleDialect::Scss,
                property: "color",
            },
            CascadeAgreementFixtureV0 {
                name: "comment-leading-cross-rule",
                path: "comment-leading-cross-rule.scss",
                source: include_str!(
                    "../../omena-query/src/tests/fixtures/cascade-input-authority/comment-leading-cross-rule.scss"
                ),
                dialect: StyleDialect::Scss,
                property: "color",
            },
            CascadeAgreementFixtureV0 {
                name: "guarded-winner-reconciliation",
                path: "guarded-winner.css",
                source: "@media (min-width: 1px) { .target { color: red; color: blue; } }",
                dialect: StyleDialect::Css,
                property: "color",
            },
            CascadeAgreementFixtureV0 {
                name: "ranked-set-refusal",
                path: "ranked-set-refusal.css",
                source: ":is(:unknown(.target), .target) { color: red; color: blue; }",
                dialect: StyleDialect::Css,
                property: "color",
            },
        ]
    }

    #[test]
    fn cascade_input_authority_corpus_reconciles_diagnostics_and_emission_winners()
    -> Result<(), String> {
        for fixture in cascade_input_authority_fixtures() {
            let seed = (std::env::var_os("OMENA_DIFF_CASCADE_INPUT_INJECT_SCANNER_MIS_SPLIT")
                .is_some()
                && fixture.name == "url-semicolon")
                .then_some("url(clean.png)");
            assert_reconciled_winner_agreement(fixture, seed)?;
        }
        Ok(())
    }

    #[test]
    fn seeded_scanner_style_mis_split_is_rejected_by_winner_agreement() -> Result<(), String> {
        let fixture = cascade_input_authority_fixtures()[0];
        let Err(error) = assert_reconciled_winner_agreement(fixture, Some("url(clean.png)")) else {
            return Err("dropping the important URL winner did not violate plane agreement".into());
        };
        if !error.contains("disagree with parser-normalized authority winner") {
            return Err(format!("unexpected seeded disagreement: {error}"));
        }
        Ok(())
    }

    #[test]
    fn shared_transform_corpus_has_no_observed_winner_flip() {
        let report = summarize_transform_winner_equality_audit_v0();

        eprintln!(
            "winner equality audit: samples={} obligations={} equal={} different={} absent={}",
            report.sample_count,
            report.obligation_count,
            report.observed_equal_count,
            report.observed_different_count,
            report.typed_absence_count
        );
        assert!(report.sample_count > 0);
        assert!(report.obligation_count > 0);
        assert_eq!(report.observed_different_count, 0, "{:#?}", report.findings);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn audit_classifier_detects_a_known_layer_order_flip() {
        let input = "@layer low, high; @layer low { .a { color: red; } } @layer high { .a { color: blue; } }";
        let output = "@layer high, low; @layer low { .a { color: red; } } @layer high { .a { color: blue; } }";
        let obligations = compare_transform_winner_equality_for_conformance_v0(
            input,
            output,
            StyleDialect::Css,
            TransformPassKind::LayerFlatten,
        );
        let mut report = empty_report(
            "omena-diff-test.transform-winner-equality-audit-calibration",
            1,
        );
        record_obligations(&mut report, "layer-order-flip", obligations.as_slice());

        assert!(report.observed_different_count > 0);
        assert!(!report.findings.is_empty());
    }

    #[test]
    fn winner_equality_audit_finding_identity_uses_standard_property_keys() -> Result<(), String> {
        let input = "@layer low, high; @layer low { .a { color: red; } } @layer high { .a { color: blue; } }";
        let output = "@layer high, low; @layer low { .a { color: red; } } @layer high { .a { color: blue; } }";
        let obligations = compare_transform_winner_equality_for_conformance_v0(
            input,
            output,
            StyleDialect::Css,
            TransformPassKind::LayerFlatten,
        );
        let mut report = empty_report("identity-fixture", 1);
        record_obligations(&mut report, "layer-order-flip", obligations.as_slice());
        let decoded = report
            .findings
            .first()
            .ok_or_else(|| "known layer-order flip emits a finding".to_string())?
            .clone();
        let mut escaped = decoded.clone();
        escaped.property = omena_syntax::ident::AuthoredPropertyTextV0::new(r"C\4f LOR");

        assert_eq!(escaped, decoded);
        Ok(())
    }

    #[test]
    fn audit_classifier_counts_guarded_authority_as_observed_equality() {
        let source = "@media (min-width: 1px) { .a { color: red; } }";
        let obligations = compare_transform_winner_equality_for_conformance_v0(
            source,
            source,
            StyleDialect::Css,
            TransformPassKind::RuleMerging,
        );
        let expected_equal_count = obligations
            .iter()
            .filter(|obligation| {
                matches!(
                    &obligation.observation,
                    TransformWinnerEqualityObservationV0::ObservedEqual { .. }
                        | TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. }
                )
            })
            .count();
        assert!(obligations.iter().any(|obligation| matches!(
            &obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. }
        )));

        let mut report = empty_report(
            "omena-diff-test.transform-winner-equality-audit-guarded-calibration",
            1,
        );
        record_obligations(&mut report, "guarded-equality", obligations.as_slice());

        assert_eq!(report.observed_equal_count, expected_equal_count);
        assert_eq!(report.observed_different_count, 0);
        assert!(report.findings.is_empty());
    }
}
