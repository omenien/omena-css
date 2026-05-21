use omena_incremental::{IncrementalComputationPlanV0, IncrementalSnapshotV0};
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
    pub reduced_product_structure_ready: bool,
    pub reduced_product_axes: Vec<&'static str>,
    pub reduced_product_operations: Vec<&'static str>,
    pub reduced_product_consumers: Vec<&'static str>,
    pub selector_projection_certainties: Vec<&'static str>,
    pub provenance_tree_ready: bool,
    pub provenance_tree_scopes: Vec<&'static str>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_provenance: Option<AbstractClassValueProvenanceV0>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueProductDomainV0 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    pub must_chars: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_chars: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueProductV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_value_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<ReducedClassValuePrefixAxisV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<ReducedClassValueSuffixAxisV0>,
    pub char_inclusion: ReducedClassValueCharInclusionAxisV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    pub lower_bound_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueProductIterationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_count: usize,
    pub applied_constraint_count: usize,
    pub iteration_count: usize,
    pub converged: bool,
    pub monotone_witness_valid: bool,
    pub result_kind: &'static str,
    pub result_value: AbstractClassValueV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_product: Option<ReducedClassValueProductV0>,
    pub steps: Vec<ReducedClassValueProductIterationStepV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueProductIterationStepV0 {
    pub iteration: usize,
    pub operation: &'static str,
    pub input_value_kind: &'static str,
    pub result_kind: &'static str,
    pub changed: bool,
    pub monotone_with_previous: bool,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValuePrefixAxisV0 {
    pub prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueSuffixAxisV0 {
    pub suffix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReducedClassValueCharInclusionAxisV0 {
    pub must_chars: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_chars: Option<String>,
    pub may_include_other_chars: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AbstractPropertyValueV0 {
    Bottom {
        property_name: String,
    },
    Exact {
        property_name: String,
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pseudo_state: Option<String>,
    },
    FiniteSet {
        property_name: String,
        values: Vec<String>,
        pseudo_states: Vec<String>,
    },
    CustomPropertyReference {
        property_name: String,
        custom_property_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pseudo_state: Option<String>,
    },
    Top {
        property_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractPropertyValueCandidateV0 {
    pub property_name: String,
    pub value: String,
    pub pseudo_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractPropertyValueNarrowingV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub stylesheet_scope: &'static str,
    pub property_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_pseudo_state: Option<String>,
    pub candidate_count: usize,
    pub matched_candidate_count: usize,
    pub value: AbstractPropertyValueV0,
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

pub trait ProvenanceSemiringV0: Default + Clone + PartialEq + Eq + Serialize {
    const IDENTIFIER: &'static str;

    fn semiring_identifier(&self) -> &'static str {
        Self::IDENTIFIER
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Lin01ProvenanceSemiringV0 {
    pub zero: &'static str,
    pub one: &'static str,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl Lin01ProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: "0",
            one: "1",
            addition: "or",
            multiplication: "andThen",
            idempotent_addition: true,
        }
    }
}

impl Default for Lin01ProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for Lin01ProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "lin01";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalCountProvenanceSemiringV0 {
    pub zero: u8,
    pub one: u8,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl NaturalCountProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: 0,
            one: 1,
            addition: "plus",
            multiplication: "times",
            idempotent_addition: false,
        }
    }
}

impl Default for NaturalCountProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for NaturalCountProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "naturalCount";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TropicalProvenanceSemiringV0 {
    pub zero: &'static str,
    pub one: u8,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl TropicalProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: "infinity",
            one: 0,
            addition: "min",
            multiplication: "plus",
            idempotent_addition: true,
        }
    }
}

impl Default for TropicalProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for TropicalProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "tropical";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViterbiProvenanceSemiringV0 {
    pub zero: u8,
    pub one: u8,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl ViterbiProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: 0,
            one: 1,
            addition: "max",
            multiplication: "times",
            idempotent_addition: true,
        }
    }
}

impl Default for ViterbiProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for ViterbiProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "viterbi";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityLabelProvenanceSemiringV0 {
    pub zero: &'static str,
    pub one: &'static str,
    pub addition: &'static str,
    pub multiplication: &'static str,
    pub idempotent_addition: bool,
}

impl SecurityLabelProvenanceSemiringV0 {
    pub const fn new() -> Self {
        Self {
            zero: "public",
            one: "trusted",
            addition: "leastUpperBound",
            multiplication: "flowThen",
            idempotent_addition: true,
        }
    }
}

impl Default for SecurityLabelProvenanceSemiringV0 {
    fn default() -> Self {
        Self::new()
    }
}

impl ProvenanceSemiringV0 for SecurityLabelProvenanceSemiringV0 {
    const IDENTIFIER: &'static str = "securityLabel";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearProvenanceTermV0 {
    pub coefficient: u8,
    pub label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(serialize = "K: Serialize"))]
pub struct LinearProvenanceV0<K: ProvenanceSemiringV0> {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub semiring_identifier: &'static str,
    pub semiring: K,
    pub term_count: usize,
    pub terms: Vec<LinearProvenanceTermV0>,
}

impl<K: ProvenanceSemiringV0> LinearProvenanceV0<K> {
    pub fn semiring_identifier(&self) -> &'static str {
        self.semiring_identifier
    }

    pub fn labels(&self) -> Vec<&'static str> {
        self.terms.iter().map(|term| term.label).collect()
    }
}

impl LinearProvenanceV0<Lin01ProvenanceSemiringV0> {
    pub fn from_static_labels(labels: &[&'static str]) -> Self {
        let terms = labels
            .iter()
            .map(|label| LinearProvenanceTermV0 {
                coefficient: 1,
                label,
            })
            .collect::<Vec<_>>();
        Self {
            schema_version: "0",
            product: "omena-abstract-value.linear-provenance",
            semiring_identifier: Lin01ProvenanceSemiringV0::IDENTIFIER,
            semiring: Lin01ProvenanceSemiringV0::new(),
            term_count: terms.len(),
            terms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractClassValueProvenanceTreeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub value_kind: &'static str,
    pub value: AbstractClassValueV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_provenance: Option<AbstractClassValueProvenanceV0>,
    pub root: AbstractClassValueProvenanceNodeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractClassValueProvenanceNodeV0 {
    pub operation: &'static str,
    pub result_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_provenance: Option<AbstractClassValueProvenanceV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub reason: &'static str,
    pub children: Vec<AbstractClassValueProvenanceNodeV0>,
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
pub struct ClassValueControlFlowGraphV0 {
    pub context_key: Option<String>,
    pub entry_block_id: String,
    pub blocks: Vec<ClassValueControlFlowBlockV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassValueControlFlowBlockV0 {
    pub id: String,
    pub nodes: Vec<ClassValueFlowNodeV0>,
    pub successor_block_ids: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassValueControlFlowAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub context_sensitivity: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_key: Option<String>,
    pub block_count: usize,
    pub edge_count: usize,
    pub reachable_block_count: usize,
    pub unreachable_block_ids: Vec<String>,
    pub branch_block_ids: Vec<String>,
    pub join_block_ids: Vec<String>,
    pub flow_analysis: ClassValueFlowAnalysisV0,
    pub blocks: Vec<ClassValueControlFlowBlockResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassValueControlFlowBlockResultV0 {
    pub block_id: String,
    pub reachable: bool,
    pub node_ids: Vec<String>,
    pub successor_block_ids: Vec<String>,
    pub exit_value_kind: &'static str,
    pub exit_value: AbstractClassValueV0,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KLimitedCallSiteFlowInputV0 {
    pub callee_key: String,
    pub call_site_stack: Vec<String>,
    pub graph: ClassValueFlowGraphV0,
    pub exit_node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KLimitedCallSiteFlowAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub context_sensitivity: String,
    pub max_context_depth: usize,
    pub call_site_count: usize,
    pub callee_count: usize,
    pub entries: Vec<KLimitedCallSiteFlowEntryV0>,
    pub callee_summaries: Vec<OneCfaCalleeFlowSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KLimitedCallSiteFlowEntryV0 {
    pub callee_key: String,
    pub call_site_stack: Vec<String>,
    pub context_key: String,
    pub exit_node_id: String,
    pub exit_value_kind: &'static str,
    pub exit_value: AbstractClassValueV0,
    pub analysis: ClassValueFlowAnalysisV0,
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

fn is_false(value: &bool) -> bool {
    !value
}
