use std::collections::{BTreeMap, BTreeSet};

use omena_incremental::{
    IncrementalComputationPlanV0, IncrementalGraphInputV0, IncrementalNodeInputV0,
    IncrementalRevisionV0, IncrementalSnapshotV0, OmenaIncrementalDatabaseV0,
    plan_incremental_computation, snapshot_from_graph_input,
};
use serde::Serialize;

pub const MAX_FINITE_CLASS_VALUES: usize = 8;
pub const MAX_FLOW_ANALYSIS_ITERATIONS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractValueDomainSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub domain_kinds: Vec<&'static str>,
    pub max_finite_class_values: usize,
    pub selector_projection_certainties: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractValueFlowAnalysisSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub context_sensitivity: &'static str,
    pub incremental_engine: &'static str,
    pub analysis_scopes: Vec<&'static str>,
    pub reuse_policy: &'static str,
    pub transfer_kinds: Vec<&'static str>,
    pub max_iterations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueDerivationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_fact_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_constraint_kind: Option<String>,
    pub input_value_count: usize,
    pub reduced_kind: &'static str,
    pub steps: Vec<ReducedClassValueDerivationStepV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueDerivationStepV0 {
    pub operation: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refinement_kind: Option<&'static str>,
    pub result_kind: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AbstractClassValueV0 {
    Bottom,
    Exact {
        value: String,
    },
    FiniteSet {
        values: Vec<String>,
    },
    Prefix {
        prefix: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provenance: Option<AbstractClassValueProvenanceV0>,
    },
    Suffix {
        suffix: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provenance: Option<AbstractClassValueProvenanceV0>,
    },
    PrefixSuffix {
        prefix: String,
        suffix: String,
        min_length: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        provenance: Option<AbstractClassValueProvenanceV0>,
    },
    CharInclusion {
        must_chars: String,
        may_chars: String,
        #[serde(skip_serializing_if = "is_false")]
        may_include_other_chars: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        provenance: Option<AbstractClassValueProvenanceV0>,
    },
    Composite {
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        suffix: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        must_chars: String,
        may_chars: String,
        #[serde(skip_serializing_if = "is_false")]
        may_include_other_chars: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        provenance: Option<AbstractClassValueProvenanceV0>,
    },
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AbstractClassValueProvenanceV0 {
    FiniteSetWideningChars,
    FiniteSetWideningComposite,
    PrefixJoinLcp,
    SuffixJoinLcs,
    PrefixSuffixJoin,
    CompositeJoin,
    CompositeConcat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeClassValueInputV0 {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub min_length: Option<usize>,
    pub must_chars: String,
    pub may_chars: String,
    pub may_include_other_chars: bool,
    pub provenance: Option<AbstractClassValueProvenanceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalStringTypeFactsV0 {
    pub kind: String,
    pub constraint_kind: Option<String>,
    pub values: Option<Vec<String>>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub min_len: Option<usize>,
    pub max_len: Option<usize>,
    pub char_must: Option<String>,
    pub char_may: Option<String>,
    pub may_include_other_chars: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassValueFlowGraphV0 {
    pub context_key: Option<String>,
    pub nodes: Vec<ClassValueFlowNodeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassValueFlowNodeV0 {
    pub id: String,
    pub predecessors: Vec<String>,
    pub transfer: ClassValueFlowTransferV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassValueFlowTransferV0 {
    AssignFacts(ExternalStringTypeFactsV0),
    RefineFacts(ExternalStringTypeFactsV0),
    ConcatFacts(ExternalStringTypeFactsV0),
    Join,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassValueFlowAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub context_sensitivity: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_key: Option<String>,
    pub converged: bool,
    pub iteration_count: usize,
    pub nodes: Vec<ClassValueFlowNodeResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassValueFlowIncrementalAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub reused_previous_analysis: bool,
    pub incremental_plan: IncrementalComputationPlanV0,
    pub next_snapshot: IncrementalSnapshotV0,
    pub analysis: ClassValueFlowAnalysisV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassValueFlowIncrementalBatchAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub revision: u64,
    pub context_count: usize,
    pub dirty_context_count: usize,
    pub reused_context_count: usize,
    pub entries: Vec<ClassValueFlowIncrementalBatchEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassValueFlowIncrementalBatchEntryV0 {
    pub context_key: String,
    pub analysis: ClassValueFlowIncrementalAnalysisV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneCfaCallSiteFlowInputV0 {
    pub callee_key: String,
    pub call_site_id: String,
    pub graph: ClassValueFlowGraphV0,
    pub exit_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneCfaCallSiteFlowAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub context_sensitivity: &'static str,
    pub call_site_count: usize,
    pub callee_count: usize,
    pub entries: Vec<OneCfaCallSiteFlowEntryV0>,
    pub callee_summaries: Vec<OneCfaCalleeFlowSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneCfaCallSiteFlowEntryV0 {
    pub callee_key: String,
    pub call_site_id: String,
    pub context_key: String,
    pub exit_node_id: String,
    pub exit_value_kind: &'static str,
    pub exit_value: AbstractClassValueV0,
    pub analysis: ClassValueFlowAnalysisV0,
    pub derivation: OneCfaCallSiteDerivationV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneCfaCalleeFlowSummaryV0 {
    pub callee_key: String,
    pub call_site_count: usize,
    pub joined_exit_value_kind: &'static str,
    pub joined_exit_value: AbstractClassValueV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneCfaCallSiteDerivationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub call_site_id: String,
    pub context_key: String,
    pub steps: Vec<OneCfaCallSiteDerivationStepV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OneCfaCallSiteDerivationStepV0 {
    pub operation: &'static str,
    pub result_kind: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassValueFlowNodeResultV0 {
    pub id: String,
    pub predecessor_ids: Vec<String>,
    pub transfer_kind: &'static str,
    pub value_kind: &'static str,
    pub value: AbstractClassValueV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SelectorProjectionCertaintyV0 {
    Exact,
    Inferred,
    Possible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractSelectorProjectionV0 {
    pub selector_names: Vec<String>,
    pub certainty: SelectorProjectionCertaintyV0,
}

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
        analysis_scopes: vec!["singleContext", "multiContextBatch", "callSiteBatch"],
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
            if prefix.is_empty() {
                top_class_value()
            } else {
                prefix_class_value(prefix, Some(AbstractClassValueProvenanceV0::PrefixJoinLcp))
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
            if suffix.is_empty() {
                top_class_value()
            } else {
                suffix_class_value(suffix, Some(AbstractClassValueProvenanceV0::SuffixJoinLcs))
            }
        }
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
        _ => top_class_value(),
    }
}

pub fn analyze_class_value_flow(graph: &ClassValueFlowGraphV0) -> ClassValueFlowAnalysisV0 {
    let mut values = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), bottom_class_value()))
        .collect::<BTreeMap<_, _>>();
    let mut converged = false;
    let mut iteration_count = 0;

    for iteration in 1..=MAX_FLOW_ANALYSIS_ITERATIONS {
        iteration_count = iteration;
        let mut changed = false;

        for node in &graph.nodes {
            let incoming = join_predecessor_flow_values(node, &values);
            let next = apply_flow_transfer(&incoming, &node.transfer);

            if values.get(&node.id) != Some(&next) {
                values.insert(node.id.clone(), next);
                changed = true;
            }
        }

        if !changed {
            converged = true;
            break;
        }
    }

    ClassValueFlowAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.flow-analysis",
        context_sensitivity: "1-cfa",
        context_key: graph.context_key.clone(),
        converged,
        iteration_count,
        nodes: graph
            .nodes
            .iter()
            .map(|node| {
                let value = values
                    .get(&node.id)
                    .cloned()
                    .unwrap_or_else(bottom_class_value);
                ClassValueFlowNodeResultV0 {
                    id: node.id.clone(),
                    predecessor_ids: node.predecessors.clone(),
                    transfer_kind: flow_transfer_kind(&node.transfer),
                    value_kind: abstract_class_value_kind(&value),
                    value,
                }
            })
            .collect(),
    }
}

pub fn analyze_class_value_flow_incremental(
    graph: &ClassValueFlowGraphV0,
    previous_snapshot: Option<&IncrementalSnapshotV0>,
    revision: u64,
) -> ClassValueFlowIncrementalAnalysisV0 {
    analyze_class_value_flow_incremental_with_reuse(graph, previous_snapshot, None, revision)
}

pub fn analyze_class_value_flow_incremental_with_reuse(
    graph: &ClassValueFlowGraphV0,
    previous_snapshot: Option<&IncrementalSnapshotV0>,
    previous_analysis: Option<&ClassValueFlowAnalysisV0>,
    revision: u64,
) -> ClassValueFlowIncrementalAnalysisV0 {
    let incremental_input = class_value_flow_incremental_input(graph, revision);
    let incremental_plan = plan_incremental_computation(&incremental_input, previous_snapshot);
    let next_snapshot = snapshot_from_graph_input(&incremental_input);
    let reused_previous_analysis =
        incremental_plan.dirty_node_count == 0 && previous_analysis.is_some();
    let analysis = match (incremental_plan.dirty_node_count, previous_analysis) {
        (0, Some(previous_analysis)) => previous_analysis.clone(),
        _ => analyze_class_value_flow(graph),
    };

    ClassValueFlowIncrementalAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.incremental-flow-analysis",
        reused_previous_analysis,
        incremental_plan,
        next_snapshot,
        analysis,
    }
}

pub fn analyze_class_value_flow_incremental_with_database(
    graph: &ClassValueFlowGraphV0,
    incremental_database: &mut OmenaIncrementalDatabaseV0,
    previous_analysis: Option<&ClassValueFlowAnalysisV0>,
    revision: u64,
) -> ClassValueFlowIncrementalAnalysisV0 {
    let incremental_input = class_value_flow_incremental_input(graph, revision);
    let update = incremental_database.plan_and_upsert_graph_input(&incremental_input);
    let reused_previous_analysis =
        update.incremental_plan.dirty_node_count == 0 && previous_analysis.is_some();
    let analysis = match (update.incremental_plan.dirty_node_count, previous_analysis) {
        (0, Some(previous_analysis)) => previous_analysis.clone(),
        _ => analyze_class_value_flow(graph),
    };

    ClassValueFlowIncrementalAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.incremental-flow-analysis",
        reused_previous_analysis,
        incremental_plan: update.incremental_plan,
        next_snapshot: update.next_snapshot,
        analysis,
    }
}

pub fn analyze_class_value_flow_incremental_batch_with_reuse(
    graphs: &[ClassValueFlowGraphV0],
    previous_snapshots: &BTreeMap<String, IncrementalSnapshotV0>,
    previous_analyses: &BTreeMap<String, ClassValueFlowAnalysisV0>,
    revision: u64,
) -> ClassValueFlowIncrementalBatchAnalysisV0 {
    let entries = graphs
        .iter()
        .enumerate()
        .map(|(index, graph)| {
            let context_key = flow_graph_batch_context_key(graph, index);
            let analysis = analyze_class_value_flow_incremental_with_reuse(
                graph,
                previous_snapshots.get(context_key.as_str()),
                previous_analyses.get(context_key.as_str()),
                revision,
            );
            ClassValueFlowIncrementalBatchEntryV0 {
                context_key,
                analysis,
            }
        })
        .collect::<Vec<_>>();
    let reused_context_count = entries
        .iter()
        .filter(|entry| entry.analysis.reused_previous_analysis)
        .count();
    let dirty_context_count = entries
        .iter()
        .filter(|entry| entry.analysis.incremental_plan.dirty_node_count > 0)
        .count();

    ClassValueFlowIncrementalBatchAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.incremental-flow-analysis-batch",
        revision,
        context_count: entries.len(),
        dirty_context_count,
        reused_context_count,
        entries,
    }
}

pub fn analyze_one_cfa_call_site_flows(
    inputs: &[OneCfaCallSiteFlowInputV0],
) -> OneCfaCallSiteFlowAnalysisV0 {
    let entries = inputs
        .iter()
        .map(|input| {
            let context_key = one_cfa_context_key(input);
            let mut graph = input.graph.clone();
            graph.context_key = Some(context_key.clone());
            let analysis = analyze_class_value_flow(&graph);
            let exit_value = flow_analysis_node_value(&analysis, &input.exit_node_id)
                .cloned()
                .unwrap_or_else(bottom_class_value);
            let exit_value_kind = abstract_class_value_kind(&exit_value);

            OneCfaCallSiteFlowEntryV0 {
                callee_key: input.callee_key.clone(),
                call_site_id: input.call_site_id.clone(),
                context_key: context_key.clone(),
                exit_node_id: input.exit_node_id.clone(),
                exit_value_kind,
                exit_value: exit_value.clone(),
                analysis,
                derivation: one_cfa_call_site_derivation(input, &context_key, &exit_value),
            }
        })
        .collect::<Vec<_>>();
    let callee_summaries = summarize_one_cfa_callees(&entries);

    OneCfaCallSiteFlowAnalysisV0 {
        schema_version: "0",
        product: "omena-abstract-value.one-cfa-call-site-flow",
        context_sensitivity: "1-cfa",
        call_site_count: entries.len(),
        callee_count: callee_summaries.len(),
        entries,
        callee_summaries,
    }
}

pub fn class_value_flow_incremental_input(
    graph: &ClassValueFlowGraphV0,
    revision: u64,
) -> IncrementalGraphInputV0 {
    IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: revision },
        nodes: graph
            .nodes
            .iter()
            .map(|node| IncrementalNodeInputV0 {
                id: node.id.clone(),
                digest: flow_node_incremental_digest(node),
                dependency_ids: node.predecessors.clone(),
            })
            .collect(),
    }
}

fn flow_graph_batch_context_key(graph: &ClassValueFlowGraphV0, index: usize) -> String {
    graph
        .context_key
        .clone()
        .unwrap_or_else(|| format!("anonymous-context-{index}"))
}

fn one_cfa_context_key(input: &OneCfaCallSiteFlowInputV0) -> String {
    format!("{}@{}", input.callee_key, input.call_site_id)
}

fn flow_analysis_node_value<'a>(
    analysis: &'a ClassValueFlowAnalysisV0,
    node_id: &str,
) -> Option<&'a AbstractClassValueV0> {
    analysis
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .map(|node| &node.value)
}

fn one_cfa_call_site_derivation(
    input: &OneCfaCallSiteFlowInputV0,
    context_key: &str,
    exit_value: &AbstractClassValueV0,
) -> OneCfaCallSiteDerivationV0 {
    OneCfaCallSiteDerivationV0 {
        schema_version: "0",
        product: "omena-abstract-value.one-cfa-call-site-derivation",
        call_site_id: input.call_site_id.clone(),
        context_key: context_key.to_string(),
        steps: vec![
            OneCfaCallSiteDerivationStepV0 {
                operation: "contextFromCallSite",
                result_kind: "context",
                reason: "1-CFA separates flow facts by the immediate call-site identity",
            },
            OneCfaCallSiteDerivationStepV0 {
                operation: "analyzeFlowGraph",
                result_kind: "flowAnalysis",
                reason: "ran the class-value flow graph inside the call-site context",
            },
            OneCfaCallSiteDerivationStepV0 {
                operation: "projectExitNode",
                result_kind: abstract_class_value_kind(exit_value),
                reason: "projected the requested exit node as the call-site result",
            },
        ],
    }
}

fn summarize_one_cfa_callees(
    entries: &[OneCfaCallSiteFlowEntryV0],
) -> Vec<OneCfaCalleeFlowSummaryV0> {
    let mut by_callee = BTreeMap::<String, Vec<AbstractClassValueV0>>::new();
    for entry in entries {
        by_callee
            .entry(entry.callee_key.clone())
            .or_default()
            .push(entry.exit_value.clone());
    }

    by_callee
        .into_iter()
        .map(|(callee_key, values)| {
            let call_site_count = values.len();
            let joined_exit_value = values
                .into_iter()
                .reduce(|left, right| join_abstract_class_values(&left, &right))
                .unwrap_or_else(bottom_class_value);
            OneCfaCalleeFlowSummaryV0 {
                callee_key,
                call_site_count,
                joined_exit_value_kind: abstract_class_value_kind(&joined_exit_value),
                joined_exit_value,
            }
        })
        .collect()
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
    selector_universe_count: usize,
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
        "constrained" | "top" => {
            if matched_selector_count == 0 {
                "possible"
            } else if matched_selector_count == selector_universe_count {
                "exact"
            } else {
                "inferred"
            }
        }
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

fn join_predecessor_flow_values(
    node: &ClassValueFlowNodeV0,
    values: &BTreeMap<String, AbstractClassValueV0>,
) -> AbstractClassValueV0 {
    node.predecessors
        .iter()
        .map(|id| values.get(id).cloned().unwrap_or_else(top_class_value))
        .reduce(|left, right| join_abstract_class_values(&left, &right))
        .unwrap_or_else(bottom_class_value)
}

fn apply_flow_transfer(
    incoming: &AbstractClassValueV0,
    transfer: &ClassValueFlowTransferV0,
) -> AbstractClassValueV0 {
    match transfer {
        ClassValueFlowTransferV0::AssignFacts(facts) => {
            reduced_abstract_class_value_from_facts(facts)
        }
        ClassValueFlowTransferV0::RefineFacts(facts) => {
            let refinement = reduced_abstract_class_value_from_facts(facts);
            intersect_abstract_class_values(incoming, &refinement)
        }
        ClassValueFlowTransferV0::ConcatFacts(facts) => {
            let right = reduced_abstract_class_value_from_facts(facts);
            concatenate_abstract_class_values(incoming, &right)
        }
        ClassValueFlowTransferV0::Join => incoming.clone(),
    }
}

fn flow_transfer_kind(transfer: &ClassValueFlowTransferV0) -> &'static str {
    match transfer {
        ClassValueFlowTransferV0::AssignFacts(_) => "assignFacts",
        ClassValueFlowTransferV0::RefineFacts(_) => "refineFacts",
        ClassValueFlowTransferV0::ConcatFacts(_) => "concatFacts",
        ClassValueFlowTransferV0::Join => "join",
    }
}

fn flow_node_incremental_digest(node: &ClassValueFlowNodeV0) -> String {
    let mut parts = vec![
        format!("id={}", node.id),
        format!("deps={}", node.predecessors.join(",")),
        format!("transfer={}", flow_transfer_kind(&node.transfer)),
    ];

    match &node.transfer {
        ClassValueFlowTransferV0::AssignFacts(facts)
        | ClassValueFlowTransferV0::RefineFacts(facts)
        | ClassValueFlowTransferV0::ConcatFacts(facts) => {
            push_external_facts_digest_parts(&mut parts, facts);
        }
        ClassValueFlowTransferV0::Join => {}
    }

    parts.join(";")
}

fn push_external_facts_digest_parts(parts: &mut Vec<String>, facts: &ExternalStringTypeFactsV0) {
    parts.push(format!("kind={}", facts.kind));
    parts.push(format!(
        "constraint={}",
        facts.constraint_kind.as_deref().unwrap_or("")
    ));
    parts.push(format!(
        "values={}",
        facts.values.as_ref().map_or_else(String::new, |values| {
            let mut values = values.clone();
            values.sort();
            values.dedup();
            values.join(",")
        })
    ));
    parts.push(format!("prefix={}", facts.prefix.as_deref().unwrap_or("")));
    parts.push(format!("suffix={}", facts.suffix.as_deref().unwrap_or("")));
    parts.push(format!(
        "minLen={}",
        facts
            .min_len
            .map_or_else(String::new, |value| value.to_string())
    ));
    parts.push(format!(
        "maxLen={}",
        facts
            .max_len
            .map_or_else(String::new, |value| value.to_string())
    ));
    parts.push(format!(
        "charMust={}",
        facts.char_must.as_deref().unwrap_or("")
    ));
    parts.push(format!(
        "charMay={}",
        facts.char_may.as_deref().unwrap_or("")
    ));
    parts.push(format!(
        "mayOther={}",
        facts
            .may_include_other_chars
            .map_or_else(String::new, |value| value.to_string())
    ));
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

fn char_set_is_subset(left: &str, right: &str) -> bool {
    let right = right.chars().collect::<BTreeSet<_>>();
    left.chars().all(|char| right.contains(&char))
}

fn is_false(value: &bool) -> bool {
    !value
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

#[cfg(test)]
mod tests {
    use super::{
        AbstractClassValueProvenanceV0, AbstractClassValueV0, ClassValueFlowGraphV0,
        ClassValueFlowNodeV0, ClassValueFlowTransferV0, CompositeClassValueInputV0,
        ExternalStringTypeFactsV0, MAX_FINITE_CLASS_VALUES, OneCfaCallSiteFlowInputV0,
        SelectorProjectionCertaintyV0, abstract_class_value_from_facts, analyze_class_value_flow,
        analyze_class_value_flow_incremental,
        analyze_class_value_flow_incremental_batch_with_reuse,
        analyze_class_value_flow_incremental_with_database,
        analyze_class_value_flow_incremental_with_reuse, analyze_one_cfa_call_site_flows,
        bottom_class_value, char_inclusion_class_value, composite_class_value,
        concatenate_abstract_class_values, derive_selector_projection_certainty, exact_class_value,
        finite_set_class_value, finite_values_from_facts, intersect_abstract_class_values,
        join_abstract_class_values, prefix_class_value, prefix_suffix_class_value,
        project_abstract_value_selectors, reduced_abstract_class_value_from_facts,
        reduced_class_value_derivation_from_facts, reduced_value_domain_kind_from_facts,
        selector_certainty_from_facts, selector_certainty_shape_kind_from_facts,
        selector_certainty_shape_label_from_facts, suffix_class_value,
        summarize_omena_abstract_value_domain, summarize_omena_abstract_value_flow_analysis,
        top_class_value, value_certainty_from_facts, value_certainty_shape_kind_from_facts,
        value_certainty_shape_label_from_facts,
    };
    use omena_incremental::OmenaIncrementalDatabaseV0;
    use std::collections::BTreeMap;

    #[test]
    fn summarizes_domain_boundary_contract() {
        let summary = summarize_omena_abstract_value_domain();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-abstract-value.domain");
        assert_eq!(summary.max_finite_class_values, MAX_FINITE_CLASS_VALUES);
        assert!(summary.domain_kinds.contains(&"exact"));
        assert!(summary.domain_kinds.contains(&"composite"));
        assert!(
            summary
                .selector_projection_certainties
                .contains(&"inferred")
        );

        let flow_summary = summarize_omena_abstract_value_flow_analysis();
        assert_eq!(flow_summary.schema_version, "0");
        assert_eq!(flow_summary.product, "omena-abstract-value.flow-analysis");
        assert_eq!(flow_summary.context_sensitivity, "1-cfa");
        assert_eq!(flow_summary.incremental_engine, "omena-incremental");
        assert!(flow_summary.analysis_scopes.contains(&"multiContextBatch"));
        assert!(flow_summary.analysis_scopes.contains(&"callSiteBatch"));
        assert_eq!(
            flow_summary.reuse_policy,
            "reuse previous context analysis when its omena-incremental plan is clean"
        );
        assert!(flow_summary.transfer_kinds.contains(&"join"));
        assert!(flow_summary.transfer_kinds.contains(&"concatFacts"));
    }

    #[test]
    fn normalizes_finite_sets_to_bottom_exact_or_sorted_unique_values() {
        assert_eq!(
            finite_set_class_value(Vec::<String>::new()),
            AbstractClassValueV0::Bottom
        );
        assert_eq!(
            finite_set_class_value(["button"]),
            exact_class_value("button")
        );
        assert_eq!(
            finite_set_class_value(["card", "button", "card"]),
            AbstractClassValueV0::FiniteSet {
                values: vec!["button".to_string(), "card".to_string()]
            }
        );
    }

    #[test]
    fn maps_external_string_facts_to_stable_value_certainty_labels() {
        let exact = external_facts("exact").with_values(["button"]);
        assert_eq!(
            abstract_class_value_from_facts(&exact),
            exact_class_value("button")
        );
        assert_eq!(value_certainty_from_facts(&exact), Some("exact"));
        assert_eq!(value_certainty_shape_kind_from_facts(&exact), "exact");
        assert_eq!(value_certainty_shape_label_from_facts(&exact), "exact");
        assert_eq!(
            finite_values_from_facts(&exact),
            Some(vec!["button".to_string()])
        );

        let finite = external_facts("finiteSet").with_values(["card", "button", "card"]);
        assert_eq!(value_certainty_from_facts(&finite), Some("inferred"));
        assert_eq!(
            value_certainty_shape_kind_from_facts(&finite),
            "boundedFinite"
        );
        assert_eq!(
            value_certainty_shape_label_from_facts(&finite),
            "bounded finite (2)"
        );
        assert_eq!(selector_certainty_from_facts(&finite, 1, 3), "inferred");
        assert_eq!(
            selector_certainty_shape_label_from_facts(&finite, 1, 3),
            "bounded selector set (1)"
        );
    }

    #[test]
    fn maps_constrained_external_string_facts_to_stable_shape_labels() {
        let edge = external_facts("constrained")
            .with_constraint_kind("prefixSuffix")
            .with_prefix("btn-")
            .with_suffix("-active")
            .with_min_len(11);

        assert_eq!(
            abstract_class_value_from_facts(&edge),
            AbstractClassValueV0::PrefixSuffix {
                prefix: "btn-".to_string(),
                suffix: "-active".to_string(),
                min_length: 11,
                provenance: None,
            }
        );
        assert_eq!(value_certainty_from_facts(&edge), Some("inferred"));
        assert_eq!(value_certainty_shape_kind_from_facts(&edge), "constrained");
        assert_eq!(
            value_certainty_shape_label_from_facts(&edge),
            "constrained prefix `btn-` + suffix `-active`"
        );
        assert_eq!(selector_certainty_from_facts(&edge, 1, 3), "inferred");
        assert_eq!(
            selector_certainty_shape_kind_from_facts(&edge, 1, 3),
            "constrained"
        );
        assert_eq!(
            selector_certainty_shape_label_from_facts(&edge, 1, 3),
            "constrained edge selector set (1)"
        );
    }

    #[test]
    fn widens_large_finite_sets_to_composite_when_edges_survive() {
        let values = (0..=MAX_FINITE_CLASS_VALUES)
            .map(|index| format!("btn-{index}-active"))
            .collect::<Vec<_>>();

        let value = finite_set_class_value(values);

        assert_eq!(
            value,
            AbstractClassValueV0::Composite {
                prefix: Some("btn-".to_string()),
                suffix: Some("-active".to_string()),
                min_length: Some("btn-0-active".len()),
                must_chars: "-abceintv".to_string(),
                may_chars: "-012345678abceintv".to_string(),
                may_include_other_chars: false,
                provenance: Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite),
            }
        );
    }

    #[test]
    fn builds_char_inclusion_and_composite_values_with_normalized_chars() {
        assert_eq!(
            char_inclusion_class_value(
                "ba",
                "cad",
                Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars),
                false,
            ),
            AbstractClassValueV0::CharInclusion {
                must_chars: "ab".to_string(),
                may_chars: "abcd".to_string(),
                may_include_other_chars: false,
                provenance: Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars),
            }
        );

        assert_eq!(
            composite_class_value(CompositeClassValueInputV0 {
                prefix: Some("btn-".to_string()),
                suffix: Some("-active".to_string()),
                min_length: None,
                must_chars: "z".to_string(),
                may_chars: "za".to_string(),
                may_include_other_chars: true,
                provenance: None,
            }),
            AbstractClassValueV0::Composite {
                prefix: Some("btn-".to_string()),
                suffix: Some("-active".to_string()),
                min_length: Some("btn--active".len()),
                must_chars: "-abceintvz".to_string(),
                may_chars: "-abceintvz".to_string(),
                may_include_other_chars: true,
                provenance: None,
            }
        );
    }

    #[test]
    fn intersects_finite_values_with_constrained_domains() {
        let finite = finite_set_class_value(["btn-primary", "card", "btn-secondary"]);
        let prefix = prefix_class_value("btn-", None);

        assert_eq!(
            intersect_abstract_class_values(&finite, &prefix),
            AbstractClassValueV0::FiniteSet {
                values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
            }
        );

        assert_eq!(
            intersect_abstract_class_values(
                &exact_class_value("card"),
                &prefix_class_value("btn-", None),
            ),
            AbstractClassValueV0::Bottom
        );
    }

    #[test]
    fn intersects_prefix_suffix_and_char_constraints_into_reduced_product() {
        let edge = intersect_abstract_class_values(
            &prefix_class_value("btn-", None),
            &suffix_class_value("-active", None),
        );

        assert_eq!(
            edge,
            AbstractClassValueV0::PrefixSuffix {
                prefix: "btn-".to_string(),
                suffix: "-active".to_string(),
                min_length: "btn--active".len(),
                provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
            }
        );

        let reduced = intersect_abstract_class_values(
            &edge,
            &char_inclusion_class_value("ab", "-abceintv", None, false),
        );

        assert_eq!(
            reduced,
            AbstractClassValueV0::Composite {
                prefix: Some("btn-".to_string()),
                suffix: Some("-active".to_string()),
                min_length: Some("btn--active".len()),
                must_chars: "-abceintv".to_string(),
                may_chars: "-abceintv".to_string(),
                may_include_other_chars: false,
                provenance: Some(AbstractClassValueProvenanceV0::CompositeJoin),
            }
        );
    }

    #[test]
    fn rejects_incompatible_reduced_product_constraints() {
        assert_eq!(
            intersect_abstract_class_values(
                &prefix_class_value("btn-", None),
                &prefix_class_value("card-", None),
            ),
            AbstractClassValueV0::Bottom
        );

        assert_eq!(
            intersect_abstract_class_values(
                &prefix_class_value("btn-", None),
                &char_inclusion_class_value("", "abc", None, false),
            ),
            AbstractClassValueV0::Bottom
        );
    }

    #[test]
    fn reduced_product_laws_hold_over_selector_projection() {
        let selectors = selector_universe([
            "btn-primary",
            "btn-secondary",
            "btn-active",
            "card",
            "card-active",
            "nav-active",
        ]);
        let finite = finite_set_class_value([
            "btn-primary",
            "btn-secondary",
            "card",
            "card-active",
            "missing",
        ]);
        let prefix = prefix_class_value("btn-", None);
        let suffix = suffix_class_value("-active", None);
        let chars = char_inclusion_class_value("ab", "-abceintv", None, false);
        let composite = composite_class_value(CompositeClassValueInputV0 {
            prefix: Some("btn-".to_string()),
            suffix: Some("-active".to_string()),
            min_length: Some("btn--active".len()),
            must_chars: "ab".to_string(),
            may_chars: "-abceintv".to_string(),
            may_include_other_chars: false,
            provenance: None,
        });

        for (left, right) in [
            (&finite, &prefix),
            (&prefix, &suffix),
            (&suffix, &chars),
            (&prefix, &composite),
        ] {
            assert_projection_equivalent(
                &intersect_abstract_class_values(left, right),
                &intersect_abstract_class_values(right, left),
                &selectors,
            );
        }

        for value in [&finite, &prefix, &suffix, &chars, &composite] {
            assert_projection_equivalent(
                &intersect_abstract_class_values(value, value),
                value,
                &selectors,
            );
        }

        assert_eq!(
            intersect_abstract_class_values(&top_class_value(), &finite),
            finite
        );
        assert_eq!(
            intersect_abstract_class_values(&finite, &top_class_value()),
            finite
        );
        assert_eq!(
            intersect_abstract_class_values(&bottom_class_value(), &finite),
            bottom_class_value()
        );
        assert_eq!(
            intersect_abstract_class_values(&finite, &bottom_class_value()),
            bottom_class_value()
        );
    }

    #[test]
    fn reduced_product_projection_matches_intersected_projection_sets() {
        let selectors = selector_universe([
            "btn-primary",
            "btn-secondary",
            "btn-active",
            "card",
            "card-active",
            "nav-active",
        ]);
        let finite = finite_set_class_value([
            "btn-primary",
            "btn-secondary",
            "card",
            "card-active",
            "missing",
        ]);
        let prefix = prefix_class_value("btn-", None);
        let suffix = suffix_class_value("-active", None);
        let prefix_suffix = intersect_abstract_class_values(&prefix, &suffix);

        assert_eq!(
            projected_names(
                &intersect_abstract_class_values(&finite, &prefix),
                &selectors
            ),
            vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        );
        assert_eq!(
            projected_names(
                &intersect_abstract_class_values(&finite, &prefix),
                &selectors
            ),
            intersect_projected_names(&finite, &prefix, &selectors)
        );
        assert_eq!(
            projected_names(
                &intersect_abstract_class_values(&finite, &prefix_suffix),
                &selectors,
            ),
            intersect_projected_names(&finite, &prefix_suffix, &selectors)
        );
    }

    #[test]
    fn joins_abstract_values_for_branch_merges() {
        assert_eq!(
            join_abstract_class_values(
                &exact_class_value("btn-primary"),
                &exact_class_value("btn-secondary"),
            ),
            AbstractClassValueV0::FiniteSet {
                values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
            }
        );

        assert_eq!(
            join_abstract_class_values(
                &prefix_class_value("btn-primary-", None),
                &prefix_class_value("btn-secondary-", None),
            ),
            prefix_class_value("btn-", Some(AbstractClassValueProvenanceV0::PrefixJoinLcp))
        );

        assert_eq!(
            join_abstract_class_values(
                &prefix_class_value("btn-", None),
                &exact_class_value("btn-primary"),
            ),
            prefix_class_value("btn-", None)
        );
    }

    #[test]
    fn concatenates_abstract_values_for_template_edges() {
        assert_eq!(
            concatenate_abstract_class_values(
                &exact_class_value("btn-"),
                &finite_set_class_value(["primary", "secondary"]),
            ),
            AbstractClassValueV0::FiniteSet {
                values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
            }
        );

        assert_eq!(
            concatenate_abstract_class_values(
                &prefix_class_value("btn-", None),
                &suffix_class_value("-active", None),
            ),
            prefix_suffix_class_value("btn-", "-active", Some("btn--active".len()), None)
        );

        assert_eq!(
            concatenate_abstract_class_values(
                &finite_set_class_value(["card-primary", "card-secondary"]),
                &prefix_class_value("--", None),
            ),
            prefix_class_value("card-", None)
        );
    }

    #[test]
    fn analyzes_flow_concat_facts_before_refinement() {
        let graph = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@concat".to_string()),
            nodes: vec![
                flow_assign_node("base", external_facts("exact").with_values(["btn-"])),
                ClassValueFlowNodeV0 {
                    id: "variant".to_string(),
                    predecessors: vec!["base".to_string()],
                    transfer: ClassValueFlowTransferV0::ConcatFacts(
                        external_facts("finiteSet").with_values(["primary", "secondary", "icon"]),
                    ),
                },
                ClassValueFlowNodeV0 {
                    id: "btn-only".to_string(),
                    predecessors: vec!["variant".to_string()],
                    transfer: ClassValueFlowTransferV0::RefineFacts(
                        external_facts("constrained")
                            .with_constraint_kind("suffix")
                            .with_suffix("primary"),
                    ),
                },
            ],
        };

        let analysis = analyze_class_value_flow(&graph);

        assert_eq!(
            flow_value(&analysis, "variant"),
            Some(&AbstractClassValueV0::FiniteSet {
                values: vec![
                    "btn-icon".to_string(),
                    "btn-primary".to_string(),
                    "btn-secondary".to_string(),
                ]
            })
        );
        assert_eq!(
            flow_value(&analysis, "btn-only"),
            Some(&AbstractClassValueV0::Exact {
                value: "btn-primary".to_string()
            })
        );
        assert_eq!(analysis.nodes[1].transfer_kind, "concatFacts");
    }

    #[test]
    fn analyzes_one_cfa_class_value_flow_with_branch_merge_and_refinement() {
        let graph = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@primary".to_string()),
            nodes: vec![
                flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
                flow_assign_node(
                    "else-if",
                    external_facts("exact").with_values(["btn-secondary"]),
                ),
                flow_assign_node("else", external_facts("exact").with_values(["card"])),
                ClassValueFlowNodeV0 {
                    id: "merge".to_string(),
                    predecessors: vec![
                        "then".to_string(),
                        "else-if".to_string(),
                        "else".to_string(),
                    ],
                    transfer: ClassValueFlowTransferV0::Join,
                },
                ClassValueFlowNodeV0 {
                    id: "btn-only".to_string(),
                    predecessors: vec!["merge".to_string()],
                    transfer: ClassValueFlowTransferV0::RefineFacts(
                        external_facts("constrained")
                            .with_constraint_kind("prefix")
                            .with_prefix("btn-"),
                    ),
                },
            ],
        };

        let analysis = analyze_class_value_flow(&graph);

        assert_eq!(analysis.schema_version, "0");
        assert_eq!(analysis.product, "omena-abstract-value.flow-analysis");
        assert_eq!(analysis.context_sensitivity, "1-cfa");
        assert_eq!(
            analysis.context_key.as_deref(),
            Some("Button.tsx:render@primary")
        );
        assert!(analysis.converged);

        assert_eq!(
            flow_value(&analysis, "merge"),
            Some(&AbstractClassValueV0::FiniteSet {
                values: vec![
                    "btn-primary".to_string(),
                    "btn-secondary".to_string(),
                    "card".to_string(),
                ]
            })
        );
        assert_eq!(
            flow_value(&analysis, "btn-only"),
            Some(&AbstractClassValueV0::FiniteSet {
                values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
            })
        );
    }

    #[test]
    fn analyzes_one_cfa_call_site_flows_with_context_discrimination() {
        let primary_graph = ClassValueFlowGraphV0 {
            context_key: None,
            nodes: vec![
                flow_assign_node(
                    "variant",
                    external_facts("exact").with_values(["btn-primary"]),
                ),
                ClassValueFlowNodeV0 {
                    id: "exit".to_string(),
                    predecessors: vec!["variant".to_string()],
                    transfer: ClassValueFlowTransferV0::RefineFacts(
                        external_facts("constrained")
                            .with_constraint_kind("prefix")
                            .with_prefix("btn-"),
                    ),
                },
            ],
        };
        let secondary_graph = ClassValueFlowGraphV0 {
            context_key: None,
            nodes: vec![
                flow_assign_node(
                    "variant",
                    external_facts("exact").with_values(["btn-secondary"]),
                ),
                ClassValueFlowNodeV0 {
                    id: "exit".to_string(),
                    predecessors: vec!["variant".to_string()],
                    transfer: ClassValueFlowTransferV0::RefineFacts(
                        external_facts("constrained")
                            .with_constraint_kind("prefix")
                            .with_prefix("btn-"),
                    ),
                },
            ],
        };

        let analysis = analyze_one_cfa_call_site_flows(&[
            OneCfaCallSiteFlowInputV0 {
                callee_key: "classForVariant".to_string(),
                call_site_id: "Button.tsx:10:className".to_string(),
                graph: primary_graph,
                exit_node_id: "exit".to_string(),
            },
            OneCfaCallSiteFlowInputV0 {
                callee_key: "classForVariant".to_string(),
                call_site_id: "Card.tsx:22:className".to_string(),
                graph: secondary_graph,
                exit_node_id: "exit".to_string(),
            },
        ]);

        assert_eq!(analysis.schema_version, "0");
        assert_eq!(
            analysis.product,
            "omena-abstract-value.one-cfa-call-site-flow"
        );
        assert_eq!(analysis.context_sensitivity, "1-cfa");
        assert_eq!(analysis.call_site_count, 2);
        assert_eq!(analysis.callee_count, 1);
        assert_eq!(
            analysis.entries[0].context_key,
            "classForVariant@Button.tsx:10:className"
        );
        assert_eq!(
            analysis.entries[1].context_key,
            "classForVariant@Card.tsx:22:className"
        );
        assert_eq!(
            analysis.entries[0].exit_value,
            exact_class_value("btn-primary")
        );
        assert_eq!(
            analysis.entries[1].exit_value,
            exact_class_value("btn-secondary")
        );
        assert_eq!(
            analysis.callee_summaries[0].joined_exit_value,
            AbstractClassValueV0::FiniteSet {
                values: vec!["btn-primary".to_string(), "btn-secondary".to_string()]
            }
        );
        assert_eq!(analysis.entries[0].derivation.steps.len(), 3);
        assert_eq!(
            analysis.entries[0].derivation.steps[0].operation,
            "contextFromCallSite"
        );
        assert_eq!(
            analysis.entries[0].derivation.steps[2].operation,
            "projectExitNode"
        );
        assert_eq!(analysis.entries[0].derivation.steps[2].result_kind, "exact");
    }

    #[test]
    fn analyzes_class_value_flow_on_incremental_plan() {
        let graph = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@primary".to_string()),
            nodes: vec![
                flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
                flow_assign_node("else", external_facts("exact").with_values(["card"])),
                ClassValueFlowNodeV0 {
                    id: "merge".to_string(),
                    predecessors: vec!["then".to_string(), "else".to_string()],
                    transfer: ClassValueFlowTransferV0::Join,
                },
            ],
        };

        let first = analyze_class_value_flow_incremental(&graph, None, 1);
        assert_eq!(
            first.product,
            "omena-abstract-value.incremental-flow-analysis"
        );
        assert!(!first.reused_previous_analysis);
        assert_eq!(first.incremental_plan.dirty_node_count, 3);
        assert_eq!(first.incremental_plan.new_node_count, 3);
        assert_eq!(
            flow_value(&first.analysis, "merge"),
            Some(&AbstractClassValueV0::FiniteSet {
                values: vec!["btn-primary".to_string(), "card".to_string()]
            })
        );

        let unchanged = analyze_class_value_flow_incremental(&graph, Some(&first.next_snapshot), 2);
        assert_eq!(unchanged.incremental_plan.dirty_node_count, 0);
        assert!(!unchanged.reused_previous_analysis);
        assert!(unchanged.analysis.converged);

        let changed_graph = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@primary".to_string()),
            nodes: vec![
                flow_assign_node(
                    "then",
                    external_facts("exact").with_values(["btn-secondary"]),
                ),
                flow_assign_node("else", external_facts("exact").with_values(["card"])),
                ClassValueFlowNodeV0 {
                    id: "merge".to_string(),
                    predecessors: vec!["then".to_string(), "else".to_string()],
                    transfer: ClassValueFlowTransferV0::Join,
                },
            ],
        };
        let changed =
            analyze_class_value_flow_incremental(&changed_graph, Some(&first.next_snapshot), 3);

        assert_eq!(changed.incremental_plan.changed_input_count, 1);
        assert_eq!(changed.incremental_plan.dependency_dirty_count, 1);
        assert_eq!(
            flow_value(&changed.analysis, "merge"),
            Some(&AbstractClassValueV0::FiniteSet {
                values: vec!["btn-secondary".to_string(), "card".to_string()]
            })
        );
    }

    #[test]
    fn reuses_previous_class_value_flow_analysis_when_incremental_plan_is_clean() {
        let graph = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@primary".to_string()),
            nodes: vec![
                flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
                flow_assign_node("else", external_facts("exact").with_values(["card"])),
                ClassValueFlowNodeV0 {
                    id: "merge".to_string(),
                    predecessors: vec!["then".to_string(), "else".to_string()],
                    transfer: ClassValueFlowTransferV0::Join,
                },
            ],
        };
        let first = analyze_class_value_flow_incremental(&graph, None, 1);

        let reused = analyze_class_value_flow_incremental_with_reuse(
            &graph,
            Some(&first.next_snapshot),
            Some(&first.analysis),
            2,
        );

        assert_eq!(reused.incremental_plan.dirty_node_count, 0);
        assert!(reused.reused_previous_analysis);
        assert_eq!(reused.analysis, first.analysis);
    }

    #[test]
    fn reuses_previous_class_value_flow_analysis_through_salsa_database() {
        let graph = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@primary".to_string()),
            nodes: vec![
                flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
                flow_assign_node("else", external_facts("exact").with_values(["card"])),
                ClassValueFlowNodeV0 {
                    id: "merge".to_string(),
                    predecessors: vec!["then".to_string(), "else".to_string()],
                    transfer: ClassValueFlowTransferV0::Join,
                },
            ],
        };
        let mut incremental_database = OmenaIncrementalDatabaseV0::default();
        let first = analyze_class_value_flow_incremental_with_database(
            &graph,
            &mut incremental_database,
            None,
            1,
        );

        assert_eq!(
            first.next_snapshot.product,
            "omena-incremental.salsa-snapshot"
        );
        assert_eq!(first.incremental_plan.dirty_node_count, 3);
        assert!(!first.reused_previous_analysis);

        let reused = analyze_class_value_flow_incremental_with_database(
            &graph,
            &mut incremental_database,
            Some(&first.analysis),
            2,
        );

        assert_eq!(reused.incremental_plan.dirty_node_count, 0);
        assert!(reused.reused_previous_analysis);
        assert_eq!(reused.analysis, first.analysis);
    }

    #[test]
    fn reuses_clean_contexts_in_incremental_flow_batch() {
        let primary = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@primary".to_string()),
            nodes: vec![
                flow_assign_node("then", external_facts("exact").with_values(["btn-primary"])),
                flow_assign_node("else", external_facts("exact").with_values(["card"])),
                ClassValueFlowNodeV0 {
                    id: "merge".to_string(),
                    predecessors: vec!["then".to_string(), "else".to_string()],
                    transfer: ClassValueFlowTransferV0::Join,
                },
            ],
        };
        let secondary = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@secondary".to_string()),
            nodes: vec![
                flow_assign_node(
                    "base",
                    external_facts("exact").with_values(["btn-secondary"]),
                ),
                ClassValueFlowNodeV0 {
                    id: "refined".to_string(),
                    predecessors: vec!["base".to_string()],
                    transfer: ClassValueFlowTransferV0::RefineFacts(
                        external_facts("prefix").with_prefix("btn-"),
                    ),
                },
            ],
        };
        let first = analyze_class_value_flow_incremental_batch_with_reuse(
            &[primary.clone(), secondary.clone()],
            &BTreeMap::new(),
            &BTreeMap::new(),
            1,
        );
        let previous_snapshots = first
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.context_key.clone(),
                    entry.analysis.next_snapshot.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let previous_analyses = first
            .entries
            .iter()
            .map(|entry| (entry.context_key.clone(), entry.analysis.analysis.clone()))
            .collect::<BTreeMap<_, _>>();
        let changed_secondary = ClassValueFlowGraphV0 {
            context_key: Some("Button.tsx:render@secondary".to_string()),
            nodes: vec![
                flow_assign_node(
                    "base",
                    external_facts("exact").with_values(["btn-tertiary"]),
                ),
                ClassValueFlowNodeV0 {
                    id: "refined".to_string(),
                    predecessors: vec!["base".to_string()],
                    transfer: ClassValueFlowTransferV0::RefineFacts(
                        external_facts("prefix").with_prefix("btn-"),
                    ),
                },
            ],
        };

        let second = analyze_class_value_flow_incremental_batch_with_reuse(
            &[primary, changed_secondary],
            &previous_snapshots,
            &previous_analyses,
            2,
        );

        assert_eq!(
            second.product,
            "omena-abstract-value.incremental-flow-analysis-batch"
        );
        assert_eq!(second.context_count, 2);
        assert_eq!(second.reused_context_count, 1);
        assert_eq!(second.dirty_context_count, 1);
        assert!(second.entries[0].analysis.reused_previous_analysis);
        assert!(!second.entries[1].analysis.reused_previous_analysis);
        assert_eq!(
            flow_value(&second.entries[1].analysis.analysis, "refined"),
            Some(&AbstractClassValueV0::Exact {
                value: "btn-tertiary".to_string()
            })
        );
    }

    #[test]
    fn reduces_external_facts_before_reporting_domain_kind() {
        let finite_with_prefix = external_facts("finiteSet")
            .with_values(["btn-primary", "card"])
            .with_constraint_kind("prefix")
            .with_prefix("btn-");

        assert_eq!(
            reduced_abstract_class_value_from_facts(&finite_with_prefix),
            exact_class_value("btn-primary")
        );
        assert_eq!(
            reduced_value_domain_kind_from_facts(&finite_with_prefix),
            "exact"
        );

        let constrained_with_values = external_facts("constrained")
            .with_values(["btn-primary", "card"])
            .with_constraint_kind("prefix")
            .with_prefix("btn-");

        assert_eq!(
            reduced_abstract_class_value_from_facts(&constrained_with_values),
            exact_class_value("btn-primary")
        );

        let finite_with_conflicting_prefix = external_facts("finiteSet")
            .with_values(["btn-primary", "card"])
            .with_constraint_kind("prefix")
            .with_prefix("nav-");

        assert_eq!(
            reduced_abstract_class_value_from_facts(&finite_with_conflicting_prefix),
            bottom_class_value()
        );
        assert_eq!(
            reduced_value_domain_kind_from_facts(&finite_with_conflicting_prefix),
            "bottom"
        );
        assert_eq!(
            reduced_value_domain_kind_from_facts(&external_facts("unknown")),
            "none"
        );
    }

    #[test]
    fn explains_reduced_external_fact_derivation_steps() {
        let finite_with_prefix = external_facts("finiteSet")
            .with_values(["btn-primary", "card"])
            .with_constraint_kind("prefix")
            .with_prefix("btn-");

        let derivation = reduced_class_value_derivation_from_facts(&finite_with_prefix);

        assert_eq!(derivation.schema_version, "0");
        assert_eq!(
            derivation.product,
            "omena-abstract-value.reduced-class-value-derivation"
        );
        assert_eq!(derivation.input_fact_kind, "finiteSet");
        assert_eq!(derivation.input_constraint_kind.as_deref(), Some("prefix"));
        assert_eq!(derivation.input_value_count, 2);
        assert_eq!(derivation.reduced_kind, "exact");
        assert_eq!(derivation.steps.len(), 2);
        assert_eq!(derivation.steps[0].operation, "baseFromFacts");
        assert_eq!(derivation.steps[0].result_kind, "finiteSet");
        assert_eq!(derivation.steps[1].operation, "intersectConstraint");
        assert_eq!(derivation.steps[1].input_kind, Some("finiteSet"));
        assert_eq!(derivation.steps[1].refinement_kind, Some("prefix"));
        assert_eq!(derivation.steps[1].result_kind, "exact");
    }

    #[test]
    fn explains_constrained_finite_value_derivation_steps() {
        let constrained_with_values = external_facts("constrained")
            .with_values(["btn-primary", "btn-secondary", "card"])
            .with_constraint_kind("prefix")
            .with_prefix("btn-");

        let derivation = reduced_class_value_derivation_from_facts(&constrained_with_values);

        assert_eq!(derivation.input_fact_kind, "constrained");
        assert_eq!(derivation.input_constraint_kind.as_deref(), Some("prefix"));
        assert_eq!(derivation.input_value_count, 3);
        assert_eq!(derivation.reduced_kind, "finiteSet");
        assert_eq!(derivation.steps.len(), 2);
        assert_eq!(derivation.steps[0].operation, "baseFromFacts");
        assert_eq!(derivation.steps[0].result_kind, "prefix");
        assert_eq!(derivation.steps[1].operation, "intersectFiniteValues");
        assert_eq!(derivation.steps[1].input_kind, Some("prefix"));
        assert_eq!(derivation.steps[1].refinement_kind, Some("finiteSet"));
        assert_eq!(derivation.steps[1].result_kind, "finiteSet");
    }

    #[test]
    fn projects_exact_and_finite_values_into_selector_universe() {
        let selectors = selector_universe(["button", "card", "link"]);

        let exact = project_abstract_value_selectors(&exact_class_value("button"), &selectors);
        assert_eq!(exact.selector_names, vec!["button".to_string()]);
        assert_eq!(exact.certainty, SelectorProjectionCertaintyV0::Exact);

        let finite = project_abstract_value_selectors(
            &finite_set_class_value(["button", "missing"]),
            &selectors,
        );
        assert_eq!(finite.selector_names, vec!["button".to_string()]);
        assert_eq!(finite.certainty, SelectorProjectionCertaintyV0::Inferred);
    }

    #[test]
    fn projects_constrained_values_into_selector_universe() {
        let selectors = selector_universe(["btn-primary", "btn-secondary", "card", "link-active"]);

        let prefix = project_abstract_value_selectors(
            &prefix_class_value("btn-", Some(AbstractClassValueProvenanceV0::PrefixJoinLcp)),
            &selectors,
        );
        assert_eq!(
            prefix.selector_names,
            vec!["btn-primary".to_string(), "btn-secondary".to_string()]
        );
        assert_eq!(prefix.certainty, SelectorProjectionCertaintyV0::Inferred);

        let edge = project_abstract_value_selectors(
            &prefix_suffix_class_value("btn-", "primary", None, None),
            &selectors,
        );
        assert_eq!(edge.selector_names, vec!["btn-primary".to_string()]);
        assert_eq!(edge.certainty, SelectorProjectionCertaintyV0::Inferred);

        let chars = project_abstract_value_selectors(
            &char_inclusion_class_value("ac", "acdr", None, false),
            &selectors,
        );
        assert_eq!(chars.selector_names, vec!["card".to_string()]);
        assert_eq!(chars.certainty, SelectorProjectionCertaintyV0::Inferred);
    }

    #[test]
    fn derives_projection_certainty_from_domain_and_selector_coverage() {
        assert_eq!(
            derive_selector_projection_certainty(&AbstractClassValueV0::Bottom, 0, 3),
            SelectorProjectionCertaintyV0::Possible
        );
        assert_eq!(
            derive_selector_projection_certainty(&prefix_class_value("btn-", None), 3, 3,),
            SelectorProjectionCertaintyV0::Exact
        );
        assert_eq!(
            derive_selector_projection_certainty(&AbstractClassValueV0::Top, 3, 3),
            SelectorProjectionCertaintyV0::Possible
        );
    }

    fn selector_universe(values: impl IntoIterator<Item = &'static str>) -> Vec<String> {
        values.into_iter().map(str::to_string).collect()
    }

    fn assert_projection_equivalent(
        left: &AbstractClassValueV0,
        right: &AbstractClassValueV0,
        selectors: &[String],
    ) {
        assert_eq!(
            projected_names(left, selectors),
            projected_names(right, selectors)
        );
    }

    fn projected_names(value: &AbstractClassValueV0, selectors: &[String]) -> Vec<String> {
        project_abstract_value_selectors(value, selectors).selector_names
    }

    fn intersect_projected_names(
        left: &AbstractClassValueV0,
        right: &AbstractClassValueV0,
        selectors: &[String],
    ) -> Vec<String> {
        let right_names = projected_names(right, selectors)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        projected_names(left, selectors)
            .into_iter()
            .filter(|name| right_names.contains(name))
            .collect()
    }

    fn flow_assign_node(id: &str, facts: ExternalStringTypeFactsV0) -> ClassValueFlowNodeV0 {
        ClassValueFlowNodeV0 {
            id: id.to_string(),
            predecessors: Vec::new(),
            transfer: ClassValueFlowTransferV0::AssignFacts(facts),
        }
    }

    fn flow_value<'a>(
        analysis: &'a super::ClassValueFlowAnalysisV0,
        id: &str,
    ) -> Option<&'a AbstractClassValueV0> {
        analysis
            .nodes
            .iter()
            .find(|node| node.id == id)
            .map(|node| &node.value)
    }

    fn external_facts(kind: &str) -> ExternalStringTypeFactsV0 {
        ExternalStringTypeFactsV0 {
            kind: kind.to_string(),
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

    trait ExternalFactsTestExt {
        fn with_values(self, values: impl IntoIterator<Item = &'static str>) -> Self;
        fn with_constraint_kind(self, value: &'static str) -> Self;
        fn with_prefix(self, value: &'static str) -> Self;
        fn with_suffix(self, value: &'static str) -> Self;
        fn with_min_len(self, value: usize) -> Self;
    }

    impl ExternalFactsTestExt for ExternalStringTypeFactsV0 {
        fn with_values(mut self, values: impl IntoIterator<Item = &'static str>) -> Self {
            self.values = Some(values.into_iter().map(str::to_string).collect());
            self
        }

        fn with_constraint_kind(mut self, value: &'static str) -> Self {
            self.constraint_kind = Some(value.to_string());
            self
        }

        fn with_prefix(mut self, value: &'static str) -> Self {
            self.prefix = Some(value.to_string());
            self
        }

        fn with_suffix(mut self, value: &'static str) -> Self {
            self.suffix = Some(value.to_string());
            self
        }

        fn with_min_len(mut self, value: usize) -> Self {
            self.min_len = Some(value);
            self
        }
    }
}
