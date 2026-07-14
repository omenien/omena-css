use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const SASS_UNSUPPORTED_LEDGER_PRODUCT_VIEW_JSON: &str =
    include_str!("../data/sass-unsupported-ledger.json");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonicalSassUnsupportedLedgerReportV0 {
    schema_version: String,
    product: String,
    semantic_site_count: usize,
    ledger_semantic_site_count: usize,
    raw_pattern_hit_count: usize,
    non_semantic_pattern_hit_count: usize,
    linked_site_count: usize,
    named_gap_site_count: usize,
    linked_case_count: usize,
    ledger_metadata_valid: bool,
    all_semantic_sites_match_ledger: bool,
    all_sites_linked_or_named_gap: bool,
    all_linked_cases_match_reason_class: bool,
    all_linked_cases_are_imported_sound_bail_cases: bool,
    all_bail_site_ledger_checks_hold: bool,
    records: Vec<OmenaQuerySassUnsupportedLedgerRecordV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassUnsupportedLedgerRecordV0 {
    pub file: String,
    pub ordinal: usize,
    pub reason: String,
    pub current_line: Option<usize>,
    pub ledger_line_hint: usize,
    pub present_in_current_sources: bool,
    pub linked_fixture_ids: Vec<String>,
    pub gap: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassUnsupportedCountV0 {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassUnsupportedLedgerViewV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_product: String,
    pub semantic_site_count: usize,
    pub ledger_semantic_site_count: usize,
    pub surface_record_count: usize,
    pub raw_pattern_hit_count: usize,
    pub non_semantic_pattern_hit_count: usize,
    pub linked_site_count: usize,
    pub named_gap_site_count: usize,
    pub linked_case_count: usize,
    pub ledger_metadata_valid: bool,
    pub all_semantic_sites_match_ledger: bool,
    pub all_sites_linked_or_named_gap: bool,
    pub all_linked_cases_match_reason_class: bool,
    pub all_linked_cases_are_imported_sound_bail_cases: bool,
    pub all_bail_site_ledger_checks_hold: bool,
    pub surface_matches_ledger: bool,
    pub summary_view_ready: bool,
    pub file_counts: Vec<OmenaQuerySassUnsupportedCountV0>,
    pub reason_counts: Vec<OmenaQuerySassUnsupportedCountV0>,
    pub records: Vec<OmenaQuerySassUnsupportedLedgerRecordV0>,
}

pub fn summarize_omena_query_sass_unsupported_ledger_view_v0()
-> Result<OmenaQuerySassUnsupportedLedgerViewV0, serde_json::Error> {
    let canonical = serde_json::from_str::<CanonicalSassUnsupportedLedgerReportV0>(
        SASS_UNSUPPORTED_LEDGER_PRODUCT_VIEW_JSON,
    )?;
    Ok(project_sass_unsupported_ledger_view(canonical))
}

fn project_sass_unsupported_ledger_view(
    canonical: CanonicalSassUnsupportedLedgerReportV0,
) -> OmenaQuerySassUnsupportedLedgerViewV0 {
    let file_counts = count_records_by(&canonical.records, |record| record.file.as_str());
    let reason_counts = count_records_by(&canonical.records, |record| record.reason.as_str());
    let surface_record_count = canonical.records.len();
    let surface_matches_ledger = canonical.all_semantic_sites_match_ledger
        && surface_record_count == canonical.semantic_site_count
        && surface_record_count == canonical.ledger_semantic_site_count;
    let summary_view_ready = canonical.all_bail_site_ledger_checks_hold && surface_matches_ledger;
    OmenaQuerySassUnsupportedLedgerViewV0 {
        schema_version: "0",
        product: "omena-query.sass-unsupported-ledger-view",
        source_product: canonical.product,
        semantic_site_count: canonical.semantic_site_count,
        ledger_semantic_site_count: canonical.ledger_semantic_site_count,
        surface_record_count,
        raw_pattern_hit_count: canonical.raw_pattern_hit_count,
        non_semantic_pattern_hit_count: canonical.non_semantic_pattern_hit_count,
        linked_site_count: canonical.linked_site_count,
        named_gap_site_count: canonical.named_gap_site_count,
        linked_case_count: canonical.linked_case_count,
        ledger_metadata_valid: canonical.ledger_metadata_valid,
        all_semantic_sites_match_ledger: canonical.all_semantic_sites_match_ledger,
        all_sites_linked_or_named_gap: canonical.all_sites_linked_or_named_gap,
        all_linked_cases_match_reason_class: canonical.all_linked_cases_match_reason_class,
        all_linked_cases_are_imported_sound_bail_cases: canonical
            .all_linked_cases_are_imported_sound_bail_cases,
        all_bail_site_ledger_checks_hold: canonical.all_bail_site_ledger_checks_hold,
        surface_matches_ledger,
        summary_view_ready,
        file_counts,
        reason_counts,
        records: canonical.records,
    }
}

fn count_records_by(
    records: &[OmenaQuerySassUnsupportedLedgerRecordV0],
    key: impl Fn(&OmenaQuerySassUnsupportedLedgerRecordV0) -> &str,
) -> Vec<OmenaQuerySassUnsupportedCountV0> {
    let mut counts = BTreeMap::<String, usize>::new();
    for record in records {
        *counts.entry(key(record).to_string()).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(key, count)| OmenaQuerySassUnsupportedCountV0 { key, count })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_view_reuses_the_independent_ledger_projection() -> Result<(), serde_json::Error> {
        let view = summarize_omena_query_sass_unsupported_ledger_view_v0()?;
        assert_eq!(
            view.source_product,
            "omena-diff-test.sass-spec-bail-site-ledger"
        );
        assert_eq!(view.semantic_site_count, 33);
        assert_eq!(view.surface_record_count, 33);
        assert!(view.all_semantic_sites_match_ledger);
        assert!(view.surface_matches_ledger);
        assert!(view.summary_view_ready);
        assert_eq!(
            view.file_counts.iter().map(|row| row.count).sum::<usize>(),
            view.surface_record_count
        );
        assert_eq!(
            view.reason_counts
                .iter()
                .map(|row| row.count)
                .sum::<usize>(),
            view.surface_record_count
        );
        Ok(())
    }

    #[test]
    fn hiding_a_ledger_record_invalidates_the_surface_projection() -> Result<(), serde_json::Error>
    {
        let mut canonical = serde_json::from_str::<CanonicalSassUnsupportedLedgerReportV0>(
            SASS_UNSUPPORTED_LEDGER_PRODUCT_VIEW_JSON,
        )?;
        canonical.records.pop();
        let view = project_sass_unsupported_ledger_view(canonical);
        assert!(!view.surface_matches_ledger);
        assert!(!view.summary_view_ready);
        Ok(())
    }
}
