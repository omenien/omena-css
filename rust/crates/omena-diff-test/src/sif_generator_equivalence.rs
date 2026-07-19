//! Differential evidence for the static Sass interface generator migration.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use omena_sif::{
    OmenaSifExportsV1, OmenaSifSourceSyntaxV1, OmenaSifStaticGeneratorInputV1,
    generate_static_omena_lif_exports_v1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaSifGeneratorCorrectionKindV0 {
    CommentDelimiterIsolation,
    InterpolationBoundaryPreservation,
    IndentedSassCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifGeneratorEquivalenceFixtureReportV0 {
    pub id: String,
    pub syntax: &'static str,
    pub source_hash: String,
    pub scanner_hash: String,
    pub parser_fact_hash: String,
    pub scanner_baseline_present: bool,
    pub scanner_baseline_current: bool,
    pub exact_match: bool,
    pub correction_kind: Option<OmenaSifGeneratorCorrectionKindV0>,
    pub correction_witness_holds: bool,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifGeneratorEquivalenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub exact_match_count: usize,
    pub intended_correction_count: usize,
    pub scanner_baseline_valid: bool,
    pub adjudication_table_valid: bool,
    pub missing_scanner_baseline_fixture_ids: Vec<String>,
    pub stale_scanner_baseline_fixture_ids: Vec<String>,
    pub changed_scanner_baseline_fixture_ids: Vec<String>,
    pub unadjudicated_divergence_fixture_ids: Vec<String>,
    pub stale_adjudication_fixture_ids: Vec<String>,
    pub all_fixtures_accounted_for: bool,
    pub fixtures: Vec<OmenaSifGeneratorEquivalenceFixtureReportV0>,
}

#[derive(Debug, Clone)]
struct SifGeneratorFixtureV0 {
    id: String,
    syntax: OmenaSifSourceSyntaxV1,
    source: String,
}

#[derive(Debug, Clone, Copy)]
struct SifGeneratorCorrectionAdjudicationV0 {
    fixture_id: &'static str,
    kind: OmenaSifGeneratorCorrectionKindV0,
    scanner_hash: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SifGeneratorScannerBaselineV0 {
    schema_version: String,
    product: String,
    fixtures: Vec<SifGeneratorScannerBaselineFixtureV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SifGeneratorScannerBaselineFixtureV0 {
    id: String,
    syntax: String,
    source_hash: String,
    scanner_hash: String,
}

const CORRECTION_ADJUDICATIONS: &[SifGeneratorCorrectionAdjudicationV0] = &[
    SifGeneratorCorrectionAdjudicationV0 {
        fixture_id: "static-generator.comment-delimiter-isolation",
        kind: OmenaSifGeneratorCorrectionKindV0::CommentDelimiterIsolation,
        scanner_hash: "cd1f8ab3777e8c8dfdcba7eb4b05413fb733c8ba6021bc67979082c3148ae38a",
    },
    SifGeneratorCorrectionAdjudicationV0 {
        fixture_id: "static-generator.interpolation-boundary-preservation",
        kind: OmenaSifGeneratorCorrectionKindV0::InterpolationBoundaryPreservation,
        scanner_hash: "960445acc3fb4ddd48fd6a75820d0a0b2c65692d96a8d7e8a12adc33171b52ed",
    },
    SifGeneratorCorrectionAdjudicationV0 {
        fixture_id: "static-generator.indented-sass-coverage",
        kind: OmenaSifGeneratorCorrectionKindV0::IndentedSassCoverage,
        scanner_hash: "6e20b5aac02554381bfbfc4be90d5aaac0dee5ea843e3b1ec755c086b410a424",
    },
    SifGeneratorCorrectionAdjudicationV0 {
        fixture_id: "sass-spec-corpus/language-core.json:sass-indented-rule",
        kind: OmenaSifGeneratorCorrectionKindV0::IndentedSassCoverage,
        scanner_hash: "39f7b37b01774173feb9c29b8c109587e87e38972f433b81c9907a4b958da0b5",
    },
];

pub fn summarize_sif_generator_fact_equivalence_v0() -> OmenaSifGeneratorEquivalenceReportV0 {
    let fixtures = sif_generator_equivalence_corpus();
    let baseline = read_scanner_baseline();
    summarize_sif_generator_fact_equivalence_with_baseline(fixtures, baseline)
}

fn summarize_sif_generator_fact_equivalence_with_baseline(
    fixtures: Vec<SifGeneratorFixtureV0>,
    baseline: SifGeneratorScannerBaselineV0,
) -> OmenaSifGeneratorEquivalenceReportV0 {
    let baseline_fixture_ids = baseline
        .fixtures
        .iter()
        .map(|fixture| fixture.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let scanner_baseline_valid = baseline.schema_version == "0"
        && baseline.product == "omena-diff-test.sif-generator-scanner-baseline"
        && !baseline.fixtures.is_empty()
        && baseline_fixture_ids.len() == baseline.fixtures.len();
    let baseline_by_id = baseline
        .fixtures
        .iter()
        .map(|fixture| (fixture.id.as_str(), fixture))
        .collect::<BTreeMap<_, _>>();
    let fixture_ids = fixtures
        .iter()
        .map(|fixture| fixture.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let adjudication_fixture_ids = CORRECTION_ADJUDICATIONS
        .iter()
        .map(|adjudication| adjudication.fixture_id)
        .collect::<std::collections::BTreeSet<_>>();
    let adjudication_table_valid = adjudication_fixture_ids.len() == CORRECTION_ADJUDICATIONS.len()
        && adjudication_fixture_ids.is_subset(&fixture_ids);
    let reports = fixtures
        .iter()
        .map(|fixture| {
            sif_generator_fixture_report(fixture, baseline_by_id.get(fixture.id.as_str()).copied())
        })
        .collect::<Vec<_>>();
    let exact_match_count = reports.iter().filter(|report| report.exact_match).count();
    let intended_correction_count = reports
        .iter()
        .filter(|report| !report.exact_match && report.accepted)
        .count();
    let missing_scanner_baseline_fixture_ids = reports
        .iter()
        .filter(|report| !report.scanner_baseline_present)
        .map(|report| report.id.clone())
        .collect::<Vec<_>>();
    let stale_scanner_baseline_fixture_ids = baseline
        .fixtures
        .iter()
        .filter(|fixture| !fixture_ids.contains(fixture.id.as_str()))
        .map(|fixture| fixture.id.clone())
        .collect::<Vec<_>>();
    let changed_scanner_baseline_fixture_ids = reports
        .iter()
        .filter(|report| report.scanner_baseline_present && !report.scanner_baseline_current)
        .map(|report| report.id.clone())
        .collect::<Vec<_>>();
    let unadjudicated_divergence_fixture_ids = reports
        .iter()
        .filter(|report| !report.exact_match && !report.accepted)
        .map(|report| report.id.clone())
        .collect::<Vec<_>>();
    let stale_adjudication_fixture_ids = reports
        .iter()
        .filter(|report| report.exact_match && report.correction_kind.is_some())
        .map(|report| report.id.clone())
        .collect::<Vec<_>>();
    let all_fixtures_accounted_for = scanner_baseline_valid
        && adjudication_table_valid
        && reports.iter().all(|report| report.accepted)
        && missing_scanner_baseline_fixture_ids.is_empty()
        && stale_scanner_baseline_fixture_ids.is_empty()
        && changed_scanner_baseline_fixture_ids.is_empty()
        && unadjudicated_divergence_fixture_ids.is_empty()
        && stale_adjudication_fixture_ids.is_empty();

    OmenaSifGeneratorEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sif-generator-fact-equivalence",
        fixture_count: reports.len(),
        exact_match_count,
        intended_correction_count,
        scanner_baseline_valid,
        adjudication_table_valid,
        missing_scanner_baseline_fixture_ids,
        stale_scanner_baseline_fixture_ids,
        changed_scanner_baseline_fixture_ids,
        unadjudicated_divergence_fixture_ids,
        stale_adjudication_fixture_ids,
        all_fixtures_accounted_for,
        fixtures: reports,
    }
}

fn sif_generator_fixture_report(
    fixture: &SifGeneratorFixtureV0,
    baseline: Option<&SifGeneratorScannerBaselineFixtureV0>,
) -> OmenaSifGeneratorEquivalenceFixtureReportV0 {
    let parser_fact = generate_static_omena_lif_exports_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: fixture.id.as_str(),
        source: fixture.source.as_str(),
        syntax: fixture.syntax.clone(),
    })
    .sif_exports;
    let source_hash = sha256_hex(fixture.source.as_bytes());
    let parser_fact_hash = exports_hash(&parser_fact);
    let scanner_baseline_present = baseline.is_some();
    let scanner_baseline_current = baseline.is_some_and(|baseline| {
        baseline.syntax == syntax_label(&fixture.syntax) && baseline.source_hash == source_hash
    });
    let scanner_hash = baseline
        .map(|baseline| baseline.scanner_hash.clone())
        .unwrap_or_default();
    let exact_match = scanner_baseline_current && scanner_hash == parser_fact_hash;
    let correction = correction_adjudication_for_fixture(fixture.id.as_str());
    let correction_kind = correction.map(|adjudication| adjudication.kind);
    let scanner_baseline_matches_adjudication =
        correction.is_none_or(|adjudication| scanner_hash == adjudication.scanner_hash);
    let correction_witness_holds = correction_kind
        .is_some_and(|kind| correction_witness_holds(fixture.id.as_str(), kind, &parser_fact));
    let accepted = exact_match && correction_kind.is_none()
        || scanner_baseline_current
            && !exact_match
            && correction_kind.is_some()
            && scanner_baseline_matches_adjudication
            && correction_witness_holds;

    OmenaSifGeneratorEquivalenceFixtureReportV0 {
        id: fixture.id.clone(),
        syntax: syntax_label(&fixture.syntax),
        source_hash,
        scanner_hash,
        parser_fact_hash,
        scanner_baseline_present,
        scanner_baseline_current,
        exact_match,
        correction_kind,
        correction_witness_holds,
        accepted,
    }
}

#[cfg(test)]
fn correction_kind_for_fixture(id: &str) -> Option<OmenaSifGeneratorCorrectionKindV0> {
    correction_adjudication_for_fixture(id).map(|entry| entry.kind)
}

fn correction_adjudication_for_fixture(
    id: &str,
) -> Option<&'static SifGeneratorCorrectionAdjudicationV0> {
    CORRECTION_ADJUDICATIONS
        .iter()
        .find(|entry| entry.fixture_id == id)
}

fn correction_witness_holds(
    fixture_id: &str,
    kind: OmenaSifGeneratorCorrectionKindV0,
    parser_fact: &OmenaSifExportsV1,
) -> bool {
    match kind {
        OmenaSifGeneratorCorrectionKindV0::CommentDelimiterIsolation => {
            parser_fact.variables.len() == 1 && parser_fact.variables[0].name == "$brand"
        }
        OmenaSifGeneratorCorrectionKindV0::InterpolationBoundaryPreservation => {
            parser_fact.variables.len() == 1
                && parser_fact.variables[0]
                    .value_repr
                    .as_deref()
                    .is_some_and(|value| value.ends_with("-wide"))
        }
        OmenaSifGeneratorCorrectionKindV0::IndentedSassCoverage => {
            let variable_is_preserved = parser_fact.variables.len() == 1
                && parser_fact.variables[0].name == "$gap"
                && parser_fact.variables[0].value_repr.as_deref() == Some("1rem")
                && !parser_fact.variables[0].defaulted;
            match fixture_id {
                "static-generator.indented-sass-coverage" => {
                    variable_is_preserved
                        && parser_fact.mixins.len() == 1
                        && parser_fact.mixins[0].name == "tone"
                }
                "sass-spec-corpus/language-core.json:sass-indented-rule" => {
                    variable_is_preserved && parser_fact.mixins.is_empty()
                }
                _ => false,
            }
        }
    }
}

fn read_scanner_baseline() -> SifGeneratorScannerBaselineV0 {
    serde_json::from_str(include_str!("sif_generator_scanner_baseline.json")).unwrap_or_else(|_| {
        SifGeneratorScannerBaselineV0 {
            schema_version: String::new(),
            product: String::new(),
            fixtures: Vec::new(),
        }
    })
}

fn sif_generator_equivalence_corpus() -> Vec<SifGeneratorFixtureV0> {
    let mut fixtures = BTreeMap::<String, SifGeneratorFixtureV0>::new();
    for fixture in correction_fixtures() {
        fixtures.insert(fixture.id.clone(), fixture);
    }
    fixtures.insert(
        "static-generator.comment-trait-control".to_string(),
        SifGeneratorFixtureV0 {
            id: "static-generator.comment-trait-control".to_string(),
            syntax: OmenaSifSourceSyntaxV1::Scss,
            source: "/* ; { } */ .card { color: red; }".to_string(),
        },
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
        .unwrap_or_default();
    for root in [repo_root.join("examples"), repo_root.join("test")] {
        collect_style_file_fixtures(&root, &repo_root, &mut fixtures);
    }
    for relative_path in [
        "static-stylesheet-external-differential/manifest.json",
        "sass-spec-corpus/language-core.json",
        "sass-spec-corpus/imported-smoke.json",
        "sass-spec-corpus/conformance-smoke.json",
    ] {
        let path = manifest_dir.join(relative_path);
        if let Ok(source) = fs::read_to_string(&path)
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&source)
        {
            collect_json_source_fixtures(&value, relative_path, &mut fixtures);
        }
    }
    fixtures.into_values().collect()
}

fn correction_fixtures() -> [SifGeneratorFixtureV0; 3] {
    [
        SifGeneratorFixtureV0 {
            id: "static-generator.comment-delimiter-isolation".to_string(),
            syntax: OmenaSifSourceSyntaxV1::Scss,
            source: "/* scanner delimiters ; { } */ $brand: red;".to_string(),
        },
        SifGeneratorFixtureV0 {
            id: "static-generator.interpolation-boundary-preservation".to_string(),
            syntax: OmenaSifSourceSyntaxV1::Scss,
            source: "$token: size-#{1 + 1}-wide;".to_string(),
        },
        SifGeneratorFixtureV0 {
            id: "static-generator.indented-sass-coverage".to_string(),
            syntax: OmenaSifSourceSyntaxV1::Sass,
            source: "$gap: 1rem\n@mixin tone($color: red)\n  color: $color\n".to_string(),
        },
    ]
}

fn collect_style_file_fixtures(
    root: &Path,
    repo_root: &Path,
    fixtures: &mut BTreeMap<String, SifGeneratorFixtureV0>,
) {
    if root.file_name().is_some_and(|name| {
        matches!(
            name.to_str(),
            Some("node_modules" | "target" | "dist" | ".git")
        )
    }) {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_style_file_fixtures(&path, repo_root, fixtures);
            continue;
        }
        let syntax = match path.extension().and_then(|value| value.to_str()) {
            Some("scss") => OmenaSifSourceSyntaxV1::Scss,
            Some("sass") => OmenaSifSourceSyntaxV1::Sass,
            _ => continue,
        };
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        if source.trim().is_empty() {
            continue;
        }
        let relative_path = path.strip_prefix(repo_root).unwrap_or(path.as_path());
        let portable_path = relative_path
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let id = format!("workspace:{portable_path}");
        fixtures.insert(id.clone(), SifGeneratorFixtureV0 { id, syntax, source });
    }
}

fn collect_json_source_fixtures(
    value: &serde_json::Value,
    origin: &str,
    fixtures: &mut BTreeMap<String, SifGeneratorFixtureV0>,
) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                collect_json_source_fixtures(value, origin, fixtures);
            }
        }
        serde_json::Value::Object(object) => {
            if let Some(source) = object.get("source").and_then(serde_json::Value::as_str)
                && let Some(syntax) = object
                    .get("dialect")
                    .or_else(|| object.get("syntax"))
                    .and_then(serde_json::Value::as_str)
                    .and_then(source_syntax)
            {
                let label = object
                    .get("id")
                    .or_else(|| object.get("label"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("anonymous");
                let id = format!("{origin}:{label}");
                fixtures.insert(
                    id.clone(),
                    SifGeneratorFixtureV0 {
                        id,
                        syntax,
                        source: source.to_string(),
                    },
                );
            }
            for value in object.values() {
                collect_json_source_fixtures(value, origin, fixtures);
            }
        }
        _ => {}
    }
}

fn source_syntax(value: &str) -> Option<OmenaSifSourceSyntaxV1> {
    match value.to_ascii_lowercase().as_str() {
        "scss" => Some(OmenaSifSourceSyntaxV1::Scss),
        "sass" => Some(OmenaSifSourceSyntaxV1::Sass),
        _ => None,
    }
}

fn syntax_label(syntax: &OmenaSifSourceSyntaxV1) -> &'static str {
    match syntax {
        OmenaSifSourceSyntaxV1::Scss => "scss",
        OmenaSifSourceSyntaxV1::Sass => "sass",
        OmenaSifSourceSyntaxV1::Css => "css",
        OmenaSifSourceSyntaxV1::Less => "less",
    }
}

fn exports_hash(exports: &OmenaSifExportsV1) -> String {
    let bytes = serde_json::to_vec(exports).unwrap_or_default();
    sha256_hex(&bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_equivalence_accepts_only_enumerated_corrections() {
        let report = summarize_sif_generator_fact_equivalence_v0();
        assert!(
            report.fixture_count >= 50,
            "equivalence corpus unexpectedly shrank: {}",
            report.fixture_count
        );
        assert_eq!(report.intended_correction_count, 4);
        assert!(report.scanner_baseline_valid);
        assert!(report.adjudication_table_valid);
        assert!(
            report.unadjudicated_divergence_fixture_ids.is_empty(),
            "unadjudicated generator divergences: {:?}",
            report.unadjudicated_divergence_fixture_ids
        );
        assert!(
            report.stale_adjudication_fixture_ids.is_empty(),
            "stale generator adjudications: {:?}",
            report.stale_adjudication_fixture_ids
        );
        assert!(report.all_fixtures_accounted_for);
    }

    #[test]
    fn source_traits_do_not_auto_classify_matching_outputs() {
        let report = summarize_sif_generator_fact_equivalence_v0();
        let control = report
            .fixtures
            .iter()
            .find(|fixture| fixture.id == "static-generator.comment-trait-control");
        assert!(control.is_some(), "comment-trait control fixture");
        let Some(control) = control else {
            return;
        };
        assert!(control.exact_match);
        assert!(control.correction_kind.is_none());
        assert!(control.accepted);
    }

    #[test]
    fn scanner_baseline_source_and_hash_are_load_bearing() {
        let fixtures = sif_generator_equivalence_corpus();
        let mut source_changed = read_scanner_baseline();
        let source_changed_id = source_changed.fixtures[0].id.clone();
        source_changed.fixtures[0].source_hash = "changed".to_string();
        let report = summarize_sif_generator_fact_equivalence_with_baseline(
            fixtures.clone(),
            source_changed,
        );
        assert!(!report.all_fixtures_accounted_for);
        assert_eq!(
            report.changed_scanner_baseline_fixture_ids,
            vec![source_changed_id]
        );

        let mut hash_changed = read_scanner_baseline();
        let exact_fixture = hash_changed
            .fixtures
            .iter_mut()
            .find(|fixture| correction_kind_for_fixture(&fixture.id).is_none());
        assert!(exact_fixture.is_some(), "exact scanner baseline fixture");
        let Some(exact_fixture) = exact_fixture else {
            return;
        };
        let hash_changed_id = exact_fixture.id.clone();
        exact_fixture.scanner_hash = "changed".to_string();
        let report = summarize_sif_generator_fact_equivalence_with_baseline(fixtures, hash_changed);
        assert!(!report.all_fixtures_accounted_for);
        assert!(
            report
                .unadjudicated_divergence_fixture_ids
                .contains(&hash_changed_id)
        );

        let mut correction_changed = read_scanner_baseline();
        let correction_fixture = correction_changed
            .fixtures
            .iter_mut()
            .find(|fixture| correction_kind_for_fixture(&fixture.id).is_some());
        assert!(
            correction_fixture.is_some(),
            "adjudicated scanner baseline fixture"
        );
        let Some(correction_fixture) = correction_fixture else {
            return;
        };
        let correction_changed_id = correction_fixture.id.clone();
        correction_fixture.scanner_hash = "changed".to_string();
        let report = summarize_sif_generator_fact_equivalence_with_baseline(
            sif_generator_equivalence_corpus(),
            correction_changed,
        );
        assert!(!report.all_fixtures_accounted_for);
        assert!(
            report
                .unadjudicated_divergence_fixture_ids
                .contains(&correction_changed_id)
        );
    }
}
