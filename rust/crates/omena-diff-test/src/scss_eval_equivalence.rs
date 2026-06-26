use omena_parser::StyleDialect;
use omena_scss_eval::{
    summarize_omena_scss_eval_oracle, summarize_scss_call_return_ir,
    summarize_scss_call_return_ir_scanner_oracle, summarize_scss_control_flow_ir,
    summarize_scss_control_flow_ir_scanner_oracle, summarize_static_stylesheet_value_resolution,
    summarize_static_stylesheet_value_resolution_scanner_oracle,
    summarize_typed_value_lattice_witness,
};
use serde::{Deserialize, Serialize};

const SCSS_EVAL_PUBLIC_SUMMARY_SNAPSHOT_SOURCE: &str =
    include_str!("../regressions/scss-eval-public-summaries.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffScssEvalPublicSummarySnapshotV0 {
    pub schema_version: String,
    pub product: String,
    pub fixtures: Vec<OmenaDiffScssEvalPublicSummaryFixtureSnapshotV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffScssEvalPublicSummaryFixtureSnapshotV0 {
    pub id: String,
    pub summaries: Vec<OmenaDiffScssEvalPublicSummaryHashSnapshotV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffScssEvalPublicSummaryHashSnapshotV0 {
    pub summary: String,
    pub json_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffScssEvalPublicSummaryEquivalenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub comparison_count: usize,
    pub matching_comparison_count: usize,
    pub all_summaries_match: bool,
    pub fixtures: Vec<OmenaDiffScssEvalPublicSummaryFixtureReportV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffScssEvalPublicSummaryFixtureReportV0 {
    pub id: &'static str,
    pub dialect: &'static str,
    pub comparisons: Vec<OmenaDiffScssEvalPublicSummaryHashReportV0>,
    pub all_summaries_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffScssEvalPublicSummaryHashReportV0 {
    pub summary: &'static str,
    pub cst_json_hash: String,
    pub legacy_scanner_json_hash: String,
    pub expected_json_hash: Option<String>,
    pub scanner_matches_cst: bool,
    pub cst_matches_snapshot: bool,
    pub matches: bool,
}

struct OmenaDiffScssEvalPublicSummaryHashV0 {
    summary: &'static str,
    json_hash: String,
}

struct ScssEvalPublicSummaryFixtureV0 {
    id: &'static str,
    dialect: StyleDialect,
    source: &'static str,
    candidate_evaluated_css: &'static str,
}

pub fn summarize_scss_eval_public_summary_equivalence_v0()
-> OmenaDiffScssEvalPublicSummaryEquivalenceReportV0 {
    let snapshot = scss_eval_public_summary_snapshot();
    let fixtures = SCSS_EVAL_PUBLIC_SUMMARY_FIXTURES
        .iter()
        .map(|fixture| scss_eval_public_summary_fixture_report(fixture, &snapshot))
        .collect::<Vec<_>>();
    let comparison_count = fixtures
        .iter()
        .map(|fixture| fixture.comparisons.len())
        .sum();
    let matching_comparison_count = fixtures
        .iter()
        .flat_map(|fixture| fixture.comparisons.iter())
        .filter(|comparison| comparison.matches)
        .count();
    OmenaDiffScssEvalPublicSummaryEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.scss-eval-public-summary-equivalence",
        fixture_count: fixtures.len(),
        comparison_count,
        matching_comparison_count,
        all_summaries_match: matching_comparison_count == comparison_count,
        fixtures,
    }
}

fn scss_eval_public_summary_fixture_report(
    fixture: &ScssEvalPublicSummaryFixtureV0,
    snapshot: &OmenaDiffScssEvalPublicSummarySnapshotV0,
) -> OmenaDiffScssEvalPublicSummaryFixtureReportV0 {
    let cst = scss_eval_public_summary_hashes(fixture);
    let legacy_scanner = scss_eval_public_summary_legacy_scanner_hashes(fixture);
    let comparisons = cst
        .into_iter()
        .zip(legacy_scanner)
        .map(|(cst, legacy_scanner)| {
            debug_assert_eq!(cst.summary, legacy_scanner.summary);
            let expected_json_hash =
                scss_eval_public_summary_expected_hash(snapshot, fixture.id, cst.summary);
            let scanner_matches_cst = cst.json_hash == legacy_scanner.json_hash;
            let cst_matches_snapshot = expected_json_hash
                .as_deref()
                .is_some_and(|expected| expected == cst.json_hash);
            OmenaDiffScssEvalPublicSummaryHashReportV0 {
                summary: cst.summary,
                cst_json_hash: cst.json_hash,
                legacy_scanner_json_hash: legacy_scanner.json_hash,
                expected_json_hash,
                scanner_matches_cst,
                cst_matches_snapshot,
                matches: scanner_matches_cst && cst_matches_snapshot,
            }
        })
        .collect::<Vec<_>>();
    let all_summaries_match = comparisons.iter().all(|comparison| comparison.matches);
    OmenaDiffScssEvalPublicSummaryFixtureReportV0 {
        id: fixture.id,
        dialect: dialect_label(fixture.dialect),
        comparisons,
        all_summaries_match,
    }
}

fn scss_eval_public_summary_hashes(
    fixture: &ScssEvalPublicSummaryFixtureV0,
) -> Vec<OmenaDiffScssEvalPublicSummaryHashV0> {
    vec![
        scss_eval_public_summary_hash(
            "controlFlowIr",
            summarize_scss_control_flow_ir(fixture.source, fixture.dialect),
        ),
        scss_eval_public_summary_hash(
            "callReturnIr",
            summarize_scss_call_return_ir(fixture.source, fixture.dialect),
        ),
        scss_eval_public_summary_hash(
            "typedValueLatticeWitness",
            summarize_typed_value_lattice_witness(),
        ),
        scss_eval_public_summary_hash(
            "scssEvalOracle",
            summarize_omena_scss_eval_oracle(
                fixture.source,
                fixture.dialect,
                fixture.candidate_evaluated_css,
            ),
        ),
        scss_eval_public_summary_hash(
            "staticStylesheetValueResolution",
            summarize_static_stylesheet_value_resolution(fixture.source, fixture.dialect),
        ),
    ]
}

fn scss_eval_public_summary_legacy_scanner_hashes(
    fixture: &ScssEvalPublicSummaryFixtureV0,
) -> Vec<OmenaDiffScssEvalPublicSummaryHashV0> {
    vec![
        scss_eval_public_summary_hash(
            "controlFlowIr",
            summarize_scss_control_flow_ir_scanner_oracle(fixture.source, fixture.dialect),
        ),
        scss_eval_public_summary_hash(
            "callReturnIr",
            summarize_scss_call_return_ir_scanner_oracle(fixture.source, fixture.dialect),
        ),
        scss_eval_public_summary_hash(
            "typedValueLatticeWitness",
            summarize_typed_value_lattice_witness(),
        ),
        scss_eval_public_summary_hash(
            "scssEvalOracle",
            summarize_omena_scss_eval_oracle(
                fixture.source,
                fixture.dialect,
                fixture.candidate_evaluated_css,
            ),
        ),
        scss_eval_public_summary_hash(
            "staticStylesheetValueResolution",
            summarize_static_stylesheet_value_resolution_scanner_oracle(
                fixture.source,
                fixture.dialect,
            ),
        ),
    ]
}

fn scss_eval_public_summary_hash<T: Serialize>(
    summary: &'static str,
    value: T,
) -> OmenaDiffScssEvalPublicSummaryHashV0 {
    let json = serde_json::to_string(&value)
        .unwrap_or_else(|error| format!("{{\"serializationError\":\"{}\"}}", error));
    OmenaDiffScssEvalPublicSummaryHashV0 {
        summary,
        json_hash: stable_json_hash(json.as_bytes()),
    }
}

fn scss_eval_public_summary_snapshot() -> OmenaDiffScssEvalPublicSummarySnapshotV0 {
    serde_json::from_str(SCSS_EVAL_PUBLIC_SUMMARY_SNAPSHOT_SOURCE).unwrap_or_else(|_| {
        OmenaDiffScssEvalPublicSummarySnapshotV0 {
            schema_version: "0".to_string(),
            product: "omena-diff-test.scss-eval-public-summary-snapshot".to_string(),
            fixtures: Vec::new(),
        }
    })
}

fn scss_eval_public_summary_expected_hash(
    snapshot: &OmenaDiffScssEvalPublicSummarySnapshotV0,
    fixture_id: &str,
    summary: &str,
) -> Option<String> {
    snapshot
        .fixtures
        .iter()
        .find(|fixture| fixture.id == fixture_id)?
        .summaries
        .iter()
        .find(|entry| entry.summary == summary)
        .map(|entry| entry.json_hash.clone())
}

fn stable_json_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

const SCSS_EVAL_PUBLIC_SUMMARY_FIXTURES: &[ScssEvalPublicSummaryFixtureV0] = &[
    ScssEvalPublicSummaryFixtureV0 {
        id: "scss.control-flow-call-return-static-values",
        dialect: StyleDialect::Scss,
        source: "$enabled: true;\n$gap: 1px + 1px;\n@function tone($name) { @if $name == primary { @return red; } @return blue; }\n@if $enabled { .button { color: tone(primary); margin: $gap; } }\n",
        candidate_evaluated_css: ".button { color: red; margin: 2px; }\n",
    },
    ScssEvalPublicSummaryFixtureV0 {
        id: "scss.loop-and-comparison-condition",
        dialect: StyleDialect::Scss,
        source: "$i: 0;\n@while $i < 2 { $i: $i + 1; .item-#{$i} { order: $i; } }\n",
        candidate_evaluated_css: ".item-1 { order: 1; }\n.item-2 { order: 2; }\n",
    },
    ScssEvalPublicSummaryFixtureV0 {
        id: "sass.indented-return-branches",
        dialect: StyleDialect::Sass,
        source: "@function tone($enabled)\n  @if $enabled\n    @return red\n  @else if not $enabled\n    @return blue\n  @return green\n\n.button\n  color: tone(true)\n",
        candidate_evaluated_css: ".button { color: red; }\n",
    },
    ScssEvalPublicSummaryFixtureV0 {
        id: "css.native-condition-control-flow",
        dialect: StyleDialect::Css,
        source: "@when supports(display: grid) { .grid { display: grid; } } @else { .grid { display: block; } } .card { margin: if(media(width >= 1px): 1rem; else: 2rem); }\n",
        candidate_evaluated_css: ".grid { display: grid; }\n.card { margin: 1rem; }\n",
    },
];
