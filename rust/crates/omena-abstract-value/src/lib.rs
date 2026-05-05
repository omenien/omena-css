use std::collections::BTreeSet;

mod facts;
mod flow;
mod types;

pub use facts::*;
pub use flow::*;
pub use types::*;

pub fn summarize_omena_abstract_value_domain() -> AbstractValueDomainSummaryV0 {
    AbstractValueDomainSummaryV0 {
        schema_version: "0",
        product: "omena-abstract-value.domain",
        domain_kinds: vec![
            "bottom",
            "exact",
            "finiteSet",
            "prefix",
            "suffix",
            "prefixSuffix",
            "charInclusion",
            "composite",
            "top",
        ],
        max_finite_class_values: MAX_FINITE_CLASS_VALUES,
        selector_projection_certainties: vec!["exact", "inferred", "possible"],
    }
}

pub fn summarize_omena_abstract_value_flow_analysis() -> AbstractValueFlowAnalysisSummaryV0 {
    AbstractValueFlowAnalysisSummaryV0 {
        schema_version: "0",
        product: "omena-abstract-value.flow-analysis",
        context_sensitivity: "1-cfa",
        incremental_engine: "omena-incremental",
        analysis_scopes: vec![
            "singleContext",
            "multiContextBatch",
            "callSiteBatch",
            "kLimitedCallSiteBatch",
            "controlFlowGraph",
        ],
        reuse_policy: "reuse previous context analysis when its omena-incremental plan is clean",
        transfer_kinds: vec!["assignFacts", "refineFacts", "concatFacts", "join"],
        max_iterations: MAX_FLOW_ANALYSIS_ITERATIONS,
    }
}

pub fn bottom_class_value() -> AbstractClassValueV0 {
    AbstractClassValueV0::Bottom
}

pub fn top_class_value() -> AbstractClassValueV0 {
    AbstractClassValueV0::Top
}

pub fn exact_class_value(value: impl Into<String>) -> AbstractClassValueV0 {
    AbstractClassValueV0::Exact {
        value: value.into(),
    }
}

pub fn finite_set_class_value<I, S>(values: I) -> AbstractClassValueV0
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let normalized = normalize_values(values);
    match normalized.len() {
        0 => bottom_class_value(),
        1 => exact_class_value(normalized[0].clone()),
        2..=MAX_FINITE_CLASS_VALUES => AbstractClassValueV0::FiniteSet { values: normalized },
        _ => widen_large_finite_set(&normalized),
    }
}

pub fn prefix_class_value(
    prefix: impl Into<String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    AbstractClassValueV0::Prefix {
        prefix: prefix.into(),
        provenance,
    }
}

pub fn suffix_class_value(
    suffix: impl Into<String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    AbstractClassValueV0::Suffix {
        suffix: suffix.into(),
        provenance,
    }
}

pub fn prefix_suffix_class_value(
    prefix: impl Into<String>,
    suffix: impl Into<String>,
    min_length: Option<usize>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    let prefix = prefix.into();
    let suffix = suffix.into();
    if prefix.is_empty() && suffix.is_empty() {
        return top_class_value();
    }
    if prefix.is_empty() {
        return suffix_class_value(suffix, provenance);
    }
    if suffix.is_empty() {
        return prefix_class_value(prefix, provenance);
    }

    AbstractClassValueV0::PrefixSuffix {
        min_length: min_length
            .unwrap_or(prefix.len() + suffix.len())
            .max(prefix.len() + suffix.len()),
        prefix,
        suffix,
        provenance,
    }
}

pub fn char_inclusion_class_value(
    must_chars: impl Into<String>,
    may_chars: impl Into<String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
    may_include_other_chars: bool,
) -> AbstractClassValueV0 {
    let must_chars = normalize_char_set(must_chars.into());
    let may_chars = normalize_char_set(format!("{}{}", may_chars.into(), must_chars));

    if may_include_other_chars && must_chars.is_empty() {
        return top_class_value();
    }
    if !may_include_other_chars && may_chars.is_empty() {
        return top_class_value();
    }

    AbstractClassValueV0::CharInclusion {
        must_chars,
        may_chars,
        may_include_other_chars,
        provenance,
    }
}

pub fn composite_class_value(input: CompositeClassValueInputV0) -> AbstractClassValueV0 {
    let prefix = input.prefix.unwrap_or_default();
    let suffix = input.suffix.unwrap_or_default();
    let edge_chars = char_set_for_string(format!("{prefix}{suffix}"));
    let must_chars = normalize_char_set(format!("{}{}", input.must_chars, edge_chars));
    let may_chars = normalize_char_set(format!("{}{}", input.may_chars, must_chars));
    let has_char_info =
        !must_chars.is_empty() || (!input.may_include_other_chars && !may_chars.is_empty());

    if !has_char_info {
        return prefix_suffix_class_value(prefix, suffix, input.min_length, input.provenance);
    }
    if prefix.is_empty() && suffix.is_empty() {
        return char_inclusion_class_value(
            must_chars,
            may_chars,
            input.provenance,
            input.may_include_other_chars,
        );
    }

    let guaranteed_distinct_char_count = must_chars.chars().count();
    let edge_min_length = prefix.len() + suffix.len();
    let min_length = input
        .min_length
        .map(|value| value.max(edge_min_length))
        .or(Some(edge_min_length))
        .map(|value| value.max(guaranteed_distinct_char_count));

    AbstractClassValueV0::Composite {
        prefix: (!prefix.is_empty()).then_some(prefix),
        suffix: (!suffix.is_empty()).then_some(suffix),
        min_length,
        must_chars,
        may_chars,
        may_include_other_chars: input.may_include_other_chars,
        provenance: input.provenance,
    }
}

pub fn enumerate_finite_class_values(value: &AbstractClassValueV0) -> Option<Vec<String>> {
    match value {
        AbstractClassValueV0::Bottom => Some(Vec::new()),
        AbstractClassValueV0::Exact { value } => Some(vec![value.clone()]),
        AbstractClassValueV0::FiniteSet { values } => Some(values.clone()),
        _ => None,
    }
}

pub fn abstract_class_value_kind(value: &AbstractClassValueV0) -> &'static str {
    match value {
        AbstractClassValueV0::Bottom => "bottom",
        AbstractClassValueV0::Exact { .. } => "exact",
        AbstractClassValueV0::FiniteSet { .. } => "finiteSet",
        AbstractClassValueV0::Prefix { .. } => "prefix",
        AbstractClassValueV0::Suffix { .. } => "suffix",
        AbstractClassValueV0::PrefixSuffix { .. } => "prefixSuffix",
        AbstractClassValueV0::CharInclusion { .. } => "charInclusion",
        AbstractClassValueV0::Composite { .. } => "composite",
        AbstractClassValueV0::Top => "top",
    }
}

pub fn intersect_abstract_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> AbstractClassValueV0 {
    match (left, right) {
        (AbstractClassValueV0::Bottom, _) | (_, AbstractClassValueV0::Bottom) => {
            bottom_class_value()
        }
        (AbstractClassValueV0::Top, value) | (value, AbstractClassValueV0::Top) => value.clone(),
        _ => intersect_non_top_class_values(left, right),
    }
}

pub fn join_abstract_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> AbstractClassValueV0 {
    if abstract_value_is_subset(left, right) {
        return right.clone();
    }
    if abstract_value_is_subset(right, left) {
        return left.clone();
    }

    match (
        enumerate_finite_class_values(left),
        enumerate_finite_class_values(right),
    ) {
        (Some(left_values), Some(right_values)) => {
            return finite_set_class_value(left_values.into_iter().chain(right_values));
        }
        (Some(values), None)
            if values
                .iter()
                .all(|value| abstract_value_matches_string(right, value)) =>
        {
            return right.clone();
        }
        (None, Some(values))
            if values
                .iter()
                .all(|value| abstract_value_matches_string(left, value)) =>
        {
            return left.clone();
        }
        _ => {}
    }

    match (left, right) {
        (
            AbstractClassValueV0::Prefix {
                prefix: left_prefix,
                ..
            },
            AbstractClassValueV0::Prefix {
                prefix: right_prefix,
                ..
            },
        ) => {
            let prefix =
                meaningful_longest_common_prefix(&[left_prefix.clone(), right_prefix.clone()]);
            if !prefix.is_empty() {
                return prefix_class_value(
                    prefix,
                    Some(AbstractClassValueProvenanceV0::PrefixJoinLcp),
                );
            }
        }
        (
            AbstractClassValueV0::Suffix {
                suffix: left_suffix,
                ..
            },
            AbstractClassValueV0::Suffix {
                suffix: right_suffix,
                ..
            },
        ) => {
            let suffix =
                meaningful_longest_common_suffix(&[left_suffix.clone(), right_suffix.clone()]);
            if !suffix.is_empty() {
                return suffix_class_value(
                    suffix,
                    Some(AbstractClassValueProvenanceV0::SuffixJoinLcs),
                );
            }
        }
        _ => {}
    }

    match (
        ClassValueReductionFacts::from_abstract_value(left),
        ClassValueReductionFacts::from_abstract_value(right),
    ) {
        (Some(left), Some(right)) => left
            .join(&right)
            .map_or_else(top_class_value, |facts| facts.into_abstract_value()),
        _ => top_class_value(),
    }
}

pub fn concatenate_abstract_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> AbstractClassValueV0 {
    match (left, right) {
        (AbstractClassValueV0::Bottom, _) | (_, AbstractClassValueV0::Bottom) => {
            return bottom_class_value();
        }
        (AbstractClassValueV0::Top, _) | (_, AbstractClassValueV0::Top) => {
            return top_class_value();
        }
        _ => {}
    }

    if let (Some(left_values), Some(right_values)) = (
        enumerate_finite_class_values(left),
        enumerate_finite_class_values(right),
    ) {
        return finite_set_class_value(left_values.into_iter().flat_map(|left_value| {
            right_values
                .iter()
                .map(move |right_value| format!("{left_value}{right_value}"))
        }));
    }

    match (left, right) {
        (AbstractClassValueV0::Exact { value }, AbstractClassValueV0::Prefix { prefix, .. }) => {
            prefix_class_value(format!("{value}{prefix}"), None)
        }
        (AbstractClassValueV0::Exact { value }, AbstractClassValueV0::Suffix { suffix, .. }) => {
            prefix_suffix_class_value(value, suffix, Some(value.len() + suffix.len()), None)
        }
        (
            AbstractClassValueV0::Exact { value },
            AbstractClassValueV0::PrefixSuffix {
                prefix,
                suffix,
                min_length,
                ..
            },
        ) => prefix_suffix_class_value(
            format!("{value}{prefix}"),
            suffix,
            Some(value.len() + min_length),
            None,
        ),
        (AbstractClassValueV0::Prefix { prefix, .. }, AbstractClassValueV0::Exact { value }) => {
            prefix_suffix_class_value(prefix, value, Some(prefix.len() + value.len()), None)
        }
        (
            AbstractClassValueV0::Prefix { prefix, .. },
            AbstractClassValueV0::FiniteSet { values },
        ) => {
            let suffix = meaningful_longest_common_suffix(values);
            if suffix.is_empty() {
                prefix_class_value(prefix, None)
            } else {
                prefix_suffix_class_value(
                    prefix,
                    suffix.clone(),
                    Some(prefix.len() + suffix.len()),
                    None,
                )
            }
        }
        (AbstractClassValueV0::Prefix { prefix, .. }, AbstractClassValueV0::Prefix { .. }) => {
            prefix_class_value(prefix, None)
        }
        (
            AbstractClassValueV0::Prefix { prefix, .. },
            AbstractClassValueV0::Suffix { suffix, .. },
        )
        | (
            AbstractClassValueV0::Prefix { prefix, .. },
            AbstractClassValueV0::PrefixSuffix { suffix, .. },
        ) => prefix_suffix_class_value(prefix, suffix, Some(prefix.len() + suffix.len()), None),
        (
            AbstractClassValueV0::FiniteSet { values },
            AbstractClassValueV0::Prefix { prefix, .. },
        ) => {
            let values = values
                .iter()
                .map(|value| format!("{value}{prefix}"))
                .collect::<Vec<_>>();
            let prefix = meaningful_longest_common_prefix(&values);
            if prefix.is_empty() {
                top_class_value()
            } else {
                prefix_class_value(prefix, None)
            }
        }
        (
            AbstractClassValueV0::FiniteSet { values },
            AbstractClassValueV0::Suffix { suffix, .. },
        ) => {
            let prefix = meaningful_longest_common_prefix(values);
            if prefix.is_empty() {
                suffix_class_value(suffix, None)
            } else {
                prefix_suffix_class_value(prefix, suffix, Some(suffix.len()), None)
            }
        }
        (AbstractClassValueV0::Suffix { .. }, AbstractClassValueV0::FiniteSet { values }) => {
            let suffix = meaningful_longest_common_suffix(values);
            if suffix.is_empty() {
                top_class_value()
            } else {
                suffix_class_value(suffix, None)
            }
        }
        (AbstractClassValueV0::Suffix { .. }, AbstractClassValueV0::Suffix { suffix, .. }) => {
            suffix_class_value(suffix, None)
        }
        (
            AbstractClassValueV0::Suffix { .. },
            AbstractClassValueV0::PrefixSuffix { suffix, .. },
        ) => suffix_class_value(suffix, None),
        (
            AbstractClassValueV0::PrefixSuffix { prefix, .. },
            AbstractClassValueV0::Prefix { .. },
        ) => prefix_class_value(prefix, None),
        (
            AbstractClassValueV0::PrefixSuffix {
                prefix,
                suffix,
                min_length,
                ..
            },
            AbstractClassValueV0::Exact { value },
        ) => prefix_suffix_class_value(
            prefix,
            format!("{suffix}{value}"),
            Some(min_length + value.len()),
            None,
        ),
        (
            AbstractClassValueV0::PrefixSuffix {
                prefix,
                suffix,
                min_length,
                ..
            },
            AbstractClassValueV0::FiniteSet { values },
        ) => {
            let shared_suffix = meaningful_longest_common_suffix(values);
            if shared_suffix.is_empty() {
                prefix_class_value(prefix, None)
            } else {
                prefix_suffix_class_value(
                    prefix,
                    format!("{suffix}{shared_suffix}"),
                    Some(min_length + shared_suffix.len()),
                    None,
                )
            }
        }
        (
            AbstractClassValueV0::PrefixSuffix { prefix, .. },
            AbstractClassValueV0::Suffix { suffix, .. },
        ) => prefix_suffix_class_value(prefix, suffix, Some(prefix.len() + suffix.len()), None),
        (
            AbstractClassValueV0::PrefixSuffix { prefix, .. },
            AbstractClassValueV0::PrefixSuffix {
                suffix, min_length, ..
            },
        ) => prefix_suffix_class_value(prefix, suffix, Some(prefix.len() + min_length), None),
        _ => match (
            ClassValueReductionFacts::from_abstract_value(left),
            ClassValueReductionFacts::from_abstract_value(right),
        ) {
            (Some(left), Some(right)) => left
                .concat(&right)
                .map_or_else(top_class_value, |facts| facts.into_abstract_value()),
            _ => top_class_value(),
        },
    }
}

pub fn project_abstract_value_selectors(
    value: &AbstractClassValueV0,
    selector_universe: &[String],
) -> AbstractSelectorProjectionV0 {
    let selector_names = resolve_abstract_value_selectors(value, selector_universe);
    let certainty =
        derive_selector_projection_certainty(value, selector_names.len(), selector_universe.len());

    AbstractSelectorProjectionV0 {
        selector_names,
        certainty,
    }
}

pub fn resolve_abstract_value_selectors(
    value: &AbstractClassValueV0,
    selector_universe: &[String],
) -> Vec<String> {
    match value {
        AbstractClassValueV0::Bottom => Vec::new(),
        AbstractClassValueV0::Exact { value } => find_selectors(selector_universe, value),
        AbstractClassValueV0::FiniteSet { values } => unique_selector_names(
            values
                .iter()
                .flat_map(|value| find_selectors(selector_universe, value)),
        ),
        AbstractClassValueV0::Prefix { prefix, .. } => selector_universe
            .iter()
            .filter(|selector| selector.starts_with(prefix))
            .cloned()
            .collect(),
        AbstractClassValueV0::Suffix { suffix, .. } => selector_universe
            .iter()
            .filter(|selector| selector.ends_with(suffix))
            .cloned()
            .collect(),
        AbstractClassValueV0::PrefixSuffix { prefix, suffix, .. } => selector_universe
            .iter()
            .filter(|selector| selector.starts_with(prefix) && selector.ends_with(suffix))
            .cloned()
            .collect(),
        AbstractClassValueV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => selector_universe
            .iter()
            .filter(|selector| {
                matches_char_constraints(selector, must_chars, may_chars, *may_include_other_chars)
            })
            .cloned()
            .collect(),
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => selector_universe
            .iter()
            .filter(|selector| {
                min_length.is_none_or(|min_length| selector.len() >= min_length)
                    && prefix
                        .as_ref()
                        .is_none_or(|prefix| selector.starts_with(prefix))
                    && suffix
                        .as_ref()
                        .is_none_or(|suffix| selector.ends_with(suffix))
                    && matches_char_constraints(
                        selector,
                        must_chars,
                        may_chars,
                        *may_include_other_chars,
                    )
            })
            .cloned()
            .collect(),
        AbstractClassValueV0::Top => selector_universe.to_vec(),
    }
}

pub fn derive_selector_projection_certainty(
    value: &AbstractClassValueV0,
    matched_selector_count: usize,
    selector_universe_count: usize,
) -> SelectorProjectionCertaintyV0 {
    match value {
        AbstractClassValueV0::Bottom => SelectorProjectionCertaintyV0::Possible,
        AbstractClassValueV0::Exact { .. } => {
            if matched_selector_count == 1 {
                SelectorProjectionCertaintyV0::Exact
            } else {
                SelectorProjectionCertaintyV0::Possible
            }
        }
        AbstractClassValueV0::FiniteSet { values } => {
            if values.is_empty() || matched_selector_count == 0 {
                SelectorProjectionCertaintyV0::Possible
            } else if matched_selector_count == values.len() {
                SelectorProjectionCertaintyV0::Exact
            } else {
                SelectorProjectionCertaintyV0::Inferred
            }
        }
        AbstractClassValueV0::Prefix { .. }
        | AbstractClassValueV0::Suffix { .. }
        | AbstractClassValueV0::PrefixSuffix { .. }
        | AbstractClassValueV0::CharInclusion { .. }
        | AbstractClassValueV0::Composite { .. } => {
            if matched_selector_count == 0 {
                SelectorProjectionCertaintyV0::Possible
            } else if matched_selector_count == selector_universe_count {
                SelectorProjectionCertaintyV0::Exact
            } else {
                SelectorProjectionCertaintyV0::Inferred
            }
        }
        AbstractClassValueV0::Top => SelectorProjectionCertaintyV0::Possible,
    }
}

fn widen_large_finite_set(values: &[String]) -> AbstractClassValueV0 {
    let prefix = meaningful_longest_common_prefix(values);
    let suffix = meaningful_longest_common_suffix(values);
    let (must_chars, may_chars) = char_inclusion_from_finite_values(values);

    if !prefix.is_empty() || !suffix.is_empty() {
        return composite_class_value(CompositeClassValueInputV0 {
            prefix: (!prefix.is_empty()).then_some(prefix),
            suffix: (!suffix.is_empty()).then_some(suffix),
            min_length: values.iter().map(String::len).min(),
            must_chars,
            may_chars,
            may_include_other_chars: false,
            provenance: Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite),
        });
    }

    char_inclusion_class_value(
        must_chars,
        may_chars,
        Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars),
        false,
    )
}

fn normalize_values<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    values
        .into_iter()
        .map(Into::into)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalize_char_set(chars: impl AsRef<str>) -> String {
    chars
        .as_ref()
        .chars()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn union_char_sets(left: &str, right: &str) -> String {
    normalize_char_set(format!("{left}{right}"))
}

fn intersect_char_sets(left: &str, right: &str) -> String {
    let right_set = right.chars().collect::<BTreeSet<_>>();
    left.chars()
        .filter(|char| right_set.contains(char))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn char_set_for_string(value: impl AsRef<str>) -> String {
    normalize_char_set(value)
}

fn char_inclusion_from_finite_values(values: &[String]) -> (String, String) {
    let mut sets = values.iter().map(char_set_for_string);
    let Some(first) = sets.next() else {
        return (String::new(), String::new());
    };

    sets.fold((first.clone(), first), |(must_chars, may_chars), next| {
        (
            intersect_char_sets(&must_chars, &next),
            union_char_sets(&may_chars, &next),
        )
    })
}

fn longest_common_prefix(values: &[String]) -> String {
    let Some(first) = values.first() else {
        return String::new();
    };
    let mut prefix = first.clone();

    for value in values.iter().skip(1) {
        let mut match_length = 0usize;
        for (left, right) in prefix.chars().zip(value.chars()) {
            if left != right {
                break;
            }
            match_length += left.len_utf8();
        }
        prefix.truncate(match_length);
        if prefix.is_empty() {
            break;
        }
    }

    prefix
}

fn meaningful_longest_common_prefix(values: &[String]) -> String {
    let prefix = longest_common_prefix(values);
    if prefix.is_empty() || !is_meaningful_class_prefix(&prefix, values) {
        return String::new();
    }
    prefix
}

fn longest_common_suffix(values: &[String]) -> String {
    let reversed = values
        .iter()
        .map(|value| value.chars().rev().collect::<String>())
        .collect::<Vec<_>>();
    longest_common_prefix(&reversed)
        .chars()
        .rev()
        .collect::<String>()
}

fn meaningful_longest_common_suffix(values: &[String]) -> String {
    let suffix = longest_common_suffix(values);
    if suffix.is_empty() || !is_meaningful_class_suffix(&suffix, values) {
        return String::new();
    }
    suffix
}

fn is_meaningful_class_prefix(prefix: &str, values: &[String]) -> bool {
    if prefix.is_empty() {
        return false;
    }
    if ends_at_class_boundary(prefix) {
        return true;
    }
    values.iter().all(|value| {
        value.len() == prefix.len()
            || value[prefix.len()..]
                .chars()
                .next()
                .is_some_and(is_class_boundary_char)
    })
}

fn is_meaningful_class_suffix(suffix: &str, values: &[String]) -> bool {
    if suffix.is_empty() {
        return false;
    }
    if starts_at_class_boundary(suffix) {
        return true;
    }
    values.iter().all(|value| {
        if value.len() == suffix.len() {
            return true;
        }
        value[..value.len() - suffix.len()]
            .chars()
            .next_back()
            .is_some_and(is_class_boundary_char)
    })
}

fn ends_at_class_boundary(value: &str) -> bool {
    value
        .chars()
        .next_back()
        .is_some_and(is_class_boundary_char)
}

fn starts_at_class_boundary(value: &str) -> bool {
    value.chars().next().is_some_and(is_class_boundary_char)
}

fn is_class_boundary_char(char: char) -> bool {
    char == '-' || char == '_'
}

fn find_selectors(selector_universe: &[String], value: &str) -> Vec<String> {
    selector_universe
        .iter()
        .filter(|selector| selector.as_str() == value)
        .cloned()
        .collect()
}

fn unique_selector_names<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn matches_char_constraints(
    value: &str,
    must_chars: &str,
    may_chars: &str,
    may_include_other_chars: bool,
) -> bool {
    let value_chars = value.chars().collect::<BTreeSet<_>>();
    let must_chars = must_chars.chars().collect::<BTreeSet<_>>();
    if !must_chars.iter().all(|char| value_chars.contains(char)) {
        return false;
    }
    if may_include_other_chars {
        return true;
    }
    let may_chars = may_chars.chars().collect::<BTreeSet<_>>();
    value_chars.iter().all(|char| may_chars.contains(char))
}

fn abstract_value_matches_string(value: &AbstractClassValueV0, candidate: &str) -> bool {
    match value {
        AbstractClassValueV0::Bottom => false,
        AbstractClassValueV0::Exact { value } => value == candidate,
        AbstractClassValueV0::FiniteSet { values } => values.iter().any(|value| value == candidate),
        AbstractClassValueV0::Prefix { prefix, .. } => candidate.starts_with(prefix),
        AbstractClassValueV0::Suffix { suffix, .. } => candidate.ends_with(suffix),
        AbstractClassValueV0::PrefixSuffix {
            prefix,
            suffix,
            min_length,
            ..
        } => {
            candidate.len() >= *min_length
                && candidate.starts_with(prefix)
                && candidate.ends_with(suffix)
        }
        AbstractClassValueV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => matches_char_constraints(candidate, must_chars, may_chars, *may_include_other_chars),
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => {
            min_length.is_none_or(|min_length| candidate.len() >= min_length)
                && prefix
                    .as_ref()
                    .is_none_or(|prefix| candidate.starts_with(prefix))
                && suffix
                    .as_ref()
                    .is_none_or(|suffix| candidate.ends_with(suffix))
                && matches_char_constraints(
                    candidate,
                    must_chars,
                    may_chars,
                    *may_include_other_chars,
                )
        }
        AbstractClassValueV0::Top => true,
    }
}

fn intersect_non_top_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> AbstractClassValueV0 {
    match (
        enumerate_finite_class_values(left),
        enumerate_finite_class_values(right),
    ) {
        (Some(left_values), Some(right_values)) => {
            let right_values = right_values.into_iter().collect::<BTreeSet<_>>();
            return finite_set_class_value(
                left_values
                    .into_iter()
                    .filter(|value| right_values.contains(value)),
            );
        }
        (Some(values), None) => {
            return finite_set_class_value(
                values
                    .into_iter()
                    .filter(|value| abstract_value_matches_string(right, value)),
            );
        }
        (None, Some(values)) => {
            return finite_set_class_value(
                values
                    .into_iter()
                    .filter(|value| abstract_value_matches_string(left, value)),
            );
        }
        (None, None) => {}
    }

    match (
        ClassValueReductionFacts::from_abstract_value(left),
        ClassValueReductionFacts::from_abstract_value(right),
    ) {
        (Some(left), Some(right)) => left
            .intersect(&right)
            .map_or_else(bottom_class_value, |facts| facts.into_abstract_value()),
        _ => bottom_class_value(),
    }
}

fn abstract_value_is_subset(left: &AbstractClassValueV0, right: &AbstractClassValueV0) -> bool {
    if left == right {
        return true;
    }

    match (left, right) {
        (AbstractClassValueV0::Bottom, _) | (_, AbstractClassValueV0::Top) => true,
        (AbstractClassValueV0::Top, _) => false,
        _ => {
            enumerate_finite_class_values(left).is_some_and(|values| {
                values
                    .iter()
                    .all(|value| abstract_value_matches_string(right, value))
            }) || constrained_value_is_subset(left, right)
        }
    }
}

fn constrained_value_is_subset(left: &AbstractClassValueV0, right: &AbstractClassValueV0) -> bool {
    match (left, right) {
        (
            AbstractClassValueV0::Prefix {
                prefix: left_prefix,
                ..
            },
            AbstractClassValueV0::Prefix {
                prefix: right_prefix,
                ..
            },
        ) => left_prefix.starts_with(right_prefix),
        (
            AbstractClassValueV0::Suffix {
                suffix: left_suffix,
                ..
            },
            AbstractClassValueV0::Suffix {
                suffix: right_suffix,
                ..
            },
        ) => left_suffix.ends_with(right_suffix),
        (
            AbstractClassValueV0::PrefixSuffix {
                prefix: left_prefix,
                suffix: _,
                ..
            },
            AbstractClassValueV0::Prefix {
                prefix: right_prefix,
                ..
            },
        ) => left_prefix.starts_with(right_prefix),
        (
            AbstractClassValueV0::PrefixSuffix {
                prefix: left_prefix,
                suffix: left_suffix,
                min_length: left_min_length,
                ..
            },
            AbstractClassValueV0::PrefixSuffix {
                prefix: right_prefix,
                suffix: right_suffix,
                min_length: right_min_length,
                ..
            },
        ) => {
            left_prefix.starts_with(right_prefix)
                && left_suffix.ends_with(right_suffix)
                && left_min_length >= right_min_length
        }
        (
            AbstractClassValueV0::PrefixSuffix {
                suffix: left_suffix,
                ..
            },
            AbstractClassValueV0::Suffix {
                suffix: right_suffix,
                ..
            },
        ) => left_suffix.ends_with(right_suffix),
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassValueReductionFacts {
    prefix: Option<String>,
    suffix: Option<String>,
    min_length: Option<usize>,
    must_chars: String,
    allowed_chars: Option<String>,
}

impl ClassValueReductionFacts {
    fn from_abstract_value(value: &AbstractClassValueV0) -> Option<Self> {
        match value {
            AbstractClassValueV0::Bottom
            | AbstractClassValueV0::Exact { .. }
            | AbstractClassValueV0::FiniteSet { .. } => None,
            AbstractClassValueV0::Prefix { prefix, .. } => Some(Self {
                prefix: Some(prefix.clone()),
                suffix: None,
                min_length: None,
                must_chars: String::new(),
                allowed_chars: None,
            }),
            AbstractClassValueV0::Suffix { suffix, .. } => Some(Self {
                prefix: None,
                suffix: Some(suffix.clone()),
                min_length: None,
                must_chars: String::new(),
                allowed_chars: None,
            }),
            AbstractClassValueV0::PrefixSuffix {
                prefix,
                suffix,
                min_length,
                ..
            } => Some(Self {
                prefix: Some(prefix.clone()),
                suffix: Some(suffix.clone()),
                min_length: Some(*min_length),
                must_chars: String::new(),
                allowed_chars: None,
            }),
            AbstractClassValueV0::CharInclusion {
                must_chars,
                may_chars,
                may_include_other_chars,
                ..
            } => Some(Self {
                prefix: None,
                suffix: None,
                min_length: None,
                must_chars: must_chars.clone(),
                allowed_chars: (!*may_include_other_chars).then_some(may_chars.clone()),
            }),
            AbstractClassValueV0::Composite {
                prefix,
                suffix,
                min_length,
                must_chars,
                may_chars,
                may_include_other_chars,
                ..
            } => Some(Self {
                prefix: prefix.clone(),
                suffix: suffix.clone(),
                min_length: *min_length,
                must_chars: must_chars.clone(),
                allowed_chars: (!*may_include_other_chars).then_some(may_chars.clone()),
            }),
            AbstractClassValueV0::Top => Some(Self {
                prefix: None,
                suffix: None,
                min_length: None,
                must_chars: String::new(),
                allowed_chars: None,
            }),
        }
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        let prefix = intersect_prefixes(self.prefix.as_deref(), other.prefix.as_deref())?;
        let suffix = intersect_suffixes(self.suffix.as_deref(), other.suffix.as_deref())?;
        let min_length = max_optional_usize(self.min_length, other.min_length);
        let edge_chars = char_set_for_string(format!(
            "{}{}",
            prefix.as_deref().unwrap_or(""),
            suffix.as_deref().unwrap_or("")
        ));
        let must_chars = union_char_sets(
            &union_char_sets(&self.must_chars, &other.must_chars),
            &edge_chars,
        );
        let allowed_chars = intersect_allowed_char_sets(
            self.allowed_chars.as_deref(),
            other.allowed_chars.as_deref(),
        );

        if let Some(allowed_chars) = &allowed_chars
            && !char_set_is_subset(&must_chars, allowed_chars)
        {
            return None;
        }

        Some(Self {
            prefix,
            suffix,
            min_length,
            must_chars,
            allowed_chars,
        })
    }

    fn join(&self, other: &Self) -> Option<Self> {
        let prefix = join_prefixes(self.prefix.as_deref(), other.prefix.as_deref());
        let suffix = join_suffixes(self.suffix.as_deref(), other.suffix.as_deref());
        let min_length = Some(self.lower_bound_length().min(other.lower_bound_length()));
        let must_chars = intersect_char_sets(&self.guaranteed_chars(), &other.guaranteed_chars());
        let allowed_chars = join_allowed_char_sets(
            self.allowed_chars.as_deref(),
            other.allowed_chars.as_deref(),
        );

        if prefix.is_none() && suffix.is_none() && must_chars.is_empty() && allowed_chars.is_none()
        {
            return None;
        }

        Some(Self {
            prefix,
            suffix,
            min_length,
            must_chars,
            allowed_chars,
        })
    }

    fn concat(&self, other: &Self) -> Option<Self> {
        let prefix = self.prefix.clone();
        let suffix = other.suffix.clone();
        let min_length = Some(self.lower_bound_length() + other.lower_bound_length());
        let must_chars = union_char_sets(&self.guaranteed_chars(), &other.guaranteed_chars());
        let allowed_chars = join_allowed_char_sets(
            self.allowed_chars.as_deref(),
            other.allowed_chars.as_deref(),
        );

        if prefix.is_none() && suffix.is_none() && must_chars.is_empty() && allowed_chars.is_none()
        {
            return None;
        }

        Some(Self {
            prefix,
            suffix,
            min_length,
            must_chars,
            allowed_chars,
        })
    }

    fn lower_bound_length(&self) -> usize {
        self.min_length.unwrap_or_else(|| {
            let edge_len = self.prefix.as_deref().unwrap_or("").len()
                + self.suffix.as_deref().unwrap_or("").len();
            edge_len.max(self.must_chars.chars().count())
        })
    }

    fn guaranteed_chars(&self) -> String {
        union_char_sets(
            &self.must_chars,
            &char_set_for_string(format!(
                "{}{}",
                self.prefix.as_deref().unwrap_or(""),
                self.suffix.as_deref().unwrap_or("")
            )),
        )
    }

    fn into_abstract_value(self) -> AbstractClassValueV0 {
        let edge_chars = char_set_for_string(format!(
            "{}{}",
            self.prefix.as_deref().unwrap_or(""),
            self.suffix.as_deref().unwrap_or("")
        ));
        if self.allowed_chars.is_none()
            && (!edge_chars.is_empty() || self.prefix.is_some() || self.suffix.is_some())
            && char_set_is_subset(&self.must_chars, &edge_chars)
        {
            return prefix_suffix_class_value(
                self.prefix.unwrap_or_default(),
                self.suffix.unwrap_or_default(),
                self.min_length,
                Some(AbstractClassValueProvenanceV0::CompositeJoin),
            );
        }

        let may_include_other_chars = self.allowed_chars.is_none();
        let may_chars = self
            .allowed_chars
            .unwrap_or_else(|| self.must_chars.clone());

        if self.prefix.is_none()
            && self.suffix.is_none()
            && self.must_chars.is_empty()
            && may_include_other_chars
        {
            return top_class_value();
        }

        if self.prefix.is_none()
            && self.suffix.is_none()
            && self.must_chars.is_empty()
            && may_chars.is_empty()
            && !may_include_other_chars
        {
            return bottom_class_value();
        }

        composite_class_value(CompositeClassValueInputV0 {
            prefix: self.prefix,
            suffix: self.suffix,
            min_length: self.min_length,
            must_chars: self.must_chars,
            may_chars,
            may_include_other_chars,
            provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
        })
    }
}

fn intersect_prefixes(left: Option<&str>, right: Option<&str>) -> Option<Option<String>> {
    match (left, right) {
        (None, None) => Some(None),
        (Some(value), None) | (None, Some(value)) => Some(Some(value.to_string())),
        (Some(left), Some(right)) if left.starts_with(right) => Some(Some(left.to_string())),
        (Some(left), Some(right)) if right.starts_with(left) => Some(Some(right.to_string())),
        (Some(_), Some(_)) => None,
    }
}

fn intersect_suffixes(left: Option<&str>, right: Option<&str>) -> Option<Option<String>> {
    match (left, right) {
        (None, None) => Some(None),
        (Some(value), None) | (None, Some(value)) => Some(Some(value.to_string())),
        (Some(left), Some(right)) if left.ends_with(right) => Some(Some(left.to_string())),
        (Some(left), Some(right)) if right.ends_with(left) => Some(Some(right.to_string())),
        (Some(_), Some(_)) => None,
    }
}

fn join_prefixes(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let prefix = meaningful_longest_common_prefix(&[left.to_string(), right.to_string()]);
            (!prefix.is_empty()).then_some(prefix)
        }
        _ => None,
    }
}

fn join_suffixes(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let suffix = meaningful_longest_common_suffix(&[left.to_string(), right.to_string()]);
            (!suffix.is_empty()).then_some(suffix)
        }
        _ => None,
    }
}

fn max_optional_usize(left: Option<usize>, right: Option<usize>) -> Option<usize> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn intersect_allowed_char_sets(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => Some(intersect_char_sets(left, right)),
        (Some(value), None) | (None, Some(value)) => Some(value.to_string()),
        (None, None) => None,
    }
}

fn join_allowed_char_sets(left: Option<&str>, right: Option<&str>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => Some(union_char_sets(left, right)),
        _ => None,
    }
}

fn char_set_is_subset(left: &str, right: &str) -> bool {
    let right = right.chars().collect::<BTreeSet<_>>();
    left.chars().all(|char| right.contains(&char))
}

#[cfg(test)]
mod tests;
