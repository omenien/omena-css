use std::collections::BTreeSet;

use crate::*;

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
        let refinement = finite_set_class_value(values.clone());
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
            .map_or_else(top_class_value, |value| exact_class_value(value.clone())),
        "finiteSet" => finite_set_class_value(facts.values.clone().unwrap_or_default()),
        "constrained" => constrained_class_value_from_facts(facts),
        "unknown" | "top" => top_class_value(),
        _ => top_class_value(),
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
        || facts.char_must.is_some()
        || facts.char_may.is_some()
        || facts.may_include_other_chars.is_some()
}

fn constrained_class_value_from_facts(facts: &ExternalStringTypeFactsV0) -> AbstractClassValueV0 {
    match facts.constraint_kind.as_deref() {
        Some("prefix") => prefix_class_value(facts.prefix.clone().unwrap_or_default(), None),
        Some("suffix") => suffix_class_value(facts.suffix.clone().unwrap_or_default(), None),
        Some("prefixSuffix") => prefix_suffix_class_value(
            facts.prefix.clone().unwrap_or_default(),
            facts.suffix.clone().unwrap_or_default(),
            facts.min_len,
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
            min_length: facts.min_len,
            must_chars: facts.char_must.clone().unwrap_or_default(),
            may_chars: facts.char_may.clone().unwrap_or_default(),
            may_include_other_chars: facts.may_include_other_chars.unwrap_or(false),
            provenance: None,
        }),
        _ => top_class_value(),
    }
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
        | AbstractClassValueV0::Composite { provenance, .. } => *provenance,
        AbstractClassValueV0::Bottom
        | AbstractClassValueV0::Exact { .. }
        | AbstractClassValueV0::FiniteSet { .. }
        | AbstractClassValueV0::Top => None,
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
