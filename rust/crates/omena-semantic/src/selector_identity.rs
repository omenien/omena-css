//! Canonical selector identity summaries.
//!
//! This module exposes the semantic identity layer that normalizes selector
//! aliases, tracks rewrite safety, and provides stable counts for parser-to-LSP
//! selector equivalence gates.

use std::collections::BTreeSet;

use omena_syntax::ident::{CanonicalClassKeyV0, ClassNameV0};
use serde::Serialize;

use crate::StyleSelectorIdentityFactsV0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorIdentityEngineSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub canonical_id_count: usize,
    pub canonical_ids: Vec<SelectorCanonicalIdentityV0>,
    pub rewrite_safety: SelectorIdentityRewriteSafetyV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorCanonicalIdentityV0 {
    canonical_id: String,
    local_name: String,
    identity_kind: &'static str,
    rewrite_safety: &'static str,
    blockers: Vec<&'static str>,
}

impl SelectorCanonicalIdentityV0 {
    fn from_canonical_key(
        key: &CanonicalClassKeyV0,
        identity_kind: &'static str,
        blockers: Vec<&'static str>,
    ) -> Self {
        let local_name = key.as_str().to_string();
        Self {
            canonical_id: format!("selector:{local_name}"),
            local_name,
            identity_kind,
            rewrite_safety: if blockers.is_empty() {
                "safe"
            } else {
                "blocked"
            },
            blockers,
        }
    }

    pub fn canonical_id(&self) -> &str {
        self.canonical_id.as_str()
    }

    pub fn local_name(&self) -> &str {
        self.local_name.as_str()
    }

    pub fn identity_kind(&self) -> &'static str {
        self.identity_kind
    }

    pub fn rewrite_safety(&self) -> &'static str {
        self.rewrite_safety
    }

    pub fn blockers(&self) -> &[&'static str] {
        self.blockers.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorIdentityRewriteSafetyV0 {
    pub all_canonical_ids_rewrite_safe: bool,
    pub safe_canonical_ids: Vec<String>,
    pub blocked_canonical_ids: Vec<String>,
    pub blockers: Vec<&'static str>,
}

pub fn summarize_selector_identity_engine(
    facts: &StyleSelectorIdentityFactsV0,
) -> SelectorIdentityEngineSummaryV0 {
    let bem_safe = facts
        .bem_suffix_safe_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let nested_unsafe = facts
        .nested_unsafe_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut authority_binding_missing = false;
    let mut issued_names = BTreeSet::<String>::new();
    let mut canonical_ids = Vec::new();
    for name in &facts.canonical_names {
        let blockers = if nested_unsafe.contains(name) {
            vec!["nested-expansion"]
        } else {
            Vec::new()
        };
        let decoded_key = ClassNameV0::new(name).canonical_key();
        let authority_key = facts
            .selector_authority_definitions
            .iter()
            .filter(|definition| definition.name == *name)
            .find_map(|definition| {
                let inferred_nesting_parent = facts
                    .selector_authority_definitions
                    .iter()
                    // These ordering, length, and prefix clauses only narrow the
                    // candidate set. They cannot issue an identity: the parser-owned
                    // AST substitution and decoded-key equality below remain the
                    // authority, so a mistaken exclusion can only lose precision.
                    .filter(|candidate| {
                        candidate.source_order < definition.source_order
                            && candidate.name.len() < name.len()
                            && name.starts_with(candidate.name.as_str())
                            && issued_names.contains(candidate.name.as_str())
                    })
                    .max_by_key(|candidate| (candidate.name.len(), candidate.source_order))
                    .map(|candidate| candidate.name.as_str());
                let nesting_parent_names = [
                    definition.bem_suffix_parent_name.as_deref(),
                    inferred_nesting_parent,
                ];
                facts.selector_asts.iter().find_map(|ast| {
                    nesting_parent_names.into_iter().find_map(|parent_name| {
                        ast.canonical_class_key_for_source_span(
                            name,
                            definition.byte_span.start..definition.byte_span.end,
                            parent_name,
                        )
                    })
                })
            })
            .filter(|authority_key| authority_key == &decoded_key);
        let Some(authority_key) = authority_key else {
            authority_binding_missing = true;
            continue;
        };
        issued_names.insert(name.clone());
        canonical_ids.push(SelectorCanonicalIdentityV0::from_canonical_key(
            &authority_key,
            if bem_safe.contains(name) {
                "bemSuffix"
            } else {
                "localClass"
            },
            blockers,
        ));
    }

    let safe_canonical_ids = canonical_ids
        .iter()
        .filter(|identity| identity.blockers.is_empty())
        .map(|identity| identity.canonical_id.clone())
        .collect::<Vec<_>>();
    let blocked_canonical_ids = canonical_ids
        .iter()
        .filter(|identity| !identity.blockers.is_empty())
        .map(|identity| identity.canonical_id.clone())
        .collect::<Vec<_>>();
    let nested_expansion_present = !blocked_canonical_ids.is_empty();

    SelectorIdentityEngineSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.selector-identity",
        canonical_id_count: canonical_ids.len(),
        canonical_ids,
        rewrite_safety: SelectorIdentityRewriteSafetyV0 {
            all_canonical_ids_rewrite_safe: blocked_canonical_ids.is_empty()
                && !authority_binding_missing,
            safe_canonical_ids,
            blocked_canonical_ids,
            blockers: match (nested_expansion_present, authority_binding_missing) {
                (false, false) => Vec::new(),
                (true, false) => vec!["nested-expansion"],
                (false, true) => vec!["authority-binding-missing"],
                (true, true) => vec!["nested-expansion", "authority-binding-missing"],
            },
        },
    }
}
