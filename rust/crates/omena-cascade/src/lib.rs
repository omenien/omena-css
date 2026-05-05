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
pub struct CascadeBoundarySummary {
    pub product: &'static str,
    pub ordering_model: &'static str,
    pub substitution_model: &'static str,
    pub ready_surfaces: Vec<&'static str>,
    pub not_ready_surfaces: Vec<&'static str>,
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
            "selectorContextWitness",
            "customPropertySubstitution",
            "cycleToGuaranteedInvalid",
        ],
        not_ready_surfaces: vec![
            "selectorMatchWitness",
            "readCascadeAtPosition",
            "wptCascadeCorpus",
        ],
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
        assert!(summary.ready_surfaces.contains(&"selectorContextWitness"));
        assert!(summary.not_ready_surfaces.contains(&"wptCascadeCorpus"));
    }
}
