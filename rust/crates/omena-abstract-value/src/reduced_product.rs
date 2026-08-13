use crate::automaton::{concatenate_automaton_class_values, join_automaton_class_values};
use crate::domain::{abstract_class_value_kind, composite_min_length_for_constraints};
use crate::{
    AbstractClassValueProvenanceV0, AbstractClassValueV0, CompositeClassValueInputV0,
    ReducedClassValueCharInclusionAxisV0, ReducedClassValuePrefixAxisV0,
    ReducedClassValueProductDomainV0, ReducedClassValueProductIterationStepV0,
    ReducedClassValueProductIterationV0, ReducedClassValueProductV0, ReducedClassValueSuffixAxisV0,
    ReducedProductConstraintFactorV0, ReducedProductConstraintGraphV0,
    ReducedProductConstraintMessageV0, ReducedProductConstraintPropagationV0,
    ReducedProductConstraintVariableV0, bottom_class_value, char_set_for_string,
    char_set_is_subset, composite_class_value, intersect_char_sets,
    meaningful_longest_common_prefix, meaningful_longest_common_suffix, prefix_suffix_class_value,
    top_class_value_with_provenance, union_char_sets,
};

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_CONSTRAINT_PROPAGATION_PRODUCT_V0: &str =
    "omena-abstract-value.belief-propagation-iteration";

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_CONSTRAINT_GRAPH_PRODUCT_V0: &str =
    "omena-abstract-value.belief-propagation-domain-graph";

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_CONSTRAINT_EQUATION_SYSTEM_V0: &str = "Pr x Su x CI finite-height meet constraints";

#[cfg(test)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_CONSTRAINT_WIRE_PRODUCT_BYTES_V0: &[u8] = br#"["omena-abstract-value.belief-propagation-iteration","Pr x Su x CI finite-height meet constraints","omena-abstract-value.belief-propagation-domain-graph"]"#;

pub fn summarize_reduced_class_value_product(
    value: &AbstractClassValueV0,
) -> Option<ReducedClassValueProductV0> {
    reduce_class_value_product(value)
        .map(|facts| summarize_reduced_product_domain(&facts, abstract_class_value_kind(value)))
}

pub fn reduce_class_value_product(
    value: &AbstractClassValueV0,
) -> Option<ReducedClassValueProductDomainV0> {
    match value {
        AbstractClassValueV0::Bottom
        | AbstractClassValueV0::Exact { .. }
        | AbstractClassValueV0::FiniteSet { .. }
        | AbstractClassValueV0::Automaton { .. } => None,
        AbstractClassValueV0::Prefix { prefix, .. } => Some(ReducedClassValueProductDomainV0 {
            prefix: Some(prefix.clone()),
            suffix: None,
            min_length: None,
            must_chars: String::new(),
            allowed_chars: None,
        }),
        AbstractClassValueV0::Suffix { suffix, .. } => Some(ReducedClassValueProductDomainV0 {
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
        } => Some(ReducedClassValueProductDomainV0 {
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
        } => Some(ReducedClassValueProductDomainV0 {
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
        } => Some(ReducedClassValueProductDomainV0 {
            prefix: prefix.clone(),
            suffix: suffix.clone(),
            min_length: *min_length,
            must_chars: must_chars.clone(),
            allowed_chars: (!*may_include_other_chars).then_some(may_chars.clone()),
        }),
        AbstractClassValueV0::Top { .. } => Some(ReducedClassValueProductDomainV0 {
            prefix: None,
            suffix: None,
            min_length: None,
            must_chars: String::new(),
            allowed_chars: None,
        }),
    }
}

pub fn summarize_reduced_product_domain(
    product: &ReducedClassValueProductDomainV0,
    source_value_kind: &'static str,
) -> ReducedClassValueProductV0 {
    product.clone().into_product_summary(source_value_kind)
}

pub fn intersect_reduced_class_value_products(
    left: &ReducedClassValueProductDomainV0,
    right: &ReducedClassValueProductDomainV0,
) -> Option<ReducedClassValueProductDomainV0> {
    left.intersect(right)
}

pub fn join_reduced_class_value_products(
    left: &ReducedClassValueProductDomainV0,
    right: &ReducedClassValueProductDomainV0,
) -> Option<ReducedClassValueProductDomainV0> {
    left.join(right)
}

pub fn concatenate_reduced_class_value_products(
    left: &ReducedClassValueProductDomainV0,
    right: &ReducedClassValueProductDomainV0,
) -> Option<ReducedClassValueProductDomainV0> {
    left.concat(right)
}

pub fn reduced_class_value_product_is_subset(
    left: &ReducedClassValueProductDomainV0,
    right: &ReducedClassValueProductDomainV0,
) -> bool {
    left.is_subset_of(right)
}

pub fn reduced_class_value_product_matches_string(
    product: &ReducedClassValueProductDomainV0,
    candidate: &str,
) -> bool {
    product.matches_string(candidate)
}

pub fn iterate_reduced_class_value_product_constraints(
    values: &[AbstractClassValueV0],
) -> ReducedClassValueProductIterationV0 {
    let mut current = ReducedClassValueProductDomainV0 {
        prefix: None,
        suffix: None,
        min_length: None,
        must_chars: String::new(),
        allowed_chars: None,
    };
    let mut bottom = false;
    let mut applied_constraint_count = 0usize;
    let mut steps = Vec::new();

    for value in values {
        let input_value_kind = abstract_class_value_kind(value);
        let Some(input_product) = reduce_class_value_product(value) else {
            steps.push(ReducedClassValueProductIterationStepV0 {
                iteration: steps.len() + 1,
                operation: "skipNonReducedProductInput",
                input_value_kind,
                result_kind: abstract_class_value_kind(
                    &current
                        .clone()
                        .into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin),
                ),
                changed: false,
                monotone_with_previous: true,
                reason: "exact, finite, and bottom inputs are handled by the outer value lattice",
            });
            continue;
        };

        applied_constraint_count += 1;
        let previous = current.clone();
        let Some(next) = current.intersect(&input_product) else {
            bottom = true;
            steps.push(ReducedClassValueProductIterationStepV0 {
                iteration: steps.len() + 1,
                operation: "meetReducedProductConstraint",
                input_value_kind,
                result_kind: "bottom",
                changed: true,
                monotone_with_previous: true,
                reason: "incompatible reduced-product axes collapse to bottom",
            });
            break;
        };

        let monotone_with_previous = next.is_subset_of(&previous);
        let changed = next != previous;
        current = next;
        let result_value = current
            .clone()
            .into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin);
        steps.push(ReducedClassValueProductIterationStepV0 {
            iteration: steps.len() + 1,
            operation: "meetReducedProductConstraint",
            input_value_kind,
            result_kind: abstract_class_value_kind(&result_value),
            changed,
            monotone_with_previous,
            reason: "intersection refines Pr x Su x CI axes without widening",
        });
    }

    let result_value = if bottom {
        bottom_class_value()
    } else {
        current
            .clone()
            .into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin)
    };
    let result_kind = abstract_class_value_kind(&result_value);
    let final_product = (!bottom).then(|| current.clone().into_product_summary(result_kind));
    let converged = bottom || reduced_product_constraints_reached_fixed_point(&current, values);
    let monotone_witness_valid = steps.iter().all(|step| step.monotone_with_previous);

    ReducedClassValueProductIterationV0 {
        schema_version: "0",
        product: "omena-abstract-value.reduced-product-iteration",
        input_count: values.len(),
        applied_constraint_count,
        iteration_count: steps.len(),
        converged,
        monotone_witness_valid,
        result_kind,
        result_value,
        final_product,
        steps,
    }
}

pub fn summarize_reduced_product_constraint_propagation_v0(
    values: &[AbstractClassValueV0],
) -> ReducedProductConstraintPropagationV0 {
    let source_iteration = iterate_reduced_class_value_product_constraints(values);
    let messages = source_iteration
        .steps
        .iter()
        .map(|step| ReducedProductConstraintMessageV0 {
            iteration: step.iteration,
            from_factor: step.input_value_kind,
            to_variable: step.result_kind,
            operation: step.operation,
            result_kind: step.result_kind,
            monotone_with_previous: step.monotone_with_previous,
        })
        .collect::<Vec<_>>();

    ReducedProductConstraintPropagationV0 {
        schema_version: "0",
        product: "omena-abstract-value.reduced-product-constraint-propagation",
        algorithm_view: "reducedProductConstraintMessagePassing",
        substrate: "omena-abstract-value.reduced-product-iteration",
        equation_system: "Pr x Su x CI; cardinality/byte bounds before automaton construction; reverse-postorder flow plus loop-header widening",
        input_count: source_iteration.input_count,
        message_count: messages.len(),
        iteration_count: source_iteration.iteration_count,
        converged: source_iteration.converged,
        monotone_witness_valid: source_iteration.monotone_witness_valid,
        fixed_point_reached: source_iteration.converged,
        messages,
        source_iteration,
    }
}

pub fn summarize_reduced_product_constraint_graph_v0(
    values: &[AbstractClassValueV0],
) -> ReducedProductConstraintGraphV0 {
    let iteration = summarize_reduced_product_constraint_propagation_v0(values);
    let variables = reduced_product_constraint_variables_v0();
    let factors = iteration
        .messages
        .iter()
        .map(|message| ReducedProductConstraintFactorV0 {
            factor_id: format!("constraint:{}:{}", message.iteration, message.from_factor),
            input_value_kind: message.from_factor,
            operation: message.operation,
            result_kind: message.result_kind,
        })
        .collect::<Vec<_>>();
    let edge_count = factors.len().saturating_mul(2);

    ReducedProductConstraintGraphV0 {
        schema_version: "0",
        product: "omena-abstract-value.reduced-product-constraint-graph",
        claim_level: "fixtureWitnessReducedProductDomainGraph",
        theorem_claimed: false,
        algorithm_view: "reducedProductDomainGraphMessagePassing",
        substrate: iteration.product,
        variable_count: variables.len(),
        factor_count: factors.len(),
        edge_count,
        converged: iteration.converged,
        monotone_witness_valid: iteration.monotone_witness_valid,
        variables,
        factors,
        messages: iteration.messages.clone(),
        source_iteration: iteration.source_iteration,
    }
}

fn reduced_product_constraint_variables_v0() -> Vec<ReducedProductConstraintVariableV0> {
    vec![
        ReducedProductConstraintVariableV0 {
            variable_id: "Pr",
            axis: "prefix",
        },
        ReducedProductConstraintVariableV0 {
            variable_id: "Su",
            axis: "suffix",
        },
        ReducedProductConstraintVariableV0 {
            variable_id: "CI",
            axis: "charInclusion",
        },
        ReducedProductConstraintVariableV0 {
            variable_id: "Len",
            axis: "minLength",
        },
    ]
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
#[allow(deprecated)]
pub fn summarize_belief_propagation_iteration_v0(
    values: &[AbstractClassValueV0],
) -> crate::BeliefPropagationIterationV0 {
    #[allow(deprecated)]
    legacy_constraint_propagation_wire_compatibility_v0(values)
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
#[allow(deprecated)]
fn legacy_constraint_propagation_wire_compatibility_v0(
    values: &[AbstractClassValueV0],
) -> crate::BeliefPropagationIterationV0 {
    let summary = summarize_reduced_product_constraint_propagation_v0(values);
    #[allow(deprecated)]
    let messages = summary
        .messages
        .into_iter()
        .map(|message| crate::BeliefPropagationMessageV0 {
            iteration: message.iteration,
            from_factor: message.from_factor,
            to_variable: message.to_variable,
            operation: message.operation,
            result_kind: message.result_kind,
            monotone_with_previous: message.monotone_with_previous,
        })
        .collect();

    #[allow(deprecated)]
    crate::BeliefPropagationIterationV0 {
        schema_version: summary.schema_version,
        product: LEGACY_CONSTRAINT_PROPAGATION_PRODUCT_V0,
        algorithm_view: summary.algorithm_view,
        substrate: summary.substrate,
        equation_system: LEGACY_CONSTRAINT_EQUATION_SYSTEM_V0,
        input_count: summary.input_count,
        message_count: summary.message_count,
        iteration_count: summary.iteration_count,
        converged: summary.converged,
        monotone_witness_valid: summary.monotone_witness_valid,
        fixed_point_reached: summary.fixed_point_reached,
        messages,
        source_iteration: summary.source_iteration,
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
#[allow(deprecated)]
pub fn summarize_reduced_product_belief_propagation_domain_graph_v0(
    values: &[AbstractClassValueV0],
) -> crate::BeliefPropagationDomainGraphV0 {
    #[allow(deprecated)]
    legacy_constraint_graph_wire_compatibility_v0(values)
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-abstract-value maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
#[allow(deprecated)]
fn legacy_constraint_graph_wire_compatibility_v0(
    values: &[AbstractClassValueV0],
) -> crate::BeliefPropagationDomainGraphV0 {
    let summary = summarize_reduced_product_constraint_graph_v0(values);
    #[allow(deprecated)]
    let variables = summary
        .variables
        .into_iter()
        .map(|variable| crate::BeliefPropagationDomainVariableV0 {
            variable_id: variable.variable_id,
            axis: variable.axis,
        })
        .collect();
    #[allow(deprecated)]
    let factors = summary
        .factors
        .into_iter()
        .map(|factor| crate::BeliefPropagationDomainFactorV0 {
            factor_id: factor.factor_id,
            input_value_kind: factor.input_value_kind,
            operation: factor.operation,
            result_kind: factor.result_kind,
        })
        .collect();
    #[allow(deprecated)]
    let messages = summary
        .messages
        .into_iter()
        .map(|message| crate::BeliefPropagationMessageV0 {
            iteration: message.iteration,
            from_factor: message.from_factor,
            to_variable: message.to_variable,
            operation: message.operation,
            result_kind: message.result_kind,
            monotone_with_previous: message.monotone_with_previous,
        })
        .collect();

    #[allow(deprecated)]
    crate::BeliefPropagationDomainGraphV0 {
        schema_version: summary.schema_version,
        product: LEGACY_CONSTRAINT_GRAPH_PRODUCT_V0,
        claim_level: summary.claim_level,
        theorem_claimed: summary.theorem_claimed,
        algorithm_view: summary.algorithm_view,
        substrate: LEGACY_CONSTRAINT_PROPAGATION_PRODUCT_V0,
        variable_count: summary.variable_count,
        factor_count: summary.factor_count,
        edge_count: summary.edge_count,
        converged: summary.converged,
        monotone_witness_valid: summary.monotone_witness_valid,
        variables,
        factors,
        messages,
        source_iteration: summary.source_iteration,
    }
}

#[cfg(test)]
mod compatibility_tests {
    use super::*;
    use crate::{char_inclusion_class_value, prefix_class_value, suffix_class_value};

    #[test]
    #[allow(deprecated)]
    fn legacy_constraint_adapters_preserve_exact_pre_rename_wire_bytes() {
        let values = [
            prefix_class_value("btn-", None),
            suffix_class_value("-active", None),
            char_inclusion_class_value("a", "abcde-5intv", None, false),
        ];

        let canonical_propagation = summarize_reduced_product_constraint_propagation_v0(&values);
        let legacy_propagation = legacy_constraint_propagation_wire_compatibility_v0(&values);
        let canonical_propagation_bytes = serde_json::to_string(&canonical_propagation);
        assert!(canonical_propagation_bytes.is_ok());
        let canonical_propagation_bytes = canonical_propagation_bytes.unwrap_or_default();
        let expected_legacy_propagation_bytes = canonical_propagation_bytes
            .replace(
                canonical_propagation.product,
                LEGACY_CONSTRAINT_PROPAGATION_PRODUCT_V0,
            )
            .replace(
                canonical_propagation.equation_system,
                LEGACY_CONSTRAINT_EQUATION_SYSTEM_V0,
            )
            .into_bytes();
        let actual_legacy_propagation_bytes = serde_json::to_vec(&legacy_propagation);
        assert!(actual_legacy_propagation_bytes.is_ok());
        assert_eq!(
            actual_legacy_propagation_bytes.unwrap_or_default(),
            expected_legacy_propagation_bytes
        );

        let canonical_graph = summarize_reduced_product_constraint_graph_v0(&values);
        let legacy_graph = legacy_constraint_graph_wire_compatibility_v0(&values);
        let canonical_graph_bytes = serde_json::to_string(&canonical_graph);
        assert!(canonical_graph_bytes.is_ok());
        let canonical_graph_bytes = canonical_graph_bytes.unwrap_or_default();
        let expected_legacy_graph_bytes = canonical_graph_bytes
            .replace(canonical_graph.product, LEGACY_CONSTRAINT_GRAPH_PRODUCT_V0)
            .replace(
                canonical_graph.substrate,
                LEGACY_CONSTRAINT_PROPAGATION_PRODUCT_V0,
            )
            .into_bytes();
        let actual_legacy_graph_bytes = serde_json::to_vec(&legacy_graph);
        assert!(actual_legacy_graph_bytes.is_ok());
        assert_eq!(
            actual_legacy_graph_bytes.unwrap_or_default(),
            expected_legacy_graph_bytes
        );
    }
}

pub(crate) fn intersect_reduced_product_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    let left = reduce_class_value_product(left)?;
    let right = reduce_class_value_product(right)?;
    intersect_reduced_class_value_products(&left, &right)
        .map(|facts| facts.into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin))
}

pub(crate) fn join_reduced_product_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    if let Some(joined) = join_automaton_class_values(left, right) {
        return Some(joined);
    }
    let left = reduce_class_value_product(left)?;
    let right = reduce_class_value_product(right)?;
    join_reduced_class_value_products(&left, &right)
        .map(|facts| facts.into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin))
}

pub(crate) fn concatenate_reduced_product_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
    if let Some(concatenated) = concatenate_automaton_class_values(left, right) {
        return Some(concatenated);
    }
    let left = reduce_class_value_product(left)?;
    let right = reduce_class_value_product(right)?;
    concatenate_reduced_class_value_products(&left, &right)
        .map(|facts| facts.into_abstract_value(AbstractClassValueProvenanceV0::CompositeConcat))
}

pub(crate) fn reduced_product_class_value_is_subset(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<bool> {
    let left = reduce_class_value_product(left)?;
    let right = reduce_class_value_product(right)?;
    Some(reduced_class_value_product_is_subset(&left, &right))
}

fn reduced_product_constraints_reached_fixed_point(
    current: &ReducedClassValueProductDomainV0,
    values: &[AbstractClassValueV0],
) -> bool {
    values
        .iter()
        .filter_map(reduce_class_value_product)
        .all(|constraint| current.intersect(&constraint).as_ref() == Some(current))
}

impl ReducedClassValueProductDomainV0 {
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

    fn is_subset_of(&self, other: &Self) -> bool {
        if let Some(other_prefix) = other.prefix.as_deref()
            && !self
                .prefix
                .as_deref()
                .is_some_and(|prefix| prefix.starts_with(other_prefix))
        {
            return false;
        }

        if let Some(other_suffix) = other.suffix.as_deref()
            && !self
                .suffix
                .as_deref()
                .is_some_and(|suffix| suffix.ends_with(other_suffix))
        {
            return false;
        }

        if let Some(other_min_length) = other.min_length
            && self.lower_bound_length() < other_min_length
        {
            return false;
        }

        if !char_set_is_subset(&other.must_chars, &self.guaranteed_chars()) {
            return false;
        }

        if let Some(other_allowed_chars) = other.allowed_chars.as_deref() {
            let Some(self_allowed_chars) = self.allowed_chars.as_deref() else {
                return false;
            };
            if !char_set_is_subset(self_allowed_chars, other_allowed_chars) {
                return false;
            }
        }

        true
    }

    fn matches_string(&self, candidate: &str) -> bool {
        if let Some(min_length) = self.min_length
            && candidate.len() < min_length
        {
            return false;
        }

        if let Some(prefix) = self.prefix.as_deref()
            && !candidate.starts_with(prefix)
        {
            return false;
        }

        if let Some(suffix) = self.suffix.as_deref()
            && !candidate.ends_with(suffix)
        {
            return false;
        }

        let candidate_chars = char_set_for_string(candidate);
        if !char_set_is_subset(&self.guaranteed_chars(), &candidate_chars) {
            return false;
        }

        if let Some(allowed_chars) = self.allowed_chars.as_deref()
            && !char_set_is_subset(&candidate_chars, allowed_chars)
        {
            return false;
        }

        true
    }

    fn lower_bound_length(&self) -> usize {
        self.min_length.unwrap_or_else(|| {
            composite_min_length_for_constraints(
                self.prefix.as_deref().unwrap_or(""),
                self.suffix.as_deref().unwrap_or(""),
                &self.must_chars,
            )
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

    fn into_abstract_value(
        self,
        provenance: AbstractClassValueProvenanceV0,
    ) -> AbstractClassValueV0 {
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
                Some(provenance),
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
            return top_class_value_with_provenance(
                AbstractClassValueProvenanceV0::ReducedProductUnconstrained,
            );
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
            provenance: Some(provenance),
        })
    }

    fn into_product_summary(self, source_value_kind: &'static str) -> ReducedClassValueProductV0 {
        let lower_bound_length = self.lower_bound_length();
        let may_include_other_chars = self.allowed_chars.is_none();
        ReducedClassValueProductV0 {
            schema_version: "0",
            product: "omena-abstract-value.reduced-product",
            source_value_kind,
            prefix: self
                .prefix
                .map(|prefix| ReducedClassValuePrefixAxisV0 { prefix }),
            suffix: self
                .suffix
                .map(|suffix| ReducedClassValueSuffixAxisV0 { suffix }),
            char_inclusion: ReducedClassValueCharInclusionAxisV0 {
                must_chars: self.must_chars,
                allowed_chars: self.allowed_chars,
                may_include_other_chars,
            },
            min_length: self.min_length,
            lower_bound_length,
        }
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
mod legacy_wire_tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn compatibility_adapters_preserve_legacy_product_bytes() {
        let propagation = legacy_constraint_propagation_wire_compatibility_v0(&[]);
        let graph = legacy_constraint_graph_wire_compatibility_v0(&[]);
        let actual = serde_json::to_vec(&(
            propagation.product,
            propagation.equation_system,
            graph.product,
        ));

        assert!(
            actual.is_ok(),
            "legacy compatibility products should serialize"
        );
        assert_eq!(
            actual.unwrap_or_default(),
            LEGACY_CONSTRAINT_WIRE_PRODUCT_BYTES_V0
        );
    }
}
