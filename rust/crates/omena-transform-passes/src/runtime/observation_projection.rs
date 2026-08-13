//! Executable observation projections derived from real transform outputs.

use omena_parser::{ParsedSelectorFactKind, StyleDialect, collect_style_facts};
use omena_transform_cst::{
    ObservationKindV0, TransformObservationEquivalenceV0, TransformPassKind,
    compare_raw_transform_observation_bytes_v0, compare_transform_observation_projection_values_v0,
};
use serde::Serialize;

use crate::{
    TransformWinnerEqualityObservationV0, execute_transform_passes_on_source,
    runtime::winner_equality::compare_transform_winner_equality_for_conformance_v0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutableObservationProjectionV0 {
    pub kind: ObservationKindV0,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutableObservationReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_id: &'static str,
    pub pass_executed: bool,
    pub input_projection: TransformExecutableObservationProjectionV0,
    pub output_projection: TransformExecutableObservationProjectionV0,
    pub projection_equivalence: TransformObservationEquivalenceV0,
    pub raw_equivalence: TransformObservationEquivalenceV0,
    pub output_css: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutableCascadeWinnerEquivalenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub profile_id: &'static str,
    pub compared_obligation_count: usize,
    pub equivalent: bool,
}

/// Execute one product pass and project selector matching from parser-owned
/// selector facts on both the input and the actual pass output.
pub fn execute_transform_pass_selector_matching_observation_v0(
    source: &str,
    pass: TransformPassKind,
) -> TransformExecutableObservationReportV0 {
    let execution = execute_transform_passes_on_source(source, &[pass]);
    let input_projection = selector_matching_projection_v0(source);
    let output_projection = selector_matching_projection_v0(execution.output_css.as_str());
    TransformExecutableObservationReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.executable-observation-projection",
        pass_id: pass.id(),
        pass_executed: execution.executed_pass_ids.contains(&pass.id()),
        projection_equivalence: compare_transform_observation_projection_values_v0(
            "selector-matching-executable-v0",
            ObservationKindV0::SelectorMatching,
            input_projection.as_str(),
            output_projection.as_str(),
        ),
        raw_equivalence: compare_raw_transform_observation_bytes_v0(
            "raw-bytes-executable-v0",
            source.as_bytes(),
            execution.output_css.as_bytes(),
        ),
        input_projection: TransformExecutableObservationProjectionV0 {
            kind: ObservationKindV0::SelectorMatching,
            value: input_projection,
        },
        output_projection: TransformExecutableObservationProjectionV0 {
            kind: ObservationKindV0::SelectorMatching,
            value: output_projection,
        },
        output_css: execution.output_css,
    }
}

/// Compare cascade winners using the cascade authority's executable
/// conformance projection. Empty or absent observations are conservative RED.
pub fn compare_transform_cascade_winner_observation_v0(
    input: &str,
    output: &str,
    pass: TransformPassKind,
) -> TransformExecutableCascadeWinnerEquivalenceV0 {
    let obligations = compare_transform_winner_equality_for_conformance_v0(
        input,
        output,
        StyleDialect::Css,
        pass,
    );
    let equivalent = !obligations.is_empty()
        && obligations.iter().all(|obligation| {
            matches!(
                obligation.observation,
                TransformWinnerEqualityObservationV0::ObservedEqual { .. }
            )
        });
    TransformExecutableCascadeWinnerEquivalenceV0 {
        schema_version: "0",
        product: "omena-transform-passes.executable-cascade-winner-equivalence",
        profile_id: "cascade-winner-executable-v0",
        compared_obligation_count: obligations.len(),
        equivalent,
    }
}

fn selector_matching_projection_v0(source: &str) -> String {
    let mut selectors = collect_style_facts(source, StyleDialect::Css)
        .selectors
        .into_iter()
        .map(|selector| {
            let kind = match selector.kind {
                ParsedSelectorFactKind::Class => "class",
                ParsedSelectorFactKind::Id => "id",
                ParsedSelectorFactKind::Placeholder => "placeholder",
            };
            format!("{kind}:{}", selector.name)
        })
        .collect::<Vec<_>>();
    selectors.sort();
    selectors.dedup();
    selectors.join("\n")
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ExecutableTruthRowV0 {
        case_id: String,
        pass_id: String,
        observation_kind: String,
        input_css: String,
        candidate_output_css: Option<String>,
        expected_equivalent: bool,
    }

    #[test]
    fn executable_truth_table_runs_product_pass_and_authority_projections()
    -> Result<(), Box<dyn std::error::Error>> {
        let rows = serde_json::from_str::<Vec<ExecutableTruthRowV0>>(include_str!(
            "../../data/observation-executable-truth-table-v0.json"
        ))?;
        let mut positive_rows = 0usize;
        for row in rows {
            if row.pass_id != TransformPassKind::WhitespaceStrip.id() {
                return Err(format!("unknown executable pass in {}", row.case_id).into());
            }
            let report = execute_transform_pass_selector_matching_observation_v0(
                row.input_css.as_str(),
                TransformPassKind::WhitespaceStrip,
            );
            assert!(report.pass_executed, "{}", row.case_id);
            let equivalent = match row.observation_kind.as_str() {
                "selectorMatching" => report.projection_equivalence.equivalent,
                "rawBytes" => report.raw_equivalence.equivalent,
                "cascadeWinner" => {
                    compare_transform_cascade_winner_observation_v0(
                        row.input_css.as_str(),
                        row.candidate_output_css
                            .as_deref()
                            .unwrap_or(report.output_css.as_str()),
                        TransformPassKind::WhitespaceStrip,
                    )
                    .equivalent
                }
                other => return Err(format!("unknown executable observer: {other}").into()),
            };
            assert_eq!(equivalent, row.expected_equivalent, "{}", row.case_id);
            positive_rows += usize::from(equivalent);
        }
        assert!(
            positive_rows > 0,
            "executable truth table has no positive row"
        );
        Ok(())
    }

    #[test]
    fn r15_whitespace_pass_is_selector_equivalent_but_not_raw_equivalent() {
        let report = execute_transform_pass_selector_matching_observation_v0(
            ".a { color: red; }",
            TransformPassKind::WhitespaceStrip,
        );
        println!(
            "selectorProjection passExecuted={} pass={} selectorMatching={} rawBytes={} output={:?}",
            report.pass_executed,
            report.pass_id,
            report.projection_equivalence.equivalent,
            report.raw_equivalence.equivalent,
            report.output_css,
        );
        assert!(report.pass_executed);
        assert!(report.projection_equivalence.equivalent);
        assert!(!report.raw_equivalence.equivalent);
    }

    #[test]
    fn r16_executed_pass_baseline_detects_cascade_winner_corruption() {
        let source = ".a { color: red; } .a { color: blue; }";
        let report = execute_transform_pass_selector_matching_observation_v0(
            source,
            TransformPassKind::WhitespaceStrip,
        );
        let pass_relation = compare_transform_cascade_winner_observation_v0(
            source,
            report.output_css.as_str(),
            TransformPassKind::WhitespaceStrip,
        );
        let corrupted = ".a{color:blue}.a{color:red}";
        let corrupted_relation = compare_transform_cascade_winner_observation_v0(
            source,
            corrupted,
            TransformPassKind::WhitespaceStrip,
        );
        println!(
            "cascadeWinnerProjection passExecuted={} pass={} passCascadeWinner={} corruptedCascadeWinner={} disjointSelectorMatching={} passOutput={:?}",
            report.pass_executed,
            report.pass_id,
            pass_relation.equivalent,
            corrupted_relation.equivalent,
            report.projection_equivalence.equivalent,
            report.output_css,
        );
        assert!(report.pass_executed);
        assert!(pass_relation.equivalent);
        assert!(!corrupted_relation.equivalent);
        assert!(report.projection_equivalence.equivalent);
    }
}
