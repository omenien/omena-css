use std::collections::BTreeSet;

mod domain;
mod facts;
mod flow;
mod reduced_product;
mod selector_projection;
mod types;

pub use domain::*;
pub use facts::*;
pub use flow::*;
pub use selector_projection::*;
pub use types::*;

use domain::{meaningful_longest_common_prefix, meaningful_longest_common_suffix};
use reduced_product::{
    concatenate_reduced_product_class_values, intersect_reduced_product_class_values,
    join_reduced_product_class_values,
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

    join_reduced_product_class_values(left, right).unwrap_or_else(top_class_value)
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
        _ => concatenate_reduced_product_class_values(left, right).unwrap_or_else(top_class_value),
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

    intersect_reduced_product_class_values(left, right).unwrap_or_else(bottom_class_value)
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

#[cfg(test)]
mod tests;
