//! Shared Rust fixture and scenario substrate for omena-css tests.
//!
//! M4 uses this crate to move reusable fixture grammar out of product-specific
//! harnesses. Later testkit layers can add scenario macros and snapshot
//! governance on top of the same `cme-fixture-v0` parser.

use serde::Serialize;
use std::collections::BTreeSet;

/// One reusable fixture seed consumed by the testkit boundary report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmenaTestkitFixtureSeedV0 {
    /// Stable fixture label.
    pub label: &'static str,
    /// Fixture lane such as `sass-language` or `cascade-proof`.
    pub lane: &'static str,
    /// Raw `cme-fixture-v0` text.
    pub raw: &'static str,
    /// Product surfaces expected to consume this fixture.
    pub expected_products: &'static [&'static str],
    /// Promotion target for M4.
    pub promotion_target: &'static str,
}

/// Parsed reusable fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeFixtureV0 {
    /// Fixture grammar version.
    pub schema_version: &'static str,
    /// Parsed files.
    pub files: Vec<CmeFixtureFileV0>,
    /// Parsed expectations.
    pub expectations: Vec<CmeFixtureExpectationV0>,
}

/// One file section in a reusable fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeFixtureFileV0 {
    /// Workspace-relative file path.
    pub path: String,
    /// File text.
    pub source: String,
}

/// One expectation section in a reusable fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeFixtureExpectationV0 {
    /// Expectation key.
    pub key: String,
    /// Expectation text.
    pub value: String,
}

/// Parsed fixture seed evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitFixtureSeedReportV0 {
    /// Stable fixture label.
    pub label: &'static str,
    /// Fixture lane.
    pub lane: &'static str,
    /// Whether the fixture parses with `cme-fixture-v0`.
    pub parses: bool,
    /// Parse error when present.
    pub parse_error: Option<String>,
    /// Parsed file count.
    pub file_count: usize,
    /// Parsed expectation count.
    pub expectation_count: usize,
    /// Expected product surfaces.
    pub expected_products: Vec<&'static str>,
    /// Promotion target for M4.
    pub promotion_target: &'static str,
}

/// Fixture seed corpus summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaTestkitFixtureSeedCorpusReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Fixture grammar.
    pub fixture_grammar: &'static str,
    /// Fixture count.
    pub fixture_count: usize,
    /// Covered lane count.
    pub lane_count: usize,
    /// Whether every seed parses with the shared fixture grammar.
    pub all_seeds_parse: bool,
    /// Seed reports.
    pub reports: Vec<OmenaTestkitFixtureSeedReportV0>,
}

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
];

/// Summarize the shared Rust testkit boundary.
pub fn summarize_omena_testkit_boundary() -> OmenaTestkitBoundarySummaryV0 {
    let fixture_seed_report = summarize_omena_testkit_fixture_seed_corpus(BOUNDARY_FIXTURE_SEEDS);

    OmenaTestkitBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-testkit.boundary",
        owner: "omena-css/testkit",
        fixture_grammar: "cme-fixture-v0",
        all_fixture_seeds_parse: fixture_seed_report.all_seeds_parse,
        closed_gates: vec![
            "sharedFixtureParserOwnedByOmenaTestkit",
            "crossLanguageFixtureGrammar",
            "m4TestkitPromotionSubstrate",
        ],
        fixture_seed_report,
    }
}

/// Summarize any `cme-fixture-v0` fixture seed corpus.
pub fn summarize_omena_testkit_fixture_seed_corpus(
    seeds: &[OmenaTestkitFixtureSeedV0],
) -> OmenaTestkitFixtureSeedCorpusReportV0 {
    let reports = seeds
        .iter()
        .copied()
        .map(report_fixture_seed)
        .collect::<Vec<_>>();
    let lane_count = reports
        .iter()
        .map(|report| report.lane)
        .collect::<BTreeSet<_>>()
        .len();
    let all_seeds_parse = reports.iter().all(|report| report.parses);

    OmenaTestkitFixtureSeedCorpusReportV0 {
        schema_version: "0",
        product: "omena-testkit.fixture-seed-corpus",
        fixture_grammar: "cme-fixture-v0",
        fixture_count: reports.len(),
        lane_count,
        all_seeds_parse,
        reports,
    }
}

/// Parse a reusable `cme-fixture-v0` fixture.
pub fn parse_cme_fixture_v0(raw: &str) -> Result<CmeFixtureV0, String> {
    enum Section {
        File { path: String, source: String },
        Expect { key: String, value: String },
    }

    let mut sections = Vec::new();
    let mut current = None::<Section>;

    for line in raw.lines() {
        if let Some(path) = line.strip_prefix("--- file: ") {
            finish_fixture_section(&mut sections, current.take());
            current = Some(Section::File {
                path: path.trim().to_string(),
                source: String::new(),
            });
            continue;
        }
        if let Some(key) = line.strip_prefix("--- expect: ") {
            finish_fixture_section(&mut sections, current.take());
            current = Some(Section::Expect {
                key: key.trim().to_string(),
                value: String::new(),
            });
            continue;
        }

        match current.as_mut() {
            Some(Section::File { source, .. }) => {
                push_fixture_line(source, line);
            }
            Some(Section::Expect { value, .. }) => {
                push_fixture_line(value, line);
            }
            None if line.trim().is_empty() => {}
            None => {
                return Err("fixture content must start with a file or expect marker".to_string());
            }
        }
    }

    finish_fixture_section(&mut sections, current);

    let mut files = Vec::new();
    let mut expectations = Vec::new();
    for section in sections {
        match section {
            Section::File { path, source } => files.push(CmeFixtureFileV0 { path, source }),
            Section::Expect { key, value } => expectations.push(CmeFixtureExpectationV0 {
                key,
                value: value.trim().to_string(),
            }),
        }
    }

    if files.is_empty() {
        return Err("fixture must contain at least one file section".to_string());
    }
    if expectations.is_empty() {
        return Err("fixture must contain at least one expectation section".to_string());
    }

    Ok(CmeFixtureV0 {
        schema_version: "0",
        files,
        expectations,
    })
}

fn report_fixture_seed(seed: OmenaTestkitFixtureSeedV0) -> OmenaTestkitFixtureSeedReportV0 {
    match parse_cme_fixture_v0(seed.raw) {
        Ok(fixture) => OmenaTestkitFixtureSeedReportV0 {
            label: seed.label,
            lane: seed.lane,
            parses: true,
            parse_error: None,
            file_count: fixture.files.len(),
            expectation_count: fixture.expectations.len(),
            expected_products: seed.expected_products.to_vec(),
            promotion_target: seed.promotion_target,
        },
        Err(error) => OmenaTestkitFixtureSeedReportV0 {
            label: seed.label,
            lane: seed.lane,
            parses: false,
            parse_error: Some(error),
            file_count: 0,
            expectation_count: 0,
            expected_products: seed.expected_products.to_vec(),
            promotion_target: seed.promotion_target,
        },
    }
}

fn finish_fixture_section<T>(sections: &mut Vec<T>, current: Option<T>) {
    if let Some(section) = current {
        sections.push(section);
    }
}

fn push_fixture_line(buffer: &mut String, line: &str) {
    if !buffer.is_empty() {
        buffer.push('\n');
    }
    buffer.push_str(line);
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
    }

    #[test]
    fn parses_reusable_cme_fixture_v0_sections() -> Result<(), String> {
        let fixture = parse_cme_fixture_v0(
            r#"--- file: src/proof.css
.a { color: red; }
--- expect: product
omena-transform-passes.cascade-proof-obligations
--- expect: assertion
proof obligations remain product-visible
"#,
        )?;

        assert_eq!(fixture.schema_version, "0");
        assert_eq!(fixture.files.len(), 1);
        assert_eq!(fixture.files[0].path, "src/proof.css");
        assert!(fixture.files[0].source.contains(".a"));
        assert_eq!(fixture.expectations.len(), 2);
        assert_eq!(fixture.expectations[0].key, "product");
        assert_eq!(
            fixture.expectations[0].value,
            "omena-transform-passes.cascade-proof-obligations"
        );

        Ok(())
    }

    #[test]
    fn keeps_source_and_style_files_in_one_workspace_fixture() -> Result<(), String> {
        let fixture = parse_cme_fixture_v0(BOUNDARY_FIXTURE_SEEDS[1].raw)?;

        assert_eq!(fixture.files.len(), 2);
        assert_eq!(fixture.files[0].path, "src/App.tsx");
        assert_eq!(fixture.files[1].path, "src/Button.module.scss");
        assert!(
            fixture
                .expectations
                .iter()
                .any(|expectation| expectation.value == "omena-query.source-syntax-index")
        );

        Ok(())
    }

    #[test]
    fn rejects_fixture_without_sections() {
        let error = parse_cme_fixture_v0("plain text").err();

        assert_eq!(
            error.as_deref(),
            Some("fixture content must start with a file or expect marker")
        );
    }

    #[test]
    fn rejects_fixture_without_expectations() {
        let error = parse_cme_fixture_v0(
            r#"--- file: src/Button.module.scss
.button { color: red; }
"#,
        )
        .err();

        assert_eq!(
            error.as_deref(),
            Some("fixture must contain at least one expectation section")
        );
    }

    #[test]
    fn summarizes_external_fixture_seed_corpus() {
        let seeds = [OmenaTestkitFixtureSeedV0 {
            label: "external",
            lane: "consumer",
            raw: r#"--- file: src/input.css
.x { color: red; }
--- expect: product
consumer.product
"#,
            expected_products: &["consumer.product"],
            promotion_target: "omena-testkit/consumer",
        }];

        let report = summarize_omena_testkit_fixture_seed_corpus(&seeds);

        assert_eq!(report.product, "omena-testkit.fixture-seed-corpus");
        assert_eq!(report.fixture_count, 1);
        assert_eq!(report.lane_count, 1);
        assert!(report.all_seeds_parse);
        assert_eq!(report.reports[0].file_count, 1);
        assert_eq!(report.reports[0].expectation_count, 1);
    }
}
