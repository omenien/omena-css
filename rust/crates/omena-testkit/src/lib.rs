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
    /// File-header metadata such as dialect or layer.
    pub metadata: Vec<CmeFixtureFileMetadataV0>,
    /// Markers removed from the source while preserving clean-source offsets.
    pub markers: Vec<CmeFixtureMarkerV0>,
    /// File text.
    pub source: String,
}

/// One metadata key/value pair from a fixture file header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeFixtureFileMetadataV0 {
    /// Metadata key.
    pub key: String,
    /// Metadata value.
    pub value: String,
}

/// One marker extracted from fixture source text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmeFixtureMarkerV0 {
    /// Marker kind such as `cursor`, `namedPoint`, or `rangeStart`.
    pub kind: &'static str,
    /// Optional marker payload.
    pub name: Option<String>,
    /// Byte offset in the cleaned source.
    pub byte_start: usize,
    /// Byte end in the cleaned source.
    pub byte_end: usize,
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
    /// Parsed file-header metadata count.
    pub metadata_count: usize,
    /// Parsed marker count.
    pub marker_count: usize,
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
    /// Parsed metadata count across all seed files.
    pub metadata_count: usize,
    /// Parsed marker count across all seed files.
    pub marker_count: usize,
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
    let metadata_count = reports.iter().map(|report| report.metadata_count).sum();
    let marker_count = reports.iter().map(|report| report.marker_count).sum();
    let all_seeds_parse = reports.iter().all(|report| report.parses);

    OmenaTestkitFixtureSeedCorpusReportV0 {
        schema_version: "0",
        product: "omena-testkit.fixture-seed-corpus",
        fixture_grammar: "cme-fixture-v0",
        fixture_count: reports.len(),
        lane_count,
        metadata_count,
        marker_count,
        all_seeds_parse,
        reports,
    }
}

/// Parse a reusable `cme-fixture-v0` fixture.
pub fn parse_cme_fixture_v0(raw: &str) -> Result<CmeFixtureV0, String> {
    enum Section {
        File {
            path: String,
            metadata: Vec<CmeFixtureFileMetadataV0>,
            source: String,
        },
        Expect {
            key: String,
            value: String,
        },
    }

    let mut sections = Vec::new();
    let mut current = None::<Section>;

    for line in raw.lines() {
        if let Some(path) = line.strip_prefix("--- file: ") {
            finish_fixture_section(&mut sections, current.take());
            current = Some(Section::File {
                path: path.trim().to_string(),
                metadata: Vec::new(),
                source: String::new(),
            });
            continue;
        }
        if let Some(header) = line.strip_prefix("//-") {
            let (path, metadata) = parse_cme_fixture_file_header(header.trim())?;
            finish_fixture_section(&mut sections, current.take());
            current = Some(Section::File {
                path,
                metadata,
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
            Section::File {
                path,
                metadata,
                source,
            } => {
                let (source, markers) = extract_cme_fixture_markers(source.as_str())?;
                files.push(CmeFixtureFileV0 {
                    path,
                    metadata,
                    markers,
                    source,
                });
            }
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
        Ok(fixture) => {
            let metadata_count = fixture.files.iter().map(|file| file.metadata.len()).sum();
            let marker_count = fixture.files.iter().map(|file| file.markers.len()).sum();
            OmenaTestkitFixtureSeedReportV0 {
                label: seed.label,
                lane: seed.lane,
                parses: true,
                parse_error: None,
                file_count: fixture.files.len(),
                expectation_count: fixture.expectations.len(),
                metadata_count,
                marker_count,
                expected_products: seed.expected_products.to_vec(),
                promotion_target: seed.promotion_target,
            }
        }
        Err(error) => OmenaTestkitFixtureSeedReportV0 {
            label: seed.label,
            lane: seed.lane,
            parses: false,
            parse_error: Some(error),
            file_count: 0,
            expectation_count: 0,
            metadata_count: 0,
            marker_count: 0,
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

fn parse_cme_fixture_file_header(
    header: &str,
) -> Result<(String, Vec<CmeFixtureFileMetadataV0>), String> {
    let mut parts = header.split_whitespace();
    let path = parts
        .next()
        .ok_or_else(|| "fixture file header must include a path".to_string())?;
    if path.contains(':') {
        return Err("fixture file header path must precede metadata".to_string());
    }

    let mut metadata = Vec::new();
    for part in parts {
        let Some((key, value)) = part.split_once(':') else {
            return Err(format!("fixture metadata `{part}` must use key:value"));
        };
        validate_cme_fixture_metadata(key, value)?;
        metadata.push(CmeFixtureFileMetadataV0 {
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    Ok((path.to_string(), metadata))
}

fn validate_cme_fixture_metadata(key: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("fixture metadata `{key}` must have a value"));
    }
    match key {
        "dialect" => match value {
            "css" | "scss" | "less" => Ok(()),
            _ => Err("fixture dialect metadata must be css, scss, or less".to_string()),
        },
        "layer" | "composes-from" | "consumer-of" => Ok(()),
        _ => Err(format!("fixture metadata key `{key}` is not supported")),
    }
}

fn extract_cme_fixture_markers(source: &str) -> Result<(String, Vec<CmeFixtureMarkerV0>), String> {
    let mut cleaned = String::new();
    let mut markers = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = source[cursor..].find("/*") {
        let start = cursor + relative_start;
        cleaned.push_str(&source[cursor..start]);
        let Some(relative_end) = source[start + 2..].find("*/") else {
            return Err("fixture marker comment is unterminated".to_string());
        };
        let end = start + 2 + relative_end + 2;
        let body = &source[start + 2..end - 2];
        if let Some(marker) = parse_cme_fixture_marker(body, cleaned.len())? {
            markers.push(marker);
        } else {
            cleaned.push_str(&source[start..end]);
        }
        cursor = end;
    }

    cleaned.push_str(&source[cursor..]);
    Ok((cleaned, markers))
}

fn parse_cme_fixture_marker(
    body: &str,
    byte_offset: usize,
) -> Result<Option<CmeFixtureMarkerV0>, String> {
    if body == "|" {
        return Ok(Some(cme_fixture_marker("cursor", None, byte_offset)));
    }
    if let Some(name) = body.strip_prefix("at:") {
        return Ok(Some(cme_fixture_marker(
            "namedPoint",
            Some(validate_cme_fixture_marker_payload("at", name)?),
            byte_offset,
        )));
    }
    if let Some(name) = body
        .strip_prefix("</")
        .and_then(|name| name.strip_suffix('>'))
    {
        return Ok(Some(cme_fixture_marker(
            "rangeEnd",
            Some(validate_cme_fixture_marker_payload("range end", name)?),
            byte_offset,
        )));
    }
    if let Some(name) = body
        .strip_prefix('<')
        .and_then(|name| name.strip_suffix('>'))
    {
        return Ok(Some(cme_fixture_marker(
            "rangeStart",
            Some(validate_cme_fixture_marker_payload("range start", name)?),
            byte_offset,
        )));
    }
    if let Some(name) = body.strip_prefix("name:") {
        return Ok(Some(cme_fixture_marker(
            "nameAnchor",
            Some(validate_cme_fixture_marker_payload("name", name)?),
            byte_offset,
        )));
    }
    if let Some(target) = body.strip_prefix("from:") {
        return Ok(Some(cme_fixture_marker(
            "linkEnd",
            Some(validate_cme_fixture_marker_payload("from", target)?),
            byte_offset,
        )));
    }
    Ok(None)
}

fn cme_fixture_marker(
    kind: &'static str,
    name: Option<String>,
    byte_offset: usize,
) -> CmeFixtureMarkerV0 {
    CmeFixtureMarkerV0 {
        kind,
        name,
        byte_start: byte_offset,
        byte_end: byte_offset,
    }
}

fn validate_cme_fixture_marker_payload(kind: &str, value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Err(format!("fixture marker `{kind}` must have a value"));
    }
    Ok(value.to_string())
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
    fn parses_cme_fixture_v0_metadata_and_markers() -> Result<(), String> {
        let fixture = parse_cme_fixture_v0(
            r#"//- src/Card.module.scss dialect:scss layer:style
.card { color: /*|*/red; }
.card/*at:selector*/ { color: blue; }
.card { color: /*<colorRange>*/green/*</colorRange>*/; }
.card { composes: item/*from:src/Base.module.scss#item*/; }
--- expect: product
omena-testkit.fixture-markers
"#,
        )?;

        assert_eq!(fixture.files.len(), 1);
        assert_eq!(fixture.files[0].path, "src/Card.module.scss");
        assert_eq!(
            fixture.files[0]
                .metadata
                .iter()
                .map(|metadata| (metadata.key.as_str(), metadata.value.as_str()))
                .collect::<Vec<_>>(),
            vec![("dialect", "scss"), ("layer", "style")]
        );
        assert_eq!(
            fixture.files[0]
                .markers
                .iter()
                .map(|marker| (marker.kind, marker.name.as_deref()))
                .collect::<Vec<_>>(),
            vec![
                ("cursor", None),
                ("namedPoint", Some("selector")),
                ("rangeStart", Some("colorRange")),
                ("rangeEnd", Some("colorRange")),
                ("linkEnd", Some("src/Base.module.scss#item"))
            ]
        );
        assert!(fixture.files[0].source.contains(".card { color: red; }"));
        assert!(!fixture.files[0].source.contains("/*|*/"));
        assert_eq!(
            fixture.files[0].markers[0].byte_start,
            ".card { color: ".len()
        );

        Ok(())
    }

    #[test]
    fn keeps_non_fixture_comments_in_source() -> Result<(), String> {
        let fixture = parse_cme_fixture_v0(
            r#"//- src/Card.module.css dialect:css
.card { /* regular comment */ color: red; }
--- expect: product
omena-testkit.fixture-markers
"#,
        )?;

        assert!(fixture.files[0].source.contains("/* regular comment */"));
        assert!(fixture.files[0].markers.is_empty());

        Ok(())
    }

    #[test]
    fn rejects_unknown_fixture_metadata() {
        let error = parse_cme_fixture_v0(
            r#"//- src/Card.module.css unknown:value
.card { color: red; }
--- expect: product
omena-testkit.fixture-markers
"#,
        )
        .err();

        assert_eq!(
            error.as_deref(),
            Some("fixture metadata key `unknown` is not supported")
        );
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
