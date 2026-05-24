//! Selector-context witnesses for cascade-aware diagnostics and transforms.
//!
//! This module exposes conservative selector matching over explicit element
//! signatures. Unsupported selector branches become witness data instead of
//! silent acceptance so proof consumers can block unsafe rewrites.

use std::collections::BTreeSet;

use crate::{
    ElementSignature, SelectorContextMatchKind, SelectorContextWitness, SelectorMatchReason,
    SelectorMatchVerdict, SelectorMatchWitness, SelectorSignature, Specificity,
};

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
