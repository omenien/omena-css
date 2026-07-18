use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AbstractClassValueProvenanceV0, AbstractClassValueV0, AbstractStringAutomatonTransitionV0,
    AbstractStringAutomatonV0, MAX_FINITE_CLASS_VALUES, MAX_STRING_AUTOMATON_STATES,
    bottom_class_value, exact_class_value, top_class_value_with_provenance,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TrieNode {
    accepting: bool,
    transitions: BTreeMap<char, usize>,
}

impl TrieNode {
    fn empty() -> Self {
        Self {
            accepting: false,
            transitions: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StateSignature {
    accepting: bool,
    transitions: Vec<(char, usize)>,
}

#[derive(Debug, Clone)]
struct CanonicalState {
    accepting: bool,
    transitions: BTreeMap<char, usize>,
}

pub(crate) fn automaton_class_value_from_values(
    values: &[String],
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    let values = normalize_automaton_values(values);
    match values.len() {
        0 => bottom_class_value(),
        1 => exact_class_value(values[0].clone()),
        2..=MAX_FINITE_CLASS_VALUES => AbstractClassValueV0::FiniteSet { values },
        _ => build_minimized_automaton(&values)
            .filter(|automaton| automaton.state_count <= MAX_STRING_AUTOMATON_STATES)
            .map_or_else(
                || {
                    top_class_value_with_provenance(
                        AbstractClassValueProvenanceV0::AutomatonStateLimit,
                    )
                },
                |automaton| AbstractClassValueV0::Automaton {
                    automaton: Box::new(automaton),
                    provenance,
                },
            ),
    }
}

pub(crate) fn join_automaton_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    let mut values = finite_language_values(left)?;
    values.extend(finite_language_values(right)?);
    Some(automaton_class_value_from_values(
        &values,
        Some(AbstractClassValueProvenanceV0::AutomatonJoin),
    ))
}

pub(crate) fn concatenate_automaton_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    let left_values = finite_language_values(left)?;
    let right_values = finite_language_values(right)?;
    let values = left_values
        .into_iter()
        .flat_map(|left_value| {
            right_values
                .iter()
                .map(move |right_value| format!("{left_value}{right_value}"))
        })
        .collect::<Vec<_>>();
    Some(automaton_class_value_from_values(
        &values,
        Some(AbstractClassValueProvenanceV0::AutomatonConcat),
    ))
}

pub(crate) fn finite_language_values(value: &AbstractClassValueV0) -> Option<Vec<String>> {
    match value {
        AbstractClassValueV0::Bottom => Some(Vec::new()),
        AbstractClassValueV0::Exact { value } => Some(vec![value.clone()]),
        AbstractClassValueV0::FiniteSet { values } => Some(values.clone()),
        AbstractClassValueV0::Automaton { automaton, .. } => {
            accepted_strings_from_automaton(automaton)
        }
        _ => None,
    }
}

pub(crate) fn automaton_matches_string(
    automaton: &AbstractStringAutomatonV0,
    candidate: &str,
) -> bool {
    let transitions = transitions_by_state(automaton);
    let accept_states = automaton
        .accept_states
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut state = automaton.start_state;
    for symbol in candidate.chars() {
        let Some(next_state) = transitions.get(&state).and_then(|edges| edges.get(&symbol)) else {
            return false;
        };
        state = *next_state;
    }
    accept_states.contains(&state)
}

pub fn automaton_key(automaton: &AbstractStringAutomatonV0) -> String {
    let accept_states = automaton
        .accept_states
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let transitions = automaton
        .transitions
        .iter()
        .map(|transition| {
            format!(
                "{}:{}:{}",
                transition.from, transition.symbol, transition.to
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "automaton:{}:{}:{accept_states}:{transitions}",
        automaton.state_count, automaton.start_state
    )
}

fn normalize_automaton_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn build_minimized_automaton(values: &[String]) -> Option<AbstractStringAutomatonV0> {
    let trie = build_trie(values);
    let mut registry = BTreeMap::<StateSignature, usize>::new();
    let mut canonical_states = BTreeMap::<usize, CanonicalState>::new();
    let start_state = minimize_trie_node(0, &trie, &mut registry, &mut canonical_states);
    Some(reindex_automaton(start_state, &canonical_states))
}

fn build_trie(values: &[String]) -> Vec<TrieNode> {
    let mut trie = vec![TrieNode::empty()];
    for value in values {
        let mut state = 0;
        for symbol in value.chars() {
            let next_state = trie[state].transitions.get(&symbol).copied();
            state = match next_state {
                Some(next_state) => next_state,
                None => {
                    let next_state = trie.len();
                    trie.push(TrieNode::empty());
                    trie[state].transitions.insert(symbol, next_state);
                    next_state
                }
            };
        }
        trie[state].accepting = true;
    }
    trie
}

fn minimize_trie_node(
    trie_state: usize,
    trie: &[TrieNode],
    registry: &mut BTreeMap<StateSignature, usize>,
    canonical_states: &mut BTreeMap<usize, CanonicalState>,
) -> usize {
    let transitions = trie[trie_state]
        .transitions
        .iter()
        .map(|(symbol, child)| {
            (
                *symbol,
                minimize_trie_node(*child, trie, registry, canonical_states),
            )
        })
        .collect::<Vec<_>>();
    let signature = StateSignature {
        accepting: trie[trie_state].accepting,
        transitions: transitions.clone(),
    };
    if let Some(state_id) = registry.get(&signature) {
        return *state_id;
    }

    let state_id = registry.len();
    registry.insert(signature, state_id);
    canonical_states.insert(
        state_id,
        CanonicalState {
            accepting: trie[trie_state].accepting,
            transitions: transitions.into_iter().collect(),
        },
    );
    state_id
}

fn reindex_automaton(
    start_state: usize,
    canonical_states: &BTreeMap<usize, CanonicalState>,
) -> AbstractStringAutomatonV0 {
    let mut old_to_new = BTreeMap::<usize, usize>::new();
    let mut accept_states = BTreeSet::<usize>::new();
    let mut transitions = Vec::<AbstractStringAutomatonTransitionV0>::new();
    reindex_state(
        start_state,
        canonical_states,
        &mut old_to_new,
        &mut accept_states,
        &mut transitions,
    );
    transitions.sort_by(|left, right| {
        (left.from, left.symbol.as_str(), left.to).cmp(&(
            right.from,
            right.symbol.as_str(),
            right.to,
        ))
    });

    AbstractStringAutomatonV0 {
        state_count: old_to_new.len(),
        start_state: 0,
        accept_states: accept_states.into_iter().collect(),
        transitions,
    }
}

fn reindex_state(
    old_state: usize,
    canonical_states: &BTreeMap<usize, CanonicalState>,
    old_to_new: &mut BTreeMap<usize, usize>,
    accept_states: &mut BTreeSet<usize>,
    transitions: &mut Vec<AbstractStringAutomatonTransitionV0>,
) -> usize {
    if let Some(new_state) = old_to_new.get(&old_state) {
        return *new_state;
    }

    let new_state = old_to_new.len();
    old_to_new.insert(old_state, new_state);
    let Some(state) = canonical_states.get(&old_state) else {
        return new_state;
    };
    if state.accepting {
        accept_states.insert(new_state);
    }

    for (symbol, old_target) in &state.transitions {
        let new_target = reindex_state(
            *old_target,
            canonical_states,
            old_to_new,
            accept_states,
            transitions,
        );
        transitions.push(AbstractStringAutomatonTransitionV0 {
            from: new_state,
            symbol: symbol.to_string(),
            to: new_target,
        });
    }

    new_state
}

fn accepted_strings_from_automaton(automaton: &AbstractStringAutomatonV0) -> Option<Vec<String>> {
    let transitions = transitions_by_state(automaton);
    let accept_states = automaton
        .accept_states
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let mut visiting = BTreeSet::<usize>::new();
    let mut output = BTreeSet::<String>::new();
    collect_accepted_strings(
        automaton.start_state,
        String::new(),
        &transitions,
        &accept_states,
        &mut visiting,
        &mut output,
    )?;
    Some(output.into_iter().collect())
}

fn collect_accepted_strings(
    state: usize,
    prefix: String,
    transitions: &BTreeMap<usize, BTreeMap<char, usize>>,
    accept_states: &BTreeSet<usize>,
    visiting: &mut BTreeSet<usize>,
    output: &mut BTreeSet<String>,
) -> Option<()> {
    if !visiting.insert(state) {
        return None;
    }
    if accept_states.contains(&state) {
        output.insert(prefix.clone());
    }
    if let Some(edges) = transitions.get(&state) {
        for (symbol, next_state) in edges {
            let mut next_prefix = prefix.clone();
            next_prefix.push(*symbol);
            collect_accepted_strings(
                *next_state,
                next_prefix,
                transitions,
                accept_states,
                visiting,
                output,
            )?;
        }
    }
    visiting.remove(&state);
    Some(())
}

fn transitions_by_state(
    automaton: &AbstractStringAutomatonV0,
) -> BTreeMap<usize, BTreeMap<char, usize>> {
    let mut transitions = BTreeMap::<usize, BTreeMap<char, usize>>::new();
    for transition in &automaton.transitions {
        let Some(symbol) = single_transition_symbol(&transition.symbol) else {
            continue;
        };
        transitions
            .entry(transition.from)
            .or_default()
            .insert(symbol, transition.to);
    }
    transitions
}

fn single_transition_symbol(symbol: &str) -> Option<char> {
    let mut chars = symbol.chars();
    let first = chars.next()?;
    chars.next().is_none().then_some(first)
}
