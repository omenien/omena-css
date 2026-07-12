//! Cascade conformance verdicts for transform passes against recorded oracles.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassCascadeConformanceVerdictV0 {
    ModelConformant,
    DivergentWithReason,
    NotExercised,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassCascadeOracleV0 {
    DartSass,
    Wpt,
}

impl TransformPassCascadeOracleV0 {
    const fn id(self) -> &'static str {
        match self {
            Self::DartSass => "dart-sass",
            Self::Wpt => "wpt",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassCascadeConformanceRecordV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub record_key: String,
    pub pass_id: &'static str,
    pub pass_kind: TransformPassKind,
    pub pass_class: TransformPassClassV0,
    pub oracle: TransformPassCascadeOracleV0,
    pub fixture_id: String,
    pub property: String,
    pub observed_facts: Vec<ObservationKindV0>,
    pub preserved_facts: Vec<ObservationKindV0>,
    pub compared_facts: Vec<ObservationKindV0>,
    pub runtime_status: TransformPassRuntimeStatus,
    pub mutation_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_baseline_match: Option<bool>,
    pub comparison_performed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_match: Option<bool>,
    pub expected_value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<String>,
    pub verdict: TransformPassCascadeConformanceVerdictV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassCascadeConformanceFamilyReportV0 {
    pub pass_class: TransformPassClassV0,
    pub pass_count: usize,
    pub exercised_record_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_gap: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassCascadeConformanceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub pass_count: usize,
    pub case_count: usize,
    pub record_count: usize,
    pub model_conformant_count: usize,
    pub divergent_count: usize,
    pub not_exercised_count: usize,
    pub measured_comparison_count: usize,
    pub aggregate_policy: &'static str,
    pub named_gap_count: usize,
    pub all_passes_accounted_for: bool,
    pub all_records_have_one_verdict: bool,
    pub all_oracle_baselines_match: bool,
    pub all_verdicts_match_measurements: bool,
    pub all_divergences_reasoned: bool,
    pub all_families_non_vacuous_or_named_gap: bool,
    pub property_corpus_witness_earned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_corpus_witness: Option<EvidenceNodeSeedV0>,
    pub family_reports: Vec<TransformPassCascadeConformanceFamilyReportV0>,
    pub records: Vec<TransformPassCascadeConformanceRecordV0>,
}

pub fn summarize_transform_pass_cascade_conformance() -> TransformPassCascadeConformanceReportV0 {
    let oracle_cases = transform_pass_cascade_oracle_cases();
    let pass_kinds = all_transform_pass_kinds();
    let mut records = Vec::new();

    for pass_kind in pass_kinds {
        for oracle_case in &oracle_cases {
            records.push(transform_pass_cascade_conformance_record(
                pass_kind,
                oracle_case,
            ));
        }
    }

    let model_conformant_count = records
        .iter()
        .filter(|record| {
            record.verdict == TransformPassCascadeConformanceVerdictV0::ModelConformant
        })
        .count();
    let divergent_count = records
        .iter()
        .filter(|record| {
            record.verdict == TransformPassCascadeConformanceVerdictV0::DivergentWithReason
        })
        .count();
    let not_exercised_count = records
        .iter()
        .filter(|record| record.verdict == TransformPassCascadeConformanceVerdictV0::NotExercised)
        .count();
    let measured_comparison_count = records
        .iter()
        .filter(|record| record.comparison_performed)
        .count();
    let family_reports = transform_pass_cascade_conformance_family_reports(&records);
    let all_passes_accounted_for = pass_kinds.iter().all(|pass_kind| {
        records
            .iter()
            .any(|record| record.pass_kind == *pass_kind && record.pass_id == pass_kind.id())
    });
    let all_records_have_one_verdict =
        records.len() == model_conformant_count + divergent_count + not_exercised_count;
    let all_oracle_baselines_match = records
        .iter()
        .all(|record| record.oracle_baseline_match != Some(false));
    let all_verdicts_match_measurements = serialized_verdicts_match_measurements(&records);
    let all_divergences_reasoned = records.iter().all(|record| {
        record.verdict != TransformPassCascadeConformanceVerdictV0::DivergentWithReason
            || record
                .reason
                .as_ref()
                .is_some_and(|reason| !reason.trim().is_empty())
    });
    let all_families_non_vacuous_or_named_gap = family_reports
        .iter()
        .all(|report| report.exercised_record_count > 0 || report.named_gap.is_some());
    let witness_evidence = PropertyCorpusWitnessEvidenceV0 {
        record_count: records.len(),
        measured_comparison_count,
        all_records_have_one_verdict,
        all_oracle_baselines_match,
        all_verdicts_match_measurements,
        all_divergences_reasoned,
        all_passes_accounted_for,
        all_families_non_vacuous_or_named_gap,
    };
    let property_corpus_witness = transform_pass_cascade_conformance_witness(witness_evidence);

    TransformPassCascadeConformanceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.transform-pass-cascade-conformance",
        pass_count: pass_kinds.len(),
        case_count: oracle_cases.len(),
        record_count: records.len(),
        model_conformant_count,
        divergent_count,
        not_exercised_count,
        measured_comparison_count,
        aggregate_policy: "observationalCoverageSnapshot",
        named_gap_count: family_reports
            .iter()
            .filter(|report| report.named_gap.is_some())
            .count(),
        all_passes_accounted_for,
        all_records_have_one_verdict,
        all_oracle_baselines_match,
        all_verdicts_match_measurements,
        all_divergences_reasoned,
        all_families_non_vacuous_or_named_gap,
        property_corpus_witness_earned: property_corpus_witness.is_some(),
        property_corpus_witness,
        family_reports,
        records,
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SerializedConformanceVerdictInputV0 {
    comparison_performed: bool,
    oracle_match: Option<bool>,
    verdict: TransformPassCascadeConformanceVerdictV0,
}

fn serialized_verdicts_match_measurements(
    records: &[TransformPassCascadeConformanceRecordV0],
) -> bool {
    let Ok(encoded) = serde_json::to_vec(records) else {
        return false;
    };
    let Ok(serialized_records) =
        serde_json::from_slice::<Vec<SerializedConformanceVerdictInputV0>>(&encoded)
    else {
        return false;
    };
    serialized_records.iter().all(|record| {
        record.verdict
            == transform_pass_cascade_conformance_verdict(
                record.comparison_performed,
                record.oracle_match,
            )
    })
}

fn transform_pass_cascade_conformance_witness(
    evidence: PropertyCorpusWitnessEvidenceV0,
) -> Option<EvidenceNodeSeedV0> {
    let token = PropertyCorpusWitnessTokenV0::from_conformance_ledger(evidence)?;
    let record_count = evidence.record_count;
    let guarantee = GuaranteeKindV0::from_existing_label("fixtureWitnessMetricInput")
        .unwrap_or(GuaranteeKindV0::MetricInputFixtureWitness);

    Some(EvidenceNodeSeedV0::with_family(
        EvidenceNodeKeyV0::new(
            "omena-diff-test.transform-pass-cascade-conformance",
            format!("records:{record_count}"),
        ),
        vec![
            "property-corpus-witness:transform-pass-cascade-conformance".to_string(),
            format!("records:{record_count}"),
        ],
        guarantee,
        FamilyStampV0::property_corpus_witness(&token),
    ))
}

#[derive(Debug, Clone)]
struct TransformPassCascadeOracleCaseV0 {
    oracle: TransformPassCascadeOracleV0,
    fixture_id: String,
    source: String,
    property: String,
    expected_value: String,
    cascade_projection_supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransformPassCascadeSiteProjectionV0 {
    selector: String,
    value: String,
}

fn transform_pass_cascade_oracle_cases() -> Vec<TransformPassCascadeOracleCaseV0> {
    use external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json;

    let mut cases = BTreeMap::new();
    if let (Ok(chunk), Ok(capture)) = (
        serde_json::from_str::<ImportedSassSpecChunkV0>(SASS_SPEC_IMPORTED_CHUNK_SOURCE),
        serde_json::from_str::<SassSpecOracleCaptureV0>(SASS_SPEC_IMPORTED_ORACLE_CAPTURE_SOURCE),
    ) {
        let oracle_by_fixture_id = capture
            .records
            .into_iter()
            .map(|record| (record.fixture_id.clone(), record))
            .collect::<BTreeMap<_, _>>();
        for fixture in chunk.fixtures.into_iter().filter(|fixture| {
            fixture.expectation_kind.as_ref()
                == Some(&ExternalCorpusExpectationKindV1Json::StaticMustMatch)
        }) {
            let Some(expected_css) = fixture.expected_css.as_ref() else {
                continue;
            };
            let Some(oracle_record) = oracle_by_fixture_id.get(fixture.id.as_str()) else {
                continue;
            };
            if !oracle_record.compiled {
                continue;
            }
            let Some(value_pairs) = oracle_record.declaration_value_pairs.as_ref() else {
                continue;
            };
            for pair in value_pairs {
                let key = format!("dart-sass:{}:{}", fixture.id, pair.property);
                cases.insert(
                    key,
                    TransformPassCascadeOracleCaseV0 {
                        oracle: TransformPassCascadeOracleV0::DartSass,
                        fixture_id: fixture.id.clone(),
                        source: expected_css.clone(),
                        property: pair.property.clone(),
                        expected_value: pair.value.clone(),
                        cascade_projection_supported: true,
                    },
                );
            }
        }
    }

    for fixture in wpt_value_differential_fixtures() {
        if !wpt_values_agree(
            fixture.wpt_value.as_str(),
            fixture.wpt_expected_value.as_str(),
        ) {
            continue;
        }
        let key = format!("wpt:{}:{}", fixture.id, fixture.property);
        let cascade_projection_supported = !fixture.source.contains("{{");
        cases.insert(
            key,
            TransformPassCascadeOracleCaseV0 {
                oracle: TransformPassCascadeOracleV0::Wpt,
                fixture_id: fixture.id,
                source: fixture.source,
                property: fixture.property,
                expected_value: fixture.wpt_expected_value,
                cascade_projection_supported,
            },
        );
    }

    cases.into_values().collect()
}

fn transform_pass_cascade_conformance_record(
    pass_kind: TransformPassKind,
    oracle_case: &TransformPassCascadeOracleCaseV0,
) -> TransformPassCascadeConformanceRecordV0 {
    let pass_class = transform_pass_class(pass_kind);
    let (observed_facts, preserved_facts, contract_known) =
        transform_pass_observation_facts(pass_kind);
    let compared_facts = transform_pass_cascade_comparable_facts(
        preserved_facts.as_slice(),
        oracle_case.property.as_str(),
    );
    let execution = execute_transform_passes_on_source_with_dialect(
        oracle_case.source.as_str(),
        StyleDialect::Css,
        &[pass_kind],
    );
    let outcome = execution
        .outcomes
        .iter()
        .find(|outcome| outcome.pass_id == pass_kind.id());
    let runtime_status = outcome
        .map(|outcome| outcome.status)
        .unwrap_or(TransformPassRuntimeStatus::PlannedOnly);
    let mutation_count = outcome.map(|outcome| outcome.mutation_count).unwrap_or(0);
    let reference_site = transform_pass_cascade_site_projection(
        oracle_case.source.as_str(),
        oracle_case.property.as_str(),
    );
    let oracle_baseline_match = oracle_case.cascade_projection_supported.then(|| {
        reference_site.as_ref().is_some_and(|site| {
            transform_pass_cascade_values_match(
                oracle_case.oracle,
                oracle_case.property.as_str(),
                site.value.as_str(),
                oracle_case.expected_value.as_str(),
            )
        })
    });
    let actual_site = transform_pass_cascade_site_projection(
        execution.output_css.as_str(),
        oracle_case.property.as_str(),
    );
    let comparison_performed = contract_known
        && !compared_facts.is_empty()
        && oracle_case.cascade_projection_supported
        && runtime_status == TransformPassRuntimeStatus::Applied
        && mutation_count > 0
        && oracle_baseline_match == Some(true);
    let oracle_match = comparison_performed.then(|| {
        reference_site
            .as_ref()
            .zip(actual_site.as_ref())
            .is_some_and(|(reference, actual)| {
                reference.selector == actual.selector
                    && transform_pass_cascade_values_match(
                        oracle_case.oracle,
                        oracle_case.property.as_str(),
                        actual.value.as_str(),
                        oracle_case.expected_value.as_str(),
                    )
            })
    });
    let verdict = transform_pass_cascade_conformance_verdict(comparison_performed, oracle_match);
    let reason = match verdict {
        TransformPassCascadeConformanceVerdictV0::ModelConformant => None,
        TransformPassCascadeConformanceVerdictV0::DivergentWithReason => Some(format!(
            "transformed cascade site does not match the {} oracle value",
            oracle_case.oracle.id()
        )),
        TransformPassCascadeConformanceVerdictV0::NotExercised => Some(
            transform_pass_cascade_not_exercised_reason(
                contract_known,
                compared_facts.as_slice(),
                oracle_case.cascade_projection_supported,
                runtime_status,
                mutation_count,
                oracle_baseline_match,
            )
            .to_string(),
        ),
    };

    TransformPassCascadeConformanceRecordV0 {
        schema_version: "0",
        product: "omena-diff-test.transform-pass-cascade-conformance-record",
        record_key: format!(
            "{}:{}:{}:{}",
            oracle_case.oracle.id(),
            oracle_case.fixture_id,
            oracle_case.property,
            pass_kind.id()
        ),
        pass_id: pass_kind.id(),
        pass_kind,
        pass_class,
        oracle: oracle_case.oracle,
        fixture_id: oracle_case.fixture_id.clone(),
        property: oracle_case.property.clone(),
        observed_facts,
        preserved_facts,
        compared_facts,
        runtime_status,
        mutation_count,
        oracle_baseline_match,
        comparison_performed,
        oracle_match,
        expected_value: oracle_case.expected_value.clone(),
        actual_value: actual_site.map(|site| site.value),
        verdict,
        reason,
    }
}

fn transform_pass_observation_facts(
    pass_kind: TransformPassKind,
) -> (Vec<ObservationKindV0>, Vec<ObservationKindV0>, bool) {
    match pass_observation_contract(pass_kind) {
        PassObservationSurfaceV0::Declared(PassSemanticContractV0 {
            observes,
            preserves,
            ..
        }) => (observes, preserves, true),
        PassObservationSurfaceV0::UnknownGap { .. } => (Vec::new(), Vec::new(), false),
    }
}

fn transform_pass_cascade_comparable_facts(
    preserved_facts: &[ObservationKindV0],
    property: &str,
) -> Vec<ObservationKindV0> {
    preserved_facts
        .iter()
        .copied()
        .filter(|fact| match fact {
            ObservationKindV0::SelectorMatching
            | ObservationKindV0::CascadeWinner
            | ObservationKindV0::LayerRank
            | ObservationKindV0::Specificity
            | ObservationKindV0::DeclarationOrder => true,
            ObservationKindV0::Inheritance => {
                css_property_metadata_for_property(property).is_some()
            }
            ObservationKindV0::CustomPropertyComputedValue => property.starts_with("--"),
            _ => false,
        })
        .collect()
}

fn transform_pass_cascade_site_projection(
    source: &str,
    property: &str,
) -> Option<TransformPassCascadeSiteProjectionV0> {
    let mut matching = summarize_omena_query_cascade_site_outcomes_from_source(source)
        .into_iter()
        .filter(|outcome| outcome.property.eq_ignore_ascii_case(property));
    let first = matching.next()?;
    if matching.next().is_some() {
        return None;
    }
    Some(TransformPassCascadeSiteProjectionV0 {
        selector: first.selector,
        value: first.winning_value,
    })
}

fn transform_pass_cascade_values_match(
    oracle: TransformPassCascadeOracleV0,
    property: &str,
    actual: &str,
    expected: &str,
) -> bool {
    match oracle {
        TransformPassCascadeOracleV0::DartSass => sass_spec_css_values_match(actual, expected),
        TransformPassCascadeOracleV0::Wpt => {
            wpt_values_agree(actual, expected)
                || actual.trim().eq_ignore_ascii_case(expected.trim())
                || (property.eq_ignore_ascii_case("opacity")
                    && css_opacity_values_agree(actual, expected))
                || css_url_values_agree(actual, expected)
        }
    }
}

fn css_opacity_values_agree(actual: &str, expected: &str) -> bool {
    let Some(actual) = css_opacity_numeric_value(actual) else {
        return false;
    };
    let Some(expected) = css_opacity_numeric_value(expected) else {
        return false;
    };
    (actual - expected).abs() <= f64::EPSILON * 8.0
}

fn css_opacity_numeric_value(value: &str) -> Option<f64> {
    let reduced = reduce_static_math(value).unwrap_or_else(|| value.trim().to_string());
    let normalized = reduced.trim();
    if let Some(percent) = normalized.strip_suffix('%') {
        return percent
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| value / 100.0);
    }
    normalized.parse::<f64>().ok()
}

fn css_url_values_agree(actual: &str, expected: &str) -> bool {
    css_url_payload(actual)
        .zip(css_url_payload(expected))
        .is_some_and(|(actual, expected)| actual == expected)
}

fn css_url_payload(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let head = trimmed.get(..4)?;
    if !head.eq_ignore_ascii_case("url(") || !trimmed.ends_with(')') {
        return None;
    }
    let payload = trimmed[4..trimmed.len() - 1].trim();
    if payload.len() >= 2 {
        let first = payload.as_bytes()[0];
        let last = payload.as_bytes()[payload.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return Some(&payload[1..payload.len() - 1]);
        }
    }
    Some(payload)
}

fn transform_pass_cascade_conformance_verdict(
    comparison_performed: bool,
    oracle_match: Option<bool>,
) -> TransformPassCascadeConformanceVerdictV0 {
    match (comparison_performed, oracle_match) {
        (true, Some(true)) => TransformPassCascadeConformanceVerdictV0::ModelConformant,
        (true, Some(false)) => TransformPassCascadeConformanceVerdictV0::DivergentWithReason,
        _ => TransformPassCascadeConformanceVerdictV0::NotExercised,
    }
}

fn transform_pass_cascade_not_exercised_reason(
    contract_known: bool,
    compared_facts: &[ObservationKindV0],
    cascade_projection_supported: bool,
    runtime_status: TransformPassRuntimeStatus,
    mutation_count: usize,
    oracle_baseline_match: Option<bool>,
) -> &'static str {
    if !contract_known {
        "pass observation surface is a named gap"
    } else if compared_facts.is_empty() {
        "pass contract has no cascade fact measurable by this oracle"
    } else if !cascade_projection_supported {
        "oracle case requires harness substitution before cascade projection"
    } else if runtime_status == TransformPassRuntimeStatus::PlannedOnly {
        "pass requires execution context not supplied by this oracle case"
    } else if mutation_count == 0 {
        "oracle case does not drive a pass mutation"
    } else if oracle_baseline_match == Some(false) {
        "oracle CSS cannot be projected to one matching cascade site"
    } else {
        "oracle comparison was not performed"
    }
}

fn transform_pass_cascade_conformance_family_reports(
    records: &[TransformPassCascadeConformanceRecordV0],
) -> Vec<TransformPassCascadeConformanceFamilyReportV0> {
    [
        TransformPassClassV0::Structural,
        TransformPassClassV0::TextLocal,
        TransformPassClassV0::ModuleEvaluation,
        TransformPassClassV0::Emission,
    ]
    .into_iter()
    .map(|pass_class| {
        let pass_count = all_transform_pass_kinds()
            .iter()
            .filter(|pass_kind| transform_pass_class(**pass_kind) == pass_class)
            .count();
        let exercised_record_count = records
            .iter()
            .filter(|record| {
                record.pass_class == pass_class
                    && record.verdict != TransformPassCascadeConformanceVerdictV0::NotExercised
            })
            .count();
        TransformPassCascadeConformanceFamilyReportV0 {
            pass_class,
            pass_count,
            exercised_record_count,
            named_gap: (exercised_record_count == 0)
                .then_some("property oracle corpus has no exercising cases for this pass family"),
        }
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_evidence_graph::GuaranteeFamilyV0;

    #[test]
    fn transform_pass_cascade_conformance_ledger_covers_static_match_oracle() {
        let report = summarize_transform_pass_cascade_conformance();
        assert_eq!(
            report.product,
            "omena-diff-test.transform-pass-cascade-conformance"
        );
        assert_eq!(report.pass_count, 44);
        assert!(report.case_count >= 1);
        assert_eq!(report.record_count, report.pass_count * report.case_count);
        assert!(report.model_conformant_count >= 1, "{report:#?}");
        assert!(report.not_exercised_count >= 1, "{report:#?}");
        assert!(report.measured_comparison_count >= 1, "{report:#?}");
        assert!(report.all_passes_accounted_for, "{report:#?}");
        assert!(report.all_records_have_one_verdict, "{report:#?}");
        assert!(report.all_oracle_baselines_match, "{report:#?}");
        assert!(report.all_verdicts_match_measurements, "{report:#?}");
        assert!(report.all_divergences_reasoned, "{report:#?}");
        assert!(report.all_families_non_vacuous_or_named_gap, "{report:#?}");
        assert!(report.records.iter().any(|record| {
            record.oracle == TransformPassCascadeOracleV0::DartSass && record.comparison_performed
        }));
        assert!(report.records.iter().any(|record| {
            record.oracle == TransformPassCascadeOracleV0::Wpt && record.comparison_performed
        }));
        assert!(report.property_corpus_witness_earned, "{report:#?}");
        assert!(report.property_corpus_witness.is_some(), "{report:#?}");
        if let Some(witness) = report.property_corpus_witness.as_ref() {
            assert_eq!(witness.earned_via, GuaranteeFamilyV0::PropertyCorpusWitness);
            assert_eq!(
                witness.guarantee,
                GuaranteeKindV0::MetricInputFixtureWitness
            );
        }
        assert_eq!(report.family_reports.len(), 4);
        assert!(
            report
                .family_reports
                .iter()
                .all(|family| family.pass_count >= 1),
            "{report:#?}"
        );
    }

    #[test]
    fn cascade_value_comparison_respects_property_level_external_oracle_semantics() {
        assert!(transform_pass_cascade_values_match(
            TransformPassCascadeOracleV0::Wpt,
            "color",
            "currentColor",
            "currentcolor",
        ));
        assert!(transform_pass_cascade_values_match(
            TransformPassCascadeOracleV0::Wpt,
            "opacity",
            "0.5",
            "calc(50%)",
        ));
        assert!(transform_pass_cascade_values_match(
            TransformPassCascadeOracleV0::Wpt,
            "background-image",
            "url(asset.svg)",
            "url(\"asset.svg\")",
        ));
        assert!(!transform_pass_cascade_values_match(
            TransformPassCascadeOracleV0::Wpt,
            "color",
            "red",
            "blue",
        ));
    }

    #[test]
    fn inheritance_preservation_contract_rejects_a_changed_cascade_value() {
        let contract = match pass_observation_contract(TransformPassKind::ShorthandCombining) {
            PassObservationSurfaceV0::Declared(contract) => Some(contract),
            PassObservationSurfaceV0::UnknownGap { .. } => None,
        };
        assert!(contract.is_some());
        let Some(contract) = contract else {
            return;
        };
        let compared_facts = transform_pass_cascade_comparable_facts(&contract.preserves, "color");
        assert!(compared_facts.contains(&ObservationKindV0::Inheritance));

        let reference = transform_pass_cascade_site_projection(".a { color: red; }", "color");
        let changed = transform_pass_cascade_site_projection(".a { color: blue; }", "color");
        let oracle_match =
            reference
                .as_ref()
                .zip(changed.as_ref())
                .is_some_and(|(reference, changed)| {
                    reference.selector == changed.selector
                        && transform_pass_cascade_values_match(
                            TransformPassCascadeOracleV0::Wpt,
                            "color",
                            changed.value.as_str(),
                            reference.value.as_str(),
                        )
                });

        assert!(!oracle_match);
        assert_eq!(
            transform_pass_cascade_conformance_verdict(true, Some(oracle_match)),
            TransformPassCascadeConformanceVerdictV0::DivergentWithReason
        );
    }
}
