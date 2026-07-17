use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const CASCADE_DRIVER_CASES_JSON: &str = include_str!("../wpt-corpus/cascade-driver-cases.json");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverCaseLedgerV0 {
    schema_version: String,
    product: String,
    source_pin: String,
    cases: Vec<CascadeDriverCaseV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CascadeDriverCaseV0 {
    id: String,
    capability: String,
    oracle_kind: String,
    wpt_path: String,
    wpt_source_line: usize,
    expected_outcome: String,
    replacement_gate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeDriverConformanceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_pin: String,
    pub case_count: usize,
    pub capabilities: Vec<String>,
    pub interim_case_count: usize,
    pub all_cases_valid: bool,
}

pub fn summarize_cascade_driver_conformance_v0() -> CascadeDriverConformanceReportV0 {
    let ledger: CascadeDriverCaseLedgerV0 = serde_json::from_str(CASCADE_DRIVER_CASES_JSON)
        .unwrap_or(CascadeDriverCaseLedgerV0 {
            schema_version: String::new(),
            product: String::new(),
            source_pin: String::new(),
            cases: Vec::new(),
        });
    let ids = ledger
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    let capabilities = ledger
        .cases
        .iter()
        .map(|case| case.capability.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let all_cases_valid = ledger.schema_version == "0"
        && ledger.product == "omena-diff-test.cascade-driver-cases"
        && ledger.source_pin.starts_with("web-platform-tests/wpt@")
        && ids.len() == ledger.cases.len()
        && ledger.cases.iter().all(|case| {
            !case.id.is_empty()
                && !case.capability.is_empty()
                && case.oracle_kind == "interimWptPath"
                && case.wpt_path.starts_with("css/css-cascade/")
                && case.wpt_source_line > 0
                && !case.expected_outcome.is_empty()
                && case.replacement_gate == "computed-testcommon-extraction"
        });

    CascadeDriverConformanceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.cascade-driver-conformance",
        source_pin: ledger.source_pin,
        case_count: ledger.cases.len(),
        capabilities,
        interim_case_count: ledger
            .cases
            .iter()
            .filter(|case| case.oracle_kind == "interimWptPath")
            .count(),
        all_cases_valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_driver_cases_have_pinned_interim_oracle_provenance() {
        let report = summarize_cascade_driver_conformance_v0();

        assert!(report.all_cases_valid);
        assert_eq!(report.case_count, 1);
        assert_eq!(report.interim_case_count, 1);
        assert_eq!(report.capabilities, vec!["elementParentChain"]);
    }
}
