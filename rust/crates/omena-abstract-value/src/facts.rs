use std::collections::BTreeSet;

use crate::automaton::finite_language_values;
use crate::domain::{
    composite_min_length_for_constraints, prefix_suffix_min_length, prefix_suffix_overlap_len,
};
use crate::*;

/// Measures one Rust string in the unit carried by
/// [`ExternalStringTypeFactsV0`]. This is the single concrete-string
/// measurement authority for that boundary; callers reuse the measured value
/// for both bounds.
pub fn external_utf16_code_unit_length(value: &str) -> Utf16CodeUnitLengthV0 {
    value.encode_utf16().count()
}

/// Projects the byte-based lower bound of an internal abstract value into a
/// sound UTF-16-code-unit lower bound for the external contract.
///
/// A byte count cannot be converted exactly without the represented strings.
/// This projection retains the shape's overlap-aware minimum and applies the
/// universal `utf8_bytes <= 3 * utf16_code_units` inequality only to byte
/// length beyond that structural witness. It may weaken a bound, but it never
/// turns an internal member into an external rejection.
pub fn external_utf16_code_unit_min_length_lower_bound(
    value: &AbstractClassValueV0,
) -> Option<Utf16CodeUnitLengthV0> {
    match value {
        AbstractClassValueV0::PrefixSuffix {
            prefix,
            suffix,
            min_length,
            ..
        } => Some(external_utf16_lower_bound_from_byte_minimum(
            prefix_suffix_min_length(prefix, suffix),
            prefix_suffix_utf16_code_unit_min_length(prefix, suffix),
            *min_length,
        )),
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            ..
        } => {
            let prefix = prefix.as_deref().unwrap_or("");
            let suffix = suffix.as_deref().unwrap_or("");
            let structural_utf16_minimum =
                composite_utf16_code_unit_min_length(prefix, suffix, must_chars);
            let structural_byte_minimum =
                composite_min_length_for_constraints(prefix, suffix, must_chars);
            Some(external_utf16_lower_bound_from_byte_minimum(
                structural_byte_minimum,
                structural_utf16_minimum,
                min_length.unwrap_or(structural_byte_minimum),
            ))
        }
        _ => None,
    }
}

pub fn reduced_abstract_class_value_from_facts(
    facts: &ExternalStringTypeFactsV0,
) -> AbstractClassValueV0 {
    reduce_abstract_class_value_with_steps(facts).0
}

pub fn reduced_class_value_derivation_from_facts(
    facts: &ExternalStringTypeFactsV0,
) -> ReducedClassValueDerivationV0 {
    let (value, steps) = reduce_abstract_class_value_with_steps(facts);

    ReducedClassValueDerivationV0 {
        schema_version: "0",
        product: "omena-abstract-value.reduced-class-value-derivation",
        input_fact_kind: facts.kind.clone(),
        input_constraint_kind: facts.constraint_kind.clone(),
        input_value_count: finite_value_count_for_facts(facts),
        reduced_kind: reduced_class_value_kind(facts, &value),
        steps,
    }
}

fn reduce_abstract_class_value_with_steps(
    facts: &ExternalStringTypeFactsV0,
) -> (AbstractClassValueV0, Vec<ReducedClassValueDerivationStepV0>) {
    let mut value = abstract_class_value_from_facts(facts);
    let mut steps = vec![ReducedClassValueDerivationStepV0 {
        operation: "baseFromFacts",
        input_kind: None,
        refinement_kind: None,
        result_kind: abstract_class_value_kind(&value),
        result_provenance: abstract_class_value_provenance(&value),
        reason: "mapped input facts to the base abstract value",
    }];

    if facts_have_constraint_details(facts) && matches!(facts.kind.as_str(), "exact" | "finiteSet")
    {
        let refinement = constrained_class_value_from_facts(facts);
        let result = intersect_abstract_class_values(&value, &refinement);
        steps.push(ReducedClassValueDerivationStepV0 {
            operation: "intersectConstraint",
            input_kind: Some(abstract_class_value_kind(&value)),
            refinement_kind: Some(abstract_class_value_kind(&refinement)),
            result_kind: abstract_class_value_kind(&result),
            result_provenance: abstract_class_value_provenance(&result),
            reason: "refined exact or finite facts with constraint details",
        });
        value = result;
    }

    if !matches!(facts.kind.as_str(), "exact" | "finiteSet")
        && let Some(values) = facts.values.as_ref().filter(|values| !values.is_empty())
    {
        let refinement = finite_set_class_value(
            values
                .iter()
                .filter(|value| value_matches_external_length_bounds(value, facts))
                .cloned()
                .collect::<Vec<_>>(),
        );
        let result = intersect_abstract_class_values(&value, &refinement);
        steps.push(ReducedClassValueDerivationStepV0 {
            operation: "intersectFiniteValues",
            input_kind: Some(abstract_class_value_kind(&value)),
            refinement_kind: Some(abstract_class_value_kind(&refinement)),
            result_kind: abstract_class_value_kind(&result),
            result_provenance: abstract_class_value_provenance(&result),
            reason: "refined constrained facts with explicit finite values",
        });
        value = result;
    }

    (value, steps)
}

pub fn reduced_value_domain_kind_from_facts(facts: &ExternalStringTypeFactsV0) -> &'static str {
    if facts.kind == "unknown" {
        return "none";
    }

    abstract_class_value_kind(&reduced_abstract_class_value_from_facts(facts))
}

fn reduced_class_value_kind(
    facts: &ExternalStringTypeFactsV0,
    value: &AbstractClassValueV0,
) -> &'static str {
    if facts.kind == "unknown" {
        return "none";
    }

    abstract_class_value_kind(value)
}

pub fn abstract_class_value_from_facts(facts: &ExternalStringTypeFactsV0) -> AbstractClassValueV0 {
    match facts.kind.as_str() {
        "exact" => facts
            .values
            .as_ref()
            .and_then(|values| values.first())
            .map_or_else(top_class_value, |value| {
                if value_matches_external_length_bounds(value, facts) {
                    exact_class_value(value.clone())
                } else {
                    bottom_class_value()
                }
            }),
        "finiteSet" => finite_set_class_value(
            facts
                .values
                .as_ref()
                .map(|values| {
                    values
                        .iter()
                        .filter(|value| value_matches_external_length_bounds(value, facts))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        ),
        "constrained" => constrained_class_value_from_facts(facts),
        "unknown" | "top" => top_class_value(),
        _ => top_class_value(),
    }
}

/// Lower an abstract class value back into the external-string-type facts that
/// the flow analyser consumes as an `AssignFacts` transfer.
///
/// This sound external lowering lets a product-path caller seed a
/// [`crate::ClassValueFlowGraphV0`] node from an already-computed abstract value
/// (for example, a dynamic-className call-site exit value). A byte-only lower
/// bound cannot be inverted into UTF-16 exactly, so a round trip may weaken that
/// bound while retaining the value's structural constraints.
pub fn external_string_type_facts_from_abstract_class_value(
    value: &AbstractClassValueV0,
) -> ExternalStringTypeFactsV0 {
    let mut facts = empty_external_string_type_facts("top");
    let external_min_len = external_utf16_code_unit_min_length_lower_bound(value);
    match value {
        AbstractClassValueV0::Bottom => {
            facts.kind = "finiteSet".to_string();
            facts.values = Some(Vec::new());
        }
        AbstractClassValueV0::Exact { value } => {
            facts.kind = "exact".to_string();
            facts.values = Some(vec![value.clone()]);
        }
        AbstractClassValueV0::FiniteSet { values } => {
            facts.kind = "finiteSet".to_string();
            facts.values = Some(values.clone());
        }
        AbstractClassValueV0::Automaton { .. } => {
            facts.kind = "finiteSet".to_string();
            facts.values = finite_language_values(value);
        }
        AbstractClassValueV0::Prefix { prefix, .. } => {
            facts.kind = "constrained".to_string();
            facts.constraint_kind = Some("prefix".to_string());
            facts.prefix = Some(prefix.clone());
        }
        AbstractClassValueV0::Suffix { suffix, .. } => {
            facts.kind = "constrained".to_string();
            facts.constraint_kind = Some("suffix".to_string());
            facts.suffix = Some(suffix.clone());
        }
        AbstractClassValueV0::PrefixSuffix { prefix, suffix, .. } => {
            facts.kind = "constrained".to_string();
            facts.constraint_kind = Some("prefixSuffix".to_string());
            facts.prefix = Some(prefix.clone());
            facts.suffix = Some(suffix.clone());
            facts.min_len = external_min_len;
        }
        AbstractClassValueV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => {
            facts.kind = "constrained".to_string();
            facts.constraint_kind = Some("charInclusion".to_string());
            facts.char_must = Some(must_chars.clone());
            facts.char_may = Some(may_chars.clone());
            facts.may_include_other_chars = Some(*may_include_other_chars);
        }
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => {
            facts.kind = "constrained".to_string();
            facts.constraint_kind = Some("composite".to_string());
            facts.prefix = prefix.clone();
            facts.suffix = suffix.clone();
            facts.min_len = external_min_len;
            facts.char_must = Some(must_chars.clone());
            facts.char_may = Some(may_chars.clone());
            facts.may_include_other_chars = Some(*may_include_other_chars);
        }
        AbstractClassValueV0::Top { .. } => {
            facts.kind = "top".to_string();
        }
    }
    facts
}

fn empty_external_string_type_facts(kind: impl Into<String>) -> ExternalStringTypeFactsV0 {
    ExternalStringTypeFactsV0 {
        kind: kind.into(),
        constraint_kind: None,
        values: None,
        prefix: None,
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
    }
}

pub fn expression_value_domain_kind_from_facts(facts: &ExternalStringTypeFactsV0) -> String {
    match facts.kind.as_str() {
        "unknown" => "none".to_string(),
        other => other.to_string(),
    }
}

pub fn value_certainty_from_facts(facts: &ExternalStringTypeFactsV0) -> Option<&'static str> {
    match facts.kind.as_str() {
        "exact" => Some("exact"),
        "finiteSet" | "constrained" => Some("inferred"),
        "unknown" | "top" => Some("possible"),
        _ => None,
    }
}

pub fn value_certainty_shape_kind_from_facts(facts: &ExternalStringTypeFactsV0) -> &'static str {
    match facts.kind.as_str() {
        "exact" => "exact",
        "finiteSet" => "boundedFinite",
        "constrained" => "constrained",
        _ => "unknown",
    }
}

pub fn value_certainty_shape_label_from_facts(facts: &ExternalStringTypeFactsV0) -> String {
    match value_certainty_from_facts(facts) {
        Some("exact") => "exact".to_string(),
        Some("possible") | None => "unknown".to_string(),
        Some("inferred") => match facts.kind.as_str() {
            "finiteSet" => format!("bounded finite ({})", finite_value_count_for_facts(facts)),
            "constrained" => constrained_value_shape_label_from_facts(facts),
            _ => "unknown".to_string(),
        },
        _ => "unknown".to_string(),
    }
}

pub fn selector_certainty_from_facts(
    facts: &ExternalStringTypeFactsV0,
    matched_selector_count: usize,
    _selector_universe_count: usize,
) -> &'static str {
    match facts.kind.as_str() {
        "unknown" => "possible",
        "exact" if matched_selector_count == 1 => "exact",
        "exact" => "possible",
        "finiteSet" => {
            let finite_value_count = finite_value_count_for_facts(facts);
            if finite_value_count == 0 || matched_selector_count == 0 {
                "possible"
            } else if matched_selector_count == finite_value_count {
                "exact"
            } else {
                "inferred"
            }
        }
        "constrained" => {
            if matched_selector_count == 0 {
                "possible"
            } else {
                "inferred"
            }
        }
        "top" => "possible",
        _ => "possible",
    }
}

pub fn selector_certainty_shape_kind_from_facts(
    facts: &ExternalStringTypeFactsV0,
    matched_selector_count: usize,
    selector_universe_count: usize,
) -> &'static str {
    match selector_certainty_from_facts(facts, matched_selector_count, selector_universe_count) {
        "exact" => "exact",
        "possible" => "unknown",
        "inferred" => {
            if is_constrained_selector_shape(facts) {
                "constrained"
            } else {
                "boundedFinite"
            }
        }
        _ => "unknown",
    }
}

pub fn selector_certainty_shape_label_from_facts(
    facts: &ExternalStringTypeFactsV0,
    matched_selector_count: usize,
    selector_universe_count: usize,
) -> String {
    match selector_certainty_from_facts(facts, matched_selector_count, selector_universe_count) {
        "exact" => "exact".to_string(),
        "possible" => "unknown".to_string(),
        "inferred" => match facts.constraint_kind.as_deref() {
            Some("prefix") => {
                format!("constrained prefix selector set ({matched_selector_count})")
            }
            Some("suffix") => {
                format!("constrained suffix selector set ({matched_selector_count})")
            }
            Some("prefixSuffix") => {
                format!("constrained edge selector set ({matched_selector_count})")
            }
            Some("charInclusion") => {
                format!("constrained character selector set ({matched_selector_count})")
            }
            Some("composite") => {
                format!("constrained composite selector set ({matched_selector_count})")
            }
            _ => format!("bounded selector set ({matched_selector_count})"),
        },
        _ => "unknown".to_string(),
    }
}

pub fn finite_values_from_facts(facts: &ExternalStringTypeFactsV0) -> Option<Vec<String>> {
    match facts.kind.as_str() {
        "exact" | "finiteSet" => facts.values.clone(),
        _ => None,
    }
}

fn facts_have_constraint_details(facts: &ExternalStringTypeFactsV0) -> bool {
    facts.constraint_kind.is_some()
        || facts.prefix.is_some()
        || facts.suffix.is_some()
        || facts.min_len.is_some()
        || facts.max_len.is_some()
        || facts.char_must.is_some()
        || facts.char_may.is_some()
        || facts.may_include_other_chars.is_some()
}

fn constrained_class_value_from_facts(facts: &ExternalStringTypeFactsV0) -> AbstractClassValueV0 {
    // An arbitrary UTF-16 scalar bound cannot be represented exactly by the
    // byte-only internal domain. Because every UTF-8 string has at least as
    // many bytes as UTF-16 code units, reusing the numeric lower bound here is
    // a sound over-approximation. The exact check stays at the external
    // concrete-string boundary. The byte domain has no max-length axis, so the
    // external maximum is deliberately not lowered into it.
    let sound_byte_minimum = facts
        .min_len
        .map(sound_internal_byte_minimum_from_external_utf16_minimum);
    match facts.constraint_kind.as_deref() {
        Some("prefix") => prefix_class_value(facts.prefix.clone().unwrap_or_default(), None),
        Some("suffix") => suffix_class_value(facts.suffix.clone().unwrap_or_default(), None),
        Some("prefixSuffix") => prefix_suffix_class_value(
            facts.prefix.clone().unwrap_or_default(),
            facts.suffix.clone().unwrap_or_default(),
            sound_byte_minimum,
            None,
        ),
        Some("charInclusion") => char_inclusion_class_value(
            facts.char_must.clone().unwrap_or_default(),
            facts.char_may.clone().unwrap_or_default(),
            None,
            facts.may_include_other_chars.unwrap_or(false),
        ),
        Some("composite") => composite_class_value(CompositeClassValueInputV0 {
            prefix: facts.prefix.clone(),
            suffix: facts.suffix.clone(),
            min_length: sound_byte_minimum,
            must_chars: facts.char_must.clone().unwrap_or_default(),
            may_chars: facts.char_may.clone().unwrap_or_default(),
            may_include_other_chars: facts.may_include_other_chars.unwrap_or(false),
            provenance: None,
        }),
        _ => top_class_value(),
    }
}

fn sound_internal_byte_minimum_from_external_utf16_minimum(
    minimum: Utf16CodeUnitLengthV0,
) -> Utf8ByteLengthV0 {
    // For every valid string, UTF-8 bytes >= UTF-16 code units. Reusing the
    // numeric lower bound therefore cannot reject an external member, although
    // it may admit additional non-ASCII values in the byte-only domain.
    minimum
}

fn value_matches_external_length_bounds(value: &str, facts: &ExternalStringTypeFactsV0) -> bool {
    let length = external_utf16_code_unit_length(value);
    facts.min_len.is_none_or(|minimum| length >= minimum)
        && facts.max_len.is_none_or(|maximum| length <= maximum)
}

fn prefix_suffix_utf16_code_unit_min_length(prefix: &str, suffix: &str) -> usize {
    let overlap = prefix_suffix_overlap_len(prefix, suffix);
    external_utf16_code_unit_length(prefix) + external_utf16_code_unit_length(&suffix[overlap..])
}

fn external_utf16_lower_bound_from_byte_minimum(
    structural_byte_minimum: usize,
    structural_utf16_minimum: usize,
    byte_minimum: usize,
) -> usize {
    structural_utf16_minimum.saturating_add(
        byte_minimum
            .saturating_sub(structural_byte_minimum)
            .div_ceil(3),
    )
}

fn composite_utf16_code_unit_min_length(prefix: &str, suffix: &str, must_chars: &str) -> usize {
    let overlap = prefix_suffix_overlap_len(prefix, suffix);
    let edge = format!("{prefix}{}", &suffix[overlap..]);
    let edge_chars = edge.chars().collect::<BTreeSet<_>>();
    let missing_required_chars = must_chars
        .chars()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|character| !edge_chars.contains(character))
        .collect::<Vec<_>>();

    if missing_required_chars.is_empty() {
        return external_utf16_code_unit_length(&edge);
    }

    external_utf16_code_unit_length(prefix)
        + external_utf16_code_unit_length(suffix)
        + missing_required_chars
            .into_iter()
            .map(char::len_utf16)
            .sum::<usize>()
}

fn finite_value_count_for_facts(facts: &ExternalStringTypeFactsV0) -> usize {
    facts
        .values
        .as_ref()
        .map(|values| values.iter().collect::<BTreeSet<_>>().len())
        .unwrap_or(0)
}

fn abstract_class_value_provenance(
    value: &AbstractClassValueV0,
) -> Option<AbstractClassValueProvenanceV0> {
    match value {
        AbstractClassValueV0::Prefix { provenance, .. }
        | AbstractClassValueV0::Suffix { provenance, .. }
        | AbstractClassValueV0::PrefixSuffix { provenance, .. }
        | AbstractClassValueV0::CharInclusion { provenance, .. }
        | AbstractClassValueV0::Composite { provenance, .. }
        | AbstractClassValueV0::Automaton { provenance, .. } => *provenance,
        AbstractClassValueV0::Bottom
        | AbstractClassValueV0::Exact { .. }
        | AbstractClassValueV0::FiniteSet { .. }
        | AbstractClassValueV0::Top { .. } => None,
    }
}

fn constrained_value_shape_label_from_facts(facts: &ExternalStringTypeFactsV0) -> String {
    match facts.constraint_kind.as_deref() {
        Some("prefix") => {
            format!(
                "constrained prefix `{}`",
                facts.prefix.as_deref().unwrap_or("")
            )
        }
        Some("suffix") => {
            format!(
                "constrained suffix `{}`",
                facts.suffix.as_deref().unwrap_or("")
            )
        }
        Some("prefixSuffix") => format!(
            "constrained prefix `{}` + suffix `{}`",
            facts.prefix.as_deref().unwrap_or(""),
            facts.suffix.as_deref().unwrap_or("")
        ),
        Some("charInclusion") => format!(
            "constrained character inclusion ({})",
            facts.char_must.as_deref().unwrap_or("none")
        ),
        Some("composite") => "constrained composite".to_string(),
        _ => "unknown".to_string(),
    }
}

fn is_constrained_selector_shape(facts: &ExternalStringTypeFactsV0) -> bool {
    matches!(
        facts.constraint_kind.as_deref(),
        Some("prefix" | "suffix" | "prefixSuffix" | "charInclusion" | "composite")
    )
}
