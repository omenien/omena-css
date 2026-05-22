use std::cmp::Reverse;

use crate::{CascadeDeclaration, CascadeKey, CascadeOutcome, CascadeProof};

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

    matching.sort_by_key(|declaration| Reverse(declaration.key));
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
