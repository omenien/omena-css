use std::collections::{BTreeMap, BTreeSet};

use crate::{
    AbstractClassValueProvenanceV0, AbstractClassValueV0, AbstractStringAutomatonTransitionV0,
    AbstractStringAutomatonV0, MAX_FINITE_CLASS_VALUES, MAX_STRING_AUTOMATON_LANGUAGE_CARDINALITY,
    MAX_STRING_AUTOMATON_MATERIALIZED_BYTES, MAX_STRING_AUTOMATON_STATES,
    OmenaAbstractValueCoverageDirectionV0, OmenaAbstractValuePrecisionBasisV0,
    OmenaAbstractValuePrecisionWitnessV0, bottom_class_value, exact_class_value,
    top_class_value_with_provenance,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FiniteLanguageCost {
    cardinality: usize,
    materialized_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FiniteLanguageInputCost {
    occurrence_bytes: usize,
    unique: FiniteLanguageCost,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct AutomatonInputPreflight {
    occurrence_count: usize,
    occurrence_bytes: usize,
}

impl AutomatonInputPreflight {
    fn observe(&mut self, value: &str) -> Result<(), AutomatonCostLimit> {
        self.occurrence_count = self
            .occurrence_count
            .checked_add(1)
            .ok_or(AutomatonCostLimit::LanguageCardinality)?;
        if self.occurrence_count > MAX_STRING_AUTOMATON_LANGUAGE_CARDINALITY {
            return Err(AutomatonCostLimit::LanguageCardinality);
        }
        self.occurrence_bytes = self.occurrence_bytes.saturating_add(value.len());
        if self.occurrence_count > MAX_FINITE_CLASS_VALUES
            && self.occurrence_bytes > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES
        {
            return Err(AutomatonCostLimit::MaterializedBytes);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundedFiniteLanguage {
    values: Vec<String>,
    cost: FiniteLanguageCost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutomatonCostLimit {
    LanguageCardinality,
    MaterializedBytes,
}

impl AutomatonCostLimit {
    fn provenance(self) -> AbstractClassValueProvenanceV0 {
        match self {
            Self::LanguageCardinality => {
                AbstractClassValueProvenanceV0::AutomatonLanguageCardinalityLimit
            }
            Self::MaterializedBytes => {
                AbstractClassValueProvenanceV0::AutomatonMaterializedByteLimit
            }
        }
    }
}

pub(crate) fn automaton_class_value_from_values(
    values: &[String],
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    // Bound the raw candidate list before normalization. Up to eight occurrences retain legacy
    // exact/finite precision; above that, both raw materialization and the deduplicated automaton
    // branch must fit their declared walls.
    if let Err(limit) = preflight_automaton_branch_values(values.iter().map(String::as_str)) {
        return top_class_value_with_provenance(limit.provenance());
    }
    let values = normalize_automaton_values(values);
    automaton_class_value_from_normalized_values(values, provenance)
}

pub(crate) fn automaton_class_value_from_owned_values(
    values: impl IntoIterator<Item = String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    let mut preflight = AutomatonInputPreflight::default();
    let mut normalized = BTreeSet::<String>::new();

    for value in values {
        if let Err(limit) = preflight.observe(&value) {
            return top_class_value_with_provenance(limit.provenance());
        }
        normalized.insert(value);
    }
    automaton_class_value_from_normalized_values(normalized.into_iter().collect(), provenance)
}

fn automaton_class_value_from_normalized_values(
    values: Vec<String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
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
                    precision_witness: Some(OmenaAbstractValuePrecisionWitnessV0 {
                        direction: OmenaAbstractValueCoverageDirectionV0::SupersetOfProducible,
                        basis: OmenaAbstractValuePrecisionBasisV0::AcyclicExact,
                        authority_digest: None,
                    }),
                },
            ),
    }
}

fn preflight_automaton_input<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<FiniteLanguageInputCost, AutomatonCostLimit> {
    let mut unique_values = BTreeSet::<&str>::new();
    let mut preflight = AutomatonInputPreflight::default();
    let mut unique_materialized_bytes = 0_usize;
    for value in values {
        preflight.observe(value)?;
        if unique_values.insert(value) {
            unique_materialized_bytes = unique_materialized_bytes.saturating_add(value.len());
        }
    }
    Ok(FiniteLanguageInputCost {
        occurrence_bytes: preflight.occurrence_bytes,
        unique: FiniteLanguageCost {
            cardinality: unique_values.len(),
            materialized_bytes: unique_materialized_bytes,
        },
    })
}

fn preflight_automaton_branch_values<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<FiniteLanguageCost, AutomatonCostLimit> {
    let input = preflight_automaton_input(values)?;
    preflight_automaton_branch_cost(input.unique)
}

fn preflight_automaton_branch_cost(
    cost: FiniteLanguageCost,
) -> Result<FiniteLanguageCost, AutomatonCostLimit> {
    if cost.cardinality > MAX_FINITE_CLASS_VALUES
        && cost.materialized_bytes > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES
    {
        return Err(AutomatonCostLimit::MaterializedBytes);
    }
    Ok(cost)
}

pub(crate) fn join_automaton_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    if !has_finite_language_shape(left) || !has_finite_language_shape(right) {
        return None;
    }
    let mut left = match bounded_automaton_operand(left) {
        Ok(Some(language)) => language,
        Ok(None) => return None,
        Err(provenance) => return Some(top_class_value_with_provenance(provenance)),
    };
    let right = match bounded_automaton_operand(right) {
        Ok(Some(language)) => language,
        Ok(None) => return None,
        Err(provenance) => return Some(top_class_value_with_provenance(provenance)),
    };
    if let Err(limit) = preflight_automaton_branch_values(
        left.values.iter().chain(&right.values).map(String::as_str),
    ) {
        return Some(top_class_value_with_provenance(limit.provenance()));
    }
    left.values.extend(right.values);
    Some(automaton_class_value_from_values(
        &left.values,
        Some(AbstractClassValueProvenanceV0::AutomatonJoin),
    ))
}

pub(crate) fn concatenate_automaton_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    if !has_finite_language_shape(left) || !has_finite_language_shape(right) {
        return None;
    }
    let left = match bounded_automaton_operand(left) {
        Ok(Some(language)) => language,
        Ok(None) => return None,
        Err(provenance) => return Some(top_class_value_with_provenance(provenance)),
    };
    let right = match bounded_automaton_operand(right) {
        Ok(Some(language)) => language,
        Ok(None) => return None,
        Err(provenance) => return Some(top_class_value_with_provenance(provenance)),
    };
    let values = match materialize_concatenated_values_with(&left, &right, |left, right| {
        let capacity = left.len().saturating_add(right.len());
        let mut value = String::with_capacity(capacity);
        value.push_str(left);
        value.push_str(right);
        value
    }) {
        Ok(values) => values,
        Err(limit) => return Some(top_class_value_with_provenance(limit.provenance())),
    };
    Some(automaton_class_value_from_values(
        &values,
        Some(AbstractClassValueProvenanceV0::AutomatonConcat),
    ))
}

pub(crate) fn finite_language_values(value: &AbstractClassValueV0) -> Option<Vec<String>> {
    bounded_finite_language(value)
        .ok()
        .flatten()
        .map(|language| language.values)
}

fn bounded_finite_language(
    value: &AbstractClassValueV0,
) -> Result<Option<BoundedFiniteLanguage>, AbstractClassValueProvenanceV0> {
    bounded_finite_language_with_operand_wall(value, false)
}

fn bounded_automaton_operand(
    value: &AbstractClassValueV0,
) -> Result<Option<BoundedFiniteLanguage>, AbstractClassValueProvenanceV0> {
    bounded_finite_language_with_operand_wall(value, true)
}

fn has_finite_language_shape(value: &AbstractClassValueV0) -> bool {
    matches!(
        value,
        AbstractClassValueV0::Bottom
            | AbstractClassValueV0::Exact { .. }
            | AbstractClassValueV0::FiniteSet { .. }
            | AbstractClassValueV0::Automaton { .. }
    )
}

fn bounded_finite_language_with_operand_wall(
    value: &AbstractClassValueV0,
    enforce_operand_wall: bool,
) -> Result<Option<BoundedFiniteLanguage>, AbstractClassValueProvenanceV0> {
    match value {
        AbstractClassValueV0::Bottom => Ok(Some(BoundedFiniteLanguage {
            values: Vec::new(),
            cost: FiniteLanguageCost {
                cardinality: 0,
                materialized_bytes: 0,
            },
        })),
        AbstractClassValueV0::Exact { value } => {
            let input = preflight_automaton_input(std::iter::once(value.as_str()))
                .map_err(AutomatonCostLimit::provenance)?;
            if enforce_operand_wall
                && input.occurrence_bytes > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES
            {
                return Err(AbstractClassValueProvenanceV0::AutomatonMaterializedByteLimit);
            }
            Ok(Some(BoundedFiniteLanguage {
                values: vec![value.clone()],
                cost: input.unique,
            }))
        }
        AbstractClassValueV0::FiniteSet { values } => {
            let input = preflight_automaton_input(values.iter().map(String::as_str))
                .map_err(AutomatonCostLimit::provenance)?;
            if enforce_operand_wall
                && input.occurrence_bytes > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES
            {
                return Err(AbstractClassValueProvenanceV0::AutomatonMaterializedByteLimit);
            }
            Ok(Some(BoundedFiniteLanguage {
                values: normalize_automaton_values(values),
                cost: input.unique,
            }))
        }
        AbstractClassValueV0::Automaton { automaton, .. } => {
            if automaton.accept_states.len() > automaton.state_count {
                return Ok(None);
            }
            preflight_automaton_structure(automaton)?;
            if !automaton_is_well_formed_acyclic(automaton) {
                return Ok(None);
            }
            let Some(cost) = bounded_automaton_language_metadata(automaton)
                .map_err(AutomatonCostLimit::provenance)?
            else {
                return Ok(None);
            };
            let Some(values) = accepted_strings_from_automaton(automaton, cost.cardinality) else {
                return Err(AbstractClassValueProvenanceV0::AutomatonLanguageCardinalityLimit);
            };
            Ok(Some(BoundedFiniteLanguage { values, cost }))
        }
        _ => Ok(None),
    }
}

fn bounded_automaton_language_metadata(
    automaton: &AbstractStringAutomatonV0,
) -> Result<Option<FiniteLanguageCost>, AutomatonCostLimit> {
    let transitions = transitions_by_state(automaton);
    let accept_states = automaton
        .accept_states
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let Some(cost) = bounded_language_metadata_from_state(
        automaton.start_state,
        &transitions,
        &accept_states,
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    ) else {
        return Ok(None);
    };
    if cost.cardinality > MAX_STRING_AUTOMATON_LANGUAGE_CARDINALITY {
        return Err(AutomatonCostLimit::LanguageCardinality);
    }
    if cost.materialized_bytes > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES {
        return Err(AutomatonCostLimit::MaterializedBytes);
    }
    Ok(Some(cost))
}

fn bounded_language_metadata_from_state(
    state: usize,
    transitions: &BTreeMap<usize, BTreeMap<char, usize>>,
    accept_states: &BTreeSet<usize>,
    visiting: &mut BTreeSet<usize>,
    memo: &mut BTreeMap<usize, FiniteLanguageCost>,
) -> Option<FiniteLanguageCost> {
    if let Some(cost) = memo.get(&state) {
        return Some(*cost);
    }
    if !visiting.insert(state) {
        return None;
    }

    let mut cost = FiniteLanguageCost {
        cardinality: usize::from(accept_states.contains(&state)),
        materialized_bytes: 0,
    };
    if let Some(edges) = transitions.get(&state) {
        for (symbol, next_state) in edges {
            let next_cost = bounded_language_metadata_from_state(
                *next_state,
                transitions,
                accept_states,
                visiting,
                memo,
            )?;
            cost.cardinality = cost.cardinality.saturating_add(next_cost.cardinality);

            let symbol_bytes = next_cost.cardinality.saturating_mul(symbol.len_utf8());
            let prefixed_bytes = next_cost.materialized_bytes.saturating_add(symbol_bytes);
            cost.materialized_bytes = cost.materialized_bytes.saturating_add(prefixed_bytes);
        }
    }
    visiting.remove(&state);
    memo.insert(state, cost);
    Some(cost)
}

fn concatenated_language_cost(
    left: FiniteLanguageCost,
    right: FiniteLanguageCost,
) -> Result<FiniteLanguageCost, AutomatonCostLimit> {
    let cardinality = left
        .cardinality
        .checked_mul(right.cardinality)
        .ok_or(AutomatonCostLimit::LanguageCardinality)?;
    if cardinality > MAX_STRING_AUTOMATON_LANGUAGE_CARDINALITY {
        return Err(AutomatonCostLimit::LanguageCardinality);
    }

    let repeated_left_bytes = left
        .materialized_bytes
        .checked_mul(right.cardinality)
        .ok_or(AutomatonCostLimit::MaterializedBytes)?;
    let repeated_right_bytes = right
        .materialized_bytes
        .checked_mul(left.cardinality)
        .ok_or(AutomatonCostLimit::MaterializedBytes)?;
    let materialized_bytes = repeated_left_bytes
        .checked_add(repeated_right_bytes)
        .ok_or(AutomatonCostLimit::MaterializedBytes)?;
    if materialized_bytes > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES {
        return Err(AutomatonCostLimit::MaterializedBytes);
    }

    Ok(FiniteLanguageCost {
        cardinality,
        materialized_bytes,
    })
}

fn materialize_concatenated_values_with(
    left: &BoundedFiniteLanguage,
    right: &BoundedFiniteLanguage,
    mut concatenate: impl FnMut(&str, &str) -> String,
) -> Result<Vec<String>, AutomatonCostLimit> {
    let cost = concatenated_language_cost(left.cost, right.cost)?;
    let mut values = Vec::with_capacity(cost.cardinality);
    for left_value in &left.values {
        for right_value in &right.values {
            values.push(concatenate(left_value, right_value));
        }
    }
    debug_assert_eq!(values.len(), cost.cardinality);
    debug_assert_eq!(
        values.iter().map(String::len).sum::<usize>(),
        cost.materialized_bytes
    );
    Ok(values)
}

#[cfg(test)]
pub(crate) fn concatenation_preflight_provenance_before_materialization_for_test(
    left: &[String],
    right: &[String],
) -> Option<(Option<AbstractClassValueProvenanceV0>, bool)> {
    let Ok(Some(left)) = bounded_automaton_operand(&AbstractClassValueV0::FiniteSet {
        values: left.to_vec(),
    }) else {
        return None;
    };
    let Ok(Some(right)) = bounded_automaton_operand(&AbstractClassValueV0::FiniteSet {
        values: right.to_vec(),
    }) else {
        return None;
    };
    let materialization_ran = std::cell::Cell::new(false);
    let result = materialize_concatenated_values_with(&left, &right, |_, _| {
        materialization_ran.set(true);
        String::new()
    });
    Some((
        result.err().map(AutomatonCostLimit::provenance),
        materialization_ran.get(),
    ))
}

#[cfg(test)]
pub(crate) fn accepted_strings_with_output_bound_for_test(
    automaton: &AbstractStringAutomatonV0,
    output_bound: usize,
) -> Option<Vec<String>> {
    accepted_strings_from_automaton(automaton, output_bound)
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

fn preflight_automaton_structure(
    automaton: &AbstractStringAutomatonV0,
) -> Result<(), AbstractClassValueProvenanceV0> {
    if automaton.state_count > MAX_STRING_AUTOMATON_STATES {
        return Err(AbstractClassValueProvenanceV0::AutomatonStateLimit);
    }
    if automaton.transitions.len() > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES {
        return Err(AbstractClassValueProvenanceV0::AutomatonMaterializedByteLimit);
    }
    let mut symbol_bytes = 0_usize;
    for transition in &automaton.transitions {
        symbol_bytes = symbol_bytes
            .checked_add(transition.symbol.len())
            .ok_or(AbstractClassValueProvenanceV0::AutomatonMaterializedByteLimit)?;
        if symbol_bytes > MAX_STRING_AUTOMATON_MATERIALIZED_BYTES {
            return Err(AbstractClassValueProvenanceV0::AutomatonMaterializedByteLimit);
        }
    }
    Ok(())
}

pub(crate) fn automaton_is_well_formed_acyclic(automaton: &AbstractStringAutomatonV0) -> bool {
    if automaton.state_count == 0
        || automaton.start_state >= automaton.state_count
        || automaton.accept_states.len() > automaton.state_count
        || preflight_automaton_structure(automaton).is_err()
    {
        return false;
    }
    let accept_states = automaton
        .accept_states
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if accept_states.len() != automaton.accept_states.len()
        || accept_states
            .iter()
            .any(|state| *state >= automaton.state_count)
    {
        return false;
    }

    let mut transition_keys = BTreeSet::new();
    let mut adjacency = vec![Vec::new(); automaton.state_count];
    for transition in &automaton.transitions {
        let Some(symbol) = single_transition_symbol(&transition.symbol) else {
            return false;
        };
        if transition.from >= automaton.state_count
            || transition.to >= automaton.state_count
            || !transition_keys.insert((transition.from, symbol))
        {
            return false;
        }
        adjacency[transition.from].push(transition.to);
    }

    let mut colors = vec![0_u8; automaton.state_count];
    (0..automaton.state_count)
        .all(|state| colors[state] != 0 || visit_acyclic_state(state, &adjacency, &mut colors))
}

fn visit_acyclic_state(state: usize, adjacency: &[Vec<usize>], colors: &mut [u8]) -> bool {
    colors[state] = 1;
    for next_state in &adjacency[state] {
        match colors[*next_state] {
            0 => {
                if !visit_acyclic_state(*next_state, adjacency, colors) {
                    return false;
                }
            }
            1 => return false,
            _ => {}
        }
    }
    colors[state] = 2;
    true
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

fn accepted_strings_from_automaton(
    automaton: &AbstractStringAutomatonV0,
    output_bound: usize,
) -> Option<Vec<String>> {
    let transitions = transitions_by_state(automaton);
    let accept_states = automaton
        .accept_states
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let productive_states = productive_automaton_states(&transitions, &accept_states);
    let mut collector = AcceptedStringCollector {
        transitions: &transitions,
        accept_states: &accept_states,
        productive_states: &productive_states,
        output_bound,
        visiting: BTreeSet::new(),
        output: BTreeSet::new(),
    };
    collector.collect(automaton.start_state, String::new())?;
    Some(collector.output.into_iter().collect())
}

fn productive_automaton_states(
    transitions: &BTreeMap<usize, BTreeMap<char, usize>>,
    accept_states: &BTreeSet<usize>,
) -> BTreeSet<usize> {
    let mut productive_states = accept_states.clone();
    loop {
        let previous_count = productive_states.len();
        for (state, edges) in transitions {
            if edges
                .values()
                .any(|next_state| productive_states.contains(next_state))
            {
                productive_states.insert(*state);
            }
        }
        if productive_states.len() == previous_count {
            return productive_states;
        }
    }
}

struct AcceptedStringCollector<'a> {
    transitions: &'a BTreeMap<usize, BTreeMap<char, usize>>,
    accept_states: &'a BTreeSet<usize>,
    productive_states: &'a BTreeSet<usize>,
    output_bound: usize,
    visiting: BTreeSet<usize>,
    output: BTreeSet<String>,
}

impl AcceptedStringCollector<'_> {
    fn collect(&mut self, state: usize, prefix: String) -> Option<()> {
        if !self.productive_states.contains(&state) {
            return Some(());
        }
        if !self.visiting.insert(state) {
            return None;
        }
        if self.accept_states.contains(&state) {
            if !self.output.contains(&prefix) && self.output.len() >= self.output_bound {
                return None;
            }
            self.output.insert(prefix.clone());
        }
        if let Some(edges) = self.transitions.get(&state) {
            for (symbol, next_state) in edges {
                if !self.productive_states.contains(next_state) {
                    continue;
                }
                let mut next_prefix = prefix.clone();
                next_prefix.push(*symbol);
                self.collect(*next_state, next_prefix)?;
            }
        }
        self.visiting.remove(&state);
        Some(())
    }
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
