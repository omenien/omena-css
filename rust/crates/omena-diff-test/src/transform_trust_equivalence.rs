use std::error::Error;

use omena_benchmarks::{bundler_productization_corpus, style_corpus};
use omena_transform_cst::all_transform_pass_kinds;
use omena_transform_passes::{
    TransformExecutionContextV0, execute_transform_passes_on_source_with_dialect_and_context,
    execute_transform_passes_on_source_with_dialect_and_context_without_semantic_trust_for_measurement,
};
use serde_json::Value;

fn remove_semantic_trust_fields(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                remove_semantic_trust_fields(value);
            }
        }
        Value::Object(fields) => {
            fields.remove("semanticGuaranteeTier");
            fields.remove("winnerEqualityObligations");
            for value in fields.values_mut() {
                remove_semantic_trust_fields(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[test]
fn semantic_trust_recording_does_not_change_transform_corpus_behavior() -> Result<(), Box<dyn Error>>
{
    let requested = all_transform_pass_kinds();
    let context = TransformExecutionContextV0::default();
    let samples = style_corpus()
        .into_iter()
        .chain(bundler_productization_corpus())
        .collect::<Vec<_>>();
    let mut recorded_tier_count = 0usize;
    let mut winner_obligation_count = 0usize;

    assert!(
        !samples.is_empty(),
        "shared transform corpus must be non-empty"
    );

    for sample in samples {
        let recorded = execute_transform_passes_on_source_with_dialect_and_context(
            sample.source.as_str(),
            sample.dialect,
            &requested,
            &context,
        );
        let omitted =
            execute_transform_passes_on_source_with_dialect_and_context_without_semantic_trust_for_measurement(
                sample.source.as_str(),
                sample.dialect,
                &requested,
                &context,
            );

        recorded_tier_count += recorded
            .decisions
            .iter()
            .filter(|decision| decision.semantic_guarantee_tier().is_some())
            .count();
        winner_obligation_count += recorded.winner_equality_obligations.len();
        assert!(
            omitted
                .decisions
                .iter()
                .all(|decision| decision.semantic_guarantee_tier().is_none()),
            "measurement arm must omit trust records for {}",
            sample.name
        );
        assert_eq!(
            recorded.output_css, omitted.output_css,
            "trust recording changed emitted CSS for {}",
            sample.name
        );

        let mut recorded_json = serde_json::to_value(&recorded)?;
        let mut omitted_json = serde_json::to_value(&omitted)?;
        remove_semantic_trust_fields(&mut recorded_json);
        remove_semantic_trust_fields(&mut omitted_json);
        assert_eq!(
            recorded_json, omitted_json,
            "trust recording changed transform admission or execution for {}",
            sample.name
        );
    }

    assert!(
        recorded_tier_count > 0,
        "the shared transform corpus must exercise at least one semantic trust record"
    );
    assert!(
        winner_obligation_count > 0,
        "the shared transform corpus must exercise the cascade winner observer"
    );
    Ok(())
}
