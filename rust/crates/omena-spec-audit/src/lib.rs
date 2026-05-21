//! Pinned CSS specification source audit substrate for Omena CSS.
//!
//! M4 uses this crate to make spec provenance and P0 gap policy explicit before
//! the larger generated webref/browser-data importer lands.

use omena_meta_macros::{pass, spec};
use serde::{Deserialize, Serialize};

/// Compile-time marker proving the spec metadata attribute is available to the
/// spec audit layer.
#[spec(webref = "css-color/properties/color", priority = "P0")]
pub const SPEC_AUDIT_COLOR_MARKER: &str = "css-color/properties/color";

/// Compile-time marker proving transform pass metadata can share the same macro
/// substrate.
#[pass(id = "color-compression", ordinal = 5, layer = "value-normalization")]
pub const SPEC_AUDIT_PASS_MARKER: &str = "color-compression";

const SPEC_SOURCE_PINS_SOURCE: &str = include_str!("../data/spec-sources.json");
const OMENA_SPEC_MANIFEST_SOURCE: &str = include_str!("../data/omena-spec-manifest.json");

/// Boundary summary for the Stage 1 spec audit substrate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSpecAuditBoundarySummaryV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Current conformance stage.
    pub stage: String,
    /// Number of pinned external data sources.
    pub source_count: usize,
    /// Number of Omena manifest entries.
    pub manifest_entry_count: usize,
    /// Number of P0 manifest entries.
    pub p0_entry_count: usize,
    /// P0 entries that are missing without an explicit rationale.
    pub blocking_p0_gap_count: usize,
    /// Whether every source has a package version, tarball, and 40-char git head.
    pub all_source_pins_valid: bool,
    /// Whether source freshness metadata is present and internally consistent.
    pub source_freshness_policy_valid: bool,
    /// Whether changed generated-data surfaces require human review.
    pub generated_data_review_gate_valid: bool,
    /// Whether every P0 missing/deferred/not-applicable entry has a rationale.
    pub all_p0_gaps_have_rationale: bool,
    /// Named gates closed by this boundary.
    pub closed_gates: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecSourcePinsV0 {
    schema_version: String,
    product: String,
    refreshed_at: String,
    refresh_policy: SpecSourceRefreshPolicyV0,
    generated_data_review_gate: SpecGeneratedDataReviewGateV0,
    sources: Vec<SpecSourcePinV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecSourceRefreshPolicyV0 {
    max_age_days: u32,
    next_review_due_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecGeneratedDataReviewGateV0 {
    human_review_required: bool,
    changed_generated_data_requires_review: bool,
    auto_merge_allowed: bool,
    reviewer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecSourcePinV0 {
    name: String,
    package: String,
    version: String,
    git_head: String,
    tarball: String,
    role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSpecManifestV0 {
    schema_version: String,
    product: String,
    stage: String,
    entries: Vec<OmenaSpecManifestEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSpecManifestEntryV0 {
    id: String,
    priority: String,
    status: String,
    owner: String,
    #[serde(default)]
    rationale: Option<String>,
    #[serde(default)]
    evidence: Vec<String>,
}

/// Summarize the spec audit boundary.
pub fn summarize_omena_spec_audit_boundary() -> OmenaSpecAuditBoundarySummaryV0 {
    let source_pins = serde_json::from_str::<SpecSourcePinsV0>(SPEC_SOURCE_PINS_SOURCE).ok();
    let manifest = serde_json::from_str::<OmenaSpecManifestV0>(OMENA_SPEC_MANIFEST_SOURCE).ok();
    let sources = source_pins
        .as_ref()
        .map(|source_pins| source_pins.sources.as_slice())
        .unwrap_or(&[]);
    let entries = manifest
        .as_ref()
        .map(|manifest| manifest.entries.as_slice())
        .unwrap_or(&[]);
    let p0_entries = entries
        .iter()
        .filter(|entry| entry.priority.as_str() == "P0")
        .count();
    let blocking_p0_gap_count = entries
        .iter()
        .filter(|entry| entry.priority.as_str() == "P0")
        .filter(|entry| entry.status.as_str() == "missing")
        .filter(|entry| !entry_has_rationale(entry))
        .count();
    let all_p0_gaps_have_rationale = entries
        .iter()
        .filter(|entry| entry.priority.as_str() == "P0")
        .filter(|entry| {
            matches!(
                entry.status.as_str(),
                "missing" | "deferred" | "not-applicable"
            )
        })
        .all(entry_has_rationale);
    let all_source_pins_valid = source_pins
        .as_ref()
        .is_some_and(|source_pins| source_pins.schema_version == "0")
        && source_pins
            .as_ref()
            .is_some_and(|source_pins| source_pins.product == "omena-spec-audit.source-pins")
        && !sources.is_empty()
        && sources.iter().all(source_pin_is_valid);
    let source_freshness_policy_valid = source_pins
        .as_ref()
        .is_some_and(source_freshness_policy_is_valid);
    let generated_data_review_gate_valid = source_pins
        .as_ref()
        .is_some_and(generated_data_review_gate_is_valid);
    let stage = manifest
        .as_ref()
        .map(|manifest| manifest.stage.clone())
        .unwrap_or_else(|| "invalid".to_string());
    let manifest_valid = manifest
        .as_ref()
        .is_some_and(|manifest| manifest.schema_version == "0")
        && manifest
            .as_ref()
            .is_some_and(|manifest| manifest.product == "omena-spec-audit.single-source-manifest")
        && !entries.is_empty()
        && entries.iter().all(manifest_entry_is_valid);

    OmenaSpecAuditBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-spec-audit.boundary",
        stage,
        source_count: sources.len(),
        manifest_entry_count: entries.len(),
        p0_entry_count: p0_entries,
        blocking_p0_gap_count,
        all_source_pins_valid,
        source_freshness_policy_valid,
        generated_data_review_gate_valid,
        all_p0_gaps_have_rationale: manifest_valid && all_p0_gaps_have_rationale,
        closed_gates: vec![
            "specAuditSourcePins",
            "specAuditSingleSourceManifest",
            "specAuditP0GapRationalePolicy",
            "specAuditSourceFreshnessPolicy",
            "generatedDataHumanReviewGate",
            "metaMacroAttributeShape",
        ],
    }
}

fn source_pin_is_valid(source: &SpecSourcePinV0) -> bool {
    !source.name.is_empty()
        && !source.package.is_empty()
        && !source.version.is_empty()
        && source.git_head.len() == 40
        && source.git_head.chars().all(|char| char.is_ascii_hexdigit())
        && source.tarball.starts_with("https://registry.npmjs.org/")
        && !source.role.is_empty()
}

fn source_freshness_policy_is_valid(source_pins: &SpecSourcePinsV0) -> bool {
    let policy = &source_pins.refresh_policy;
    is_yyyy_mm_dd(source_pins.refreshed_at.as_str())
        && is_yyyy_mm_dd(policy.next_review_due_at.as_str())
        && policy.max_age_days > 0
        && policy.max_age_days <= 90
        && date_key(policy.next_review_due_at.as_str())
            >= date_key(source_pins.refreshed_at.as_str())
}

fn generated_data_review_gate_is_valid(source_pins: &SpecSourcePinsV0) -> bool {
    let gate = &source_pins.generated_data_review_gate;
    gate.human_review_required
        && gate.changed_generated_data_requires_review
        && !gate.auto_merge_allowed
        && !gate.reviewer.trim().is_empty()
}

fn is_yyyy_mm_dd(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
        && month_day_are_in_basic_range(value)
}

fn month_day_are_in_basic_range(value: &str) -> bool {
    let Some(month) = value.get(5..7).and_then(|month| month.parse::<u32>().ok()) else {
        return false;
    };
    let Some(day) = value.get(8..10).and_then(|day| day.parse::<u32>().ok()) else {
        return false;
    };
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn date_key(value: &str) -> Option<u32> {
    if !is_yyyy_mm_dd(value) {
        return None;
    }
    value.replace('-', "").parse::<u32>().ok()
}

fn manifest_entry_is_valid(entry: &OmenaSpecManifestEntryV0) -> bool {
    !entry.id.is_empty()
        && matches!(entry.priority.as_str(), "P0" | "P1" | "P2" | "P3")
        && matches!(
            entry.status.as_str(),
            "covered" | "missing" | "deferred" | "not-applicable"
        )
        && !entry.owner.is_empty()
        && (entry.status != "covered" || !entry.evidence.is_empty())
}

fn entry_has_rationale(entry: &OmenaSpecManifestEntryV0) -> bool {
    entry
        .rationale
        .as_ref()
        .is_some_and(|rationale| !rationale.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::{
        SPEC_AUDIT_COLOR_MARKER, SPEC_AUDIT_PASS_MARKER, SpecGeneratedDataReviewGateV0,
        SpecSourcePinsV0, SpecSourceRefreshPolicyV0, generated_data_review_gate_is_valid,
        source_freshness_policy_is_valid, summarize_omena_spec_audit_boundary,
    };

    #[test]
    fn boundary_reports_pinned_sources_and_p0_policy() {
        let summary = summarize_omena_spec_audit_boundary();

        assert_eq!(summary.product, "omena-spec-audit.boundary");
        assert_eq!(summary.stage, "stage1-advisory");
        assert_eq!(summary.source_count, 4);
        assert_eq!(summary.manifest_entry_count, 5);
        assert_eq!(summary.p0_entry_count, 4);
        assert_eq!(summary.blocking_p0_gap_count, 0);
        assert!(summary.all_source_pins_valid);
        assert!(summary.source_freshness_policy_valid);
        assert!(summary.generated_data_review_gate_valid);
        assert!(summary.all_p0_gaps_have_rationale);
        assert!(summary.closed_gates.contains(&"specAuditSourcePins"));
        assert!(
            summary
                .closed_gates
                .contains(&"specAuditSourceFreshnessPolicy")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"generatedDataHumanReviewGate")
        );
        assert!(summary.closed_gates.contains(&"metaMacroAttributeShape"));
    }

    #[test]
    fn metadata_macro_markers_compile_in_spec_audit_layer() {
        assert_eq!(SPEC_AUDIT_COLOR_MARKER, "css-color/properties/color");
        assert_eq!(SPEC_AUDIT_PASS_MARKER, "color-compression");
    }

    #[test]
    fn freshness_and_review_policy_reject_invalid_shapes() {
        let mut source_pins = SpecSourcePinsV0 {
            schema_version: "0".to_string(),
            product: "omena-spec-audit.source-pins".to_string(),
            refreshed_at: "2026-05-22".to_string(),
            refresh_policy: SpecSourceRefreshPolicyV0 {
                max_age_days: 30,
                next_review_due_at: "2026-06-21".to_string(),
            },
            generated_data_review_gate: SpecGeneratedDataReviewGateV0 {
                human_review_required: true,
                changed_generated_data_requires_review: true,
                auto_merge_allowed: false,
                reviewer: "maintainer".to_string(),
            },
            sources: Vec::new(),
        };

        assert!(source_freshness_policy_is_valid(&source_pins));
        assert!(generated_data_review_gate_is_valid(&source_pins));

        source_pins.refresh_policy.next_review_due_at = "2026-05-01".to_string();
        assert!(!source_freshness_policy_is_valid(&source_pins));
        source_pins.refresh_policy.next_review_due_at = "2026-06-21".to_string();

        source_pins.generated_data_review_gate.auto_merge_allowed = true;
        assert!(!generated_data_review_gate_is_valid(&source_pins));
    }
}
