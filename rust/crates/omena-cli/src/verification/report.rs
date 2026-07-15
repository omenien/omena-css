use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

use super::manifest::VerificationEvidenceReferenceV0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum VerificationOutcomeV0 {
    Passed,
    Failed,
    Indeterminate,
    NotYetAvailable,
    Skipped,
}

impl VerificationOutcomeV0 {
    pub(crate) const fn is_blocking_failure(self) -> bool {
        matches!(self, Self::Failed | Self::Indeterminate)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VerificationItemReportV0 {
    pub(crate) id: String,
    pub(crate) scope: &'static str,
    pub(crate) outcome: VerificationOutcomeV0,
    pub(crate) description: String,
    pub(crate) summary: String,
    pub(crate) evidence: Vec<VerificationEvidenceReferenceV0>,
    pub(crate) runtime_evidence: Vec<Value>,
    pub(crate) limitation: String,
    pub(crate) metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VerificationReportV0 {
    schema_version: &'static str,
    product: &'static str,
    pub(crate) workspace_root: String,
    pub(crate) engine_self_requested: bool,
    pub(crate) item_count: usize,
    pub(crate) passed_count: usize,
    pub(crate) failed_count: usize,
    pub(crate) indeterminate_count: usize,
    pub(crate) not_yet_available_count: usize,
    pub(crate) skipped_count: usize,
    pub(crate) blocking_failure_count: usize,
    pub(crate) items: Vec<VerificationItemReportV0>,
}

impl VerificationReportV0 {
    pub(crate) fn new(
        workspace_root: String,
        engine_self_requested: bool,
        items: Vec<VerificationItemReportV0>,
    ) -> Self {
        let passed_count = count_outcome(&items, VerificationOutcomeV0::Passed);
        let failed_count = count_outcome(&items, VerificationOutcomeV0::Failed);
        let indeterminate_count = count_outcome(&items, VerificationOutcomeV0::Indeterminate);
        let not_yet_available_count = count_outcome(&items, VerificationOutcomeV0::NotYetAvailable);
        let skipped_count = count_outcome(&items, VerificationOutcomeV0::Skipped);
        let blocking_failure_count = items
            .iter()
            .filter(|item| item.outcome.is_blocking_failure())
            .count();
        Self {
            schema_version: "0",
            product: "omena-cli.verify",
            workspace_root,
            engine_self_requested,
            item_count: items.len(),
            passed_count,
            failed_count,
            indeterminate_count,
            not_yet_available_count,
            skipped_count,
            blocking_failure_count,
            items,
        }
    }
}

fn count_outcome(items: &[VerificationItemReportV0], outcome: VerificationOutcomeV0) -> usize {
    items.iter().filter(|item| item.outcome == outcome).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_items_are_neither_passed_nor_failed() {
        let report = VerificationReportV0::new(
            "/workspace".to_string(),
            false,
            vec![fixture_item(
                "translation-validation",
                VerificationOutcomeV0::NotYetAvailable,
            )],
        );
        assert_eq!(report.passed_count, 0);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.blocking_failure_count, 0);
        assert_eq!(report.not_yet_available_count, 1);
    }

    #[test]
    fn failed_and_indeterminate_items_fail_closed() {
        let report = VerificationReportV0::new(
            "/workspace".to_string(),
            false,
            vec![
                fixture_item("parser", VerificationOutcomeV0::Failed),
                fixture_item("bundle", VerificationOutcomeV0::Indeterminate),
                fixture_item("precision", VerificationOutcomeV0::NotYetAvailable),
            ],
        );
        assert_eq!(report.passed_count, 0);
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.indeterminate_count, 1);
        assert_eq!(report.blocking_failure_count, 2);
    }

    fn fixture_item(id: &str, outcome: VerificationOutcomeV0) -> VerificationItemReportV0 {
        VerificationItemReportV0 {
            id: id.to_string(),
            scope: "userWorkspace",
            outcome,
            description: "fixture".to_string(),
            summary: "fixture".to_string(),
            evidence: vec![VerificationEvidenceReferenceV0 {
                path: "fixture.rs".to_string(),
                symbol: "fixture".to_string(),
            }],
            runtime_evidence: Vec::new(),
            limitation: "fixture".to_string(),
            metrics: BTreeMap::new(),
        }
    }
}
