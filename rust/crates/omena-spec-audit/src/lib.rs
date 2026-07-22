//! Pinned CSS specification source audit substrate for Omena CSS.
//!
//! M4 uses this crate to make spec provenance and P0 gap policy explicit before
//! the larger generated webref/browser-data importer lands.

use omena_meta_macros::{pass, spec};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

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
const VALUE_GRAMMAR_OVERRIDES_SOURCE: &str = include_str!("../data/value-grammar-overrides.json");
const ORACLE_SOURCE_LOCK_SOURCE: &str = include_str!("../data/oracle-source-lock.json");

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
    /// Number of pinned external oracle/corpus archive sources.
    pub external_oracle_pin_count: usize,
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
    /// Whether every source has a package or repository pin with a stable version.
    pub all_source_pins_valid: bool,
    /// Whether source freshness metadata is present and internally consistent.
    pub source_freshness_policy_valid: bool,
    /// Whether external oracle records match package and corpus manifest pins.
    pub oracle_pin_consistency_valid: bool,
    /// Whether corpus-farm consumers can enter using the pinned oracle contract.
    pub external_corpus_entry_gate_valid: bool,
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
    /// Number of closed-vocabulary terms the `SpecVocabularyV0` feed exposes from the
    /// vendored grammar (entries reduced to a finite keyword set). A bounded subset
    /// of the grammar, NOT a spec-coverage metric.
    pub spec_vocabulary_coverage: usize,
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
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    repo_pin: Option<String>,
    #[serde(default)]
    git_head: String,
    #[serde(default)]
    tarball: String,
    #[serde(default)]
    declared_version_source: Option<String>,
    role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OracleSourceLockV0 {
    schema_version: String,
    product: String,
    npm_packages: BTreeMap<String, OracleNpmPackagePinV0>,
    sass_spec_archive: OracleRepositoryPinV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OracleNpmPackagePinV0 {
    version: String,
    declared_version_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OracleRepositoryPinV0 {
    pin: String,
    declared_version_source: String,
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
    summarize_omena_spec_audit_boundary_from_sources(
        SPEC_SOURCE_PINS_SOURCE,
        OMENA_SPEC_MANIFEST_SOURCE,
    )
}

/// Summarize the spec audit boundary from explicit source payloads.
fn summarize_omena_spec_audit_boundary_from_sources(
    source_pins_source: &str,
    manifest_source: &str,
) -> OmenaSpecAuditBoundarySummaryV0 {
    let source_pins = serde_json::from_str::<SpecSourcePinsV0>(source_pins_source).ok();
    let manifest = serde_json::from_str::<OmenaSpecManifestV0>(manifest_source).ok();
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
            .filter(|source| source_requires_manifest_coverage(source))
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
    let oracle_pin_consistency_valid = source_pins
        .as_ref()
        .is_some_and(oracle_pin_consistency_is_valid);
    let external_oracle_pin_count = sources
        .iter()
        .filter(|source| source_is_external_oracle_pin(source))
        .count();
    let external_corpus_entry_gate_valid = all_source_pins_valid
        && source_freshness_policy_valid
        && oracle_pin_consistency_valid
        && external_oracle_pin_count >= 3;
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
        external_oracle_pin_count,
        manifest_entry_count: entries.len(),
        p0_entry_count: p0_entries,
        source_linked_entry_count: source_linked_entries,
        webref_entry_count: webref_entries,
        source_coverage_count: source_coverage.len(),
        blocking_p0_gap_count,
        all_source_pins_valid,
        source_freshness_policy_valid,
        oracle_pin_consistency_valid,
        external_corpus_entry_gate_valid,
        generated_data_review_gate_valid,
        all_p0_gaps_have_rationale: manifest_shape_valid && all_p0_gaps_have_rationale,
        manifest_cross_references_valid,
        manifest_source_coverage_valid,
        all_pinned_sources_have_manifest_coverage,
        webref_grammar_entry_count: webref_grammar.entry_count,
        webref_grammar_modeled_entry_count: webref_grammar.modeled_entry_count,
        all_webref_grammar_entries_valid: webref_grammar.all_entries_valid,
        webref_grammar_provenance_valid: webref_grammar.provenance_valid,
        spec_vocabulary_coverage: spec_vocabulary().closed_term_count(),
        closed_gates: vec![
            "specAuditSourcePins",
            "specAuditSingleSourceManifest",
            "specAuditManifestSourceCrossReferences",
            "specAuditCrossSourceCoverage",
            "specAuditP0GapRationalePolicy",
            "specAuditSourceFreshnessPolicy",
            "externalOraclePinConsistency",
            "externalCorpusEntryGate",
            "generatedDataHumanReviewGate",
            "metaMacroAttributeShape",
            "webrefGrammarConsumer",
        ],
    }
}

fn source_pin_is_valid(source: &SpecSourcePinV0) -> bool {
    let npm_pin_valid = source.tarball.starts_with("https://registry.npmjs.org/")
        && (source.git_head.is_empty()
            || (source.git_head.len() == 40
                && source.git_head.chars().all(|char| char.is_ascii_hexdigit())));
    let repo_pin_valid = source
        .repository
        .as_deref()
        .is_some_and(|repository| repository.starts_with("https://github.com/"))
        && source
            .repo_pin
            .as_deref()
            .is_some_and(repo_pin_has_full_sha)
        && source.tarball.starts_with("https://github.com/");
    !source.name.is_empty()
        && !source.package.is_empty()
        && !source.version.is_empty()
        && (npm_pin_valid || repo_pin_valid)
        && !source.role.is_empty()
}

fn source_requires_manifest_coverage(source: &SpecSourcePinV0) -> bool {
    !source_is_external_oracle_pin(source) && source.role != "external-wpt-corpus-module"
}

fn source_is_external_oracle_pin(source: &SpecSourcePinV0) -> bool {
    matches!(
        source.role.as_str(),
        "external-sass-oracle" | "external-css-parser-oracle" | "external-corpus-archive"
    )
}

fn repo_pin_has_full_sha(pin: &str) -> bool {
    let Some((_, sha)) = pin.rsplit_once('@') else {
        return false;
    };
    sha.len() == 40 && sha.chars().all(|char| char.is_ascii_hexdigit())
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

fn oracle_pin_consistency_is_valid(source_pins: &SpecSourcePinsV0) -> bool {
    let source_by_name = source_pins
        .sources
        .iter()
        .map(|source| (source.name.as_str(), source))
        .collect::<BTreeMap<_, _>>();
    let Some(dart_sass_pin) = source_by_name.get("dart-sass") else {
        return false;
    };
    let Some(lightningcss_pin) = source_by_name.get("lightningcss") else {
        return false;
    };
    let Some(sass_spec_pin) = source_by_name.get("sass-spec-archive") else {
        return false;
    };
    let Some(oracle_source_lock) = oracle_source_lock() else {
        return false;
    };
    if oracle_source_lock.schema_version != "0"
        || oracle_source_lock.product != "omena-spec-audit.oracle-source-lock"
    {
        return false;
    }
    oracle_source_lock
        .npm_packages
        .get("sass")
        .is_some_and(|package| {
            dart_sass_pin.package == "sass"
                && dart_sass_pin.version == package.version
                && dart_sass_pin.declared_version_source.as_deref()
                    == Some(package.declared_version_source.as_str())
        })
        && oracle_source_lock
            .npm_packages
            .get("lightningcss")
            .is_some_and(|package| {
                lightningcss_pin.package == "lightningcss"
                    && lightningcss_pin.version == package.version
                    && lightningcss_pin.declared_version_source.as_deref()
                        == Some(package.declared_version_source.as_str())
            })
        && {
            let repository = &oracle_source_lock.sass_spec_archive;
            sass_spec_pin.repo_pin.as_deref() == Some(repository.pin.as_str())
                && sass_spec_pin.version
                    == repository
                        .pin
                        .rsplit_once('@')
                        .map(|(_, sha)| sha)
                        .unwrap_or_default()
                && sass_spec_pin.declared_version_source.as_deref()
                    == Some(repository.declared_version_source.as_str())
        }
}

fn oracle_source_lock() -> Option<&'static OracleSourceLockV0> {
    static LOCK: OnceLock<Option<OracleSourceLockV0>> = OnceLock::new();
    LOCK.get_or_init(|| serde_json::from_str(ORACLE_SOURCE_LOCK_SOURCE).ok())
        .as_ref()
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
    href: String,
    syntax: Option<String>,
    boundary: SpecGrammarBoundaryV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ValueGrammarOverrideSetV0 {
    schema_version: String,
    product: String,
    human_review_required: bool,
    entries: Vec<ValueGrammarOverrideRowV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ValueGrammarOverrideRowV0 {
    category: String,
    name: String,
    expected_syntax: String,
    replacement_syntax: String,
    source_url: String,
    decision: String,
    reason: String,
    reviewer: String,
    reviewed_at: String,
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

/// Public closed-vocabulary projection of the pinned webref grammar snapshot.
///
/// For each webref category, every entry whose value-definition syntax reduces to
/// a finite, enumerable keyword set (`Keyword` or `KeywordAlternation`) is exposed
/// as `name -> keywords`. Entries whose syntax is a `<type>` reference or a grammar
/// richer than the conservative classifier models (`Raw`) are intentionally omitted
/// — never fabricated. This is a bounded closed-vocabulary tally driven entirely by
/// the vendored snapshot, NOT a spec-coverage or conformance claim.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpecVocabularyV0 {
    closed_terms: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

/// One effective value-definition-syntax record from the pinned Webref snapshot.
///
/// The base record retains its source boundary and URL. A reviewed syntax delta,
/// when present, replaces only the effective syntax and records its provenance.
/// Missing syntax remains data, so consumers must distinguish a known record with
/// no grammar from a name that is absent from the snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecGrammarEntryV0 {
    pub name: String,
    pub syntax: Option<String>,
    pub source_url: String,
    pub boundary: SpecGrammarBoundaryV0,
    pub override_provenance: Option<SpecGrammarOverrideProvenanceV0>,
}

/// Source-boundary policy attached to one pinned grammar entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecGrammarBoundaryClassificationV0 {
    InBoundary,
    ForwardTier,
}

/// Provenance for the specification tier represented by one grammar entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecGrammarBoundaryV0 {
    pub classification: SpecGrammarBoundaryClassificationV0,
    pub reason: String,
    pub rule_id: String,
    pub browser_spec_shortname: Option<String>,
}

/// Human-reviewed provenance for a syntax delta composed over pinned data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecGrammarOverrideProvenanceV0 {
    pub source_url: String,
    pub source_syntax: String,
    pub decision: String,
    pub reason: String,
    pub reviewer: String,
    pub reviewed_at: String,
}

/// Audit summary for reviewed grammar overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecGrammarOverrideAuditV0 {
    pub entry_count: usize,
    pub applied_entry_count: usize,
    pub all_entries_valid: bool,
}

/// Full grammar authority over every axis in the pinned Webref snapshot.
///
/// This is the lossless counterpart to [`SpecVocabularyV0`], whose finite
/// keyword projection intentionally omits richer grammars. Semantic consumers
/// use this registry instead of extracting or embedding a second grammar copy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpecGrammarRegistryV0 {
    categories: BTreeMap<String, Vec<SpecGrammarEntryV0>>,
}

impl SpecGrammarRegistryV0 {
    /// Looks up a known grammar record case-insensitively by category and name.
    pub fn entry(&self, category: &str, name: &str) -> Option<&SpecGrammarEntryV0> {
        let entries = self.categories.get(category)?;
        let lowered = name.trim().to_ascii_lowercase();
        entries
            .binary_search_by(|entry| entry.name.as_str().cmp(lowered.as_str()))
            .ok()
            .map(|index| &entries[index])
    }

    /// Returns the syntax string for a known record when the pinned source
    /// supplies one. `None` is also returned for an absent record; callers that
    /// need to distinguish those states should use [`Self::entry`].
    pub fn syntax(&self, category: &str, name: &str) -> Option<&str> {
        self.entry(category, name)?.syntax.as_deref()
    }

    /// Returns every record in one axis in deterministic name order.
    pub fn entries(&self, category: &str) -> &[SpecGrammarEntryV0] {
        self.categories
            .get(category)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Number of records in one registry axis.
    pub fn entry_count(&self, category: &str) -> usize {
        self.entries(category).len()
    }

    /// Number of records across all axes.
    pub fn total_entry_count(&self) -> usize {
        self.categories.values().map(Vec::len).sum()
    }
}

impl SpecVocabularyV0 {
    /// The closed keyword set for a named term in a webref category, if the term's
    /// grammar reduces to one. Lookup is case-insensitive on the term name.
    pub fn closed_keywords(&self, category: &str, name: &str) -> Option<&[String]> {
        let lowered = name.trim().to_ascii_lowercase();
        self.closed_terms
            .get(category)
            .and_then(|terms| terms.get(&lowered))
            .map(Vec::as_slice)
    }

    /// The closed keyword set for a `<type>` (e.g. `named-color`), if it is closed.
    pub fn type_keywords(&self, type_name: &str) -> Option<&[String]> {
        self.closed_keywords("types", type_name)
    }

    /// The closed keyword set for a standard property (e.g. `box-sizing`), if the
    /// property's whole grammar is a closed alternation.
    pub fn property_keywords(&self, property: &str) -> Option<&[String]> {
        self.closed_keywords("properties", property)
    }

    /// Total number of closed-vocabulary terms exposed across all categories.
    pub fn closed_term_count(&self) -> usize {
        self.closed_terms.values().map(BTreeMap::len).sum()
    }

    /// Whether a value is a member of a `<type>`'s closed keyword set
    /// (case-insensitive). Returns `None` when the type has no closed projection,
    /// which the caller must treat as undecided (never a rejection).
    pub fn type_accepts(&self, type_name: &str, value: &str) -> Option<bool> {
        let keywords = self.type_keywords(type_name)?;
        let value = value.trim();
        Some(
            keywords
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(value)),
        )
    }
}

/// The closed-vocabulary projection of the vendored webref snapshot, parsed once.
pub fn spec_vocabulary() -> &'static SpecVocabularyV0 {
    static DATA: OnceLock<SpecVocabularyV0> = OnceLock::new();
    DATA.get_or_init(build_spec_vocabulary)
}

/// The complete pinned Webref grammar registry, parsed once per process.
pub fn spec_grammar_registry() -> &'static SpecGrammarRegistryV0 {
    static DATA: OnceLock<SpecGrammarRegistryV0> = OnceLock::new();
    DATA.get_or_init(build_spec_grammar_registry)
}

/// Audits the reviewed syntax deltas composed over the pinned Webref registry.
pub fn audit_value_grammar_overrides_v0() -> SpecGrammarOverrideAuditV0 {
    let Some((snapshot, overrides)) =
        parse_value_grammar_override_sources(WEBREF_GRAMMAR_SOURCE, VALUE_GRAMMAR_OVERRIDES_SOURCE)
    else {
        return SpecGrammarOverrideAuditV0 {
            entry_count: 0,
            applied_entry_count: 0,
            all_entries_valid: false,
        };
    };
    let all_entries_valid = value_grammar_override_set_is_valid(&snapshot, &overrides);
    let applied_entry_count = spec_grammar_registry()
        .categories
        .values()
        .flatten()
        .filter(|entry| entry.override_provenance.is_some())
        .count();
    SpecGrammarOverrideAuditV0 {
        entry_count: overrides.entries.len(),
        applied_entry_count,
        all_entries_valid,
    }
}

fn build_spec_grammar_registry() -> SpecGrammarRegistryV0 {
    let Ok(snapshot) = serde_json::from_str::<WebrefGrammarSnapshotV0>(WEBREF_GRAMMAR_SOURCE)
    else {
        return SpecGrammarRegistryV0::default();
    };
    let overrides =
        serde_json::from_str::<ValueGrammarOverrideSetV0>(VALUE_GRAMMAR_OVERRIDES_SOURCE).ok();
    let overrides_are_valid = overrides
        .as_ref()
        .is_some_and(|overrides| value_grammar_override_set_is_valid(&snapshot, overrides));
    let override_by_key = overrides
        .as_ref()
        .filter(|_| overrides_are_valid)
        .into_iter()
        .flat_map(|overrides| overrides.entries.iter())
        .map(|entry| ((entry.category.clone(), entry.name.clone()), entry))
        .collect::<BTreeMap<_, _>>();
    let categories = snapshot
        .categories
        .into_iter()
        .map(|(category, entries)| {
            let mut entries = entries
                .into_iter()
                .map(|entry| {
                    let name = entry.name.trim().to_ascii_lowercase();
                    let grammar_override = override_by_key.get(&(category.clone(), name.clone()));
                    let syntax = grammar_override
                        .map(|grammar_override| grammar_override.replacement_syntax.clone())
                        .or(entry.syntax.clone());
                    let override_provenance =
                        grammar_override.map(|grammar_override| SpecGrammarOverrideProvenanceV0 {
                            source_url: grammar_override.source_url.clone(),
                            source_syntax: grammar_override.expected_syntax.clone(),
                            decision: grammar_override.decision.clone(),
                            reason: grammar_override.reason.clone(),
                            reviewer: grammar_override.reviewer.clone(),
                            reviewed_at: grammar_override.reviewed_at.clone(),
                        });
                    SpecGrammarEntryV0 {
                        name,
                        syntax,
                        source_url: entry.href,
                        boundary: entry.boundary,
                        override_provenance,
                    }
                })
                .collect::<Vec<_>>();
            entries.sort_by(|left, right| left.name.cmp(&right.name));
            (category, entries)
        })
        .collect();
    SpecGrammarRegistryV0 { categories }
}

fn parse_value_grammar_override_sources(
    snapshot_source: &str,
    override_source: &str,
) -> Option<(WebrefGrammarSnapshotV0, ValueGrammarOverrideSetV0)> {
    Some((
        serde_json::from_str(snapshot_source).ok()?,
        serde_json::from_str(override_source).ok()?,
    ))
}

fn value_grammar_override_set_is_valid(
    snapshot: &WebrefGrammarSnapshotV0,
    overrides: &ValueGrammarOverrideSetV0,
) -> bool {
    if overrides.schema_version != "0"
        || overrides.product != "omena-spec-audit.value-grammar-overrides"
        || !overrides.human_review_required
        || overrides.entries.is_empty()
    {
        return false;
    }
    let mut keys = BTreeSet::new();
    overrides.entries.iter().all(|entry| {
        let key = (entry.category.as_str(), entry.name.as_str());
        let normalized = entry.category == entry.category.trim().to_ascii_lowercase()
            && entry.name == entry.name.trim().to_ascii_lowercase();
        let unique = keys.insert(key);
        let source_entry = snapshot
            .categories
            .get(entry.category.as_str())
            .and_then(|entries| entries.iter().find(|source| source.name == entry.name));
        normalized
            && unique
            && entry.decision == "replace-syntax"
            && !entry.expected_syntax.trim().is_empty()
            && !entry.replacement_syntax.trim().is_empty()
            && entry.expected_syntax != entry.replacement_syntax
            && entry.source_url.starts_with("https://")
            && !entry.reason.trim().is_empty()
            && !entry.reviewer.trim().is_empty()
            && is_yyyy_mm_dd(entry.reviewed_at.as_str())
            && source_entry.is_some_and(|source| {
                source.href == entry.source_url
                    && source.syntax.as_deref() == Some(entry.expected_syntax.as_str())
            })
    })
}

fn build_spec_vocabulary() -> SpecVocabularyV0 {
    let mut closed_terms: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for category in ["atrules", "functions", "properties", "selectors", "types"] {
        let mut terms: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for entry in spec_grammar_registry().entries(category) {
            let Some(syntax) = entry.syntax.as_deref() else {
                continue;
            };
            let keywords = match classify_webref_syntax(syntax) {
                WebrefGrammarTermV0::Keyword(keyword) => vec![keyword],
                WebrefGrammarTermV0::KeywordAlternation(keywords) => keywords,
                WebrefGrammarTermV0::Reference(_) | WebrefGrammarTermV0::Raw(_) => continue,
            };
            if entry.name.is_empty() {
                continue;
            }
            terms.insert(entry.name.clone(), keywords);
        }
        if !terms.is_empty() {
            closed_terms.insert(category.to_string(), terms);
        }
    }
    SpecVocabularyV0 { closed_terms }
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
            if entry.name.trim().is_empty() {
                all_entries_well_formed = false;
                continue;
            }
            classified += 1;
            if let Some(syntax) = entry.syntax.as_deref() {
                if syntax.trim().is_empty() {
                    all_entries_well_formed = false;
                    continue;
                }
                if !matches!(classify_webref_syntax(syntax), WebrefGrammarTermV0::Raw(_)) {
                    modeled += 1;
                }
            }
        }
    }
    let all_entries_valid = snapshot.schema_version == "1"
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
        OMENA_SPEC_MANIFEST_SOURCE, OmenaSpecAuditBoundarySummaryV0, SPEC_AUDIT_COLOR_MARKER,
        SPEC_AUDIT_PASS_MARKER, SPEC_SOURCE_PINS_SOURCE, SpecGeneratedDataReviewGateV0,
        SpecGrammarBoundaryClassificationV0, SpecSourcePinsV0, SpecSourceRefreshPolicyV0,
        VALUE_GRAMMAR_OVERRIDES_SOURCE, WEBREF_GRAMMAR_SOURCE, WebrefGrammarTermV0,
        audit_value_grammar_overrides_v0, classify_webref_syntax,
        generated_data_review_gate_is_valid, parse_value_grammar_override_sources,
        source_freshness_policy_is_valid, spec_grammar_registry, spec_vocabulary,
        summarize_omena_spec_audit_boundary, summarize_omena_spec_audit_boundary_from_sources,
        value_grammar_override_set_is_valid,
    };
    use serde_json::{Value, json};

    fn assert_manifest_growth_contract(summary: &OmenaSpecAuditBoundarySummaryV0) {
        assert!(summary.source_count >= summary.source_coverage_count);
        assert_eq!(summary.external_oracle_pin_count, 3);
        assert!(
            summary.manifest_entry_count >= 33,
            "manifest coverage shrank to {}; re-bless the coverage floor if intended",
            summary.manifest_entry_count
        );
        assert!(
            summary.p0_entry_count >= 22,
            "P0 coverage shrank to {}; re-bless the coverage floor if intended",
            summary.p0_entry_count
        );
        assert!(
            summary.source_linked_entry_count >= 33,
            "source-linked coverage shrank to {}; re-bless the coverage floor if intended",
            summary.source_linked_entry_count
        );
        assert!(
            summary.webref_entry_count >= 23,
            "webref coverage shrank to {}; re-bless the coverage floor if intended",
            summary.webref_entry_count
        );
        assert_eq!(
            summary.source_linked_entry_count,
            summary.manifest_entry_count
        );
        assert_eq!(summary.blocking_p0_gap_count, 0);
    }

    fn embedded_manifest_value() -> Result<Value, String> {
        serde_json::from_str::<Value>(OMENA_SPEC_MANIFEST_SOURCE)
            .map_err(|error| format!("embedded manifest JSON did not parse: {error}"))
    }

    fn summary_from_manifest_value(
        manifest: Value,
    ) -> Result<OmenaSpecAuditBoundarySummaryV0, String> {
        let manifest_source = serde_json::to_string(&manifest)
            .map_err(|error| format!("mutated manifest JSON did not serialize: {error}"))?;
        Ok(summarize_omena_spec_audit_boundary_from_sources(
            SPEC_SOURCE_PINS_SOURCE,
            &manifest_source,
        ))
    }

    fn push_manifest_entry(manifest: &mut Value, entry: Value) -> bool {
        manifest
            .get_mut("entries")
            .and_then(Value::as_array_mut)
            .is_some_and(|entries| {
                entries.push(entry);
                true
            })
    }

    fn retag_one_webref_entry_to_web_features(manifest: &mut Value) -> bool {
        let Some(entries) = manifest.get_mut("entries").and_then(Value::as_array_mut) else {
            return false;
        };
        for entry in entries {
            if entry.get("sourceName").and_then(Value::as_str) == Some("webref-css") {
                entry["sourceName"] = Value::String("web-features".to_string());
                return true;
            }
        }
        false
    }

    fn duplicate_one_source_coverage_row(manifest: &mut Value) -> bool {
        let Some(source_coverage) = manifest
            .get_mut("sourceCoverage")
            .and_then(Value::as_array_mut)
        else {
            return false;
        };
        let Some(row) = source_coverage.iter().next().cloned() else {
            return false;
        };
        source_coverage.push(row);
        true
    }

    fn covered_webref_entry(id: &str) -> Value {
        json!({
            "id": id,
            "webrefId": id,
            "sourceName": "webref-css",
            "sourceCategory": "properties",
            "specUrl": "https://drafts.csswg.org/css-values/",
            "priority": "P0",
            "status": "covered",
            "owner": "omena-css",
            "rationale": "covered by an injected manifest entry",
            "evidence": ["synthetic manifest growth fixture"]
        })
    }

    #[test]
    fn boundary_reports_pinned_sources_and_p0_policy() {
        let summary = summarize_omena_spec_audit_boundary();

        assert_eq!(summary.product, "omena-spec-audit.boundary");
        assert_eq!(summary.stage, "stage1-advisory");
        assert_manifest_growth_contract(&summary);
        assert!(summary.all_source_pins_valid);
        assert!(summary.source_freshness_policy_valid);
        assert!(summary.oracle_pin_consistency_valid);
        assert!(summary.external_corpus_entry_gate_valid);
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
                .contains(&"externalOraclePinConsistency")
        );
        assert!(summary.closed_gates.contains(&"externalCorpusEntryGate"));
        assert!(
            summary
                .closed_gates
                .contains(&"generatedDataHumanReviewGate")
        );
        assert!(summary.closed_gates.contains(&"metaMacroAttributeShape"));
    }

    #[test]
    fn source_injectable_boundary_matches_embedded_summary() {
        let embedded = summarize_omena_spec_audit_boundary();
        let injected = summarize_omena_spec_audit_boundary_from_sources(
            SPEC_SOURCE_PINS_SOURCE,
            OMENA_SPEC_MANIFEST_SOURCE,
        );

        assert_eq!(embedded, injected);
    }

    #[test]
    fn grow_pass_covered_entry_keeps_boundary_green() -> Result<(), String> {
        let mut manifest = embedded_manifest_value()?;
        assert!(push_manifest_entry(
            &mut manifest,
            covered_webref_entry("css-values/properties/synthetic-growth-contract")
        ));
        let summary = summary_from_manifest_value(manifest)?;

        assert_eq!(summary.manifest_entry_count, 35);
        assert_eq!(summary.p0_entry_count, 23);
        assert_eq!(summary.source_linked_entry_count, 35);
        assert_eq!(summary.webref_entry_count, 24);
        assert_ne!(summary.manifest_entry_count, 34);
        assert_ne!(summary.p0_entry_count, 22);
        assert_ne!(summary.source_linked_entry_count, 34);
        assert!(summary.manifest_cross_references_valid);
        assert!(summary.manifest_source_coverage_valid);
        assert!(summary.all_p0_gaps_have_rationale);
        assert!(summary.all_pinned_sources_have_manifest_coverage);
        assert_manifest_growth_contract(&summary);
        Ok(())
    }

    #[test]
    fn unlinked_entry_breaks_source_link_relation() -> Result<(), String> {
        let mut manifest = embedded_manifest_value()?;
        assert!(push_manifest_entry(
            &mut manifest,
            json!({
                "id": "css-values/properties/unlinked-source-contract",
                "webrefId": "css-values/properties/unlinked-source-contract",
                "sourceName": "unregistered-source",
                "sourceCategory": "properties",
                "specUrl": "https://drafts.csswg.org/css-values/",
                "priority": "P1",
                "status": "covered",
                "owner": "omena-css",
                "rationale": "exercises source-link relation",
                "evidence": ["synthetic manifest source-link fixture"]
            })
        ));
        let summary = summary_from_manifest_value(manifest)?;

        assert_eq!(summary.manifest_entry_count, 35);
        assert_eq!(summary.source_linked_entry_count, 34);
        assert_ne!(
            summary.source_linked_entry_count,
            summary.manifest_entry_count
        );
        Ok(())
    }

    #[test]
    fn webref_shrink_breaks_coverage_floor() -> Result<(), String> {
        let mut manifest = embedded_manifest_value()?;
        assert!(retag_one_webref_entry_to_web_features(&mut manifest));
        let summary = summary_from_manifest_value(manifest)?;

        assert_eq!(summary.manifest_entry_count, 34);
        assert_eq!(summary.source_linked_entry_count, 34);
        assert_eq!(summary.webref_entry_count, 22);
        assert!(summary.webref_entry_count < 23);
        Ok(())
    }

    #[test]
    fn duplicate_source_coverage_row_breaks_source_count_relation() -> Result<(), String> {
        let mut manifest = embedded_manifest_value()?;
        assert!(duplicate_one_source_coverage_row(&mut manifest));
        let summary = summary_from_manifest_value(manifest)?;

        assert_eq!(summary.source_count, 14);
        assert_eq!(summary.source_coverage_count, 5);
        assert_ne!(summary.source_count, summary.source_coverage_count);
        assert!(summary.manifest_source_coverage_valid);
        assert!(summary.all_pinned_sources_have_manifest_coverage);
        Ok(())
    }

    #[test]
    fn oracle_pin_consistency_rejects_package_version_desync() -> Result<(), String> {
        let mut source_pins = serde_json::from_str::<Value>(SPEC_SOURCE_PINS_SOURCE)
            .map_err(|error| format!("source pins JSON did not parse: {error}"))?;
        let Some(sources) = source_pins.get_mut("sources").and_then(Value::as_array_mut) else {
            return Err("source pins must expose sources array".to_string());
        };
        let Some(dart_sass) = sources
            .iter_mut()
            .find(|source| source.get("name").and_then(Value::as_str) == Some("dart-sass"))
        else {
            return Err("source pins must include dart-sass".to_string());
        };
        dart_sass["version"] = Value::String("1.99.0".to_string());
        let mutated = serde_json::to_string(&source_pins)
            .map_err(|error| format!("source pins JSON did not serialize: {error}"))?;
        let summary =
            summarize_omena_spec_audit_boundary_from_sources(&mutated, OMENA_SPEC_MANIFEST_SOURCE);

        assert!(!summary.oracle_pin_consistency_valid);
        assert!(!summary.external_corpus_entry_gate_valid);
        Ok(())
    }

    #[test]
    fn oracle_pin_consistency_rejects_corpus_archive_desync() -> Result<(), String> {
        let mut source_pins = serde_json::from_str::<Value>(SPEC_SOURCE_PINS_SOURCE)
            .map_err(|error| format!("source pins JSON did not parse: {error}"))?;
        let Some(sources) = source_pins.get_mut("sources").and_then(Value::as_array_mut) else {
            return Err("source pins must expose sources array".to_string());
        };
        let Some(sass_spec) = sources
            .iter_mut()
            .find(|source| source.get("name").and_then(Value::as_str) == Some("sass-spec-archive"))
        else {
            return Err("source pins must include sass-spec-archive".to_string());
        };
        sass_spec["repoPin"] =
            Value::String("sass/sass-spec@ffffffffffffffffffffffffffffffffffffffff".to_string());
        let mutated = serde_json::to_string(&source_pins)
            .map_err(|error| format!("source pins JSON did not serialize: {error}"))?;
        let summary =
            summarize_omena_spec_audit_boundary_from_sources(&mutated, OMENA_SPEC_MANIFEST_SOURCE);

        assert!(!summary.oracle_pin_consistency_valid);
        assert!(!summary.external_corpus_entry_gate_valid);
        Ok(())
    }

    #[test]
    fn rationale_less_p0_missing_entry_breaks_safety_gate() -> Result<(), String> {
        let clean = summarize_omena_spec_audit_boundary();
        assert_eq!(clean.blocking_p0_gap_count, 0);

        let mut manifest = embedded_manifest_value()?;
        assert!(push_manifest_entry(
            &mut manifest,
            json!({
                "id": "css-values/properties/missing-rationale-contract",
                "webrefId": "css-values/properties/missing-rationale-contract",
                "sourceName": "webref-css",
                "sourceCategory": "properties",
                "specUrl": "https://drafts.csswg.org/css-values/",
                "priority": "P0",
                "status": "missing",
                "owner": "omena-css"
            })
        ));
        let summary = summary_from_manifest_value(manifest)?;

        assert_eq!(summary.blocking_p0_gap_count, 1);
        assert!(!summary.all_p0_gaps_have_rationale);
        Ok(())
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

    #[test]
    fn spec_vocabulary_exposes_closed_keyword_projections_from_the_snapshot() {
        let vocabulary = spec_vocabulary();

        // Named colors reduce to a closed alternation — the full vendored set, not
        // the historical 15-entry stub.
        let named_colors = vocabulary.type_keywords("named-color").unwrap_or_default();
        assert!(named_colors.iter().any(|color| color == "aliceblue"));
        assert!(named_colors.len() > 100);
        assert_eq!(
            vocabulary.type_accepts("named-color", "AliceBlue"),
            Some(true)
        );
        assert_eq!(
            vocabulary.type_accepts("named-color", "not-a-color"),
            Some(false)
        );

        // A closed-alternation property is projected verbatim, in syntax order.
        assert_eq!(
            vocabulary.property_keywords("box-sizing"),
            Some(["content-box".to_string(), "border-box".to_string()].as_slice())
        );

        // An open grammar has no closed projection -> the caller must treat it as
        // undecided, never a rejection.
        assert_eq!(vocabulary.type_accepts("color", "anything"), None);

        assert_eq!(
            vocabulary.closed_term_count(),
            summarize_omena_spec_audit_boundary().spec_vocabulary_coverage
        );
        assert!(vocabulary.closed_term_count() > 0);
    }

    #[test]
    fn full_grammar_registry_preserves_every_axis_and_missing_syntax() {
        let registry = spec_grammar_registry();
        assert_eq!(registry.total_entry_count(), 1_715);
        assert_eq!(registry.entry_count("atrules"), 56);
        assert_eq!(registry.entry_count("functions"), 162);
        assert_eq!(registry.entry_count("properties"), 815);
        assert_eq!(registry.entry_count("selectors"), 158);
        assert_eq!(registry.entry_count("types"), 524);
        assert_eq!(
            registry.syntax("properties", "box-sizing"),
            Some("content-box | border-box")
        );
        assert_eq!(
            registry.syntax("types", "color"),
            Some(
                "<color-base> | currentColor | <system-color> | <contrast-color()> | <device-cmyk()> | <light-dark-color>"
            )
        );
        assert!(registry.entry("types", "length").is_some());
        assert!(registry.syntax("types", "length").is_none());
        assert!(registry.entry("types", "not-a-webref-type").is_none());
    }

    #[test]
    fn reviewed_value_grammar_override_preserves_source_and_decision_provenance() {
        let audit = audit_value_grammar_overrides_v0();
        assert_eq!(audit.entry_count, 1);
        assert_eq!(audit.applied_entry_count, 1);
        assert!(audit.all_entries_valid);

        let registry = spec_grammar_registry();
        let entry = registry.entry("properties", "-webkit-background-clip");
        assert!(
            entry.is_some(),
            "compatibility property must remain in the registry"
        );
        let Some(entry) = entry else {
            return;
        };
        assert_eq!(entry.syntax.as_deref(), Some("[ <visual-box> | text ]#"));
        assert_eq!(
            entry.boundary.classification,
            SpecGrammarBoundaryClassificationV0::InBoundary
        );
        let provenance = entry.override_provenance.as_ref();
        assert!(
            provenance.is_some(),
            "reviewed syntax delta must retain provenance"
        );
        let Some(provenance) = provenance else {
            return;
        };
        assert_eq!(provenance.source_url, entry.source_url);
        assert_eq!(provenance.source_syntax, "<visual-box>#");
        assert_eq!(provenance.decision, "replace-syntax");
        assert_eq!(provenance.reason, "compatibility-spec-text-value");
        assert_eq!(provenance.reviewer, "maintainer");
        assert_eq!(provenance.reviewed_at, "2026-07-21");
    }

    #[test]
    fn reviewed_value_grammar_override_rejects_drift_and_missing_review_metadata() {
        let parsed = parse_value_grammar_override_sources(
            WEBREF_GRAMMAR_SOURCE,
            VALUE_GRAMMAR_OVERRIDES_SOURCE,
        );
        assert!(
            parsed.is_some(),
            "embedded grammar override sources must parse"
        );
        let Some((snapshot, mut overrides)) = parsed else {
            return;
        };
        overrides.entries[0].reviewer.clear();
        assert!(!value_grammar_override_set_is_valid(&snapshot, &overrides));

        overrides.entries[0].reviewer = "maintainer".to_string();
        overrides.entries[0].expected_syntax = "<drifted-visual-box>#".to_string();
        assert!(!value_grammar_override_set_is_valid(&snapshot, &overrides));
    }

    #[test]
    fn spec_vocabulary_never_fabricates_a_term_outside_the_closed_classification() {
        let vocabulary = spec_vocabulary();
        // Every exposed projection traces back to a Keyword/KeywordAlternation
        // classification of a vendored entry; nothing is fabricated.
        assert!(
            !vocabulary
                .type_keywords("named-color")
                .unwrap_or_default()
                .is_empty()
        );
        // `color` is a rich grammar (Raw) and must never be exposed as closed.
        assert!(vocabulary.type_keywords("color").is_none());
        // `system-color` references <deprecated-color> (Raw) -> excluded.
        assert!(vocabulary.type_keywords("system-color").is_none());
        // Sanity: the classifier and projection agree on the box-sizing shape.
        assert!(matches!(
            classify_webref_syntax("content-box | border-box"),
            WebrefGrammarTermV0::KeywordAlternation(_)
        ));
    }

    #[test]
    fn spec_vocabulary_coverage_fences_depended_on_terms_against_drift() {
        let vocabulary = spec_vocabulary();

        // The `<named-color>` closed alternation backs the registered- and
        // standard-property value diagnostics. A webref re-vendor that drops, renames,
        // or shrinks it below the historically recognized set must fail CI rather than
        // silently degrade those diagnostics; additions (a new color) keep it passing.
        let named_colors = vocabulary.type_keywords("named-color").unwrap_or_default();
        assert!(
            named_colors.len() >= 140,
            "the <named-color> closed set shrank to {}; re-bless the coverage contract if intended",
            named_colors.len()
        );
        for color in ["aliceblue", "rebeccapurple"] {
            assert!(
                named_colors.iter().any(|entry| entry == color),
                "<named-color> no longer lists {color}"
            );
        }

        // A representative closed-alternation property the standard-property diagnostic
        // validates; if its grammar stops reducing to a closed keyword set the feed
        // shape has changed and the contract must be re-reviewed.
        assert!(vocabulary.property_keywords("box-sizing").is_some());

        // Broad coverage floor: additions are non-breaking (coverage only grows); a
        // bulk drop of closed-vocabulary terms fails this contract and forces review.
        assert!(
            vocabulary.closed_term_count() >= 200,
            "closed-vocabulary coverage dropped to {}; re-bless the contract if intended",
            vocabulary.closed_term_count()
        );
    }
}
