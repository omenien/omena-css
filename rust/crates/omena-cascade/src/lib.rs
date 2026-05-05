//! Cascade-formal substrate for the Omena CSS track.
//!
//! The crate starts with the load-bearing algebra from the research plan:
//! lexicographic cascade keys, specificity, provenance proofs, and a finite
//! custom-property substitution function with explicit cycle handling.

use serde::Serialize;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, BTreeSet},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeLevel {
    UserAgentNormal,
    UserNormal,
    AuthorNormal,
    InlineNormal,
    Animation,
    AuthorImportant,
    UserImportant,
    UserAgentImportant,
    Transition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerRank(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Specificity {
    pub ids: u32,
    pub classes: u32,
    pub elements: u32,
}

impl Specificity {
    pub const ZERO: Self = Self {
        ids: 0,
        classes: 0,
        elements: 0,
    };

    pub const fn new(ids: u32, classes: u32, elements: u32) -> Self {
        Self {
            ids,
            classes,
            elements,
        }
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.ids, self.classes, self.elements).cmp(&(other.ids, other.classes, other.elements))
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeKey {
    pub level: CascadeLevel,
    pub layer_rank: LayerRank,
    pub scope_proximity: u32,
    pub specificity: Specificity,
    pub source_order: u32,
}

impl CascadeKey {
    pub const fn new(
        level: CascadeLevel,
        layer_rank: LayerRank,
        scope_proximity: u32,
        specificity: Specificity,
        source_order: u32,
    ) -> Self {
        Self {
            level,
            layer_rank,
            scope_proximity,
            specificity,
            source_order,
        }
    }
}

impl Ord for CascadeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.level
            .cmp(&other.level)
            .then_with(|| self.layer_rank.cmp(&other.layer_rank))
            .then_with(|| other.scope_proximity.cmp(&self.scope_proximity))
            .then_with(|| self.specificity.cmp(&other.specificity))
            .then_with(|| self.source_order.cmp(&other.source_order))
    }
}

impl PartialOrd for CascadeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeDeclaration {
    pub id: String,
    pub property: String,
    pub value: CascadeValue,
    pub key: CascadeKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeProof {
    pub declaration_id: String,
    pub property: String,
    pub level: CascadeLevel,
    pub layer_rank: LayerRank,
    pub scope_proximity: u32,
    pub specificity: Specificity,
    pub source_order: u32,
}

impl CascadeProof {
    pub fn from_declaration(declaration: &CascadeDeclaration) -> Self {
        Self {
            declaration_id: declaration.id.clone(),
            property: declaration.property.clone(),
            level: declaration.key.level,
            layer_rank: declaration.key.layer_rank,
            scope_proximity: declaration.key.scope_proximity,
            specificity: declaration.key.specificity,
            source_order: declaration.key.source_order,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeOutcome {
    Definite {
        winner: CascadeDeclaration,
        proof: CascadeProof,
        also_considered: Vec<CascadeDeclaration>,
    },
    RankedSet(Vec<CascadeDeclaration>),
    Inherit,
    Top,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CascadeValue {
    Literal(String),
    Var {
        name: String,
        fallback: Option<Box<CascadeValue>>,
    },
    GuaranteedInvalid,
    Unset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorContextMatchKind {
    NoMatch,
    Global,
    Root,
    Exact,
    ContainsSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorContextWitness {
    pub kind: SelectorContextMatchKind,
    pub matched: bool,
    pub rank: usize,
    pub declaration_selector: Option<String>,
    pub reference_selector: Option<String>,
}

impl SelectorContextWitness {
    pub fn no_match() -> Self {
        Self {
            kind: SelectorContextMatchKind::NoMatch,
            matched: false,
            rank: 0,
            declaration_selector: None,
            reference_selector: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementSignature {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: BTreeSet<String>,
    pub attributes: BTreeSet<String>,
    pub pseudo_states: BTreeSet<String>,
    pub classes_are_exact: bool,
    pub attributes_are_exact: bool,
    pub pseudo_states_are_exact: bool,
    pub tag_is_exact: bool,
    pub id_is_exact: bool,
}

impl ElementSignature {
    pub fn concrete(
        tag: Option<impl Into<String>>,
        id: Option<impl Into<String>>,
        classes: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            tag: tag.map(Into::into),
            id: id.map(Into::into),
            classes: classes.into_iter().map(Into::into).collect(),
            attributes: BTreeSet::new(),
            pseudo_states: BTreeSet::new(),
            classes_are_exact: true,
            attributes_are_exact: true,
            pseudo_states_are_exact: true,
            tag_is_exact: true,
            id_is_exact: true,
        }
    }

    pub fn at_least_classes(classes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            classes_are_exact: false,
            ..Self::concrete(None::<String>, None::<String>, classes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorSignature {
    pub selector: String,
    pub required_tag: Option<String>,
    pub required_id: Option<String>,
    pub required_classes: BTreeSet<String>,
    pub required_attributes: BTreeSet<String>,
    pub required_pseudo_states: BTreeSet<String>,
    pub specificity: Specificity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorMatchVerdict {
    No,
    Maybe,
    Yes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorMatchReason {
    Universal,
    SimpleCompound,
    SelectorList,
    MissingTag,
    MissingId,
    MissingClass,
    MissingAttribute,
    MissingPseudoState,
    UnsupportedSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorMatchWitness {
    pub selector: String,
    pub matched_branch: Option<String>,
    pub verdict: SelectorMatchVerdict,
    pub reason: SelectorMatchReason,
    pub specificity: Specificity,
    pub missing_tag: Option<String>,
    pub missing_id: Option<String>,
    pub missing_classes: BTreeSet<String>,
    pub missing_attributes: BTreeSet<String>,
    pub missing_pseudo_states: BTreeSet<String>,
    pub unsupported_branches: Vec<String>,
}

impl SelectorMatchWitness {
    fn unsupported(selector: &str) -> Self {
        Self {
            selector: selector.to_string(),
            matched_branch: Some(selector.to_string()),
            verdict: SelectorMatchVerdict::Maybe,
            reason: SelectorMatchReason::UnsupportedSelector,
            specificity: Specificity::ZERO,
            missing_tag: None,
            missing_id: None,
            missing_classes: BTreeSet::new(),
            missing_attributes: BTreeSet::new(),
            missing_pseudo_states: BTreeSet::new(),
            unsupported_branches: vec![selector.to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeBoundarySummary {
    pub product: &'static str,
    pub ordering_model: &'static str,
    pub substitution_model: &'static str,
    pub ready_surfaces: Vec<&'static str>,
    pub not_ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeConformanceSeedCase {
    pub name: &'static str,
    pub property: &'static str,
    pub declarations: Vec<CascadeDeclaration>,
    pub expected_outcome: &'static str,
    pub expected_winner_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeConformanceSeedResult {
    pub name: &'static str,
    pub passed: bool,
    pub expected_outcome: &'static str,
    pub actual_outcome: &'static str,
    pub expected_winner_id: Option<String>,
    pub actual_winner_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeConformanceSeedReport {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<CascadeConformanceSeedResult>,
}

pub type CustomPropertyEnv = BTreeMap<String, CascadeValue>;

pub fn summarize_cascade_boundary() -> CascadeBoundarySummary {
    CascadeBoundarySummary {
        product: "omena-cascade.boundary",
        ordering_model: "lexicographicCascadeKey",
        substitution_model: "finiteCustomPropertyLeastFixedPoint",
        ready_surfaces: vec![
            "cascadeKeyOrdering",
            "specificityOrdering",
            "cascadeOutcomeProof",
            "genericCascadeWinner",
            "semanticDesignTokenRanking",
            "queryReadCascadeAtPosition",
            "selectorContextWitness",
            "selectorMatchWitness",
            "cascadeConformanceSeedCorpus",
            "customPropertySubstitution",
            "cycleToGuaranteedInvalid",
        ],
        not_ready_surfaces: vec!["wptCascadeCorpus"],
    }
}

pub fn run_cascade_conformance_seed_corpus() -> CascadeConformanceSeedReport {
    let results = cascade_conformance_seed_cases()
        .into_iter()
        .map(run_cascade_conformance_seed_case)
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    CascadeConformanceSeedReport {
        schema_version: "0",
        product: "omena-cascade.conformance-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count.saturating_sub(passed_count),
        results,
    }
}

fn run_cascade_conformance_seed_case(
    case: CascadeConformanceSeedCase,
) -> CascadeConformanceSeedResult {
    let outcome = cascade_property(case.declarations, case.property);
    let (actual_outcome, actual_winner_id) = match outcome {
        CascadeOutcome::Definite { winner, .. } => ("definite", Some(winner.id)),
        CascadeOutcome::RankedSet(_) => ("rankedSet", None),
        CascadeOutcome::Inherit => ("inherit", None),
        CascadeOutcome::Top => ("top", None),
    };
    let passed =
        actual_outcome == case.expected_outcome && actual_winner_id == case.expected_winner_id;

    CascadeConformanceSeedResult {
        name: case.name,
        passed,
        expected_outcome: case.expected_outcome,
        actual_outcome,
        expected_winner_id: case.expected_winner_id,
        actual_winner_id,
    }
}

fn cascade_conformance_seed_cases() -> Vec<CascadeConformanceSeedCase> {
    vec![
        CascadeConformanceSeedCase {
            name: "source-order-breaks-identical-key",
            property: "color",
            declarations: vec![
                conformance_decl(
                    "source-earlier",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
                conformance_decl(
                    "source-later",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("source-later".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "specificity-beats-source-order",
            property: "color",
            declarations: vec![
                conformance_decl(
                    "specificity-low-later",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "specificity-high-earlier",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(1, 0, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("specificity-high-earlier".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "important-origin-beats-inline-normal",
            property: "color",
            declarations: vec![
                conformance_decl(
                    "inline-normal",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::InlineNormal,
                        0,
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "author-important",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorImportant,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("author-important".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "layer-rank-beats-specificity-within-level",
            property: "color",
            declarations: vec![
                conformance_decl(
                    "lower-layer-specific",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        1,
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "higher-layer",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        2,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("higher-layer".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "scope-proximity-beats-specificity-tie",
            property: "color",
            declarations: vec![
                conformance_decl(
                    "far-scope",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        5,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "near-scope",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        1,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("near-scope".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "missing-property-inherits",
            property: "background",
            declarations: vec![conformance_decl(
                "color-only",
                "color",
                "red",
                conformance_key(
                    CascadeLevel::AuthorNormal,
                    0,
                    0,
                    Specificity::new(0, 1, 0),
                    1,
                ),
            )],
            expected_outcome: "inherit",
            expected_winner_id: None,
        },
    ]
}

fn conformance_key(
    level: CascadeLevel,
    layer_rank: i32,
    scope_proximity: u32,
    specificity: Specificity,
    source_order: u32,
) -> CascadeKey {
    CascadeKey::new(
        level,
        LayerRank(layer_rank),
        scope_proximity,
        specificity,
        source_order,
    )
}

fn conformance_decl(id: &str, property: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: property.to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key,
    }
}

pub fn cascade_property(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let mut matching: Vec<CascadeDeclaration> = declarations
        .into_iter()
        .filter(|declaration| declaration.property == property)
        .collect();

    if matching.is_empty() {
        return CascadeOutcome::Inherit;
    }

    matching.sort_by(|left, right| right.key.cmp(&left.key));
    let winner = matching.remove(0);
    let proof = CascadeProof::from_declaration(&winner);
    CascadeOutcome::Definite {
        winner,
        proof,
        also_considered: matching,
    }
}

pub fn rank_cascade_items<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Vec<T> {
    let mut ranked = items.into_iter().collect::<Vec<_>>();
    ranked.sort_by_key(|item| Reverse(key_for(item)));
    ranked
}

pub fn select_cascade_winner<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Option<(T, Vec<T>)> {
    let mut ranked = rank_cascade_items(items, key_for);
    if ranked.is_empty() {
        return None;
    }

    let winner = ranked.remove(0);
    Some((winner, ranked))
}

pub fn selector_context_witness(
    declaration_selectors: &[String],
    reference_selectors: &[String],
) -> SelectorContextWitness {
    if declaration_selectors.is_empty() {
        return SelectorContextWitness {
            kind: SelectorContextMatchKind::Global,
            matched: true,
            rank: 1,
            declaration_selector: None,
            reference_selector: None,
        };
    }

    let mut best = SelectorContextWitness::no_match();
    for declaration_selector in declaration_selectors {
        let candidate = selector_context_witness_for_declaration(
            declaration_selector.as_str(),
            reference_selectors,
        );
        if candidate.rank > best.rank {
            best = candidate;
        }
    }
    best
}

pub fn selector_context_witness_for_declaration(
    declaration_selector: &str,
    reference_selectors: &[String],
) -> SelectorContextWitness {
    if declaration_selector == ":root" {
        return SelectorContextWitness {
            kind: SelectorContextMatchKind::Root,
            matched: true,
            rank: 1,
            declaration_selector: Some(declaration_selector.to_string()),
            reference_selector: None,
        };
    }

    for reference_selector in reference_selectors {
        if reference_selector == declaration_selector {
            return SelectorContextWitness {
                kind: SelectorContextMatchKind::Exact,
                matched: true,
                rank: 2,
                declaration_selector: Some(declaration_selector.to_string()),
                reference_selector: Some(reference_selector.clone()),
            };
        }
    }

    for reference_selector in reference_selectors {
        if reference_selector.contains(declaration_selector) {
            return SelectorContextWitness {
                kind: SelectorContextMatchKind::ContainsSelector,
                matched: true,
                rank: 2,
                declaration_selector: Some(declaration_selector.to_string()),
                reference_selector: Some(reference_selector.clone()),
            };
        }
    }

    SelectorContextWitness {
        kind: SelectorContextMatchKind::NoMatch,
        matched: false,
        rank: 0,
        declaration_selector: Some(declaration_selector.to_string()),
        reference_selector: None,
    }
}

pub fn selector_match_witness(selector: &str, element: &ElementSignature) -> SelectorMatchWitness {
    let branches = split_selector_list(selector);
    if branches.is_empty() {
        return SelectorMatchWitness::unsupported(selector);
    }

    let mut witnesses = branches
        .iter()
        .map(|branch| selector_match_branch_witness(branch, element))
        .collect::<Vec<_>>();

    let yes = strongest_by_verdict(&witnesses, SelectorMatchVerdict::Yes);
    if let Some(index) = yes {
        let mut witness = witnesses.remove(index);
        witness.selector = selector.to_string();
        if branches.len() > 1 {
            witness.reason = SelectorMatchReason::SelectorList;
            witness.unsupported_branches = witnesses
                .into_iter()
                .flat_map(|witness| witness.unsupported_branches)
                .collect();
        }
        return witness;
    }

    let maybe = strongest_by_verdict(&witnesses, SelectorMatchVerdict::Maybe);
    if let Some(index) = maybe {
        let mut witness = witnesses.remove(index);
        witness.selector = selector.to_string();
        if branches.len() > 1 {
            witness.reason = SelectorMatchReason::SelectorList;
            witness.unsupported_branches = witnesses
                .into_iter()
                .flat_map(|witness| witness.unsupported_branches)
                .collect();
        }
        return witness;
    }

    let mut witness = witnesses
        .into_iter()
        .max_by(|left, right| left.specificity.cmp(&right.specificity))
        .unwrap_or_else(|| SelectorMatchWitness::unsupported(selector));
    witness.selector = selector.to_string();
    if branches.len() > 1 {
        witness.reason = SelectorMatchReason::SelectorList;
    }
    witness
}

pub fn parse_simple_selector_signature(selector: &str) -> Option<SelectorSignature> {
    parse_simple_selector_signature_inner(selector.trim())
}

fn selector_match_branch_witness(
    selector: &str,
    element: &ElementSignature,
) -> SelectorMatchWitness {
    let Some(signature) = parse_simple_selector_signature(selector) else {
        return SelectorMatchWitness::unsupported(selector);
    };

    let mut witness = SelectorMatchWitness {
        selector: selector.to_string(),
        matched_branch: Some(selector.to_string()),
        verdict: SelectorMatchVerdict::Yes,
        reason: if signature.required_tag.is_none()
            && signature.required_id.is_none()
            && signature.required_classes.is_empty()
            && signature.required_attributes.is_empty()
            && signature.required_pseudo_states.is_empty()
        {
            SelectorMatchReason::Universal
        } else {
            SelectorMatchReason::SimpleCompound
        },
        specificity: signature.specificity,
        missing_tag: None,
        missing_id: None,
        missing_classes: BTreeSet::new(),
        missing_attributes: BTreeSet::new(),
        missing_pseudo_states: BTreeSet::new(),
        unsupported_branches: Vec::new(),
    };

    if let Some(required_tag) = &signature.required_tag {
        match element.tag.as_deref() {
            Some(tag) if tag == required_tag => {}
            _ if !element.tag_is_exact => {
                witness.verdict = SelectorMatchVerdict::Maybe;
                witness.reason = SelectorMatchReason::MissingTag;
                witness.missing_tag = Some(required_tag.clone());
            }
            _ => {
                witness.verdict = SelectorMatchVerdict::No;
                witness.reason = SelectorMatchReason::MissingTag;
                witness.missing_tag = Some(required_tag.clone());
            }
        }
    }

    if let Some(required_id) = &signature.required_id {
        match element.id.as_deref() {
            Some(id) if id == required_id => {}
            _ if !element.id_is_exact && witness.verdict != SelectorMatchVerdict::No => {
                witness.verdict = SelectorMatchVerdict::Maybe;
                witness.reason = SelectorMatchReason::MissingId;
                witness.missing_id = Some(required_id.clone());
            }
            _ => {
                witness.verdict = SelectorMatchVerdict::No;
                witness.reason = SelectorMatchReason::MissingId;
                witness.missing_id = Some(required_id.clone());
            }
        }
    }

    for required_class in &signature.required_classes {
        if element.classes.contains(required_class) {
            continue;
        }
        if !element.classes_are_exact && witness.verdict != SelectorMatchVerdict::No {
            witness.verdict = SelectorMatchVerdict::Maybe;
        } else {
            witness.verdict = SelectorMatchVerdict::No;
        }
        witness.reason = SelectorMatchReason::MissingClass;
        witness.missing_classes.insert(required_class.clone());
    }

    for required_attribute in &signature.required_attributes {
        if element.attributes.contains(required_attribute) {
            continue;
        }
        if !element.attributes_are_exact && witness.verdict != SelectorMatchVerdict::No {
            witness.verdict = SelectorMatchVerdict::Maybe;
        } else {
            witness.verdict = SelectorMatchVerdict::No;
        }
        witness.reason = SelectorMatchReason::MissingAttribute;
        witness
            .missing_attributes
            .insert(required_attribute.clone());
    }

    for required_pseudo_state in &signature.required_pseudo_states {
        if element.pseudo_states.contains(required_pseudo_state) {
            continue;
        }
        if !element.pseudo_states_are_exact && witness.verdict != SelectorMatchVerdict::No {
            witness.verdict = SelectorMatchVerdict::Maybe;
        } else {
            witness.verdict = SelectorMatchVerdict::No;
        }
        witness.reason = SelectorMatchReason::MissingPseudoState;
        witness
            .missing_pseudo_states
            .insert(required_pseudo_state.clone());
    }

    witness
}

fn strongest_by_verdict(
    witnesses: &[SelectorMatchWitness],
    verdict: SelectorMatchVerdict,
) -> Option<usize> {
    witnesses
        .iter()
        .enumerate()
        .filter(|(_, witness)| witness.verdict == verdict)
        .max_by(|(_, left), (_, right)| left.specificity.cmp(&right.specificity))
        .map(|(index, _)| index)
}

fn parse_simple_selector_signature_inner(selector: &str) -> Option<SelectorSignature> {
    if selector.is_empty() || selector_has_unsupported_top_level_syntax(selector) {
        return None;
    }

    let mut required_tag = None;
    let mut required_id = None;
    let mut required_classes = BTreeSet::new();
    let mut required_attributes = BTreeSet::new();
    let mut required_pseudo_states = BTreeSet::new();
    let mut specificity = Specificity::ZERO;
    let chars = selector.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        match chars[index] {
            '*' => index += 1,
            '.' => {
                index += 1;
                let (name, next) = read_identifier(&chars, index)?;
                specificity.classes += 1;
                required_classes.insert(name);
                index = next;
            }
            '#' => {
                index += 1;
                let (name, next) = read_identifier(&chars, index)?;
                specificity.ids += 1;
                required_id = Some(name);
                index = next;
            }
            '[' => {
                let close = find_closing_bracket(&chars, index)?;
                let attribute = chars[index + 1..close].iter().collect::<String>();
                let attribute_name = read_attribute_name(attribute.trim())?;
                specificity.classes += 1;
                required_attributes.insert(attribute_name);
                index = close + 1;
            }
            ':' => {
                if matches!(chars.get(index + 1), Some(':')) {
                    index += 2;
                    let (_, next) = read_identifier(&chars, index)?;
                    specificity.elements += 1;
                    index = next;
                } else {
                    index += 1;
                    let (name, next) = read_identifier(&chars, index)?;
                    if matches!(chars.get(next), Some('(')) {
                        return None;
                    }
                    specificity.classes += 1;
                    required_pseudo_states.insert(name);
                    index = next;
                }
            }
            ch if is_identifier_start(ch) => {
                let (name, next) = read_identifier(&chars, index)?;
                if required_tag.is_some() {
                    return None;
                }
                specificity.elements += 1;
                required_tag = Some(name);
                index = next;
            }
            _ => return None,
        }
    }

    Some(SelectorSignature {
        selector: selector.to_string(),
        required_tag,
        required_id,
        required_classes,
        required_attributes,
        required_pseudo_states,
        specificity,
    })
}

fn split_selector_list(selector: &str) -> Vec<String> {
    let mut branches = Vec::new();
    let mut start = 0;
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let chars = selector.char_indices().collect::<Vec<_>>();

    for (index, ch) in &chars {
        match *ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let branch = selector[start..*index].trim();
                if !branch.is_empty() {
                    branches.push(branch.to_string());
                }
                start = *index + 1;
            }
            _ => {}
        }
    }

    let tail = selector[start..].trim();
    if !tail.is_empty() {
        branches.push(tail.to_string());
    }
    branches
}

fn selector_has_unsupported_top_level_syntax(selector: &str) -> bool {
    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    for ch in selector.chars() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '>' | '+' | '~' if paren_depth == 0 && bracket_depth == 0 => return true,
            ch if ch.is_whitespace() && paren_depth == 0 && bracket_depth == 0 => return true,
            _ => {}
        }
    }
    false
}

fn find_closing_bracket(chars: &[char], open_index: usize) -> Option<usize> {
    chars
        .iter()
        .enumerate()
        .skip(open_index + 1)
        .find_map(|(index, ch)| if *ch == ']' { Some(index) } else { None })
}

fn read_attribute_name(attribute: &str) -> Option<String> {
    let name = attribute
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '=' | '~' | '|' | '^' | '$' | '*'))
        .find(|part| !part.is_empty())?;
    Some(name.to_string())
}

fn read_identifier(chars: &[char], start: usize) -> Option<(String, usize)> {
    if start >= chars.len() || !is_identifier_start(chars[start]) {
        return None;
    }
    let mut end = start + 1;
    while end < chars.len() && is_identifier_continue(chars[end]) {
        end += 1;
    }
    Some((chars[start..end].iter().collect(), end))
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '-')
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

pub fn substitute_custom_properties(value: &CascadeValue, env: &CustomPropertyEnv) -> CascadeValue {
    let mut visiting = BTreeSet::new();
    substitute_custom_properties_inner(value, env, &mut visiting)
}

fn substitute_custom_properties_inner(
    value: &CascadeValue,
    env: &CustomPropertyEnv,
    visiting: &mut BTreeSet<String>,
) -> CascadeValue {
    match value {
        CascadeValue::Literal(_) | CascadeValue::GuaranteedInvalid | CascadeValue::Unset => {
            value.clone()
        }
        CascadeValue::Var { name, fallback } => {
            if !visiting.insert(name.clone()) {
                return CascadeValue::GuaranteedInvalid;
            }
            let resolved = match env.get(name) {
                Some(CascadeValue::Unset) | None => fallback
                    .as_deref()
                    .map(|fallback| substitute_custom_properties_inner(fallback, env, visiting))
                    .unwrap_or(CascadeValue::GuaranteedInvalid),
                Some(value) => substitute_custom_properties_inner(value, env, visiting),
            };
            visiting.remove(name);
            resolved
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(id: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
        CascadeDeclaration {
            id: id.to_string(),
            property: "color".to_string(),
            value: CascadeValue::Literal(value.to_string()),
            key,
        }
    }

    fn key(
        level: CascadeLevel,
        layer_rank: i32,
        scope_proximity: u32,
        specificity: Specificity,
        source_order: u32,
    ) -> CascadeKey {
        CascadeKey::new(
            level,
            LayerRank(layer_rank),
            scope_proximity,
            specificity,
            source_order,
        )
    }

    #[test]
    fn orders_specificity_lexicographically() {
        assert!(Specificity::new(1, 0, 0) > Specificity::new(0, 99, 99));
        assert!(Specificity::new(0, 2, 0) > Specificity::new(0, 1, 99));
        assert!(Specificity::new(0, 0, 2) > Specificity::new(0, 0, 1));
    }

    #[test]
    fn orders_cascade_keys_by_level_layer_scope_specificity_and_source() {
        let base = key(
            CascadeLevel::AuthorNormal,
            0,
            3,
            Specificity::new(0, 1, 0),
            1,
        );
        assert!(
            key(
                CascadeLevel::AuthorImportant,
                0,
                3,
                Specificity::new(0, 1, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                1,
                3,
                Specificity::new(0, 1, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                0,
                3,
                Specificity::new(0, 2, 0),
                1,
            ) > base
        );
        assert!(
            key(
                CascadeLevel::AuthorNormal,
                0,
                3,
                Specificity::new(0, 1, 0),
                2,
            ) > base
        );
    }

    #[test]
    fn selects_definite_winner_with_proof() {
        let earlier = declaration(
            "earlier",
            "red",
            key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                1,
            ),
        );
        let later = declaration(
            "later",
            "blue",
            key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                2,
            ),
        );

        let outcome = cascade_property([earlier, later], "color");

        assert!(matches!(outcome, CascadeOutcome::Definite { .. }));
        if let CascadeOutcome::Definite {
            winner,
            proof,
            also_considered,
        } = outcome
        {
            assert_eq!(winner.id, "later");
            assert_eq!(proof.declaration_id, "later");
            assert_eq!(also_considered.len(), 1);
        }
    }

    #[test]
    fn selects_generic_winner_with_same_cascade_ordering() {
        let ranked = select_cascade_winner(["earlier", "later"], |item| match *item {
            "earlier" => key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                1,
            ),
            _ => key(
                CascadeLevel::AuthorNormal,
                0,
                1,
                Specificity::new(0, 1, 0),
                2,
            ),
        });

        let Some((winner, also_considered)) = ranked else {
            unreachable!("test input contains candidates")
        };
        assert_eq!(winner, "later");
        assert_eq!(also_considered, vec!["earlier"]);
    }

    #[test]
    fn reports_selector_context_witness_rank() {
        let root = selector_context_witness(&[":root".to_string()], &[".button".to_string()]);
        assert_eq!(root.kind, SelectorContextMatchKind::Root);
        assert!(root.matched);
        assert_eq!(root.rank, 1);

        let exact = selector_context_witness(&[".button".to_string()], &[".button".to_string()]);
        assert_eq!(exact.kind, SelectorContextMatchKind::Exact);
        assert_eq!(exact.rank, 2);

        let descendant =
            selector_context_witness(&[".theme".to_string()], &[".theme .button".to_string()]);
        assert_eq!(descendant.kind, SelectorContextMatchKind::ContainsSelector);
        assert_eq!(
            descendant.reference_selector.as_deref(),
            Some(".theme .button")
        );

        let miss = selector_context_witness(&[".card".to_string()], &[".button".to_string()]);
        assert_eq!(miss.kind, SelectorContextMatchKind::NoMatch);
        assert!(!miss.matched);
    }

    #[test]
    fn parses_simple_selector_specificity() {
        let signature = parse_simple_selector_signature("button#save.primary[data-state]:hover");
        assert!(signature.is_some());
        if let Some(signature) = signature {
            assert_eq!(signature.required_tag.as_deref(), Some("button"));
            assert_eq!(signature.required_id.as_deref(), Some("save"));
            assert!(signature.required_classes.contains("primary"));
            assert!(signature.required_attributes.contains("data-state"));
            assert!(signature.required_pseudo_states.contains("hover"));
            assert_eq!(signature.specificity, Specificity::new(1, 3, 1));
        }
    }

    #[test]
    fn matches_simple_compound_selectors_against_concrete_signature() {
        let mut element =
            ElementSignature::concrete(Some("button"), Some("save"), ["primary", "active"]);
        element.attributes.insert("data-state".to_string());
        element.pseudo_states.insert("hover".to_string());

        let witness = selector_match_witness("button#save.primary[data-state]:hover", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Yes);
        assert_eq!(witness.reason, SelectorMatchReason::SimpleCompound);
        assert_eq!(witness.specificity, Specificity::new(1, 3, 1));
    }

    #[test]
    fn reports_missing_class_and_id_as_no_for_exact_signature() {
        let element = ElementSignature::concrete(Some("button"), Some("save"), ["primary"]);

        let class_miss = selector_match_witness(".missing", &element);
        assert_eq!(class_miss.verdict, SelectorMatchVerdict::No);
        assert_eq!(class_miss.reason, SelectorMatchReason::MissingClass);
        assert!(class_miss.missing_classes.contains("missing"));

        let id_miss = selector_match_witness("#cancel", &element);
        assert_eq!(id_miss.verdict, SelectorMatchVerdict::No);
        assert_eq!(id_miss.reason, SelectorMatchReason::MissingId);
        assert_eq!(id_miss.missing_id.as_deref(), Some("cancel"));
    }

    #[test]
    fn returns_maybe_for_inexact_abstract_class_sets() {
        let element = ElementSignature::at_least_classes(["button"]);

        let witness = selector_match_witness(".button.primary", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Maybe);
        assert_eq!(witness.reason, SelectorMatchReason::MissingClass);
        assert!(witness.missing_classes.contains("primary"));
    }

    #[test]
    fn selector_lists_choose_strongest_matching_branch() {
        let element = ElementSignature::concrete(Some("button"), Some("save"), ["primary"]);

        let witness = selector_match_witness(".missing, button#save.primary", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Yes);
        assert_eq!(witness.reason, SelectorMatchReason::SelectorList);
        assert_eq!(
            witness.matched_branch.as_deref(),
            Some("button#save.primary")
        );
        assert_eq!(witness.specificity, Specificity::new(1, 1, 1));
    }

    #[test]
    fn unsupported_combinators_are_reported_as_maybe() {
        let element = ElementSignature::concrete(Some("span"), None::<String>, ["icon"]);

        let witness = selector_match_witness(".button > .icon", &element);

        assert_eq!(witness.verdict, SelectorMatchVerdict::Maybe);
        assert_eq!(witness.reason, SelectorMatchReason::UnsupportedSelector);
        assert_eq!(witness.unsupported_branches, vec![".button > .icon"]);
    }

    #[test]
    fn substitutes_custom_property_fallbacks_and_references() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--brand".to_string(),
            CascadeValue::Literal("red".to_string()),
        );

        let resolved = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--brand".to_string(),
                fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
            },
            &env,
        );
        assert_eq!(resolved, CascadeValue::Literal("red".to_string()));

        let fallback = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--missing".to_string(),
                fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
            },
            &env,
        );
        assert_eq!(fallback, CascadeValue::Literal("blue".to_string()));
    }

    #[test]
    fn substitutes_cycles_to_guaranteed_invalid() {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--a".to_string(),
            CascadeValue::Var {
                name: "--b".to_string(),
                fallback: None,
            },
        );
        env.insert(
            "--b".to_string(),
            CascadeValue::Var {
                name: "--a".to_string(),
                fallback: None,
            },
        );

        let resolved = substitute_custom_properties(
            &CascadeValue::Var {
                name: "--a".to_string(),
                fallback: None,
            },
            &env,
        );

        assert_eq!(resolved, CascadeValue::GuaranteedInvalid);
    }

    #[test]
    fn summarizes_current_boundary_status() {
        let summary = summarize_cascade_boundary();

        assert_eq!(summary.product, "omena-cascade.boundary");
        assert_eq!(summary.ordering_model, "lexicographicCascadeKey");
        assert!(summary.ready_surfaces.contains(&"cascadeKeyOrdering"));
        assert!(summary.ready_surfaces.contains(&"genericCascadeWinner"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"semanticDesignTokenRanking")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"queryReadCascadeAtPosition")
        );
        assert!(summary.ready_surfaces.contains(&"selectorContextWitness"));
        assert!(summary.ready_surfaces.contains(&"selectorMatchWitness"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"cascadeConformanceSeedCorpus")
        );
        assert!(!summary.not_ready_surfaces.contains(&"selectorMatchWitness"));
        assert!(summary.not_ready_surfaces.contains(&"wptCascadeCorpus"));
    }

    #[test]
    fn seed_conformance_corpus_passes_current_cascade_model() {
        let report = run_cascade_conformance_seed_corpus();

        assert_eq!(report.product, "omena-cascade.conformance-seed-corpus");
        assert_eq!(report.case_count, 6);
        assert_eq!(report.passed_count, report.case_count);
        assert_eq!(report.failed_count, 0);
        assert!(report.results.iter().all(|result| result.passed));
    }
}
