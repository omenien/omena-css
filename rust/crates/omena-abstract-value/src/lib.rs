use std::collections::BTreeSet;

mod domain;
mod facts;
mod flow;
mod selector_projection;
mod types;

pub use domain::*;
pub use facts::*;
pub use flow::*;
pub use selector_projection::*;
pub use types::*;

use domain::{
    char_set_for_string, char_set_is_subset, intersect_char_sets, meaningful_longest_common_prefix,
    meaningful_longest_common_suffix, union_char_sets,
};
use selector_projection::abstract_value_matches_string;

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

#[cfg(test)]
mod tests;
