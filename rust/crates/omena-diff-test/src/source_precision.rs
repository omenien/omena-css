use std::collections::BTreeMap;

use omena_abstract_value::AbstractClassValueV0;
use omena_query::resolve_omena_query_source_precision_for_source;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSourcePrecisionReferenceReportV0 {
    pub id: String,
    pub source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
    pub variable_name: String,
    pub reference_byte_offset: usize,
    pub resolved_tier: String,
    pub precision_stratum: String,
    pub resolved_value: AbstractClassValueV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_cause: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSourcePrecisionBaselineV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub corpus_manifest_hash: String,
    pub tier_order: Vec<&'static str>,
    pub total_reference_count: usize,
    pub non_top_reference_count: usize,
    pub top_reference_count: usize,
    pub non_top_share_basis_points: usize,
    pub tier_histogram: BTreeMap<String, usize>,
    pub precision_stratum_histogram: BTreeMap<String, usize>,
    pub top_cause_histogram: BTreeMap<String, usize>,
    pub bottom_witnesses: BTreeMap<String, String>,
    pub references: Vec<OmenaSourcePrecisionReferenceReportV0>,
}

struct SourcePrecisionFixtureV0 {
    id: &'static str,
    source_path: &'static str,
    source_language: Option<&'static str>,
    variable_name: &'static str,
    reference_needle: &'static str,
    source: &'static str,
}

pub fn summarize_omena_source_precision_baseline() -> OmenaSourcePrecisionBaselineV0 {
    let fixtures = source_precision_fixtures();
    let references = fixtures
        .iter()
        .map(source_precision_reference_report)
        .collect::<Vec<_>>();
    let mut tier_histogram = empty_tier_histogram();
    let mut precision_stratum_histogram = BTreeMap::new();
    let mut top_cause_histogram = BTreeMap::new();

    for reference in &references {
        *tier_histogram
            .entry(reference.resolved_tier.clone())
            .or_default() += 1;
        *precision_stratum_histogram
            .entry(reference.precision_stratum.clone())
            .or_default() += 1;
        if reference.resolved_tier == "top" {
            let cause = reference.top_cause.as_deref().unwrap_or("topValueFacts");
            *top_cause_histogram.entry(cause.to_string()).or_default() += 1;
        }
    }

    let total_reference_count = references.len();
    let top_reference_count = tier_histogram.get("top").copied().unwrap_or_default();
    let non_top_reference_count = total_reference_count.saturating_sub(top_reference_count);
    let non_top_share_basis_points = (non_top_reference_count * 10_000)
        .checked_div(total_reference_count)
        .unwrap_or_default();

    OmenaSourcePrecisionBaselineV0 {
        schema_version: "0",
        product: "omena-diff-test.source-precision-baseline",
        corpus_manifest_hash: source_precision_corpus_manifest_hash(fixtures.as_slice()),
        tier_order: source_precision_tier_order(),
        total_reference_count,
        non_top_reference_count,
        top_reference_count,
        non_top_share_basis_points,
        tier_histogram,
        precision_stratum_histogram,
        top_cause_histogram,
        bottom_witnesses: BTreeMap::new(),
        references,
    }
}

fn source_precision_reference_report(
    fixture: &SourcePrecisionFixtureV0,
) -> OmenaSourcePrecisionReferenceReportV0 {
    let reference_byte_offset = fixture
        .source
        .rfind(fixture.reference_needle)
        .unwrap_or(fixture.source.len());
    let resolved = resolve_omena_query_source_precision_for_source(
        fixture.source_path,
        fixture.source,
        fixture.source_language,
        fixture.variable_name,
        reference_byte_offset,
    );
    OmenaSourcePrecisionReferenceReportV0 {
        id: fixture.id.to_string(),
        source_path: resolved.source_path,
        source_language: resolved.source_language,
        variable_name: resolved.variable_name,
        reference_byte_offset: resolved.reference_byte_offset,
        resolved_tier: resolved.resolved_tier.to_string(),
        precision_stratum: source_precision_stratum(&resolved.precision),
        resolved_value: resolved.resolved_value,
        top_cause: resolved.top_cause.map(str::to_string),
    }
}

fn source_precision_stratum(precision: &omena_query::OmenaQueryAnalysisPrecisionV0) -> String {
    format!(
        "{}|{}|{}",
        precision.value_domain, precision.flow_sensitivity, precision.context_sensitivity
    )
}

fn source_precision_tier_order() -> Vec<&'static str> {
    vec![
        "top",
        "automaton",
        "prefix",
        "suffix",
        "prefixSuffix",
        "charInclusion",
        "composite",
        "finiteSet",
        "exact",
        "bottom",
    ]
}

fn empty_tier_histogram() -> BTreeMap<String, usize> {
    source_precision_tier_order()
        .into_iter()
        .map(|tier| (tier.to_string(), 0))
        .collect()
}

fn source_precision_corpus_manifest_hash(fixtures: &[SourcePrecisionFixtureV0]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for fixture in fixtures {
        for part in [
            fixture.id,
            fixture.source_path,
            fixture.source_language.unwrap_or(""),
            fixture.variable_name,
            fixture.reference_needle,
            fixture.source,
        ] {
            for byte in part.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            hash ^= 0xff;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("fnv1a64:{hash:016x}")
}

fn source_precision_fixtures() -> Vec<SourcePrecisionFixtureV0> {
    vec![
        SourcePrecisionFixtureV0 {
            id: "same-file-helper-return",
            source_path: "/fake/ws/src/StatusCard.tsx",
            source_language: Some("typescriptreact"),
            variable_name: "size",
            reference_needle: "size",
            source: r#"type Status = "idle" | "busy" | "error";
function resolveStatusClass(status: Status): string {
  switch (status) {
    case "idle": return "state-idle";
    case "busy": return "state-busy";
    case "error": return "state-error";
    default: return "state-idle";
  }
}
export function Card(status: Status) {
  const size = resolveStatusClass(status);
  return cx(size);
}
"#,
        },
        SourcePrecisionFixtureV0 {
            id: "concatenated-affix",
            source_path: "/fake/ws/src/AffixCard.tsx",
            source_language: Some("typescriptreact"),
            variable_name: "size",
            reference_needle: "size",
            source: r#"export function Card(variant: string) {
  const size = "btn-" + variant + "-chip";
  return cx(size);
}
"#,
        },
        SourcePrecisionFixtureV0 {
            id: "branch-reassignment",
            source_path: "/fake/ws/src/BranchCard.tsx",
            source_language: Some("typescriptreact"),
            variable_name: "size",
            reference_needle: "size",
            source: r#"export function Card({ enabled }: { enabled: boolean }) {
  let size = "card";
  if (enabled) {
    size = "card--active";
  }
  return <div className={size} />;
}
"#,
        },
        SourcePrecisionFixtureV0 {
            id: "ambiguous-declaration",
            source_path: "/fake/ws/src/AmbiguousCard.tsx",
            source_language: Some("typescriptreact"),
            variable_name: "size",
            reference_needle: "size",
            source: r#"export function Card(enabled: boolean) {
  var size = "card";
  if (enabled) {
    var size = "card--active";
  }
  return cx(size);
}
"#,
        },
        SourcePrecisionFixtureV0 {
            id: "style-member-access",
            source_path: "/fake/ws/src/MemberCard.tsx",
            source_language: Some("typescriptreact"),
            variable_name: "styles.primary",
            reference_needle: "styles.primary",
            source: r#"import styles from "./Card.module.css";
export function Card() {
  return cx(styles.primary);
}
"#,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_precision_baseline_is_non_vacuous() {
        let baseline = summarize_omena_source_precision_baseline();
        assert_eq!(baseline.total_reference_count, 5);
        assert!(baseline.non_top_reference_count >= 2);
        assert!(baseline.top_reference_count >= 1);
        assert_eq!(baseline.tier_histogram.get("finiteSet").copied(), Some(1));
        assert_eq!(
            baseline.tier_histogram.get("prefixSuffix").copied(),
            Some(1)
        );
        assert_eq!(
            baseline
                .precision_stratum_histogram
                .get("classValueResolution|sourceControlFlow|sameFile")
                .copied(),
            Some(5)
        );
    }

    #[test]
    fn source_precision_baseline_records_top_causes() {
        let baseline = summarize_omena_source_precision_baseline();
        assert!(
            baseline
                .top_cause_histogram
                .contains_key("ambiguousFlowSnapshot")
        );
        assert!(baseline.top_cause_histogram.contains_key("noFlowCapture"));
    }
}
