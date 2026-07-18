use omena_benchmarks::{bundler_productization_corpus, style_corpus};
use omena_transform_cst::all_transform_pass_kinds;
use omena_transform_passes::{
    TransformCascadeEnvironmentV0, TransformExecutionContextV0,
    TransformWinnerEqualityObligationV0, TransformWinnerEqualityObservationV0,
    TransformWinnerEqualityWitnessV0, execute_transform_passes_on_source_with_dialect_and_context,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformWinnerEqualityAuditFindingV0 {
    pub sample_name: String,
    pub pass_id: &'static str,
    pub property: String,
    pub input: TransformWinnerEqualityWitnessV0,
    pub output: TransformWinnerEqualityWitnessV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
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
            TransformWinnerEqualityObservationV0::ObservedEqual { .. } => {
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
    use omena_parser::StyleDialect;
    use omena_transform_cst::TransformPassKind;
    use omena_transform_passes::compare_transform_winner_equality_for_conformance_v0;

    use super::*;

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
}
