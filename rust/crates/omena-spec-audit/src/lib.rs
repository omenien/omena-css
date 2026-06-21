//! Pinned CSS specification source audit substrate for Omena CSS.
//!
//! M4 uses this crate to make spec provenance and P0 gap policy explicit before
//! the larger generated webref/browser-data importer lands.

use omena_meta_macros::{pass, spec};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
const WEBREF_GRAMMAR_SOURCE: &str = include_str!("../data/webref-grammar.json");

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
    /// Entries whose manifest metadata links back to a pinned source.
    pub source_linked_entry_count: usize,
    /// Entries sourced from the primary webref CSS package.
    pub webref_entry_count: usize,
    /// Source-coverage declarations in the single-source manifest.
    pub source_coverage_count: usize,
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
    /// Whether every manifest entry cross-references a pinned spec source.
    pub manifest_cross_references_valid: bool,
    /// Whether manifest source coverage references pinned sources and entries.
    pub manifest_source_coverage_valid: bool,
    /// Whether every pinned source is represented by manifest coverage metadata.
    pub all_pinned_sources_have_manifest_coverage: bool,
    /// Number of vendored webref value-definition-syntax grammar entries.
    pub webref_grammar_entry_count: usize,
    /// Vendored webref grammar entries the consumer modeled (non-`Raw`).
    pub webref_grammar_modeled_entry_count: usize,
    /// Whether every vendored grammar entry round-trips (parsed or `Raw`, none
    /// dropped) and the snapshot's entry count is internally consistent.
    pub all_webref_grammar_entries_valid: bool,
    /// Whether the vendored grammar's stamped version + git head match the pin.
    pub webref_grammar_provenance_valid: bool,
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
    source_coverage: Vec<OmenaSpecManifestSourceCoverageV0>,
    entries: Vec<OmenaSpecManifestEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSpecManifestSourceCoverageV0 {
    source_name: String,
    usage: String,
    entry_ids: Vec<String>,
    source_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSpecManifestEntryV0 {
    id: String,
    webref_id: String,
    source_name: String,
    source_category: String,
    spec_url: String,
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
    let source_coverage = manifest
        .as_ref()
        .map(|manifest| manifest.source_coverage.as_slice())
        .unwrap_or(&[]);
    let p0_entries = entries
        .iter()
        .filter(|entry| entry.priority.as_str() == "P0")
        .count();
    let source_by_name = sources
        .iter()
        .map(|source| (source.name.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let source_linked_entries = entries
        .iter()
        .filter(|entry| source_by_name.contains_key(entry.source_name.as_str()))
        .count();
    let webref_entries = entries
        .iter()
        .filter(|entry| {
            source_by_name
                .get(entry.source_name.as_str())
                .is_some_and(|source| source.package == "@webref/css")
        })
        .count();
    let manifest_entry_ids = entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_source_coverage_valid = !source_coverage.is_empty()
        && source_coverage.iter().all(|coverage| {
            manifest_source_coverage_is_valid(coverage, &source_by_name, &manifest_entry_ids)
        });
    let covered_source_names = source_coverage
        .iter()
        .map(|coverage| coverage.source_name.as_str())
        .collect::<BTreeSet<_>>();
    let all_pinned_sources_have_manifest_coverage = !sources.is_empty()
        && sources
            .iter()
            .all(|source| covered_source_names.contains(source.name.as_str()));
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
    let manifest_shape_valid = manifest
        .as_ref()
        .is_some_and(|manifest| manifest.schema_version == "0")
        && manifest
            .as_ref()
            .is_some_and(|manifest| manifest.product == "omena-spec-audit.single-source-manifest")
        && !entries.is_empty()
        && entries.iter().all(manifest_entry_is_valid);
    let manifest_source_coverage_valid = manifest_shape_valid
        && manifest_source_coverage_valid
        && all_pinned_sources_have_manifest_coverage;
    let manifest_cross_references_valid = manifest_shape_valid
        && entries
            .iter()
            .all(|entry| manifest_entry_cross_reference_is_valid(entry, &source_by_name));
    let webref_pin = sources
        .iter()
        .find(|source| source.package == "@webref/css");
    let webref_grammar = summarize_webref_grammar(webref_pin);

    OmenaSpecAuditBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-spec-audit.boundary",
        stage,
        source_count: sources.len(),
        manifest_entry_count: entries.len(),
        p0_entry_count: p0_entries,
        source_linked_entry_count: source_linked_entries,
        webref_entry_count: webref_entries,
        source_coverage_count: source_coverage.len(),
        blocking_p0_gap_count,
        all_source_pins_valid,
        source_freshness_policy_valid,
        generated_data_review_gate_valid,
        all_p0_gaps_have_rationale: manifest_shape_valid && all_p0_gaps_have_rationale,
        manifest_cross_references_valid,
        manifest_source_coverage_valid,
        all_pinned_sources_have_manifest_coverage,
        webref_grammar_entry_count: webref_grammar.entry_count,
        webref_grammar_modeled_entry_count: webref_grammar.modeled_entry_count,
        all_webref_grammar_entries_valid: webref_grammar.all_entries_valid,
        webref_grammar_provenance_valid: webref_grammar.provenance_valid,
        closed_gates: vec![
            "specAuditSourcePins",
            "specAuditSingleSourceManifest",
            "specAuditManifestSourceCrossReferences",
            "specAuditCrossSourceCoverage",
            "specAuditP0GapRationalePolicy",
            "specAuditSourceFreshnessPolicy",
            "generatedDataHumanReviewGate",
            "metaMacroAttributeShape",
            "webrefGrammarConsumer",
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
        && !entry.webref_id.is_empty()
        && !entry.source_name.is_empty()
        && webref_category_is_valid(entry.source_category.as_str())
        && entry.spec_url.starts_with("https://")
        && matches!(entry.priority.as_str(), "P0" | "P1" | "P2" | "P3")
        && matches!(
            entry.status.as_str(),
            "covered" | "missing" | "deferred" | "not-applicable"
        )
        && !entry.owner.is_empty()
        && (entry.status != "covered" || !entry.evidence.is_empty())
}

fn manifest_entry_cross_reference_is_valid(
    entry: &OmenaSpecManifestEntryV0,
    source_by_name: &BTreeMap<&str, &SpecSourcePinV0>,
) -> bool {
    source_by_name
        .get(entry.source_name.as_str())
        .is_some_and(|source| {
            !source.version.is_empty()
                && (source.package != "@webref/css" || entry.webref_id == entry.id)
        })
}

fn manifest_source_coverage_is_valid(
    coverage: &OmenaSpecManifestSourceCoverageV0,
    source_by_name: &BTreeMap<&str, &SpecSourcePinV0>,
    manifest_entry_ids: &BTreeSet<&str>,
) -> bool {
    source_by_name.contains_key(coverage.source_name.as_str())
        && !coverage.usage.trim().is_empty()
        && !coverage.entry_ids.is_empty()
        && coverage
            .entry_ids
            .iter()
            .all(|entry_id| manifest_entry_ids.contains(entry_id.as_str()))
        && !coverage.source_keys.is_empty()
        && coverage
            .source_keys
            .iter()
            .all(|source_key| !source_key.trim().is_empty())
}

fn webref_category_is_valid(category: &str) -> bool {
    matches!(
        category,
        "atrules" | "descriptors" | "properties" | "selectors" | "values"
    )
}

fn entry_has_rationale(entry: &OmenaSpecManifestEntryV0) -> bool {
    entry
        .rationale
        .as_ref()
        .is_some_and(|rationale| !rationale.trim().is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebrefGrammarSnapshotV0 {
    schema_version: String,
    product: String,
    source: WebrefGrammarSourceV0,
    entry_count: usize,
    categories: BTreeMap<String, Vec<WebrefGrammarEntryV0>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebrefGrammarSourceV0 {
    package: String,
    version: String,
    git_head: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct WebrefGrammarEntryV0 {
    name: String,
    syntax: String,
}

/// Coarse classification of a webref value-definition-syntax string. The consumer
/// is conservative: anything it cannot model with certainty is preserved as `Raw`,
/// so no entry is dropped and no structure is guessed.
#[derive(Debug, Clone, PartialEq, Eq)]
enum WebrefGrammarTermV0 {
    Reference(String),
    Keyword(String),
    KeywordAlternation(Vec<String>),
    Raw(String),
}

struct WebrefGrammarConsumerSummaryV0 {
    entry_count: usize,
    modeled_entry_count: usize,
    all_entries_valid: bool,
    provenance_valid: bool,
}

fn classify_webref_syntax(syntax: &str) -> WebrefGrammarTermV0 {
    let trimmed = syntax.trim();
    if is_single_type_reference(trimmed) {
        return WebrefGrammarTermV0::Reference(trimmed.to_string());
    }
    if is_bare_grammar_keyword(trimmed) {
        return WebrefGrammarTermV0::Keyword(trimmed.to_string());
    }
    if let Some(keywords) = simple_keyword_alternation(trimmed) {
        return WebrefGrammarTermV0::KeywordAlternation(keywords);
    }
    WebrefGrammarTermV0::Raw(trimmed.to_string())
}

fn is_single_type_reference(syntax: &str) -> bool {
    syntax.starts_with('<')
        && syntax.ends_with('>')
        && syntax.matches('<').count() == 1
        && syntax.matches('>').count() == 1
}

fn is_bare_grammar_keyword(syntax: &str) -> bool {
    let mut chars = syntax.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '-' || first == '_' || first.is_ascii_alphabetic())
        && chars.all(|char| char == '-' || char == '_' || char.is_ascii_alphanumeric())
}

fn simple_keyword_alternation(syntax: &str) -> Option<Vec<String>> {
    // Only the simplest `a | b | c` shape with no groups, multipliers, type
    // references, or other combinators reduces to a keyword set; anything richer
    // stays `Raw` so the consumer never mis-parses a structure it does not model.
    if syntax.contains([
        '[', ']', '(', ')', '<', '>', '{', '}', '*', '+', '?', '#', ',', '!', '&',
    ]) || syntax.contains("||")
    {
        return None;
    }
    let parts = syntax.split('|').map(str::trim).collect::<Vec<_>>();
    if parts.len() < 2 || !parts.iter().all(|part| is_bare_grammar_keyword(part)) {
        return None;
    }
    Some(parts.into_iter().map(str::to_string).collect())
}

fn summarize_webref_grammar(
    webref_pin: Option<&SpecSourcePinV0>,
) -> WebrefGrammarConsumerSummaryV0 {
    let Ok(snapshot) = serde_json::from_str::<WebrefGrammarSnapshotV0>(WEBREF_GRAMMAR_SOURCE)
    else {
        return WebrefGrammarConsumerSummaryV0 {
            entry_count: 0,
            modeled_entry_count: 0,
            all_entries_valid: false,
            provenance_valid: false,
        };
    };
    let actual_count = snapshot
        .categories
        .values()
        .map(|entries| entries.len())
        .sum::<usize>();
    let mut classified = 0usize;
    // NOTE: `modeled` is a recognition tally (entries the conservative classifier
    // shaped as a Reference/Keyword/Alternation rather than `Raw`), NOT a spec
    // coverage or conformance metric — a `<type>` reference is still an
    // unresolved forward pointer.
    let mut modeled = 0usize;
    let mut all_entries_well_formed = true;
    for entries in snapshot.categories.values() {
        for entry in entries {
            if entry.name.trim().is_empty() || entry.syntax.trim().is_empty() {
                all_entries_well_formed = false;
                continue;
            }
            classified += 1;
            if !matches!(
                classify_webref_syntax(entry.syntax.as_str()),
                WebrefGrammarTermV0::Raw(_)
            ) {
                modeled += 1;
            }
        }
    }
    let all_entries_valid = snapshot.schema_version == "0"
        && snapshot.product == "omena-spec-audit.webref-grammar"
        && actual_count == snapshot.entry_count
        && classified == snapshot.entry_count
        && all_entries_well_formed;
    let provenance_valid = webref_pin.is_some_and(|pin| {
        snapshot.source.package == pin.package
            && snapshot.source.version == pin.version
            && snapshot.source.git_head == pin.git_head
    });
    WebrefGrammarConsumerSummaryV0 {
        entry_count: snapshot.entry_count,
        modeled_entry_count: modeled,
        all_entries_valid,
        provenance_valid,
    }
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
        assert_eq!(summary.manifest_entry_count, 33);
        assert_eq!(summary.p0_entry_count, 22);
        assert_eq!(summary.source_linked_entry_count, 33);
        assert_eq!(summary.webref_entry_count, 23);
        assert_eq!(summary.source_coverage_count, 4);
        assert_eq!(summary.blocking_p0_gap_count, 0);
        assert!(summary.all_source_pins_valid);
        assert!(summary.source_freshness_policy_valid);
        assert!(summary.generated_data_review_gate_valid);
        assert!(summary.all_p0_gaps_have_rationale);
        assert!(summary.manifest_cross_references_valid);
        assert!(summary.manifest_source_coverage_valid);
        assert!(summary.all_pinned_sources_have_manifest_coverage);
        assert!(summary.closed_gates.contains(&"specAuditSourcePins"));
        assert!(
            summary
                .closed_gates
                .contains(&"specAuditManifestSourceCrossReferences")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"specAuditCrossSourceCoverage")
        );
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
    fn boundary_consumes_webref_grammar_with_count_and_provenance() {
        let summary = summarize_omena_spec_audit_boundary();

        assert!(summary.webref_grammar_entry_count > 0);
        assert!(summary.webref_grammar_modeled_entry_count <= summary.webref_grammar_entry_count);
        assert!(summary.all_webref_grammar_entries_valid);
        assert!(summary.webref_grammar_provenance_valid);
        assert!(summary.closed_gates.contains(&"webrefGrammarConsumer"));
    }

    #[test]
    fn webref_syntax_classifier_reduces_unmodelable_to_raw() {
        use super::{WebrefGrammarTermV0, classify_webref_syntax};

        assert_eq!(
            classify_webref_syntax("<length>"),
            WebrefGrammarTermV0::Reference("<length>".to_string())
        );
        assert_eq!(
            classify_webref_syntax("subgrid"),
            WebrefGrammarTermV0::Keyword("subgrid".to_string())
        );
        assert_eq!(
            classify_webref_syntax("block | inline | none"),
            WebrefGrammarTermV0::KeywordAlternation(vec![
                "block".to_string(),
                "inline".to_string(),
                "none".to_string(),
            ])
        );
        // Combinators/multipliers/groups the consumer does not model reduce to
        // Raw (never a guess, never a panic).
        assert!(matches!(
            classify_webref_syntax("none | <track-list> | subgrid <line-name-list>?"),
            WebrefGrammarTermV0::Raw(_)
        ));
        assert!(matches!(
            classify_webref_syntax("<a> || <b>"),
            WebrefGrammarTermV0::Raw(_)
        ));
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
