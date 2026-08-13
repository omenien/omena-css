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

use serde::{Deserialize, Serialize};

pub const REWRITE_CERTIFICATE_SCHEMA_VERSION_V0: &str = "0";
pub const REWRITE_RULE_CATALOG_SCHEMA_VERSION_V0: &str = "0";
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
pub enum RewriteSideConditionKindV0 {
    NoSideCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SideConditionCertV0 {
    NoSideCondition,
}

impl SideConditionCertV0 {
    fn kind(&self) -> RewriteSideConditionKindV0 {
        match self {
            Self::NoSideCondition => RewriteSideConditionKindV0::NoSideCondition,
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
    TransitiveMiddleMismatch,
    EndpointMismatch {
        endpoint: RewriteCheckInputV0,
    },
    MissingCertificate,
    MalformedCertificate {
        message: String,
    },
    DerivedTermLimitExceeded,
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
///     checked_rule_ids: Vec::new(),
///     _seal: loop {},
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteIssuanceTokenV0 {
    before_digest: [u8; 32],
    after_digest: [u8; 32],
    checked_rule_ids: Vec<String>,
    _seal: RewriteIssuanceSealV0,
}

impl RewriteIssuanceTokenV0 {
    fn issue(before: &RewriteTermV0, after: &RewriteTermV0, checked_rule_ids: Vec<String>) -> Self {
        Self {
            before_digest: term_digest_v0(before),
            after_digest: term_digest_v0(after),
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
    fn real_selector_trans_cong_rewrite_chain_issues_token() {
        let result = check_selector(&selector_rewrite_rule_catalog_v0(), &selector_certificate());
        assert!(result.is_ok(), "selector certificate rejected: {result:?}");
        let Ok(token) = result else {
            return;
        };

        assert_eq!(
            token.checked_rule_ids_v0(),
            ["selector-list-deduplicate-v0", "selector-is-single-v0"]
        );
        assert_ne!(token.before_digest_hex_v0(), token.after_digest_hex_v0());
        println!(
            "issued=true beforeDigest={} afterDigest={} checkedRules={:?}",
            token.before_digest_hex_v0(),
            token.after_digest_hex_v0(),
            token.checked_rule_ids_v0()
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
