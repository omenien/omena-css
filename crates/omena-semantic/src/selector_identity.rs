//! Canonical selector identity summaries.
//!
//! This module exposes the semantic identity layer that normalizes selector
//! aliases, tracks rewrite safety, and provides stable counts for parser-to-LSP
//! selector equivalence gates.

use std::collections::BTreeSet;

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
    pub canonical_id: String,
    pub local_name: String,
    pub identity_kind: &'static str,
    pub rewrite_safety: &'static str,
    pub blockers: Vec<&'static str>,
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

    let canonical_ids = facts
        .canonical_names
        .iter()
        .map(|name| {
            let blockers = if nested_unsafe.contains(name) {
                vec!["nested-expansion"]
            } else {
                Vec::new()
            };
            SelectorCanonicalIdentityV0 {
                canonical_id: format!("selector:{name}"),
                local_name: name.clone(),
                identity_kind: if bem_safe.contains(name) {
                    "bemSuffix"
                } else {
                    "localClass"
                },
                rewrite_safety: if blockers.is_empty() {
                    "safe"
                } else {
                    "blocked"
                },
                blockers,
            }
        })
        .collect::<Vec<_>>();

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

    SelectorIdentityEngineSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.selector-identity",
        canonical_id_count: canonical_ids.len(),
        canonical_ids,
        rewrite_safety: SelectorIdentityRewriteSafetyV0 {
            all_canonical_ids_rewrite_safe: blocked_canonical_ids.is_empty(),
            safe_canonical_ids,
            blocked_canonical_ids,
            blockers: if nested_unsafe.is_empty() {
                Vec::new()
            } else {
                vec!["nested-expansion"]
            },
        },
    }
}
