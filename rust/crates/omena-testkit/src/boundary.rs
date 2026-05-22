use serde::Serialize;

use crate::fixture::{
    OmenaTestkitFixtureSeedCorpusReportV0, OmenaTestkitFixtureSeedV0,
    summarize_omena_testkit_fixture_seed_corpus,
};
use crate::scenario::{
    OmenaTestkitScenarioMacroReportV0, summarize_omena_testkit_scenario_macro_report,
};

/// Boundary summary for the shared Rust testkit substrate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitBoundarySummaryV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Owning omena-css layer.
    pub owner: &'static str,
    /// Fixture grammar owned by this crate.
    pub fixture_grammar: &'static str,
    /// Whether every boundary seed parses with the shared grammar.
    pub all_fixture_seeds_parse: bool,
    /// Named evidence gates closed by this crate.
    pub closed_gates: Vec<&'static str>,
    /// Boundary seed corpus report.
    pub fixture_seed_report: OmenaTestkitFixtureSeedCorpusReportV0,
    /// Boundary and transform-execution scenario macro substrate report.
    pub scenario_macro_report: OmenaTestkitScenarioMacroReportV0,
}

const BOUNDARY_FIXTURE_SEEDS: &[OmenaTestkitFixtureSeedV0] = &[
    OmenaTestkitFixtureSeedV0 {
        label: "shared-style-fixture",
        lane: "style-fixture",
        raw: r#"--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
omena-parser.style-facts
--- expect: assertion
shared fixture parser preserves style source text for product consumers
"#,
        expected_products: &["omena-parser.style-facts"],
        promotion_target: "omena-testkit/shared-fixture-parser",
    },
    OmenaTestkitFixtureSeedV0 {
        label: "cross-language-fixture",
        lane: "cross-language-fixture",
        raw: r#"--- file: src/App.tsx
import styles from "./Button.module.scss";
styles.button;
--- file: src/Button.module.scss
.button { color: red; }
--- expect: product
omena-query.source-syntax-index
--- expect: assertion
shared fixture parser keeps source and style files in the same workspace fixture
"#,
        expected_products: &["omena-query.source-syntax-index"],
        promotion_target: "omena-testkit/cross-language-fixture",
    },
    OmenaTestkitFixtureSeedV0 {
        label: "marked-style-fixture",
        lane: "marked-style-fixture",
        raw: r#"//- src/Card.module.scss dialect:scss layer:style
.card { color: /*|*/red; }
--- expect: product
omena-testkit.fixture-markers
--- expect: assertion
shared fixture parser strips marker text and reports clean-source offsets
"#,
        expected_products: &["omena-testkit.fixture-markers"],
        promotion_target: "omena-testkit/fixture-markers",
    },
];

/// Summarize the shared Rust testkit boundary.
pub fn summarize_omena_testkit_boundary() -> OmenaTestkitBoundarySummaryV0 {
    let fixture_seed_report = summarize_omena_testkit_fixture_seed_corpus(BOUNDARY_FIXTURE_SEEDS);
    let scenario_macro_report = summarize_omena_testkit_scenario_macro_report();

    OmenaTestkitBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-testkit.boundary",
        owner: "omena-css/testkit",
        fixture_grammar: "cme-fixture-v0",
        all_fixture_seeds_parse: fixture_seed_report.all_seeds_parse,
        closed_gates: vec![
            "sharedFixtureParserOwnedByOmenaTestkit",
            "crossLanguageFixtureGrammar",
            "fixtureHeaderMetadata",
            "fixtureMarkerOffsets",
            "boundaryScenarioMacro",
            "transformExecuteScenarioMacro",
            "scenarioMacroCallSiteEvidence",
            "m4TestkitPromotionSubstrate",
        ],
        fixture_seed_report,
        scenario_macro_report,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_declares_shared_fixture_parser() {
        let summary = summarize_omena_testkit_boundary();

        assert_eq!(summary.product, "omena-testkit.boundary");
        assert_eq!(summary.fixture_grammar, "cme-fixture-v0");
        assert!(summary.all_fixture_seeds_parse);
        assert_eq!(summary.fixture_seed_report.fixture_count, 3);
        assert_eq!(summary.fixture_seed_report.metadata_count, 2);
        assert_eq!(summary.fixture_seed_report.marker_count, 1);
        assert!(
            summary
                .closed_gates
                .contains(&"sharedFixtureParserOwnedByOmenaTestkit")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"crossLanguageFixtureGrammar")
        );
        assert!(summary.closed_gates.contains(&"fixtureHeaderMetadata"));
        assert!(summary.closed_gates.contains(&"fixtureMarkerOffsets"));
        assert!(summary.closed_gates.contains(&"boundaryScenarioMacro"));
        assert!(
            summary
                .closed_gates
                .contains(&"transformExecuteScenarioMacro")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"scenarioMacroCallSiteEvidence")
        );
        assert!(summary.scenario_macro_report.all_scenario_macros_ready);
        assert!(summary.scenario_macro_report.call_site_evidence_supported);
        assert!(summary.scenario_macro_report.call_site_probe.present);
        assert!(
            summary
                .scenario_macro_report
                .call_site_probe
                .file_suffix_matches
        );
        assert!(summary.scenario_macro_report.call_site_probe.line_non_zero);
        assert!(
            summary
                .scenario_macro_report
                .call_site_probe
                .column_non_zero
        );
        assert_eq!(summary.scenario_macro_report.scenario_count, 2);
    }
}
