use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const CASCADE_DRIVER_CASES_JSON: &str = include_str!("../wpt-corpus/cascade-driver-cases.json");
const LAYER_TOPOLOGY_CENSUS_JSON: &str = include_str!("../wpt-corpus/layer-topology-census.json");

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerTopologyCensusV0 {
    schema_version: String,
    product: String,
    cases: Vec<LayerTopologyCaseV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LayerTopologyCaseV0 {
    id: String,
    status: String,
    source: String,
    expected_order: Vec<String>,
    minimum_unresolved_count: usize,
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
    pub layer_topology_case_count: usize,
    pub resolved_layer_topology_case_count: usize,
    pub blocked_layer_topology_case_count: usize,
    pub all_layer_topology_cases_match: bool,
    pub cascade_level_count: usize,
    pub driven_cascade_level_count: usize,
    pub deferred_cascade_level_count: usize,
    pub cascade_origin_driver_census_matches: bool,
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
                && (case.wpt_path.starts_with("css/css-cascade/")
                    || case.wpt_path.starts_with("css/css-properties-values-api/"))
                && case.wpt_source_line > 0
                && !case.expected_outcome.is_empty()
                && matches!(
                    case.replacement_gate.as_str(),
                    "computed-testcommon-extraction" | "inheritance-testcommon-extraction"
                )
        });
    let layer_census: LayerTopologyCensusV0 = serde_json::from_str(LAYER_TOPOLOGY_CENSUS_JSON)
        .unwrap_or(LayerTopologyCensusV0 {
            schema_version: String::new(),
            product: String::new(),
            cases: Vec::new(),
        });
    let layer_case_ids = layer_census
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    let all_layer_topology_cases_match = layer_census.schema_version == "0"
        && layer_census.product == "omena-diff-test.layer-topology-census"
        && layer_case_ids.len() == layer_census.cases.len()
        && layer_census.cases.iter().all(|case| {
            let index = omena_semantic::summarize_style_layer_order_from_source(
                case.source.as_str(),
                omena_parser::StyleDialect::Css,
            );
            let actual_order = index
                .order_nodes
                .iter()
                .map(|node| node.canonical_name.clone())
                .collect::<Vec<_>>();
            match case.status.as_str() {
                "resolved" => {
                    index.topology_complete
                        && index.unresolved_topology_count == 0
                        && actual_order == case.expected_order
                }
                "blocked" => {
                    !index.topology_complete
                        && index.unresolved_topology_count >= case.minimum_unresolved_count
                        && actual_order == case.expected_order
                }
                _ => false,
            }
        });
    let cascade_level_count = omena_cascade::cascade_level_catalog_v0().len();
    let driven_cascade_level_count = omena_cascade::cascade_driven_levels_v0().len();

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
        layer_topology_case_count: layer_census.cases.len(),
        resolved_layer_topology_case_count: layer_census
            .cases
            .iter()
            .filter(|case| case.status == "resolved")
            .count(),
        blocked_layer_topology_case_count: layer_census
            .cases
            .iter()
            .filter(|case| case.status == "blocked")
            .count(),
        all_layer_topology_cases_match,
        cascade_level_count,
        driven_cascade_level_count,
        deferred_cascade_level_count: cascade_level_count - driven_cascade_level_count,
        cascade_origin_driver_census_matches: omena_cascade::cascade_driver_census_is_consistent_v0(
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_driver_cases_have_pinned_interim_oracle_provenance() {
        let report = summarize_cascade_driver_conformance_v0();

        assert!(report.all_cases_valid);
        assert_eq!(report.case_count, 6);
        assert_eq!(report.interim_case_count, 6);
        assert_eq!(
            report.capabilities,
            vec![
                "computedValueInheritance",
                "elementParentChain",
                "nestedLayerOrder",
                "originImportanceLadder",
                "registeredPropertyComputedValue",
                "scopeAncestorProximity"
            ]
        );
        assert_eq!(report.layer_topology_case_count, 5);
        assert_eq!(report.resolved_layer_topology_case_count, 2);
        assert_eq!(report.blocked_layer_topology_case_count, 3);
        assert!(report.all_layer_topology_cases_match);
        assert_eq!(report.cascade_level_count, 9);
        assert_eq!(report.driven_cascade_level_count, 7);
        assert_eq!(report.deferred_cascade_level_count, 2);
        assert!(report.cascade_origin_driver_census_matches);
    }
}
