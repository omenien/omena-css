use crate::{Lin01ProvenanceSemiringV0, NaturalCountProvenanceSemiringV0, ProvenanceSemiringV0};
use omena_incremental::{IncrementalComputationPlanV0, IncrementalSnapshotV0};
use serde::{Deserialize, Serialize};

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
    pub cascade_family_substrate_ready: bool,
    pub cascade_family_framing: &'static str,
    pub cascade_family_claim_level: &'static str,
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
pub struct BeliefPropagationMessageV0 {
    pub iteration: usize,
    pub from_factor: &'static str,
    pub to_variable: &'static str,
    pub operation: &'static str,
    pub result_kind: &'static str,
    pub monotone_with_previous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// V0 algorithm-view substrate over the reduced-product class-value iterator.
///
/// This records compatibility evidence for the current message-passing view; it
/// is not a belief-propagation paper result or mechanism-completeness claim.
pub struct BeliefPropagationIterationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub algorithm_view: &'static str,
    pub substrate: &'static str,
    pub equation_system: &'static str,
    pub input_count: usize,
    pub message_count: usize,
    pub iteration_count: usize,
    pub converged: bool,
    pub monotone_witness_valid: bool,
    pub fixed_point_reached: bool,
    pub messages: Vec<BeliefPropagationMessageV0>,
    pub source_iteration: ReducedClassValueProductIterationV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeliefPropagationDomainGraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub algorithm_view: &'static str,
    pub substrate: &'static str,
    pub variable_count: usize,
    pub factor_count: usize,
    pub edge_count: usize,
    pub converged: bool,
    pub monotone_witness_valid: bool,
    pub variables: Vec<BeliefPropagationDomainVariableV0>,
    pub factors: Vec<BeliefPropagationDomainFactorV0>,
    pub messages: Vec<BeliefPropagationMessageV0>,
    pub source_iteration: ReducedClassValueProductIterationV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeliefPropagationDomainVariableV0 {
    pub variable_id: &'static str,
    pub axis: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeliefPropagationDomainFactorV0 {
    pub factor_id: String,
    pub input_value_kind: &'static str,
    pub operation: &'static str,
    pub result_kind: &'static str,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AbstractCssValueV0 {
    Bottom,
    Exact { value: String },
    FiniteSet { values: Vec<String> },
    Raw { value: String },
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
    pub condition_context: Vec<String>,
    pub layer_name: Option<String>,
    pub layer_order: Option<i32>,
    pub source_order: Option<u32>,
    pub important: bool,
    pub same_selector_ordering: bool,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requested_condition_context: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_layer_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_layer_order: Option<i32>,
    pub requested_layer_scope: &'static str,
    pub candidate_count: usize,
    pub matched_candidate_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_value: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub display_values: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinearProvenanceTermV0 {
    pub coefficient: u8,
    pub label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearProvenancePathV0 {
    pub labels: Vec<&'static str>,
    pub support_count: u8,
}

impl LinearProvenancePathV0 {
    pub fn supported(labels: &[&'static str], support_count: u8) -> Self {
        Self {
            labels: labels.to_vec(),
            support_count,
        }
    }

    pub fn unsupported(labels: &[&'static str]) -> Self {
        Self {
            labels: labels.to_vec(),
            support_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(serialize = "K: Serialize"))]
/// V0 freeze-candidate provenance contract over the existing label vector.
///
/// The shape is a strict-superset bridge for staged query provenance evidence;
/// it does not declare Cargo 1.0 API finality or a completed QTT/sheaf model.
pub struct LinearProvenanceV0<K: ProvenanceSemiringV0> {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub semiring_identifier: &'static str,
    pub semiring: K,
    pub term_count: usize,
    pub terms: Vec<LinearProvenanceTermV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolynomialProvenanceVariableV0 {
    pub variable: String,
    pub label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolynomialProvenanceTermV0 {
    pub coefficient: u16,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolynomialProvenanceProjectionV0 {
    pub projection_kind: &'static str,
    pub semiring_identifier: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolynomialProvenanceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub polynomial_kind: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub selected_ladder: String,
    pub available_ladder_tiers: Vec<&'static str>,
    pub root_operator: &'static str,
    pub variables: Vec<PolynomialProvenanceVariableV0>,
    pub terms: Vec<PolynomialProvenanceTermV0>,
    pub projections: Vec<PolynomialProvenanceProjectionV0>,
}

impl<K: ProvenanceSemiringV0> LinearProvenanceV0<K> {
    pub fn semiring_identifier(&self) -> &'static str {
        self.semiring_identifier
    }

    pub fn labels(&self) -> Vec<&'static str> {
        self.terms.iter().map(|term| term.label).collect()
    }
}

pub fn summarize_polynomial_provenance_from_linear_v0(
    linear_provenance: &LinearProvenanceV0<NaturalCountProvenanceSemiringV0>,
    selected_ladder: &str,
) -> PolynomialProvenanceV0 {
    let variables = linear_provenance
        .terms
        .iter()
        .enumerate()
        .map(|(index, term)| PolynomialProvenanceVariableV0 {
            variable: format!("x{index}"),
            label: term.label,
        })
        .collect::<Vec<_>>();
    let terms = linear_provenance
        .terms
        .iter()
        .enumerate()
        .map(|(index, term)| PolynomialProvenanceTermV0 {
            coefficient: u16::from(term.coefficient),
            variables: vec![format!("x{index}")],
        })
        .collect::<Vec<_>>();
    let total_support = terms.iter().map(|term| term.coefficient).sum::<u16>();
    let tropical_cost = terms.iter().map(|term| term.coefficient).min().unwrap_or(0);
    let why_not_value = linear_provenance
        .terms
        .iter()
        .filter(|term| term.coefficient == 0)
        .map(|term| term.label)
        .collect::<Vec<_>>();
    let why_not_value = if why_not_value.is_empty() {
        "noUnsupportedTermsInFixture".to_string()
    } else {
        why_not_value.join(" -> ")
    };

    PolynomialProvenanceV0 {
        schema_version: "0",
        product: "omena-abstract-value.polynomial-provenance",
        layer_marker: "qtt-graded",
        feature_gate: "qtt-provenance-polynomial-v0",
        polynomial_kind: "naturalCountPolynomialOverLabels",
        claim_level: "fixtureWitnessPolynomialProjection",
        theorem_claimed: false,
        selected_ladder: selected_ladder.to_string(),
        available_ladder_tiers: vec![
            "linearLabels",
            "naturalCountPolynomial",
            "homomorphicProjections",
        ],
        root_operator: "sum",
        variables,
        terms,
        projections: vec![
            PolynomialProvenanceProjectionV0 {
                projection_kind: "why",
                semiring_identifier: "lin01",
                value: linear_provenance.labels().join(" -> "),
            },
            PolynomialProvenanceProjectionV0 {
                projection_kind: "whyNot",
                semiring_identifier: "lin01",
                value: why_not_value,
            },
            PolynomialProvenanceProjectionV0 {
                projection_kind: "confidence",
                semiring_identifier: "naturalCount",
                value: format!("{total_support}/{}", linear_provenance.term_count.max(1)),
            },
            PolynomialProvenanceProjectionV0 {
                projection_kind: "tropical",
                semiring_identifier: "tropical",
                value: tropical_cost.to_string(),
            },
        ],
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
        linear_provenance_with_terms(Lin01ProvenanceSemiringV0::new(), terms)
    }
}

impl LinearProvenanceV0<NaturalCountProvenanceSemiringV0> {
    pub fn from_static_labels(labels: &[&'static str]) -> Self {
        let path = LinearProvenancePathV0::supported(labels, 1);
        Self::from_composed_paths(&[path])
    }

    pub fn from_composed_paths(paths: &[LinearProvenancePathV0]) -> Self {
        let semiring = NaturalCountProvenanceSemiringV0::new();
        let mut terms = Vec::<LinearProvenanceTermV0>::new();

        for path in paths {
            let support = u16::from(path.support_count);
            let mut path_weight = semiring.multiply(&semiring.one(), &support);
            for _ in &path.labels {
                path_weight = semiring.multiply(&path_weight, &semiring.one());
            }

            for label in &path.labels {
                if let Some(term) = terms.iter_mut().find(|term| term.label == *label) {
                    let updated = semiring.add(&u16::from(term.coefficient), &path_weight);
                    term.coefficient = u16_to_u8_saturating(updated);
                } else {
                    terms.push(LinearProvenanceTermV0 {
                        coefficient: u16_to_u8_saturating(
                            semiring.add(&semiring.zero(), &path_weight),
                        ),
                        label,
                    });
                }
            }
        }

        linear_provenance_with_terms(semiring, terms)
    }
}

fn linear_provenance_with_terms<K: ProvenanceSemiringV0>(
    semiring: K,
    terms: Vec<LinearProvenanceTermV0>,
) -> LinearProvenanceV0<K> {
    LinearProvenanceV0 {
        schema_version: "0",
        product: "omena-abstract-value.linear-provenance",
        layer_marker: "qtt-graded",
        feature_gate: "qtt-provenance",
        semiring_identifier: K::IDENTIFIER,
        semiring,
        term_count: terms.len(),
        terms,
    }
}

fn u16_to_u8_saturating(value: u16) -> u8 {
    value.min(u16::from(u8::MAX)) as u8
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
