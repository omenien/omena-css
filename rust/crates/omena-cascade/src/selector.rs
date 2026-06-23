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
            verdict: SelectorMatchVerdict::Yes,
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
            verdict: SelectorMatchVerdict::Yes,
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
                verdict: SelectorMatchVerdict::Yes,
                matched: true,
                rank: 3,
                declaration_selector: Some(declaration_selector.to_string()),
                reference_selector: Some(reference_selector.clone()),
            };
        }
    }

    let mut approximate_reference_selector = None;
    for reference_selector in reference_selectors {
        match selector_context_component_verdict(reference_selector, declaration_selector) {
            SelectorMatchVerdict::Yes => {
                return SelectorContextWitness {
                    kind: SelectorContextMatchKind::ContainsSelector,
                    verdict: SelectorMatchVerdict::Yes,
                    matched: true,
                    rank: 2,
                    declaration_selector: Some(declaration_selector.to_string()),
                    reference_selector: Some(reference_selector.clone()),
                };
            }
            SelectorMatchVerdict::Maybe => {
                approximate_reference_selector.get_or_insert_with(|| reference_selector.clone());
            }
            SelectorMatchVerdict::No => {}
        }
    }

    if let Some(reference_selector) = approximate_reference_selector {
        return SelectorContextWitness {
            kind: SelectorContextMatchKind::ApproximateSelector,
            verdict: SelectorMatchVerdict::Maybe,
            matched: true,
            rank: 1,
            declaration_selector: Some(declaration_selector.to_string()),
            reference_selector: Some(reference_selector),
        };
    }

    SelectorContextWitness {
        kind: SelectorContextMatchKind::NoMatch,
        verdict: SelectorMatchVerdict::No,
        matched: false,
        rank: 0,
        declaration_selector: Some(declaration_selector.to_string()),
        reference_selector: None,
    }
}

fn selector_context_component_verdict(
    reference_selector: &str,
    declaration_selector: &str,
) -> SelectorMatchVerdict {
    if parse_simple_selector_signature(declaration_selector).is_none() {
        return SelectorMatchVerdict::Maybe;
    }

    let branches = split_selector_list(reference_selector);
    if branches.is_empty() {
        return SelectorMatchVerdict::Maybe;
    }

    let mut saw_unmodeled_component = false;
    for branch in branches {
        let components = split_complex_selector_components(&branch);
        if components.is_empty() {
            saw_unmodeled_component = true;
            continue;
        }
        for component in components {
            if component == declaration_selector {
                return SelectorMatchVerdict::Yes;
            }
            if parse_simple_selector_signature(&component).is_none() {
                saw_unmodeled_component = true;
            }
        }
    }

    if saw_unmodeled_component {
        SelectorMatchVerdict::Maybe
    } else {
        SelectorMatchVerdict::No
    }
}

fn split_complex_selector_components(selector: &str) -> Vec<String> {
    let mut components = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let chars = selector.char_indices().collect::<Vec<_>>();

    for (index, ch) in &chars {
        match *ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '>' | '+' | '~' if paren_depth == 0 && bracket_depth == 0 => {
                push_selector_component(selector, start, *index, &mut components);
                start = *index + ch.len_utf8();
            }
            ch if ch.is_whitespace() && paren_depth == 0 && bracket_depth == 0 => {
                push_selector_component(selector, start, *index, &mut components);
                start = *index + ch.len_utf8();
            }
            _ => {}
        }
    }

    push_selector_component(selector, start, selector.len(), &mut components);
    components
}

fn push_selector_component(selector: &str, start: usize, end: usize, components: &mut Vec<String>) {
    let component = selector[start..end].trim();
    if !component.is_empty() {
        components.push(component.to_string());
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

pub fn selector_co_match_verdict(
    left_selector: &str,
    right_selector: &str,
) -> SelectorMatchVerdict {
    match (
        parse_simple_selector_signature(left_selector),
        parse_simple_selector_signature(right_selector),
    ) {
        (Some(left), Some(right)) => selector_signature_co_match_verdict(&left, &right),
        _ => SelectorMatchVerdict::Maybe,
    }
}

pub fn selector_signature_co_match_verdict(
    left: &SelectorSignature,
    right: &SelectorSignature,
) -> SelectorMatchVerdict {
    if left
        .required_tag
        .as_ref()
        .zip(right.required_tag.as_ref())
        .is_some_and(|(left, right)| left != right)
    {
        return SelectorMatchVerdict::No;
    }
    if left
        .required_id
        .as_ref()
        .zip(right.required_id.as_ref())
        .is_some_and(|(left, right)| left != right)
    {
        return SelectorMatchVerdict::No;
    }
    if selector_signature_has_lossy_co_match_constraints(left)
        || selector_signature_has_lossy_co_match_constraints(right)
    {
        return SelectorMatchVerdict::Maybe;
    }

    SelectorMatchVerdict::Yes
}

fn selector_signature_has_lossy_co_match_constraints(signature: &SelectorSignature) -> bool {
    selector_has_attribute_value_or_modifier(signature.selector.as_str())
        || selector_has_functional_pseudo(signature.selector.as_str())
        || selector_has_pseudo_element(signature.selector.as_str())
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
                        // Functional pseudo-class, e.g. `:is(...)`, `:where(...)`, `:not(...)`.
                        // Parse the argument selector list and fold its specificity in per
                        // Selectors L4 §15 instead of bailing out of the whole selector.
                        let close = find_closing_paren(&chars, next)?;
                        let arguments = chars[next + 1..close].iter().collect::<String>();
                        let argument_specificity =
                            functional_pseudo_specificity(name.as_str(), arguments.as_str())?;
                        specificity.ids += argument_specificity.ids;
                        specificity.classes += argument_specificity.classes;
                        specificity.elements += argument_specificity.elements;
                        required_pseudo_states.insert(name);
                        index = close + 1;
                    } else {
                        specificity.classes += 1;
                        required_pseudo_states.insert(name);
                        index = next;
                    }
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

fn find_closing_paren(chars: &[char], open_index: usize) -> Option<usize> {
    let mut depth: usize = 0;
    for (index, ch) in chars.iter().enumerate().skip(open_index) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

/// Specificity contribution of a functional pseudo-class per Selectors L4.
///
/// `:is()`/`:not()`/`:has()` contribute the specificity of their most specific
/// argument; `:where()` always contributes zero. Unknown functional
/// pseudo-classes return `None` so the caller keeps treating them as unsupported
/// rather than guessing.
//
// NOTE: recognizing `:has` here also lets the whole signature parse (the rule is
// no longer dropped), which sharpens the co-match verdict for a tag-mismatched
// `:has` pair (e.g. `a:has(.x)` vs `b:has(.x)`) from `Maybe` to `No`. That is an
// intentional precision gain: it only drops a spurious proof-obligation candidate,
// never turns an unsafe pair into a safe one.
fn functional_pseudo_specificity(name: &str, arguments: &str) -> Option<Specificity> {
    match name.to_ascii_lowercase().as_str() {
        "where" => Some(Specificity::ZERO),
        "is" | "not" | "matches" | "has" => Some(most_specific_argument_specificity(arguments)),
        _ => None,
    }
}

/// Highest specificity among a comma-separated argument selector list. Arguments
/// that the conservative parser cannot model contribute `Specificity::ZERO`,
/// which keeps the estimate sound (never over-counts) without dropping the rule.
fn most_specific_argument_specificity(arguments: &str) -> Specificity {
    split_selector_list(arguments)
        .iter()
        .map(|argument| {
            parse_simple_selector_signature_inner(argument.trim())
                .map(|signature| signature.specificity)
                .unwrap_or(Specificity::ZERO)
        })
        .max()
        .unwrap_or(Specificity::ZERO)
}

fn read_attribute_name(attribute: &str) -> Option<String> {
    let name = attribute
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '=' | '~' | '|' | '^' | '$' | '*'))
        .find(|part| !part.is_empty())?;
    Some(name.to_string())
}

fn selector_has_attribute_value_or_modifier(selector: &str) -> bool {
    let chars = selector.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '[' {
            let Some(close) = find_closing_bracket(&chars, index) else {
                return true;
            };
            let attribute = chars[index + 1..close].iter().collect::<String>();
            if attribute
                .chars()
                .any(|ch| ch.is_whitespace() || matches!(ch, '=' | '~' | '|' | '^' | '$' | '*'))
            {
                return true;
            }
            index = close + 1;
        } else {
            index += 1;
        }
    }
    false
}

fn selector_has_functional_pseudo(selector: &str) -> bool {
    let chars = selector.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == ':' && !matches!(chars.get(index + 1), Some(':')) {
            index += 1;
            let Some((_, next)) = read_identifier(&chars, index) else {
                return true;
            };
            if matches!(chars.get(next), Some('(')) {
                return true;
            }
            index = next;
        } else {
            index += 1;
        }
    }
    false
}

fn selector_has_pseudo_element(selector: &str) -> bool {
    selector.contains("::")
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
