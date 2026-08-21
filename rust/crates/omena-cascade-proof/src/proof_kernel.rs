//! Small, solver-free rewrite certificate checker.
//!
//! The checker derives both endpoints from a certificate and compares those
//! derived terms with the supplied endpoints. It never calls a rewrite search,
//! reads a transform proof object, or accepts a producer-owned boolean.
//!
//! This kernel has deliberately narrow authority:
//! - it cannot establish that a rule catalog is sound;
//! - it cannot establish that an observer profile models browser behaviour;
//! - it cannot detect a defect shared with an external side-condition source;
//! - it cannot see a defect that moves both sides of a comparison identically;
//! - for genuinely IR-computed requirements, it can add disclosure without
//!   adding independent semantic strength.

use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{
    CascadeKey, CascadeLevel, CascadeValue, DomClassTokenizationV0, LayerOrdinal,
    OrderedTokenWordV0, Specificity, TokenSupportV0, normalized_layer_rank,
    resolve_custom_property_env_least_fixed_point, token_support_v0,
    tokenize_dom_class_attribute_v0,
};
use omena_parser::ModuleInstanceKeyV0;
use omena_syntax::ident::ClassNameV0;
use serde::{Deserialize, Serialize};

pub const REWRITE_CERTIFICATE_SCHEMA_VERSION_V0: &str = "0";
pub const REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0: &str = "0";
pub const REWRITE_RULE_CATALOG_SCHEMA_ID_V0: &str = "omena-cascade-proof.rewrite-rule-catalog.v0";
pub const CANONICAL_REWRITE_ASSUMPTIONS_SCHEMA_VERSION_V0: &str = "0";
pub const REWRITE_CERTIFICATE_MAX_DEPTH_V0: usize = 64;
pub const REWRITE_CERTIFICATE_MAX_NODES_V0: usize = 4_096;
pub const REWRITE_RULE_CATALOG_MAX_RULES_V0: usize = 256;
pub const REWRITE_RULE_CATALOG_MAX_OPERATORS_V0: usize = 256;
const REWRITE_TERM_MAX_DEPTH_V0: usize = 64;
const REWRITE_TERM_MAX_NODES_V0: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RewriteTermV0 {
    Atom {
        value: String,
    },
    Apply {
        operator: String,
        operands: Vec<RewriteTermV0>,
    },
}

impl RewriteTermV0 {
    pub fn atom(value: impl Into<String>) -> Self {
        Self::Atom {
            value: value.into(),
        }
    }

    pub fn apply(operator: impl Into<String>, operands: Vec<Self>) -> Self {
        Self::Apply {
            operator: operator.into(),
            operands,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RewritePatternV0 {
    Atom {
        value: String,
    },
    Variable {
        name: String,
    },
    Apply {
        operator: String,
        operands: Vec<RewritePatternV0>,
    },
}

impl RewritePatternV0 {
    pub fn atom(value: impl Into<String>) -> Self {
        Self::Atom {
            value: value.into(),
        }
    }

    pub fn variable(name: impl Into<String>) -> Self {
        Self::Variable { name: name.into() }
    }

    pub fn apply(operator: impl Into<String>, operands: Vec<Self>) -> Self {
        Self::Apply {
            operator: operator.into(),
            operands,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub enum CascadeLevelCertV0 {
    UserAgentNormal,
    UserNormal,
    AuthorNormal,
    InlineNormal,
    Animation,
    AuthorImportant,
    InlineImportant,
    UserImportant,
    UserAgentImportant,
    Transition,
}

impl CascadeLevelCertV0 {
    fn to_cascade_level(self) -> CascadeLevel {
        match self {
            Self::UserAgentNormal => CascadeLevel::UserAgentNormal,
            Self::UserNormal => CascadeLevel::UserNormal,
            Self::AuthorNormal => CascadeLevel::AuthorNormal,
            Self::InlineNormal => CascadeLevel::InlineNormal,
            Self::Animation => CascadeLevel::Animation,
            Self::AuthorImportant => CascadeLevel::AuthorImportant,
            Self::InlineImportant => CascadeLevel::InlineImportant,
            Self::UserImportant => CascadeLevel::UserImportant,
            Self::UserAgentImportant => CascadeLevel::UserAgentImportant,
            Self::Transition => CascadeLevel::Transition,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeWinnerKeyCertV0 {
    pub level: CascadeLevelCertV0,
    pub layer_important: bool,
    pub layer_ordinal: Option<i32>,
    pub scope_proximity: u32,
    pub specificity_ids: u32,
    pub specificity_classes: u32,
    pub specificity_elements: u32,
    pub source_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeWinnerEqualityCertV0 {
    pub before_winner_id: String,
    pub after_winner_id: String,
    pub before_key: CascadeWinnerKeyCertV0,
    pub after_key: CascadeWinnerKeyCertV0,
    pub before_class_attribute: String,
    pub after_class_attribute: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ComputedValueTermV0 {
    Literal {
        value: String,
    },
    Composite {
        values: Vec<ComputedValueTermV0>,
    },
    Variable {
        name: String,
        fallback: Option<Box<ComputedValueTermV0>>,
    },
    Initial,
    Inherit,
    Indeterminate,
    GuaranteedInvalid,
    Unset,
}

impl ComputedValueTermV0 {
    fn to_cascade_value(&self) -> CascadeValue {
        match self {
            Self::Literal { value } => CascadeValue::Literal(value.clone()),
            Self::Composite { values } => CascadeValue::Composite(
                values
                    .iter()
                    .map(ComputedValueTermV0::to_cascade_value)
                    .collect(),
            ),
            Self::Variable { name, fallback } => CascadeValue::Var {
                name: name.clone(),
                fallback: fallback
                    .as_ref()
                    .map(|value| Box::new(value.to_cascade_value())),
            },
            Self::Initial => CascadeValue::Initial,
            Self::Inherit => CascadeValue::Inherit,
            Self::Indeterminate => CascadeValue::Indeterminate,
            Self::GuaranteedInvalid => CascadeValue::GuaranteedInvalid,
            Self::Unset => CascadeValue::Unset,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedValueEnvironmentEntryV0 {
    pub name: String,
    pub value: ComputedValueTermV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedValueEqualityCertV0 {
    pub property: String,
    pub before_environment: Vec<ComputedValueEnvironmentEntryV0>,
    pub after_environment: Vec<ComputedValueEnvironmentEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMapTraceSegmentV0 {
    pub source_path: String,
    pub source_digest: String,
    pub original_start: usize,
    pub original_end: usize,
    pub generated_start: usize,
    pub generated_end: usize,
    pub pass_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMapTraceCertV0 {
    pub before_segments: Vec<SourceMapTraceSegmentV0>,
    pub after_segments: Vec<SourceMapTraceSegmentV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenOwnershipCertEntryV0 {
    pub emitted_token: String,
    pub module_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenOwnershipSeparabilityCertV0 {
    pub complete: bool,
    pub modeled_preimage_count: usize,
    pub emitted_token_count: usize,
    pub ownerships: Vec<TokenOwnershipCertEntryV0>,
    pub unattributed_emitted_token_count: usize,
    pub interface_mismatch_count: usize,
}

/// Identity of one CSS Module export before and after a transform pass.
///
/// The authored name is never a standalone key: it is sealed together with
/// the module-instance carrier selected by the CSS Modules identity plane.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleExportKeyV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub canonical_class_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleExportObservationV0 {
    pub key: ModuleExportKeyV0,
    pub emitted_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleExportRenameDeltaV0 {
    pub key: ModuleExportKeyV0,
    pub before_token: String,
    pub after_token: String,
}

/// Premises for one pass's exported-class-name preservation claim.
///
/// The checker re-hashes both premise sets and derives the preservation
/// relation itself. No producer-owned `preserved` boolean exists.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleExportPreservationCertV0 {
    pub pass_id: String,
    pub before_exports: Vec<ModuleExportObservationV0>,
    pub after_exports: Vec<ModuleExportObservationV0>,
    pub declared_rename_delta: Vec<ModuleExportRenameDeltaV0>,
    pub before_premise_digest: String,
    pub after_premise_digest: String,
}

impl ModuleExportPreservationCertV0 {
    pub fn new(
        pass_id: impl Into<String>,
        before_exports: Vec<ModuleExportObservationV0>,
        after_exports: Vec<ModuleExportObservationV0>,
        declared_rename_delta: Vec<ModuleExportRenameDeltaV0>,
    ) -> Self {
        let before_premise_digest =
            module_export_observation_digest_hex_v0(before_exports.as_slice());
        let after_premise_digest =
            module_export_observation_digest_hex_v0(after_exports.as_slice());
        Self {
            pass_id: pass_id.into(),
            before_exports,
            after_exports,
            declared_rename_delta,
            before_premise_digest,
            after_premise_digest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIndependenceObservationCertRowV0 {
    pub fixture_id: String,
    pub observer: String,
    pub left_then_right: String,
    pub right_then_left: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIndependenceCertV0 {
    pub left_pass_id: String,
    pub right_pass_id: String,
    pub observation_profile_id: String,
    pub profile_observers: Vec<String>,
    pub observation_rows: Vec<TransformIndependenceObservationCertRowV0>,
    pub left_preconditions: Vec<String>,
    pub right_preconditions: Vec<String>,
    pub left_preserves_right_preconditions: Vec<String>,
    pub right_preserves_left_preconditions: Vec<String>,
    pub disqualifying_descriptor_edges: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub enum RewriteSideConditionKindV0 {
    NoSideCondition,
    CascadeWinnerEquality,
    ComputedValueEquality,
    SourceMapTrace,
    TokenOwnershipSeparability,
    TransformIndependence,
    ModuleExportPreservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SideConditionCertV0 {
    NoSideCondition,
    CascadeWinnerEquality {
        certificate: CascadeWinnerEqualityCertV0,
    },
    ComputedValueEquality {
        certificate: ComputedValueEqualityCertV0,
    },
    SourceMapTrace {
        certificate: SourceMapTraceCertV0,
    },
    TokenOwnershipSeparability {
        certificate: TokenOwnershipSeparabilityCertV0,
    },
    TransformIndependence {
        certificate: Box<TransformIndependenceCertV0>,
    },
    ModuleExportPreservation {
        certificate: ModuleExportPreservationCertV0,
    },
}

impl SideConditionCertV0 {
    fn kind(&self) -> RewriteSideConditionKindV0 {
        match self {
            Self::NoSideCondition => RewriteSideConditionKindV0::NoSideCondition,
            Self::CascadeWinnerEquality { .. } => RewriteSideConditionKindV0::CascadeWinnerEquality,
            Self::ComputedValueEquality { .. } => RewriteSideConditionKindV0::ComputedValueEquality,
            Self::SourceMapTrace { .. } => RewriteSideConditionKindV0::SourceMapTrace,
            Self::TokenOwnershipSeparability { .. } => {
                RewriteSideConditionKindV0::TokenOwnershipSeparability
            }
            Self::ModuleExportPreservation { .. } => {
                RewriteSideConditionKindV0::ModuleExportPreservation
            }
            Self::TransformIndependence { .. } => RewriteSideConditionKindV0::TransformIndependence,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteOperatorV0 {
    pub operator: String,
    pub arity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteRuleV0 {
    pub rule_id: String,
    pub before_pattern: RewritePatternV0,
    pub after_pattern: RewritePatternV0,
    pub side_condition_kind: RewriteSideConditionKindV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteRuleCatalogV0 {
    pub schema_version: String,
    pub operators: Vec<RewriteOperatorV0>,
    pub rules: Vec<RewriteRuleV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteSubstitutionEntryV0 {
    pub variable: String,
    pub term: RewriteTermV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RewriteCertificateV0 {
    Refl {
        term: RewriteTermV0,
    },
    Sym {
        certificate: Box<RewriteCertificateV0>,
    },
    Trans {
        left: Box<RewriteCertificateV0>,
        right: Box<RewriteCertificateV0>,
    },
    Cong {
        operator: String,
        certificates: Vec<RewriteCertificateV0>,
    },
    Rewrite {
        rule_id: String,
        substitution: Vec<RewriteSubstitutionEntryV0>,
        side_condition: SideConditionCertV0,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteCertificateEnvelopeV0 {
    pub schema_version: String,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub certificate: RewriteCertificateV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalRewriteAssumptionV0 {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalRewriteAssumptionsV0 {
    pub schema_version: String,
    pub entries: Vec<CanonicalRewriteAssumptionV0>,
}

impl Default for CanonicalRewriteAssumptionsV0 {
    fn default() -> Self {
        Self {
            schema_version: CANONICAL_REWRITE_ASSUMPTIONS_SCHEMA_VERSION_V0.to_owned(),
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub enum RewriteCheckInputV0 {
    BeforeTerm,
    AfterTerm,
    RuleCatalog,
    Certificate,
    Assumptions,
    SerializedCertificate,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteFailureSiteV0 {
    pub input: RewriteCheckInputV0,
    pub certificate_path: Vec<usize>,
    pub term_path: Vec<usize>,
    pub rule_id: Option<String>,
}

impl RewriteFailureSiteV0 {
    fn root(input: RewriteCheckInputV0) -> Self {
        Self {
            input,
            certificate_path: Vec::new(),
            term_path: Vec::new(),
            rule_id: None,
        }
    }

    fn certificate(certificate_path: &[usize]) -> Self {
        Self {
            input: RewriteCheckInputV0::Certificate,
            certificate_path: certificate_path.to_vec(),
            term_path: Vec::new(),
            rule_id: None,
        }
    }

    fn rule(rule_id: &str, term_path: Vec<usize>) -> Self {
        Self {
            input: RewriteCheckInputV0::RuleCatalog,
            certificate_path: Vec::new(),
            term_path,
            rule_id: Some(rule_id.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CertificateRejectionKindV0 {
    SchemaVersionMismatch {
        expected: String,
        observed: String,
    },
    BoundOutOfRange {
        bound: String,
        declared: usize,
        hard_maximum: usize,
    },
    DeclaredBoundExceeded {
        bound: String,
        declared: usize,
        observed: usize,
    },
    CatalogLimitExceeded {
        collection: String,
        observed: usize,
        maximum: usize,
    },
    DuplicateOperator {
        operator: String,
    },
    DuplicateRule {
        rule_id: String,
    },
    UnknownOperator {
        operator: String,
    },
    OperatorArityMismatch {
        operator: String,
        expected: usize,
        observed: usize,
    },
    EmptyVariable,
    DuplicateAssumption {
        name: String,
    },
    DuplicateSubstitutionVariable {
        variable: String,
    },
    MissingSubstitutionVariable {
        variable: String,
    },
    UnexpectedSubstitutionVariable {
        variable: String,
    },
    UnknownRule {
        rule_id: String,
    },
    SideConditionKindMismatch {
        expected: RewriteSideConditionKindV0,
        observed: RewriteSideConditionKindV0,
    },
    InvalidLayerOrdinal {
        observed: i32,
    },
    CascadeWinnerEqualityRejected {
        winner_ids_equal: bool,
        cascade_keys_equal: bool,
        token_support_equal: bool,
    },
    CascadeTokenizationUnavailable,
    DuplicateComputedValueEnvironmentEntry {
        name: String,
    },
    ComputedValueEqualityRejected {
        property: String,
        before_present: bool,
        after_present: bool,
    },
    EmptySourceMapTrace,
    SourceMapTraceRejected {
        segment_index: usize,
        reason: String,
    },
    TokenOwnershipSeparabilityRejected {
        reason: String,
        token: Option<String>,
    },
    TransformIndependenceRejected {
        reason: String,
        left_pass_id: String,
        right_pass_id: String,
    },
    TransitiveMiddleMismatch,
    EndpointMismatch {
        endpoint: RewriteCheckInputV0,
    },
    MissingCertificate,
    MalformedCertificate {
        message: String,
    },
    DerivedTermLimitExceeded,
    ModuleExportPreservationRejected {
        reason: String,
        pass_id: String,
        export_key: Option<ModuleExportKeyV0>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateRejectionV0 {
    pub site: Box<RewriteFailureSiteV0>,
    pub rejection: Box<CertificateRejectionKindV0>,
}

impl CertificateRejectionV0 {
    fn new(site: RewriteFailureSiteV0, rejection: CertificateRejectionKindV0) -> Self {
        Self {
            site: Box::new(site),
            rejection: Box::new(rejection),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RewriteIssuanceSealV0(());

/// Token issued only after the checker derives and matches both endpoints.
///
/// The fields are private and the type has no public constructor.
///
/// ```compile_fail
/// use omena_cascade_proof::RewriteIssuanceTokenV0;
///
/// let _token = RewriteIssuanceTokenV0 {
///     before_digest: [0; 32],
///     after_digest: [0; 32],
///     catalog_schema_id: "caller-owned",
///     catalog_content_digest: [0; 32],
///     checked_rule_ids: Vec::new(),
///     _seal: loop {},
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteIssuanceTokenV0 {
    before_digest: [u8; 32],
    after_digest: [u8; 32],
    catalog_schema_id: &'static str,
    catalog_content_digest: [u8; 32],
    checked_rule_ids: Vec<String>,
    _seal: RewriteIssuanceSealV0,
}

impl RewriteIssuanceTokenV0 {
    fn issue(
        before: &RewriteTermV0,
        after: &RewriteTermV0,
        catalog: &RewriteRuleCatalogV0,
        checked_rule_ids: Vec<String>,
    ) -> Self {
        Self {
            before_digest: term_digest_v0(before),
            after_digest: term_digest_v0(after),
            catalog_schema_id: REWRITE_RULE_CATALOG_SCHEMA_ID_V0,
            catalog_content_digest: catalog_content_digest_v0(catalog),
            checked_rule_ids,
            _seal: RewriteIssuanceSealV0(()),
        }
    }

    pub fn before_digest_hex_v0(&self) -> String {
        digest_hex_v0(&self.before_digest)
    }

    pub fn after_digest_hex_v0(&self) -> String {
        digest_hex_v0(&self.after_digest)
    }

    pub fn checked_rule_ids_v0(&self) -> &[String] {
        self.checked_rule_ids.as_slice()
    }

    pub const fn catalog_schema_id_v0(&self) -> &'static str {
        self.catalog_schema_id
    }

    pub fn catalog_content_digest_hex_v0(&self) -> String {
        digest_hex_v0(&self.catalog_content_digest)
    }

    /// Re-bind a sealed issuance token to the exact endpoints a consumer is
    /// about to apply. A token for a different rewrite pair is not reusable.
    pub fn matches_endpoints_v0(&self, before: &RewriteTermV0, after: &RewriteTermV0) -> bool {
        self.before_digest == term_digest_v0(before) && self.after_digest == term_digest_v0(after)
    }

    /// Compare the sealed catalog identity with catalog content selected by a
    /// consumer. Callers do not supply a digest: it is re-derived here from
    /// the catalog value the consumer trusts.
    pub fn matches_catalog_v0(&self, catalog: &RewriteRuleCatalogV0) -> bool {
        self.catalog_schema_id == REWRITE_RULE_CATALOG_SCHEMA_ID_V0
            && self.catalog_content_digest == catalog_content_digest_v0(catalog)
    }
}

struct ValidatedCatalogV0<'a> {
    operators: BTreeMap<&'a str, usize>,
    rules: BTreeMap<&'a str, &'a RewriteRuleV0>,
}

struct DerivedRewriteV0 {
    before: RewriteTermV0,
    after: RewriteTermV0,
    checked_rule_ids: Vec<String>,
}

pub fn check_rewrite_certificate_v0(
    before: &RewriteTermV0,
    after: &RewriteTermV0,
    rule_catalog: &RewriteRuleCatalogV0,
    certificate: &RewriteCertificateEnvelopeV0,
    assumptions: &CanonicalRewriteAssumptionsV0,
) -> Result<RewriteIssuanceTokenV0, CertificateRejectionV0> {
    validate_schema(
        &certificate.schema_version,
        REWRITE_CERTIFICATE_SCHEMA_VERSION_V0,
        RewriteCheckInputV0::Certificate,
    )?;
    validate_certificate_bounds(certificate)?;
    let catalog = validate_catalog(rule_catalog)?;
    validate_assumptions(assumptions)?;
    validate_term(
        before,
        RewriteCheckInputV0::BeforeTerm,
        &[],
        &catalog.operators,
    )?;
    validate_term(
        after,
        RewriteCheckInputV0::AfterTerm,
        &[],
        &catalog.operators,
    )?;

    let mut path = Vec::new();
    let derived = derive_certificate(&certificate.certificate, &catalog, &mut path)?;
    if derived.before != *before {
        let term_path = first_term_mismatch_path(before, &derived.before);
        return Err(CertificateRejectionV0::new(
            RewriteFailureSiteV0 {
                input: RewriteCheckInputV0::BeforeTerm,
                certificate_path: Vec::new(),
                term_path,
                rule_id: None,
            },
            CertificateRejectionKindV0::EndpointMismatch {
                endpoint: RewriteCheckInputV0::BeforeTerm,
            },
        ));
    }
    if derived.after != *after {
        let term_path = first_term_mismatch_path(after, &derived.after);
        return Err(CertificateRejectionV0::new(
            RewriteFailureSiteV0 {
                input: RewriteCheckInputV0::AfterTerm,
                certificate_path: Vec::new(),
                term_path,
                rule_id: None,
            },
            CertificateRejectionKindV0::EndpointMismatch {
                endpoint: RewriteCheckInputV0::AfterTerm,
            },
        ));
    }

    Ok(RewriteIssuanceTokenV0::issue(
        before,
        after,
        rule_catalog,
        derived.checked_rule_ids,
    ))
}

pub fn check_optional_rewrite_certificate_v0(
    before: &RewriteTermV0,
    after: &RewriteTermV0,
    rule_catalog: &RewriteRuleCatalogV0,
    certificate: Option<&RewriteCertificateEnvelopeV0>,
    assumptions: &CanonicalRewriteAssumptionsV0,
) -> Result<RewriteIssuanceTokenV0, CertificateRejectionV0> {
    let Some(certificate) = certificate else {
        return Err(CertificateRejectionV0::new(
            RewriteFailureSiteV0::root(RewriteCheckInputV0::Certificate),
            CertificateRejectionKindV0::MissingCertificate,
        ));
    };
    check_rewrite_certificate_v0(before, after, rule_catalog, certificate, assumptions)
}

pub fn check_serialized_rewrite_certificate_v0(
    before: &RewriteTermV0,
    after: &RewriteTermV0,
    rule_catalog: &RewriteRuleCatalogV0,
    certificate_json: &str,
    assumptions: &CanonicalRewriteAssumptionsV0,
) -> Result<RewriteIssuanceTokenV0, CertificateRejectionV0> {
    let certificate = serde_json::from_str::<RewriteCertificateEnvelopeV0>(certificate_json)
        .map_err(|error| {
            CertificateRejectionV0::new(
                RewriteFailureSiteV0::root(RewriteCheckInputV0::SerializedCertificate),
                CertificateRejectionKindV0::MalformedCertificate {
                    message: error.to_string(),
                },
            )
        })?;
    check_rewrite_certificate_v0(before, after, rule_catalog, &certificate, assumptions)
}

pub fn selector_rewrite_rule_catalog_v0() -> RewriteRuleCatalogV0 {
    RewriteRuleCatalogV0 {
        schema_version: REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0.to_owned(),
        operators: vec![
            RewriteOperatorV0 {
                operator: "selectorConcat".to_owned(),
                arity: 2,
            },
            RewriteOperatorV0 {
                operator: "selectorIs".to_owned(),
                arity: 1,
            },
            RewriteOperatorV0 {
                operator: "selectorWhere".to_owned(),
                arity: 1,
            },
            RewriteOperatorV0 {
                operator: "selectorList2".to_owned(),
                arity: 2,
            },
        ],
        rules: vec![
            RewriteRuleV0 {
                rule_id: "selector-list-deduplicate-v0".to_owned(),
                before_pattern: RewritePatternV0::apply(
                    "selectorList2",
                    vec![
                        RewritePatternV0::variable("x"),
                        RewritePatternV0::variable("x"),
                    ],
                ),
                after_pattern: RewritePatternV0::variable("x"),
                side_condition_kind: RewriteSideConditionKindV0::NoSideCondition,
            },
            RewriteRuleV0 {
                rule_id: "selector-is-single-v0".to_owned(),
                before_pattern: RewritePatternV0::apply(
                    "selectorIs",
                    vec![RewritePatternV0::variable("x")],
                ),
                after_pattern: RewritePatternV0::variable("x"),
                side_condition_kind: RewriteSideConditionKindV0::NoSideCondition,
            },
        ],
    }
}

pub fn selector_rewrite_rule_catalog_with_cascade_winner_equality_v0() -> RewriteRuleCatalogV0 {
    let mut catalog = selector_rewrite_rule_catalog_v0();
    for rule in &mut catalog.rules {
        rule.side_condition_kind = RewriteSideConditionKindV0::CascadeWinnerEquality;
    }
    catalog
}

const MODULE_EXPORT_PRESERVATION_RULES_V0: [(&str, &str); 4] = [
    (
        "selector-merging",
        "module-export-preservation-selector-merging-v0",
    ),
    (
        "css-modules-class-hashing",
        "module-export-preservation-css-modules-class-hashing-v0",
    ),
    (
        "composes-resolution",
        "module-export-preservation-composes-resolution-v0",
    ),
    (
        "tree-shake-class",
        "module-export-preservation-tree-shake-class-v0",
    ),
];

pub fn module_export_preservation_rule_id_v0(pass_id: &str) -> Option<&'static str> {
    MODULE_EXPORT_PRESERVATION_RULES_V0
        .iter()
        .find_map(|(candidate_pass_id, rule_id)| {
            (*candidate_pass_id == pass_id).then_some(*rule_id)
        })
}

pub fn module_export_preservation_rule_catalog_v0() -> RewriteRuleCatalogV0 {
    let rules = MODULE_EXPORT_PRESERVATION_RULES_V0
        .into_iter()
        .map(|(_pass_id, rule_id)| RewriteRuleV0 {
            rule_id: rule_id.to_owned(),
            before_pattern: RewritePatternV0::apply(
                "moduleExportPreservationRequested",
                vec![
                    RewritePatternV0::variable("pass"),
                    RewritePatternV0::variable("module"),
                ],
            ),
            after_pattern: RewritePatternV0::apply(
                "moduleExportPreservationGranted",
                vec![
                    RewritePatternV0::variable("pass"),
                    RewritePatternV0::variable("module"),
                ],
            ),
            side_condition_kind: RewriteSideConditionKindV0::ModuleExportPreservation,
        })
        .collect();
    RewriteRuleCatalogV0 {
        schema_version: REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0.to_owned(),
        operators: vec![
            RewriteOperatorV0 {
                operator: "moduleExportPreservationRequested".to_owned(),
                arity: 2,
            },
            RewriteOperatorV0 {
                operator: "moduleExportPreservationGranted".to_owned(),
                arity: 2,
            },
        ],
        rules,
    }
}

fn validate_schema(
    observed: &str,
    expected: &str,
    input: RewriteCheckInputV0,
) -> Result<(), CertificateRejectionV0> {
    if observed == expected {
        return Ok(());
    }
    Err(CertificateRejectionV0::new(
        RewriteFailureSiteV0::root(input),
        CertificateRejectionKindV0::SchemaVersionMismatch {
            expected: expected.to_owned(),
            observed: observed.to_owned(),
        },
    ))
}

fn validate_certificate_bounds(
    envelope: &RewriteCertificateEnvelopeV0,
) -> Result<(), CertificateRejectionV0> {
    for (bound, declared, hard_maximum) in [
        (
            "depth",
            envelope.max_depth,
            REWRITE_CERTIFICATE_MAX_DEPTH_V0,
        ),
        (
            "nodes",
            envelope.max_nodes,
            REWRITE_CERTIFICATE_MAX_NODES_V0,
        ),
    ] {
        if declared == 0 || declared > hard_maximum {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0::root(RewriteCheckInputV0::Certificate),
                CertificateRejectionKindV0::BoundOutOfRange {
                    bound: bound.to_owned(),
                    declared,
                    hard_maximum,
                },
            ));
        }
    }

    let mut stack = vec![(&envelope.certificate, 1_usize, Vec::<usize>::new())];
    let mut nodes = 0_usize;
    while let Some((certificate, depth, path)) = stack.pop() {
        nodes = nodes.saturating_add(1);
        if depth > envelope.max_depth {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0::certificate(path.as_slice()),
                CertificateRejectionKindV0::DeclaredBoundExceeded {
                    bound: "depth".to_owned(),
                    declared: envelope.max_depth,
                    observed: depth,
                },
            ));
        }
        if nodes > envelope.max_nodes {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0::certificate(path.as_slice()),
                CertificateRejectionKindV0::DeclaredBoundExceeded {
                    bound: "nodes".to_owned(),
                    declared: envelope.max_nodes,
                    observed: nodes,
                },
            ));
        }
        match certificate {
            RewriteCertificateV0::Refl { .. } | RewriteCertificateV0::Rewrite { .. } => {}
            RewriteCertificateV0::Sym { certificate } => {
                let mut child_path = path;
                child_path.push(0);
                stack.push((certificate, depth.saturating_add(1), child_path));
            }
            RewriteCertificateV0::Trans { left, right } => {
                let mut right_path = path.clone();
                right_path.push(1);
                stack.push((right, depth.saturating_add(1), right_path));
                let mut left_path = path;
                left_path.push(0);
                stack.push((left, depth.saturating_add(1), left_path));
            }
            RewriteCertificateV0::Cong { certificates, .. } => {
                for (index, child) in certificates.iter().enumerate().rev() {
                    let mut child_path = path.clone();
                    child_path.push(index);
                    stack.push((child, depth.saturating_add(1), child_path));
                }
            }
        }
    }
    Ok(())
}

fn validate_catalog(
    catalog: &RewriteRuleCatalogV0,
) -> Result<ValidatedCatalogV0<'_>, CertificateRejectionV0> {
    validate_schema(
        &catalog.schema_version,
        REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0,
        RewriteCheckInputV0::RuleCatalog,
    )?;
    validate_catalog_limit(
        "operators",
        catalog.operators.len(),
        REWRITE_RULE_CATALOG_MAX_OPERATORS_V0,
    )?;
    validate_catalog_limit(
        "rules",
        catalog.rules.len(),
        REWRITE_RULE_CATALOG_MAX_RULES_V0,
    )?;

    let mut operators = BTreeMap::new();
    for operator in &catalog.operators {
        if operators
            .insert(operator.operator.as_str(), operator.arity)
            .is_some()
        {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0::root(RewriteCheckInputV0::RuleCatalog),
                CertificateRejectionKindV0::DuplicateOperator {
                    operator: operator.operator.clone(),
                },
            ));
        }
    }

    let mut rules = BTreeMap::new();
    for rule in &catalog.rules {
        if rules.insert(rule.rule_id.as_str(), rule).is_some() {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0::rule(&rule.rule_id, Vec::new()),
                CertificateRejectionKindV0::DuplicateRule {
                    rule_id: rule.rule_id.clone(),
                },
            ));
        }
        validate_pattern(&rule.before_pattern, &rule.rule_id, &operators)?;
        validate_pattern(&rule.after_pattern, &rule.rule_id, &operators)?;
    }
    Ok(ValidatedCatalogV0 { operators, rules })
}

fn validate_catalog_limit(
    collection: &str,
    observed: usize,
    maximum: usize,
) -> Result<(), CertificateRejectionV0> {
    if observed <= maximum {
        return Ok(());
    }
    Err(CertificateRejectionV0::new(
        RewriteFailureSiteV0::root(RewriteCheckInputV0::RuleCatalog),
        CertificateRejectionKindV0::CatalogLimitExceeded {
            collection: collection.to_owned(),
            observed,
            maximum,
        },
    ))
}

fn validate_pattern(
    pattern: &RewritePatternV0,
    rule_id: &str,
    operators: &BTreeMap<&str, usize>,
) -> Result<(), CertificateRejectionV0> {
    let mut stack = vec![(pattern, 1_usize, Vec::<usize>::new())];
    let mut nodes = 0_usize;
    while let Some((current, depth, path)) = stack.pop() {
        nodes = nodes.saturating_add(1);
        if depth > REWRITE_TERM_MAX_DEPTH_V0 || nodes > REWRITE_TERM_MAX_NODES_V0 {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0::rule(rule_id, path),
                CertificateRejectionKindV0::DerivedTermLimitExceeded,
            ));
        }
        match current {
            RewritePatternV0::Atom { .. } => {}
            RewritePatternV0::Variable { name } => {
                if name.is_empty() {
                    return Err(CertificateRejectionV0::new(
                        RewriteFailureSiteV0::rule(rule_id, path),
                        CertificateRejectionKindV0::EmptyVariable,
                    ));
                }
            }
            RewritePatternV0::Apply { operator, operands } => {
                validate_operator_arity(
                    operator,
                    operands.len(),
                    operators,
                    RewriteFailureSiteV0::rule(rule_id, path.clone()),
                )?;
                for (index, child) in operands.iter().enumerate().rev() {
                    let mut child_path = path.clone();
                    child_path.push(index);
                    stack.push((child, depth.saturating_add(1), child_path));
                }
            }
        }
    }
    Ok(())
}

fn validate_assumptions(
    assumptions: &CanonicalRewriteAssumptionsV0,
) -> Result<BTreeMap<&str, &str>, CertificateRejectionV0> {
    validate_schema(
        &assumptions.schema_version,
        CANONICAL_REWRITE_ASSUMPTIONS_SCHEMA_VERSION_V0,
        RewriteCheckInputV0::Assumptions,
    )?;
    let mut canonical = BTreeMap::new();
    for assumption in &assumptions.entries {
        if canonical
            .insert(assumption.name.as_str(), assumption.value.as_str())
            .is_some()
        {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0::root(RewriteCheckInputV0::Assumptions),
                CertificateRejectionKindV0::DuplicateAssumption {
                    name: assumption.name.clone(),
                },
            ));
        }
    }
    Ok(canonical)
}

fn validate_term(
    term: &RewriteTermV0,
    input: RewriteCheckInputV0,
    certificate_path: &[usize],
    operators: &BTreeMap<&str, usize>,
) -> Result<(), CertificateRejectionV0> {
    let mut stack = vec![(term, 1_usize, Vec::<usize>::new())];
    let mut nodes = 0_usize;
    while let Some((current, depth, term_path)) = stack.pop() {
        nodes = nodes.saturating_add(1);
        if depth > REWRITE_TERM_MAX_DEPTH_V0 || nodes > REWRITE_TERM_MAX_NODES_V0 {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0 {
                    input,
                    certificate_path: certificate_path.to_vec(),
                    term_path,
                    rule_id: None,
                },
                CertificateRejectionKindV0::DerivedTermLimitExceeded,
            ));
        }
        if let RewriteTermV0::Apply { operator, operands } = current {
            validate_operator_arity(
                operator,
                operands.len(),
                operators,
                RewriteFailureSiteV0 {
                    input,
                    certificate_path: certificate_path.to_vec(),
                    term_path: term_path.clone(),
                    rule_id: None,
                },
            )?;
            for (index, child) in operands.iter().enumerate().rev() {
                let mut child_path = term_path.clone();
                child_path.push(index);
                stack.push((child, depth.saturating_add(1), child_path));
            }
        }
    }
    Ok(())
}

fn validate_operator_arity(
    operator: &str,
    observed: usize,
    operators: &BTreeMap<&str, usize>,
    site: RewriteFailureSiteV0,
) -> Result<(), CertificateRejectionV0> {
    let Some(expected) = operators.get(operator).copied() else {
        return Err(CertificateRejectionV0::new(
            site,
            CertificateRejectionKindV0::UnknownOperator {
                operator: operator.to_owned(),
            },
        ));
    };
    if expected == observed {
        return Ok(());
    }
    Err(CertificateRejectionV0::new(
        site,
        CertificateRejectionKindV0::OperatorArityMismatch {
            operator: operator.to_owned(),
            expected,
            observed,
        },
    ))
}

fn derive_certificate(
    certificate: &RewriteCertificateV0,
    catalog: &ValidatedCatalogV0<'_>,
    path: &mut Vec<usize>,
) -> Result<DerivedRewriteV0, CertificateRejectionV0> {
    match certificate {
        RewriteCertificateV0::Refl { term } => {
            validate_term(
                term,
                RewriteCheckInputV0::Certificate,
                path,
                &catalog.operators,
            )?;
            Ok(DerivedRewriteV0 {
                before: term.clone(),
                after: term.clone(),
                checked_rule_ids: Vec::new(),
            })
        }
        RewriteCertificateV0::Sym { certificate } => {
            path.push(0);
            let child = derive_certificate(certificate, catalog, path);
            path.pop();
            let child = child?;
            Ok(DerivedRewriteV0 {
                before: child.after,
                after: child.before,
                checked_rule_ids: child.checked_rule_ids,
            })
        }
        RewriteCertificateV0::Trans { left, right } => {
            path.push(0);
            let left_derived = derive_certificate(left, catalog, path);
            path.pop();
            let left_derived = left_derived?;
            path.push(1);
            let right_derived = derive_certificate(right, catalog, path);
            path.pop();
            let right_derived = right_derived?;
            if left_derived.after != right_derived.before {
                return Err(CertificateRejectionV0::new(
                    RewriteFailureSiteV0 {
                        input: RewriteCheckInputV0::Certificate,
                        certificate_path: path.clone(),
                        term_path: first_term_mismatch_path(
                            &left_derived.after,
                            &right_derived.before,
                        ),
                        rule_id: None,
                    },
                    CertificateRejectionKindV0::TransitiveMiddleMismatch,
                ));
            }
            let mut checked_rule_ids = left_derived.checked_rule_ids;
            checked_rule_ids.extend(right_derived.checked_rule_ids);
            Ok(DerivedRewriteV0 {
                before: left_derived.before,
                after: right_derived.after,
                checked_rule_ids,
            })
        }
        RewriteCertificateV0::Cong {
            operator,
            certificates,
        } => {
            validate_operator_arity(
                operator,
                certificates.len(),
                &catalog.operators,
                RewriteFailureSiteV0::certificate(path.as_slice()),
            )?;
            let mut before_operands = Vec::with_capacity(certificates.len());
            let mut after_operands = Vec::with_capacity(certificates.len());
            let mut checked_rule_ids = Vec::new();
            for (index, child) in certificates.iter().enumerate() {
                path.push(index);
                let child_derived = derive_certificate(child, catalog, path);
                path.pop();
                let child_derived = child_derived?;
                before_operands.push(child_derived.before);
                after_operands.push(child_derived.after);
                checked_rule_ids.extend(child_derived.checked_rule_ids);
            }
            let before = RewriteTermV0::apply(operator, before_operands);
            let after = RewriteTermV0::apply(operator, after_operands);
            validate_term(
                &before,
                RewriteCheckInputV0::Certificate,
                path,
                &catalog.operators,
            )?;
            validate_term(
                &after,
                RewriteCheckInputV0::Certificate,
                path,
                &catalog.operators,
            )?;
            Ok(DerivedRewriteV0 {
                before,
                after,
                checked_rule_ids,
            })
        }
        RewriteCertificateV0::Rewrite {
            rule_id,
            substitution,
            side_condition,
        } => derive_rule_application(rule_id, substitution, side_condition, catalog, path),
    }
}

fn derive_rule_application(
    rule_id: &str,
    substitution: &[RewriteSubstitutionEntryV0],
    side_condition: &SideConditionCertV0,
    catalog: &ValidatedCatalogV0<'_>,
    path: &[usize],
) -> Result<DerivedRewriteV0, CertificateRejectionV0> {
    let Some(rule) = catalog.rules.get(rule_id).copied() else {
        return Err(CertificateRejectionV0::new(
            RewriteFailureSiteV0 {
                input: RewriteCheckInputV0::Certificate,
                certificate_path: path.to_vec(),
                term_path: Vec::new(),
                rule_id: Some(rule_id.to_owned()),
            },
            CertificateRejectionKindV0::UnknownRule {
                rule_id: rule_id.to_owned(),
            },
        ));
    };
    let observed_kind = side_condition.kind();
    if observed_kind != rule.side_condition_kind {
        return Err(CertificateRejectionV0::new(
            RewriteFailureSiteV0 {
                input: RewriteCheckInputV0::Certificate,
                certificate_path: path.to_vec(),
                term_path: Vec::new(),
                rule_id: Some(rule_id.to_owned()),
            },
            CertificateRejectionKindV0::SideConditionKindMismatch {
                expected: rule.side_condition_kind,
                observed: observed_kind,
            },
        ));
    }
    check_side_condition_v0(side_condition, path, rule_id)?;

    let mut substitutions = BTreeMap::new();
    for entry in substitution {
        if substitutions
            .insert(entry.variable.as_str(), &entry.term)
            .is_some()
        {
            return Err(CertificateRejectionV0::new(
                RewriteFailureSiteV0 {
                    input: RewriteCheckInputV0::Certificate,
                    certificate_path: path.to_vec(),
                    term_path: Vec::new(),
                    rule_id: Some(rule_id.to_owned()),
                },
                CertificateRejectionKindV0::DuplicateSubstitutionVariable {
                    variable: entry.variable.clone(),
                },
            ));
        }
        validate_term(
            &entry.term,
            RewriteCheckInputV0::Certificate,
            path,
            &catalog.operators,
        )?;
    }

    let mut variables = BTreeSet::new();
    collect_pattern_variables(&rule.before_pattern, &mut variables);
    collect_pattern_variables(&rule.after_pattern, &mut variables);
    if let Some(missing) = variables
        .iter()
        .find(|variable| !substitutions.contains_key(variable.as_str()))
    {
        return Err(CertificateRejectionV0::new(
            RewriteFailureSiteV0 {
                input: RewriteCheckInputV0::Certificate,
                certificate_path: path.to_vec(),
                term_path: Vec::new(),
                rule_id: Some(rule_id.to_owned()),
            },
            CertificateRejectionKindV0::MissingSubstitutionVariable {
                variable: (*missing).clone(),
            },
        ));
    }
    if let Some(extra) = substitutions
        .keys()
        .find(|variable| !variables.contains(**variable))
    {
        return Err(CertificateRejectionV0::new(
            RewriteFailureSiteV0 {
                input: RewriteCheckInputV0::Certificate,
                certificate_path: path.to_vec(),
                term_path: Vec::new(),
                rule_id: Some(rule_id.to_owned()),
            },
            CertificateRejectionKindV0::UnexpectedSubstitutionVariable {
                variable: (*extra).to_owned(),
            },
        ));
    }

    let mut before_nodes = 0_usize;
    let before = instantiate_pattern(&rule.before_pattern, &substitutions, &mut before_nodes)
        .ok_or_else(|| {
            CertificateRejectionV0::new(
                RewriteFailureSiteV0 {
                    input: RewriteCheckInputV0::Certificate,
                    certificate_path: path.to_vec(),
                    term_path: Vec::new(),
                    rule_id: Some(rule_id.to_owned()),
                },
                CertificateRejectionKindV0::DerivedTermLimitExceeded,
            )
        })?;
    let mut after_nodes = 0_usize;
    let after = instantiate_pattern(&rule.after_pattern, &substitutions, &mut after_nodes)
        .ok_or_else(|| {
            CertificateRejectionV0::new(
                RewriteFailureSiteV0 {
                    input: RewriteCheckInputV0::Certificate,
                    certificate_path: path.to_vec(),
                    term_path: Vec::new(),
                    rule_id: Some(rule_id.to_owned()),
                },
                CertificateRejectionKindV0::DerivedTermLimitExceeded,
            )
        })?;
    Ok(DerivedRewriteV0 {
        before,
        after,
        checked_rule_ids: vec![rule_id.to_owned()],
    })
}

fn check_side_condition_v0(
    side_condition: &SideConditionCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<(), CertificateRejectionV0> {
    match side_condition {
        SideConditionCertV0::NoSideCondition => Ok(()),
        SideConditionCertV0::CascadeWinnerEquality { certificate } => {
            check_cascade_winner_equality_v0(certificate, path, rule_id)
        }
        SideConditionCertV0::ComputedValueEquality { certificate } => {
            check_computed_value_equality_v0(certificate, path, rule_id)
        }
        SideConditionCertV0::SourceMapTrace { certificate } => {
            check_source_map_trace_v0(certificate, path, rule_id)
        }
        SideConditionCertV0::TokenOwnershipSeparability { certificate } => {
            check_token_ownership_separability_v0(certificate, path, rule_id)
        }
        SideConditionCertV0::ModuleExportPreservation { certificate } => {
            check_module_export_preservation_v0(certificate, path, rule_id)
        }
        SideConditionCertV0::TransformIndependence { certificate } => {
            check_transform_independence_v0(certificate, path, rule_id)
        }
    }
}

fn check_transform_independence_v0(
    certificate: &TransformIndependenceCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<(), CertificateRejectionV0> {
    let reject = |reason: &str| {
        CertificateRejectionV0::new(
            side_condition_site_v0(path, rule_id),
            CertificateRejectionKindV0::TransformIndependenceRejected {
                reason: reason.to_owned(),
                left_pass_id: certificate.left_pass_id.clone(),
                right_pass_id: certificate.right_pass_id.clone(),
            },
        )
    };
    if certificate.left_pass_id.is_empty()
        || certificate.right_pass_id.is_empty()
        || certificate.left_pass_id == certificate.right_pass_id
    {
        return Err(reject("independence pair is empty or reflexive"));
    }
    if certificate.observation_profile_id.is_empty() || certificate.profile_observers.is_empty() {
        return Err(reject("observation profile is empty"));
    }
    if !certificate.disqualifying_descriptor_edges.is_empty() {
        return Err(reject(
            "descriptor dependency or conflict disqualifies the pair",
        ));
    }

    let profile_observers = certificate
        .profile_observers
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if profile_observers.len() != certificate.profile_observers.len()
        || profile_observers.contains("")
    {
        return Err(reject(
            "observation profile contains an empty or duplicate observer",
        ));
    }
    if certificate.observation_rows.is_empty() {
        return Err(reject("observational commutation has no checked rows"));
    }
    let mut observed_profile_members = BTreeSet::new();
    let mut row_keys = BTreeSet::new();
    for row in &certificate.observation_rows {
        if row.fixture_id.is_empty() || !profile_observers.contains(row.observer.as_str()) {
            return Err(reject(
                "observation row does not resolve to the named profile",
            ));
        }
        if !row_keys.insert((row.fixture_id.as_str(), row.observer.as_str())) {
            return Err(reject("observation row is duplicated"));
        }
        if row.left_then_right != row.right_then_left {
            return Err(reject(
                "adjacent transform orders have different observations",
            ));
        }
        observed_profile_members.insert(row.observer.as_str());
    }
    if observed_profile_members != profile_observers {
        return Err(reject("observation rows do not cover the named profile"));
    }

    let left_preconditions = certificate
        .left_preconditions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let right_preconditions = certificate
        .right_preconditions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let left_preserves_right = certificate
        .left_preserves_right_preconditions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let right_preserves_left = certificate
        .right_preserves_left_preconditions
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if left_preconditions.len() != certificate.left_preconditions.len()
        || right_preconditions.len() != certificate.right_preconditions.len()
        || left_preserves_right.len() != certificate.left_preserves_right_preconditions.len()
        || right_preserves_left.len() != certificate.right_preserves_left_preconditions.len()
    {
        return Err(reject("precondition evidence contains duplicate entries"));
    }
    if left_preserves_right != right_preconditions || right_preserves_left != left_preconditions {
        return Err(reject("mutual precondition preservation is incomplete"));
    }
    Ok(())
}

fn check_token_ownership_separability_v0(
    certificate: &TokenOwnershipSeparabilityCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<(), CertificateRejectionV0> {
    let reject = |reason: &str, token: Option<String>| {
        CertificateRejectionV0::new(
            side_condition_site_v0(path, rule_id),
            CertificateRejectionKindV0::TokenOwnershipSeparabilityRejected {
                reason: reason.to_owned(),
                token,
            },
        )
    };
    if !certificate.complete {
        return Err(reject("ownership census is incomplete", None));
    }
    if certificate.unattributed_emitted_token_count != 0 {
        return Err(reject("emitted token has no attributed owner", None));
    }
    if certificate.interface_mismatch_count != 0 {
        return Err(reject("emitted token disagrees with its interface", None));
    }
    if certificate.emitted_token_count != certificate.ownerships.len() {
        return Err(reject(
            "emitted token count does not match ownership rows",
            None,
        ));
    }
    let mut tokens = BTreeSet::new();
    for ownership in &certificate.ownerships {
        if ownership.emitted_token.is_empty() {
            return Err(reject("ownership row has an empty emitted token", None));
        }
        if !tokens.insert(ownership.emitted_token.as_str()) {
            return Err(reject(
                "ownership census repeats an emitted token",
                Some(ownership.emitted_token.clone()),
            ));
        }
        if ownership.module_paths.len() != 1 || ownership.module_paths[0].is_empty() {
            return Err(reject(
                "emitted token does not resolve to exactly one module path",
                Some(ownership.emitted_token.clone()),
            ));
        }
    }
    if certificate.modeled_preimage_count != certificate.ownerships.len() {
        return Err(reject(
            "modeled preimages do not form a one-to-one ownership relation",
            None,
        ));
    }
    Ok(())
}

fn check_module_export_preservation_v0(
    certificate: &ModuleExportPreservationCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<(), CertificateRejectionV0> {
    let reject = |reason: &str, export_key: Option<ModuleExportKeyV0>| {
        CertificateRejectionV0::new(
            side_condition_site_v0(path, rule_id),
            CertificateRejectionKindV0::ModuleExportPreservationRejected {
                reason: reason.to_owned(),
                pass_id: certificate.pass_id.clone(),
                export_key,
            },
        )
    };
    let Some(expected_rule_id) = module_export_preservation_rule_id_v0(&certificate.pass_id) else {
        return Err(reject(
            "pass does not claim exported-class-name preservation",
            None,
        ));
    };
    if expected_rule_id != rule_id {
        return Err(reject(
            "certificate pass does not match the selected catalog rule",
            None,
        ));
    }
    if certificate.before_premise_digest
        != module_export_observation_digest_hex_v0(&certificate.before_exports)
    {
        return Err(reject(
            "before export premise digest does not match its rows",
            None,
        ));
    }
    if certificate.after_premise_digest
        != module_export_observation_digest_hex_v0(&certificate.after_exports)
    {
        return Err(reject(
            "after export premise digest does not match its rows",
            None,
        ));
    }

    let before = module_export_observation_map_v0(&certificate.before_exports)
        .map_err(|(reason, key)| reject(reason, Some(key)))?;
    let after = module_export_observation_map_v0(&certificate.after_exports)
        .map_err(|(reason, key)| reject(reason, Some(key)))?;
    let mut deltas = BTreeMap::new();
    let mut delta_before_tokens = BTreeSet::new();
    let mut delta_after_tokens = BTreeSet::new();
    for delta in &certificate.declared_rename_delta {
        if deltas.insert(delta.key.clone(), delta).is_some() {
            return Err(reject(
                "rename delta repeats an identity key",
                Some(delta.key.clone()),
            ));
        }
        if !delta_before_tokens.insert(delta.before_token.as_str()) {
            return Err(reject(
                "rename delta does not form a bijection over before tokens",
                Some(delta.key.clone()),
            ));
        }
        if !delta_after_tokens.insert(delta.after_token.as_str()) {
            return Err(reject(
                "rename delta does not form a bijection over after tokens",
                Some(delta.key.clone()),
            ));
        }
    }
    if certificate.pass_id != "css-modules-class-hashing" && !deltas.is_empty() {
        return Err(reject(
            "only css-modules-class-hashing may declare an export rename delta",
            deltas.keys().next().cloned(),
        ));
    }

    for (key, before_token) in &before {
        let Some(after_token) = after.get(key) else {
            return Err(reject(
                "after export premise dropped an identity key",
                Some(key.clone()),
            ));
        };
        let expected_after = if let Some(delta) = deltas.get(key) {
            if delta.before_token.as_str() != *before_token {
                return Err(reject(
                    "rename delta before token does not match the observed premise",
                    Some(key.clone()),
                ));
            }
            delta.after_token.as_str()
        } else {
            *before_token
        };
        if expected_after != *after_token {
            return Err(reject(
                "after export token is neither preserved nor covered by the declared rename delta",
                Some(key.clone()),
            ));
        }
    }
    if let Some(extra) = after.keys().find(|key| !before.contains_key(*key)) {
        return Err(reject(
            "after export premise introduced an unmodeled identity key",
            Some((*extra).clone()),
        ));
    }
    if let Some(unused) = deltas.keys().find(|key| !before.contains_key(*key)) {
        return Err(reject(
            "rename delta names an identity key absent from the before premise",
            Some(unused.clone()),
        ));
    }

    let expected_after_tokens = before
        .iter()
        .map(|(key, before_token)| {
            deltas
                .get(key)
                .map_or(*before_token, |delta| delta.after_token.as_str())
        })
        .collect::<Vec<_>>();
    let actual_after_tokens = after.values().copied().collect::<Vec<_>>();
    let (expected_word_len, expected_support) =
        module_export_token_support_v0(expected_after_tokens.iter().copied());
    let (actual_word_len, actual_support) =
        module_export_token_support_v0(actual_after_tokens.iter().copied());
    if expected_word_len != expected_after_tokens.len()
        || actual_word_len != actual_after_tokens.len()
    {
        return Err(reject(
            "export tokens collapse under canonical class-token identity",
            None,
        ));
    }
    if expected_support != actual_support {
        return Err(reject(
            "canonical export-token support does not match the declared preservation relation",
            None,
        ));
    }
    Ok(())
}

fn module_export_token_support_v0<'a>(
    tokens: impl IntoIterator<Item = &'a str>,
) -> (usize, TokenSupportV0) {
    let word = OrderedTokenWordV0::from_keys(
        tokens
            .into_iter()
            .map(|token| ClassNameV0::new(token).canonical_key()),
    );
    let word_len = word.tokens().len();
    (word_len, token_support_v0(&word))
}

fn module_export_observation_map_v0(
    observations: &[ModuleExportObservationV0],
) -> Result<BTreeMap<ModuleExportKeyV0, &str>, (&'static str, ModuleExportKeyV0)> {
    let mut result = BTreeMap::new();
    for observation in observations {
        let canonical_class_name = ClassNameV0::new(observation.key.canonical_class_name.as_str());
        if observation.key.canonical_class_name.is_empty()
            || canonical_class_name.decoded() != observation.key.canonical_class_name
        {
            return Err((
                "export premise contains a non-canonical identity key",
                observation.key.clone(),
            ));
        }
        if observation.emitted_token.is_empty() {
            return Err((
                "export premise contains an empty emitted token",
                observation.key.clone(),
            ));
        }
        if result
            .insert(observation.key.clone(), observation.emitted_token.as_str())
            .is_some()
        {
            return Err((
                "export premise repeats an identity key",
                observation.key.clone(),
            ));
        }
    }
    Ok(result)
}

pub fn module_export_observation_digest_hex_v0(
    observations: &[ModuleExportObservationV0],
) -> String {
    let mut ordered = observations.iter().collect::<Vec<_>>();
    ordered.sort();
    let mut hasher = blake3::Hasher::new();
    update_framed_hash_v0(
        &mut hasher,
        b"omena-cascade-proof.module-export-observation.v0",
    );
    for observation in ordered {
        update_framed_hash_v0(
            &mut hasher,
            observation.key.module_instance.module().as_str().as_bytes(),
        );
        update_framed_hash_v0(
            &mut hasher,
            observation
                .key
                .module_instance
                .configuration()
                .as_str()
                .as_bytes(),
        );
        update_framed_hash_v0(&mut hasher, observation.key.canonical_class_name.as_bytes());
        update_framed_hash_v0(&mut hasher, observation.emitted_token.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn side_condition_site_v0(path: &[usize], rule_id: &str) -> RewriteFailureSiteV0 {
    RewriteFailureSiteV0 {
        input: RewriteCheckInputV0::Certificate,
        certificate_path: path.to_vec(),
        term_path: Vec::new(),
        rule_id: Some(rule_id.to_owned()),
    }
}

fn cascade_key_from_certificate_v0(
    key: &CascadeWinnerKeyCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<CascadeKey, CertificateRejectionV0> {
    let layer_ordinal = match key.layer_ordinal {
        Some(ordinal) => Some(LayerOrdinal::new(ordinal).ok_or_else(|| {
            CertificateRejectionV0::new(
                side_condition_site_v0(path, rule_id),
                CertificateRejectionKindV0::InvalidLayerOrdinal { observed: ordinal },
            )
        })?),
        None => None,
    };
    Ok(CascadeKey::new(
        key.level.to_cascade_level(),
        normalized_layer_rank(key.layer_important, layer_ordinal),
        key.scope_proximity,
        Specificity::new(
            key.specificity_ids,
            key.specificity_classes,
            key.specificity_elements,
        ),
        key.source_order,
    ))
}

fn check_cascade_winner_equality_v0(
    certificate: &CascadeWinnerEqualityCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<(), CertificateRejectionV0> {
    // The key ordering and token support are recomputed by omena-cascade,
    // outside the transform pass that benefits from this certificate.
    let before_key = cascade_key_from_certificate_v0(&certificate.before_key, path, rule_id)?;
    let after_key = cascade_key_from_certificate_v0(&certificate.after_key, path, rule_id)?;
    let before_tokenization =
        tokenize_dom_class_attribute_v0(Some(&certificate.before_class_attribute));
    let after_tokenization =
        tokenize_dom_class_attribute_v0(Some(&certificate.after_class_attribute));
    let (
        DomClassTokenizationV0::Known {
            word: before_word, ..
        },
        DomClassTokenizationV0::Known {
            word: after_word, ..
        },
    ) = (before_tokenization, after_tokenization)
    else {
        return Err(CertificateRejectionV0::new(
            side_condition_site_v0(path, rule_id),
            CertificateRejectionKindV0::CascadeTokenizationUnavailable,
        ));
    };
    let winner_ids_equal = certificate.before_winner_id == certificate.after_winner_id;
    let cascade_keys_equal = before_key == after_key;
    let token_support_equal = token_support_v0(&before_word) == token_support_v0(&after_word);
    if winner_ids_equal && cascade_keys_equal && token_support_equal {
        return Ok(());
    }
    Err(CertificateRejectionV0::new(
        side_condition_site_v0(path, rule_id),
        CertificateRejectionKindV0::CascadeWinnerEqualityRejected {
            winner_ids_equal,
            cascade_keys_equal,
            token_support_equal,
        },
    ))
}

fn computed_value_environment_v0(
    entries: &[ComputedValueEnvironmentEntryV0],
    path: &[usize],
    rule_id: &str,
) -> Result<BTreeMap<String, CascadeValue>, CertificateRejectionV0> {
    let mut environment = BTreeMap::new();
    for entry in entries {
        if environment
            .insert(entry.name.clone(), entry.value.to_cascade_value())
            .is_some()
        {
            return Err(CertificateRejectionV0::new(
                side_condition_site_v0(path, rule_id),
                CertificateRejectionKindV0::DuplicateComputedValueEnvironmentEntry {
                    name: entry.name.clone(),
                },
            ));
        }
    }
    Ok(environment)
}

fn check_computed_value_equality_v0(
    certificate: &ComputedValueEqualityCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<(), CertificateRejectionV0> {
    // Fixed-point resolution is recomputed by omena-cascade's value plane,
    // outside the transform pass and outside the obligation-family tag.
    let before_environment =
        computed_value_environment_v0(&certificate.before_environment, path, rule_id)?;
    let after_environment =
        computed_value_environment_v0(&certificate.after_environment, path, rule_id)?;
    let before_resolved = resolve_custom_property_env_least_fixed_point(&before_environment);
    let after_resolved = resolve_custom_property_env_least_fixed_point(&after_environment);
    let before_value = before_resolved.get(&certificate.property);
    let after_value = after_resolved.get(&certificate.property);
    if before_value.is_some() && before_value == after_value {
        return Ok(());
    }
    Err(CertificateRejectionV0::new(
        side_condition_site_v0(path, rule_id),
        CertificateRejectionKindV0::ComputedValueEqualityRejected {
            property: certificate.property.clone(),
            before_present: before_value.is_some(),
            after_present: after_value.is_some(),
        },
    ))
}

fn check_source_map_trace_v0(
    certificate: &SourceMapTraceCertV0,
    path: &[usize],
    rule_id: &str,
) -> Result<(), CertificateRejectionV0> {
    // These are emitted segment-table records, not a transform-owned
    // provenance boolean. The checker compares their source projections.
    if certificate.before_segments.is_empty() || certificate.after_segments.is_empty() {
        return Err(CertificateRejectionV0::new(
            side_condition_site_v0(path, rule_id),
            CertificateRejectionKindV0::EmptySourceMapTrace,
        ));
    }
    if certificate.before_segments.len() != certificate.after_segments.len() {
        return Err(CertificateRejectionV0::new(
            side_condition_site_v0(path, rule_id),
            CertificateRejectionKindV0::SourceMapTraceRejected {
                segment_index: 0,
                reason: "segment count differs".to_owned(),
            },
        ));
    }
    for (index, (before, after)) in certificate
        .before_segments
        .iter()
        .zip(&certificate.after_segments)
        .enumerate()
    {
        let ranges_valid = before.original_start <= before.original_end
            && before.generated_start <= before.generated_end
            && after.original_start <= after.original_end
            && after.generated_start <= after.generated_end;
        let source_projection_equal = before.source_path == after.source_path
            && before.source_digest == after.source_digest
            && before.original_start == after.original_start
            && before.original_end == after.original_end
            && before.pass_id == after.pass_id;
        if !ranges_valid || !source_projection_equal {
            return Err(CertificateRejectionV0::new(
                side_condition_site_v0(path, rule_id),
                CertificateRejectionKindV0::SourceMapTraceRejected {
                    segment_index: index,
                    reason: if ranges_valid {
                        "source projection differs".to_owned()
                    } else {
                        "segment range is inverted".to_owned()
                    },
                },
            ));
        }
    }
    Ok(())
}

fn collect_pattern_variables(pattern: &RewritePatternV0, variables: &mut BTreeSet<String>) {
    let mut stack = vec![pattern];
    while let Some(current) = stack.pop() {
        match current {
            RewritePatternV0::Atom { .. } => {}
            RewritePatternV0::Variable { name } => {
                variables.insert(name.clone());
            }
            RewritePatternV0::Apply { operands, .. } => stack.extend(operands),
        }
    }
}

fn instantiate_pattern(
    pattern: &RewritePatternV0,
    substitutions: &BTreeMap<&str, &RewriteTermV0>,
    nodes: &mut usize,
) -> Option<RewriteTermV0> {
    *nodes = nodes.saturating_add(1);
    if *nodes > REWRITE_TERM_MAX_NODES_V0 {
        return None;
    }
    match pattern {
        RewritePatternV0::Atom { value } => Some(RewriteTermV0::atom(value)),
        RewritePatternV0::Variable { name } => {
            let term = substitutions.get(name.as_str())?;
            let term_nodes = term_node_count_v0(term)?;
            *nodes = nodes.saturating_add(term_nodes.saturating_sub(1));
            (*nodes <= REWRITE_TERM_MAX_NODES_V0).then(|| (*term).clone())
        }
        RewritePatternV0::Apply { operator, operands } => {
            let mut instantiated = Vec::with_capacity(operands.len());
            for operand in operands {
                instantiated.push(instantiate_pattern(operand, substitutions, nodes)?);
            }
            Some(RewriteTermV0::apply(operator, instantiated))
        }
    }
}

fn term_node_count_v0(term: &RewriteTermV0) -> Option<usize> {
    let mut stack = vec![term];
    let mut nodes = 0_usize;
    while let Some(current) = stack.pop() {
        nodes = nodes.checked_add(1)?;
        if nodes > REWRITE_TERM_MAX_NODES_V0 {
            return None;
        }
        if let RewriteTermV0::Apply { operands, .. } = current {
            stack.extend(operands);
        }
    }
    Some(nodes)
}

fn first_term_mismatch_path(expected: &RewriteTermV0, observed: &RewriteTermV0) -> Vec<usize> {
    let mut stack = vec![(expected, observed, Vec::<usize>::new())];
    while let Some((left, right, path)) = stack.pop() {
        match (left, right) {
            (RewriteTermV0::Atom { value: left }, RewriteTermV0::Atom { value: right })
                if left == right => {}
            (
                RewriteTermV0::Apply {
                    operator: left_operator,
                    operands: left_operands,
                },
                RewriteTermV0::Apply {
                    operator: right_operator,
                    operands: right_operands,
                },
            ) if left_operator == right_operator && left_operands.len() == right_operands.len() => {
                for index in (0..left_operands.len()).rev() {
                    let mut child_path = path.clone();
                    child_path.push(index);
                    stack.push((&left_operands[index], &right_operands[index], child_path));
                }
            }
            _ => return path,
        }
    }
    Vec::new()
}

/// Return the order-independent digest used to bind an issuance token to the
/// exact catalog content checked by the kernel.
pub fn rewrite_rule_catalog_content_digest_hex_v0(catalog: &RewriteRuleCatalogV0) -> String {
    digest_hex_v0(&catalog_content_digest_v0(catalog))
}

fn catalog_content_digest_v0(catalog: &RewriteRuleCatalogV0) -> [u8; 32] {
    let mut operators = catalog
        .operators
        .iter()
        .map(|operator| {
            let mut material = Vec::new();
            append_framed_bytes_v0(&mut material, operator.operator.as_bytes());
            append_framed_bytes_v0(&mut material, operator.arity.to_string().as_bytes());
            material
        })
        .collect::<Vec<_>>();
    operators.sort();

    let mut rules = catalog
        .rules
        .iter()
        .map(|rule| {
            let mut material = Vec::new();
            append_framed_bytes_v0(&mut material, rule.rule_id.as_bytes());
            append_pattern_material_v0(&mut material, &rule.before_pattern);
            append_pattern_material_v0(&mut material, &rule.after_pattern);
            append_framed_bytes_v0(
                &mut material,
                rewrite_side_condition_kind_id_v0(rule.side_condition_kind).as_bytes(),
            );
            material
        })
        .collect::<Vec<_>>();
    rules.sort();

    let mut hasher = blake3::Hasher::new();
    update_framed_hash_v0(&mut hasher, REWRITE_RULE_CATALOG_SCHEMA_ID_V0.as_bytes());
    update_framed_hash_v0(&mut hasher, catalog.schema_version.as_bytes());
    update_framed_hash_v0(&mut hasher, operators.len().to_string().as_bytes());
    for operator in operators {
        update_framed_hash_v0(&mut hasher, operator.as_slice());
    }
    update_framed_hash_v0(&mut hasher, rules.len().to_string().as_bytes());
    for rule in rules {
        update_framed_hash_v0(&mut hasher, rule.as_slice());
    }
    *hasher.finalize().as_bytes()
}

fn append_pattern_material_v0(material: &mut Vec<u8>, pattern: &RewritePatternV0) {
    match pattern {
        RewritePatternV0::Atom { value } => {
            append_framed_bytes_v0(material, b"atom");
            append_framed_bytes_v0(material, value.as_bytes());
        }
        RewritePatternV0::Variable { name } => {
            append_framed_bytes_v0(material, b"variable");
            append_framed_bytes_v0(material, name.as_bytes());
        }
        RewritePatternV0::Apply { operator, operands } => {
            append_framed_bytes_v0(material, b"apply");
            append_framed_bytes_v0(material, operator.as_bytes());
            append_framed_bytes_v0(material, operands.len().to_string().as_bytes());
            for operand in operands {
                append_pattern_material_v0(material, operand);
            }
        }
    }
}

fn rewrite_side_condition_kind_id_v0(kind: RewriteSideConditionKindV0) -> &'static str {
    match kind {
        RewriteSideConditionKindV0::NoSideCondition => "noSideCondition",
        RewriteSideConditionKindV0::CascadeWinnerEquality => "cascadeWinnerEquality",
        RewriteSideConditionKindV0::ComputedValueEquality => "computedValueEquality",
        RewriteSideConditionKindV0::SourceMapTrace => "sourceMapTrace",
        RewriteSideConditionKindV0::TokenOwnershipSeparability => "tokenOwnershipSeparability",
        RewriteSideConditionKindV0::ModuleExportPreservation => "moduleExportPreservation",
        RewriteSideConditionKindV0::TransformIndependence => "transformIndependence",
    }
}

fn append_framed_bytes_v0(material: &mut Vec<u8>, value: &[u8]) {
    material.extend_from_slice(value.len().to_string().as_bytes());
    material.push(0);
    material.extend_from_slice(value);
    material.push(0xff);
}

fn update_framed_hash_v0(hasher: &mut blake3::Hasher, value: &[u8]) {
    hasher.update(value.len().to_string().as_bytes());
    hasher.update(b"\0");
    hasher.update(value);
    hasher.update(b"\xff");
}

fn term_digest_v0(term: &RewriteTermV0) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    let mut stack = vec![term];
    while let Some(current) = stack.pop() {
        match current {
            RewriteTermV0::Atom { value } => {
                hasher.update(b"atom\0");
                hasher.update(value.len().to_string().as_bytes());
                hasher.update(b"\0");
                hasher.update(value.as_bytes());
            }
            RewriteTermV0::Apply { operator, operands } => {
                hasher.update(b"apply\0");
                hasher.update(operator.len().to_string().as_bytes());
                hasher.update(b"\0");
                hasher.update(operator.as_bytes());
                hasher.update(b"\0");
                hasher.update(operands.len().to_string().as_bytes());
                for operand in operands.iter().rev() {
                    stack.push(operand);
                }
            }
        }
    }
    *hasher.finalize().as_bytes()
}

fn digest_hex_v0(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    fn selector_terms() -> (RewriteTermV0, RewriteTermV0) {
        let before = RewriteTermV0::apply(
            "selectorConcat",
            vec![
                RewriteTermV0::atom(".root"),
                RewriteTermV0::apply(
                    "selectorIs",
                    vec![RewriteTermV0::apply(
                        "selectorList2",
                        vec![RewriteTermV0::atom(".a"), RewriteTermV0::atom(".a")],
                    )],
                ),
            ],
        );
        let after = RewriteTermV0::apply(
            "selectorConcat",
            vec![RewriteTermV0::atom(".root"), RewriteTermV0::atom(".a")],
        );
        (before, after)
    }

    fn substitution(variable: &str, value: &str) -> RewriteSubstitutionEntryV0 {
        RewriteSubstitutionEntryV0 {
            variable: variable.to_owned(),
            term: RewriteTermV0::atom(value),
        }
    }

    fn selector_certificate() -> RewriteCertificateEnvelopeV0 {
        let deduplicate = RewriteCertificateV0::Rewrite {
            rule_id: "selector-list-deduplicate-v0".to_owned(),
            substitution: vec![substitution("x", ".a")],
            side_condition: SideConditionCertV0::NoSideCondition,
        };
        let inside_is = RewriteCertificateV0::Trans {
            left: Box::new(RewriteCertificateV0::Cong {
                operator: "selectorIs".to_owned(),
                certificates: vec![deduplicate],
            }),
            right: Box::new(RewriteCertificateV0::Rewrite {
                rule_id: "selector-is-single-v0".to_owned(),
                substitution: vec![substitution("x", ".a")],
                side_condition: SideConditionCertV0::NoSideCondition,
            }),
        };
        RewriteCertificateEnvelopeV0 {
            schema_version: REWRITE_CERTIFICATE_SCHEMA_VERSION_V0.to_owned(),
            max_depth: 5,
            max_nodes: 8,
            certificate: RewriteCertificateV0::Cong {
                operator: "selectorConcat".to_owned(),
                certificates: vec![
                    RewriteCertificateV0::Refl {
                        term: RewriteTermV0::atom(".root"),
                    },
                    inside_is,
                ],
            },
        }
    }

    fn cascade_winner_key() -> CascadeWinnerKeyCertV0 {
        CascadeWinnerKeyCertV0 {
            level: CascadeLevelCertV0::AuthorNormal,
            layer_important: false,
            layer_ordinal: Some(0),
            scope_proximity: 0,
            specificity_ids: 0,
            specificity_classes: 2,
            specificity_elements: 0,
            source_order: 3,
        }
    }

    fn cascade_winner_certificate() -> CascadeWinnerEqualityCertV0 {
        CascadeWinnerEqualityCertV0 {
            before_winner_id: "declaration-color".to_owned(),
            after_winner_id: "declaration-color".to_owned(),
            before_key: cascade_winner_key(),
            after_key: cascade_winner_key(),
            before_class_attribute: "root a".to_owned(),
            after_class_attribute: "root a".to_owned(),
        }
    }

    fn selector_certificate_with_cascade_side_condition() -> RewriteCertificateEnvelopeV0 {
        let mut envelope = selector_certificate();
        replace_side_condition(
            &mut envelope.certificate,
            &SideConditionCertV0::CascadeWinnerEquality {
                certificate: cascade_winner_certificate(),
            },
        );
        envelope
    }

    fn replace_side_condition(
        certificate: &mut RewriteCertificateV0,
        side_condition: &SideConditionCertV0,
    ) {
        match certificate {
            RewriteCertificateV0::Refl { .. } => {}
            RewriteCertificateV0::Sym { certificate } => {
                replace_side_condition(certificate, side_condition);
            }
            RewriteCertificateV0::Trans { left, right } => {
                replace_side_condition(left, side_condition);
                replace_side_condition(right, side_condition);
            }
            RewriteCertificateV0::Cong { certificates, .. } => {
                for child in certificates {
                    replace_side_condition(child, side_condition);
                }
            }
            RewriteCertificateV0::Rewrite {
                side_condition: observed,
                ..
            } => *observed = side_condition.clone(),
        }
    }

    fn mutate_cascade_specificity(certificate: &mut RewriteCertificateV0) {
        match certificate {
            RewriteCertificateV0::Refl { .. } => {}
            RewriteCertificateV0::Sym { certificate } => {
                mutate_cascade_specificity(certificate);
            }
            RewriteCertificateV0::Trans { left, right } => {
                mutate_cascade_specificity(left);
                mutate_cascade_specificity(right);
            }
            RewriteCertificateV0::Cong { certificates, .. } => {
                for child in certificates {
                    mutate_cascade_specificity(child);
                }
            }
            RewriteCertificateV0::Rewrite {
                side_condition: SideConditionCertV0::CascadeWinnerEquality { certificate },
                ..
            } => {
                certificate.after_key.specificity_classes =
                    certificate.after_key.specificity_classes.saturating_add(1);
            }
            RewriteCertificateV0::Rewrite { .. } => {}
        }
    }

    fn single_rule_catalog(
        rule_id: &str,
        before: &str,
        after: &str,
        side_condition_kind: RewriteSideConditionKindV0,
    ) -> RewriteRuleCatalogV0 {
        RewriteRuleCatalogV0 {
            schema_version: REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0.to_owned(),
            operators: Vec::new(),
            rules: vec![RewriteRuleV0 {
                rule_id: rule_id.to_owned(),
                before_pattern: RewritePatternV0::atom(before),
                after_pattern: RewritePatternV0::atom(after),
                side_condition_kind,
            }],
        }
    }

    fn single_rule_certificate(
        rule_id: &str,
        side_condition: SideConditionCertV0,
    ) -> RewriteCertificateEnvelopeV0 {
        RewriteCertificateEnvelopeV0 {
            schema_version: REWRITE_CERTIFICATE_SCHEMA_VERSION_V0.to_owned(),
            max_depth: 1,
            max_nodes: 1,
            certificate: RewriteCertificateV0::Rewrite {
                rule_id: rule_id.to_owned(),
                substitution: Vec::new(),
                side_condition,
            },
        }
    }

    fn module_export_observation(
        module: &str,
        authored_name: &str,
        emitted_token: &str,
    ) -> ModuleExportObservationV0 {
        ModuleExportObservationV0 {
            key: ModuleExportKeyV0 {
                module_instance: ModuleInstanceKeyV0::unconfigured(omena_parser::ModuleIdV0::new(
                    module,
                )),
                canonical_class_name: ClassNameV0::new(authored_name)
                    .canonical_key()
                    .as_str()
                    .to_owned(),
            },
            emitted_token: emitted_token.to_owned(),
        }
    }

    fn module_export_certificate(
        pass_id: &str,
        before_exports: Vec<ModuleExportObservationV0>,
        after_exports: Vec<ModuleExportObservationV0>,
        declared_rename_delta: Vec<ModuleExportRenameDeltaV0>,
    ) -> RewriteCertificateEnvelopeV0 {
        let rule_id = module_export_preservation_rule_id_v0(pass_id)
            .unwrap_or("unknown-module-export-preservation-rule-v0");
        single_rule_certificate(
            rule_id,
            SideConditionCertV0::ModuleExportPreservation {
                certificate: ModuleExportPreservationCertV0::new(
                    pass_id,
                    before_exports,
                    after_exports,
                    declared_rename_delta,
                ),
            },
        )
    }

    fn literal(value: &str) -> ComputedValueTermV0 {
        ComputedValueTermV0::Literal {
            value: value.to_owned(),
        }
    }

    fn computed_value_certificate(after_value: &str) -> ComputedValueEqualityCertV0 {
        ComputedValueEqualityCertV0 {
            property: "--space".to_owned(),
            before_environment: vec![
                ComputedValueEnvironmentEntryV0 {
                    name: "--base".to_owned(),
                    value: literal("8px"),
                },
                ComputedValueEnvironmentEntryV0 {
                    name: "--space".to_owned(),
                    value: ComputedValueTermV0::Variable {
                        name: "--base".to_owned(),
                        fallback: None,
                    },
                },
            ],
            after_environment: vec![
                ComputedValueEnvironmentEntryV0 {
                    name: "--base".to_owned(),
                    value: literal("8px"),
                },
                ComputedValueEnvironmentEntryV0 {
                    name: "--space".to_owned(),
                    value: literal(after_value),
                },
            ],
        }
    }

    fn source_map_certificate() -> SourceMapTraceCertV0 {
        SourceMapTraceCertV0 {
            before_segments: vec![SourceMapTraceSegmentV0 {
                source_path: "fixture/selector.module.css".to_owned(),
                source_digest: "c6f1172b".to_owned(),
                original_start: 0,
                original_end: 15,
                generated_start: 0,
                generated_end: 15,
                pass_id: "selector-is-where-compression".to_owned(),
            }],
            after_segments: vec![SourceMapTraceSegmentV0 {
                source_path: "fixture/selector.module.css".to_owned(),
                source_digest: "c6f1172b".to_owned(),
                original_start: 0,
                original_end: 15,
                generated_start: 0,
                generated_end: 10,
                pass_id: "selector-is-where-compression".to_owned(),
            }],
        }
    }

    fn check_selector(
        catalog: &RewriteRuleCatalogV0,
        certificate: &RewriteCertificateEnvelopeV0,
    ) -> Result<RewriteIssuanceTokenV0, CertificateRejectionV0> {
        let (before, after) = selector_terms();
        check_rewrite_certificate_v0(
            &before,
            &after,
            catalog,
            certificate,
            &CanonicalRewriteAssumptionsV0::default(),
        )
    }

    #[test]
    fn real_selector_rewrite_accepts_cascade_winner_equality_certificate() {
        let result = check_selector(
            &selector_rewrite_rule_catalog_with_cascade_winner_equality_v0(),
            &selector_certificate_with_cascade_side_condition(),
        );
        assert!(result.is_ok(), "cascade cert rejected: {result:?}");
        let Ok(token) = result else {
            return;
        };
        println!(
            "cascadeWinnerEquality=accepted checkedRules={:?}",
            token.checked_rule_ids_v0()
        );
    }

    #[test]
    fn specificity_perturbation_rejects_cascade_cert_without_producer_boolean_input() {
        let catalog = selector_rewrite_rule_catalog_with_cascade_winner_equality_v0();
        let mut certificate = selector_certificate_with_cascade_side_condition();
        mutate_cascade_specificity(&mut certificate.certificate);
        let producer_specificity_preserved_values = [true, false];
        let mut rejections = Vec::new();
        for producer_specificity_preserved in producer_specificity_preserved_values {
            let result = check_selector(&catalog, &certificate);
            assert!(
                result.is_err(),
                "specificity perturbation accepted with producer={producer_specificity_preserved}"
            );
            let Err(rejection) = result else {
                continue;
            };
            rejections.push((*rejection.rejection).clone());
        }
        assert_eq!(rejections.len(), 2);
        assert_eq!(rejections[0], rejections[1]);
        assert!(matches!(
            rejections[0],
            CertificateRejectionKindV0::CascadeWinnerEqualityRejected {
                cascade_keys_equal: false,
                ..
            }
        ));
        println!(
            "specificityPerturbation={:?} producerFieldTrueFalseSame={}",
            rejections[0],
            rejections[0] == rejections[1]
        );
    }

    #[test]
    fn custom_property_rewrite_accepts_and_rejects_from_fixed_point_values() {
        let catalog = single_rule_catalog(
            "custom-property-inline-v0",
            "var(--base)",
            "8px",
            RewriteSideConditionKindV0::ComputedValueEquality,
        );
        let before = RewriteTermV0::atom("var(--base)");
        let after = RewriteTermV0::atom("8px");
        let accepted = single_rule_certificate(
            "custom-property-inline-v0",
            SideConditionCertV0::ComputedValueEquality {
                certificate: computed_value_certificate("8px"),
            },
        );
        let accepted_result = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &accepted,
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(
            accepted_result.is_ok(),
            "computed-value cert rejected: {accepted_result:?}"
        );

        let rejected = single_rule_certificate(
            "custom-property-inline-v0",
            SideConditionCertV0::ComputedValueEquality {
                certificate: computed_value_certificate("9px"),
            },
        );
        let rejected_result = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &rejected,
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(
            rejected_result.is_err(),
            "computed-value perturbation accepted"
        );
        let Err(rejection) = rejected_result else {
            return;
        };
        assert!(matches!(
            *rejection.rejection,
            CertificateRejectionKindV0::ComputedValueEqualityRejected { .. }
        ));
        println!(
            "computedValue accepted=true perturbedRejection={:?}",
            rejection.rejection
        );
    }

    #[test]
    fn selector_rewrite_accepts_and_rejects_from_emitted_source_map_segments() {
        let catalog = single_rule_catalog(
            "selector-trace-v0",
            ".root:is(.a)",
            ".root.a",
            RewriteSideConditionKindV0::SourceMapTrace,
        );
        let before = RewriteTermV0::atom(".root:is(.a)");
        let after = RewriteTermV0::atom(".root.a");
        let accepted = single_rule_certificate(
            "selector-trace-v0",
            SideConditionCertV0::SourceMapTrace {
                certificate: source_map_certificate(),
            },
        );
        let accepted_result = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &accepted,
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(
            accepted_result.is_ok(),
            "source-map cert rejected: {accepted_result:?}"
        );

        let mut perturbed_trace = source_map_certificate();
        perturbed_trace.after_segments[0].original_start = 1;
        let rejected = single_rule_certificate(
            "selector-trace-v0",
            SideConditionCertV0::SourceMapTrace {
                certificate: perturbed_trace,
            },
        );
        let rejected_result = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &rejected,
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(rejected_result.is_err(), "source-map perturbation accepted");
        let Err(rejection) = rejected_result else {
            return;
        };
        assert!(matches!(
            *rejection.rejection,
            CertificateRejectionKindV0::SourceMapTraceRejected {
                segment_index: 0,
                ..
            }
        ));
        println!(
            "sourceMapTrace accepted=true perturbedRejection={:?}",
            rejection.rejection
        );
    }

    #[test]
    fn side_condition_certificates_round_trip_through_serde() -> Result<(), serde_json::Error> {
        let certificates = [
            SideConditionCertV0::CascadeWinnerEquality {
                certificate: cascade_winner_certificate(),
            },
            SideConditionCertV0::ComputedValueEquality {
                certificate: computed_value_certificate("8px"),
            },
            SideConditionCertV0::SourceMapTrace {
                certificate: source_map_certificate(),
            },
            SideConditionCertV0::TokenOwnershipSeparability {
                certificate: TokenOwnershipSeparabilityCertV0 {
                    complete: true,
                    modeled_preimage_count: 1,
                    emitted_token_count: 1,
                    ownerships: vec![TokenOwnershipCertEntryV0 {
                        emitted_token: "_shared_0".to_owned(),
                        module_paths: vec!["src/one.module.css".to_owned()],
                    }],
                    unattributed_emitted_token_count: 0,
                    interface_mismatch_count: 0,
                },
            },
            SideConditionCertV0::ModuleExportPreservation {
                certificate: ModuleExportPreservationCertV0::new(
                    "selector-merging",
                    vec![module_export_observation(
                        "src/card.module.css",
                        "root",
                        "root",
                    )],
                    vec![module_export_observation(
                        "src/card.module.css",
                        "root",
                        "root",
                    )],
                    Vec::new(),
                ),
            },
            SideConditionCertV0::TransformIndependence {
                certificate: Box::new(TransformIndependenceCertV0 {
                    left_pass_id: "number-compression".to_owned(),
                    right_pass_id: "color-compression".to_owned(),
                    observation_profile_id: "exact-emission-bytes-v0".to_owned(),
                    profile_observers: vec!["rawBytes".to_owned()],
                    observation_rows: vec![TransformIndependenceObservationCertRowV0 {
                        fixture_id: "disjoint-values".to_owned(),
                        observer: "rawBytes".to_owned(),
                        left_then_right: ".a{color:red;margin:.5px}".to_owned(),
                        right_then_left: ".a{color:red;margin:.5px}".to_owned(),
                    }],
                    left_preconditions: vec!["equivalentLiteralValue".to_owned()],
                    right_preconditions: vec!["equivalentLiteralValue".to_owned()],
                    left_preserves_right_preconditions: vec!["equivalentLiteralValue".to_owned()],
                    right_preserves_left_preconditions: vec!["equivalentLiteralValue".to_owned()],
                    disqualifying_descriptor_edges: Vec::new(),
                }),
            },
        ];
        for certificate in certificates {
            let encoded = serde_json::to_string(&certificate)?;
            let decoded = serde_json::from_str::<SideConditionCertV0>(&encoded)?;
            assert_eq!(decoded, certificate);
        }
        Ok(())
    }

    #[test]
    fn module_export_preservation_rejects_drop_identity_swap_and_digest_tamper() {
        let pass_id = "selector-merging";
        let rule_id = "module-export-preservation-selector-merging-v0";
        let catalog = single_rule_catalog(
            rule_id,
            "preservationRequested",
            "preservationGranted",
            RewriteSideConditionKindV0::ModuleExportPreservation,
        );
        let before_term = RewriteTermV0::atom("preservationRequested");
        let after_term = RewriteTermV0::atom("preservationGranted");
        let before = vec![module_export_observation(
            "src/card.module.css",
            "root",
            "root",
        )];
        let after = before.clone();
        let check = |certificate: RewriteCertificateEnvelopeV0| {
            check_rewrite_certificate_v0(
                &before_term,
                &after_term,
                &catalog,
                &certificate,
                &CanonicalRewriteAssumptionsV0::default(),
            )
        };

        let accepted = check(module_export_certificate(
            pass_id,
            before.clone(),
            after.clone(),
            Vec::new(),
        ));
        assert!(
            accepted.is_ok(),
            "valid preservation rejected: {accepted:?}"
        );

        let noncanonical_identity = ModuleExportObservationV0 {
            key: ModuleExportKeyV0 {
                module_instance: ModuleInstanceKeyV0::unconfigured(omena_parser::ModuleIdV0::new(
                    "src/card.module.css",
                )),
                canonical_class_name: r"\72 oot".to_owned(),
            },
            emitted_token: "root".to_owned(),
        };
        let noncanonical = check(module_export_certificate(
            pass_id,
            vec![noncanonical_identity.clone()],
            vec![noncanonical_identity],
            Vec::new(),
        ));
        assert!(matches!(
            noncanonical,
            Err(CertificateRejectionV0 { rejection, .. }) if matches!(
                *rejection,
                CertificateRejectionKindV0::ModuleExportPreservationRejected { ref reason, .. }
                    if reason.contains("non-canonical identity key")
            )
        ));

        let dropped = check(module_export_certificate(
            pass_id,
            before.clone(),
            Vec::new(),
            Vec::new(),
        ));
        assert!(matches!(
            dropped,
            Err(CertificateRejectionV0 { rejection, .. }) if matches!(
                *rejection,
                CertificateRejectionKindV0::ModuleExportPreservationRejected { ref reason, .. }
                    if reason.contains("dropped an identity key")
            )
        ));

        let swapped = check(module_export_certificate(
            pass_id,
            before.clone(),
            vec![module_export_observation(
                "src/other.module.css",
                "root",
                "root",
            )],
            Vec::new(),
        ));
        assert!(matches!(
            swapped,
            Err(CertificateRejectionV0 { rejection, .. }) if matches!(
                *rejection,
                CertificateRejectionKindV0::ModuleExportPreservationRejected { ref reason, .. }
                    if reason.contains("dropped an identity key")
            )
        ));

        let mut tampered = module_export_certificate(pass_id, before, after, Vec::new());
        let shape_mutated = if let RewriteCertificateV0::Rewrite {
            side_condition: SideConditionCertV0::ModuleExportPreservation { certificate },
            ..
        } = &mut tampered.certificate
        {
            certificate.after_premise_digest = "00".repeat(32);
            true
        } else {
            false
        };
        assert!(
            shape_mutated,
            "single-rule preservation certificate shape changed"
        );
        let tampered_result = check(tampered);
        assert!(matches!(
            tampered_result,
            Err(CertificateRejectionV0 { rejection, .. }) if matches!(
                *rejection,
                CertificateRejectionKindV0::ModuleExportPreservationRejected { ref reason, .. }
                    if reason.contains("digest")
            )
        ));
    }

    #[test]
    fn module_export_hashing_accepts_only_a_bijective_declared_rename_delta() {
        let pass_id = "css-modules-class-hashing";
        let rule_id = "module-export-preservation-css-modules-class-hashing-v0";
        let catalog = single_rule_catalog(
            rule_id,
            "preservationRequested",
            "preservationGranted",
            RewriteSideConditionKindV0::ModuleExportPreservation,
        );
        let before_term = RewriteTermV0::atom("preservationRequested");
        let after_term = RewriteTermV0::atom("preservationGranted");
        let before_root = module_export_observation("src/card.module.css", "root", "root");
        let before_title = module_export_observation("src/card.module.css", "title", "title");
        let after_root = module_export_observation("src/card.module.css", "root", "_root_a1");
        let after_title = module_export_observation("src/card.module.css", "title", "_title_b2");
        let deltas = vec![
            ModuleExportRenameDeltaV0 {
                key: before_root.key.clone(),
                before_token: before_root.emitted_token.clone(),
                after_token: after_root.emitted_token.clone(),
            },
            ModuleExportRenameDeltaV0 {
                key: before_title.key.clone(),
                before_token: before_title.emitted_token.clone(),
                after_token: after_title.emitted_token.clone(),
            },
        ];
        let check = |certificate: RewriteCertificateEnvelopeV0| {
            check_rewrite_certificate_v0(
                &before_term,
                &after_term,
                &catalog,
                &certificate,
                &CanonicalRewriteAssumptionsV0::default(),
            )
        };
        let accepted = check(module_export_certificate(
            pass_id,
            vec![before_root.clone(), before_title.clone()],
            vec![after_root.clone(), after_title.clone()],
            deltas.clone(),
        ));
        assert!(accepted.is_ok(), "bijective rename rejected: {accepted:?}");

        let canonical_before_root =
            module_export_observation("src/card.module.css", "root", "button");
        let canonical_before_title =
            module_export_observation("src/card.module.css", "title", "title");
        let canonical_after_root = canonical_before_root.clone();
        let canonical_after_title =
            module_export_observation("src/card.module.css", "title", r"\62 utton");
        let canonical_collision = check(module_export_certificate(
            pass_id,
            vec![canonical_before_root, canonical_before_title.clone()],
            vec![canonical_after_root, canonical_after_title.clone()],
            vec![ModuleExportRenameDeltaV0 {
                key: canonical_before_title.key,
                before_token: canonical_before_title.emitted_token,
                after_token: canonical_after_title.emitted_token,
            }],
        ));
        assert!(matches!(
            canonical_collision,
            Err(CertificateRejectionV0 { rejection, .. }) if matches!(
                *rejection,
                CertificateRejectionKindV0::ModuleExportPreservationRejected { ref reason, .. }
                    if reason.contains("collapse under canonical")
            )
        ));

        let mut colliding_deltas = deltas;
        colliding_deltas[1].after_token = after_root.emitted_token.clone();
        let rejected = check(module_export_certificate(
            pass_id,
            vec![before_root, before_title],
            vec![after_root, after_title],
            colliding_deltas,
        ));
        assert!(matches!(
            rejected,
            Err(CertificateRejectionV0 { rejection, .. }) if matches!(
                *rejection,
                CertificateRejectionKindV0::ModuleExportPreservationRejected { ref reason, .. }
                    if reason.contains("bijection")
            )
        ));
    }

    #[test]
    fn token_ownership_side_condition_requires_one_owner_per_emitted_token() {
        let catalog = single_rule_catalog(
            "closed-world-ownership-admission-v0",
            "admissionRequested",
            "admissionGranted",
            RewriteSideConditionKindV0::TokenOwnershipSeparability,
        );
        let before = RewriteTermV0::atom("admissionRequested");
        let after = RewriteTermV0::atom("admissionGranted");
        let certificate = |module_paths: Vec<String>, modeled_preimage_count| {
            single_rule_certificate(
                "closed-world-ownership-admission-v0",
                SideConditionCertV0::TokenOwnershipSeparability {
                    certificate: TokenOwnershipSeparabilityCertV0 {
                        complete: true,
                        modeled_preimage_count,
                        emitted_token_count: 1,
                        ownerships: vec![TokenOwnershipCertEntryV0 {
                            emitted_token: "_shared_0".to_owned(),
                            module_paths,
                        }],
                        unattributed_emitted_token_count: 0,
                        interface_mismatch_count: 0,
                    },
                },
            )
        };
        let accepted = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &certificate(vec!["src/one.module.css".to_owned()], 1),
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(accepted.is_ok(), "unique ownership rejected: {accepted:?}");

        let rejected = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &certificate(
                vec![
                    "src/one.module.css".to_owned(),
                    "src/two.module.css".to_owned(),
                ],
                2,
            ),
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(rejected.is_err(), "ambiguous ownership accepted");
        let Err(rejection) = rejected else {
            return;
        };
        assert!(matches!(
            *rejection.rejection,
            CertificateRejectionKindV0::TokenOwnershipSeparabilityRejected {
                token: Some(ref token),
                ..
            } if token == "_shared_0"
        ));
    }

    #[test]
    fn transform_independence_requires_observation_and_precondition_halves() {
        let catalog = single_rule_catalog(
            "adjacent-schedule-swap-v0",
            "numberThenColor",
            "colorThenNumber",
            RewriteSideConditionKindV0::TransformIndependence,
        );
        let before = RewriteTermV0::atom("numberThenColor");
        let after = RewriteTermV0::atom("colorThenNumber");
        let independence = TransformIndependenceCertV0 {
            left_pass_id: "number-compression".to_owned(),
            right_pass_id: "color-compression".to_owned(),
            observation_profile_id: "exact-emission-bytes-v0".to_owned(),
            profile_observers: vec!["rawBytes".to_owned()],
            observation_rows: vec![TransformIndependenceObservationCertRowV0 {
                fixture_id: "disjoint-values".to_owned(),
                observer: "rawBytes".to_owned(),
                left_then_right: ".a{color:red;margin:.5px}".to_owned(),
                right_then_left: ".a{color:red;margin:.5px}".to_owned(),
            }],
            left_preconditions: vec!["equivalentLiteralValue".to_owned()],
            right_preconditions: vec!["equivalentLiteralValue".to_owned()],
            left_preserves_right_preconditions: vec!["equivalentLiteralValue".to_owned()],
            right_preserves_left_preconditions: vec!["equivalentLiteralValue".to_owned()],
            disqualifying_descriptor_edges: Vec::new(),
        };
        let envelope = |certificate| {
            single_rule_certificate(
                "adjacent-schedule-swap-v0",
                SideConditionCertV0::TransformIndependence {
                    certificate: Box::new(certificate),
                },
            )
        };
        let accepted = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &envelope(independence.clone()),
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(accepted.is_ok(), "independence cert rejected: {accepted:?}");

        let mut dependent = independence;
        dependent
            .disqualifying_descriptor_edges
            .push("conflictsWith:color-mix-lowering:color-function-lowering".to_owned());
        let rejected = check_rewrite_certificate_v0(
            &before,
            &after,
            &catalog,
            &envelope(dependent),
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(matches!(
            rejected,
            Err(CertificateRejectionV0 {
                rejection,
                ..
            }) if matches!(
                *rejection,
                CertificateRejectionKindV0::TransformIndependenceRejected { .. }
            )
        ));
    }

    #[test]
    fn real_selector_trans_cong_rewrite_chain_issues_token() {
        let catalog = selector_rewrite_rule_catalog_v0();
        let result = check_selector(&catalog, &selector_certificate());
        assert!(result.is_ok(), "selector certificate rejected: {result:?}");
        let Ok(token) = result else {
            return;
        };

        assert_eq!(
            token.checked_rule_ids_v0(),
            ["selector-list-deduplicate-v0", "selector-is-single-v0"]
        );
        assert_ne!(token.before_digest_hex_v0(), token.after_digest_hex_v0());
        assert_eq!(
            token.catalog_schema_id_v0(),
            REWRITE_RULE_CATALOG_SCHEMA_ID_V0
        );
        assert!(token.matches_catalog_v0(&catalog));
        println!(
            "issued=true beforeDigest={} afterDigest={} catalogSchemaId={} catalogDigest={} checkedRules={:?}",
            token.before_digest_hex_v0(),
            token.after_digest_hex_v0(),
            token.catalog_schema_id_v0(),
            token.catalog_content_digest_hex_v0(),
            token.checked_rule_ids_v0()
        );
    }

    #[test]
    fn catalog_digest_is_order_independent_and_content_sensitive() {
        let catalog = selector_rewrite_rule_catalog_v0();
        let mut permuted = catalog.clone();
        permuted.operators.reverse();
        permuted.rules.reverse();
        assert_eq!(
            rewrite_rule_catalog_content_digest_hex_v0(&catalog),
            rewrite_rule_catalog_content_digest_hex_v0(&permuted)
        );

        let mut spoofed = catalog.clone();
        spoofed.rules[1].before_pattern = RewritePatternV0::variable("anything");
        spoofed.rules[1].after_pattern = RewritePatternV0::variable("whatever");
        assert_ne!(
            rewrite_rule_catalog_content_digest_hex_v0(&catalog),
            rewrite_rule_catalog_content_digest_hex_v0(&spoofed)
        );
    }

    #[test]
    fn one_token_substitution_mutation_names_the_after_subterm() {
        let mut certificate = selector_certificate();
        let RewriteCertificateV0::Cong { certificates, .. } = &mut certificate.certificate else {
            unreachable!("fixture root is congruence")
        };
        let RewriteCertificateV0::Trans { right, .. } = &mut certificates[1] else {
            unreachable!("fixture inner node is transitivity")
        };
        let RewriteCertificateV0::Rewrite { substitution, .. } = right.as_mut() else {
            unreachable!("fixture right node is rewrite")
        };
        substitution[0].term = RewriteTermV0::atom(".b");

        let result = check_selector(&selector_rewrite_rule_catalog_v0(), &certificate);
        assert!(result.is_err(), "one-token substitution mutation accepted");
        let Err(rejection) = result else {
            return;
        };
        assert_eq!(
            *rejection.rejection,
            CertificateRejectionKindV0::TransitiveMiddleMismatch
        );
        assert_eq!(rejection.site.certificate_path, vec![1]);
        assert_eq!(rejection.site.term_path, vec![0]);
        println!(
            "rejection={:?} certificatePath={:?} termPath={:?}",
            rejection.rejection, rejection.site.certificate_path, rejection.site.term_path
        );
    }

    #[test]
    fn favourable_producer_fields_cannot_replace_a_missing_certificate() {
        let producer_specificity_preserved = true;
        let producer_computed_value_preserved = true;
        let producer_provenance_preserved = true;
        assert!(
            producer_specificity_preserved
                && producer_computed_value_preserved
                && producer_provenance_preserved
        );
        let (before, after) = selector_terms();
        let result = check_optional_rewrite_certificate_v0(
            &before,
            &after,
            &selector_rewrite_rule_catalog_v0(),
            None,
            &CanonicalRewriteAssumptionsV0::default(),
        );
        assert!(result.is_err(), "missing certificate minted a token");
        let Err(rejection) = result else {
            return;
        };
        assert_eq!(
            *rejection.rejection,
            CertificateRejectionKindV0::MissingCertificate
        );
        println!(
            "producerFields=true,true,true rejection={:?}",
            rejection.rejection
        );
    }

    #[test]
    fn adversarial_corpus_returns_six_typed_rejections_without_panicking() {
        let catalog = selector_rewrite_rule_catalog_v0();
        let (before, after) = selector_terms();

        let mut unknown_rule = selector_certificate();
        let RewriteCertificateV0::Cong { certificates, .. } = &mut unknown_rule.certificate else {
            unreachable!("fixture root is congruence")
        };
        let RewriteCertificateV0::Trans { right, .. } = &mut certificates[1] else {
            unreachable!("fixture inner node is transitivity")
        };
        let RewriteCertificateV0::Rewrite { rule_id, .. } = right.as_mut() else {
            unreachable!("fixture right node is rewrite")
        };
        *rule_id = "unknown-rule-v0".to_owned();

        let mut arity = selector_certificate();
        let RewriteCertificateV0::Cong { certificates, .. } = &mut arity.certificate else {
            unreachable!("fixture root is congruence")
        };
        certificates.pop();

        let missing_substitution = RewriteCertificateEnvelopeV0 {
            schema_version: REWRITE_CERTIFICATE_SCHEMA_VERSION_V0.to_owned(),
            max_depth: 1,
            max_nodes: 1,
            certificate: RewriteCertificateV0::Rewrite {
                rule_id: "selector-is-single-v0".to_owned(),
                substitution: Vec::new(),
                side_condition: SideConditionCertV0::NoSideCondition,
            },
        };

        let mut depth = selector_certificate();
        depth.max_depth = 2;

        let trans_mismatch = RewriteCertificateEnvelopeV0 {
            schema_version: REWRITE_CERTIFICATE_SCHEMA_VERSION_V0.to_owned(),
            max_depth: 2,
            max_nodes: 3,
            certificate: RewriteCertificateV0::Trans {
                left: Box::new(RewriteCertificateV0::Refl {
                    term: RewriteTermV0::atom(".a"),
                }),
                right: Box::new(RewriteCertificateV0::Refl {
                    term: RewriteTermV0::atom(".b"),
                }),
            },
        };

        let typed_cases = [
            ("unknownRule", unknown_rule, "unknownRule"),
            ("congruenceArity", arity, "operatorArityMismatch"),
            (
                "missingSubstitution",
                missing_substitution,
                "missingSubstitutionVariable",
            ),
            ("declaredDepth", depth, "declaredBoundExceeded"),
            (
                "transitiveMiddle",
                trans_mismatch,
                "transitiveMiddleMismatch",
            ),
        ];
        let mut observed = Vec::new();
        for (name, certificate, expected_kind) in typed_cases {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                check_rewrite_certificate_v0(
                    &before,
                    &after,
                    &catalog,
                    &certificate,
                    &CanonicalRewriteAssumptionsV0::default(),
                )
            }));
            assert!(outcome.is_ok(), "{name} panicked");
            let Some(result) = outcome.ok() else {
                continue;
            };
            assert!(result.is_err(), "{name} did not produce a typed rejection");
            let Err(rejection) = result else {
                continue;
            };
            let encoded = serde_json::to_value(&rejection);
            assert!(encoded.is_ok(), "{name} rejection did not serialize");
            let Ok(value) = encoded else {
                continue;
            };
            assert_eq!(value["rejection"]["kind"], expected_kind);
            assert_eq!(value["site"]["input"], "certificate");
            println!(
                "case={name} rejection={} site={}",
                value["rejection"], value["site"]
            );
            observed.push(name);
        }

        let malformed = catch_unwind(AssertUnwindSafe(|| {
            check_serialized_rewrite_certificate_v0(
                &before,
                &after,
                &catalog,
                r#"{"schemaVersion":"0","certificate":{"kind":"trans""#,
                &CanonicalRewriteAssumptionsV0::default(),
            )
        }));
        assert!(malformed.is_ok(), "malformed serde input panicked");
        let Some(result) = malformed.ok() else {
            return;
        };
        assert!(result.is_err(), "malformed serde input was not rejected");
        let Err(rejection) = result else {
            return;
        };
        assert!(matches!(
            *rejection.rejection,
            CertificateRejectionKindV0::MalformedCertificate { .. }
        ));
        assert_eq!(
            rejection.site.input,
            RewriteCheckInputV0::SerializedCertificate
        );
        println!(
            "case=malformedSerde rejection={:?} site={:?}",
            rejection.rejection, rejection.site
        );
        observed.push("malformedSerde");
        assert_eq!(observed.len(), 6);
    }

    #[test]
    fn fixed_seed_input_order_permutations_issue_identical_tokens() {
        let baseline_catalog = selector_rewrite_rule_catalog_v0();
        let baseline_certificate = selector_certificate();
        let baseline_result = check_selector(&baseline_catalog, &baseline_certificate);
        assert!(
            baseline_result.is_ok(),
            "baseline rejected: {baseline_result:?}"
        );
        let Ok(baseline) = baseline_result else {
            return;
        };
        let mut state = 0x6a09_e667_f3bc_c909_u64;

        for _ in 0..32 {
            let mut catalog = selector_rewrite_rule_catalog_v0();
            seeded_shuffle(&mut catalog.operators, &mut state);
            seeded_shuffle(&mut catalog.rules, &mut state);
            let mut certificate = selector_certificate();
            reverse_substitution_order(&mut certificate.certificate);
            let observed_result = check_selector(&catalog, &certificate);
            assert!(
                observed_result.is_ok(),
                "permutation rejected: {observed_result:?}"
            );
            let Ok(observed) = observed_result else {
                continue;
            };
            assert_eq!(observed, baseline);
        }
    }

    fn reverse_substitution_order(certificate: &mut RewriteCertificateV0) {
        match certificate {
            RewriteCertificateV0::Refl { .. } => {}
            RewriteCertificateV0::Sym { certificate } => {
                reverse_substitution_order(certificate);
            }
            RewriteCertificateV0::Trans { left, right } => {
                reverse_substitution_order(left);
                reverse_substitution_order(right);
            }
            RewriteCertificateV0::Cong { certificates, .. } => {
                for child in certificates {
                    reverse_substitution_order(child);
                }
            }
            RewriteCertificateV0::Rewrite { substitution, .. } => substitution.reverse(),
        }
    }

    fn seeded_shuffle<T>(values: &mut [T], state: &mut u64) {
        for index in (1..values.len()).rev() {
            *state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let target = (*state as usize) % (index + 1);
            values.swap(index, target);
        }
    }

    #[test]
    fn grammar_round_trips_through_serde() -> Result<(), serde_json::Error> {
        let certificate = selector_certificate();
        let encoded = serde_json::to_string(&certificate)?;
        let decoded = serde_json::from_str::<RewriteCertificateEnvelopeV0>(&encoded)?;
        assert_eq!(decoded, certificate);
        Ok(())
    }
}
