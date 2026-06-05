use crate::domain::{abstract_class_value_kind, composite_min_length_for_constraints};
use crate::{
    AbstractClassValueProvenanceV0, AbstractClassValueV0, BeliefPropagationDomainFactorV0,
    BeliefPropagationDomainGraphV0, BeliefPropagationDomainVariableV0,
    BeliefPropagationIterationV0, BeliefPropagationMessageV0, CompositeClassValueInputV0,
    ReducedClassValueCharInclusionAxisV0, ReducedClassValuePrefixAxisV0,
    ReducedClassValueProductDomainV0, ReducedClassValueProductIterationStepV0,
    ReducedClassValueProductIterationV0, ReducedClassValueProductV0, ReducedClassValueSuffixAxisV0,
    bottom_class_value, char_set_for_string, char_set_is_subset, composite_class_value,
    intersect_char_sets, meaningful_longest_common_prefix, meaningful_longest_common_suffix,
    prefix_suffix_class_value, top_class_value, union_char_sets,
};

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
        | AbstractClassValueV0::FiniteSet { .. } => None,
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
        AbstractClassValueV0::Top => Some(ReducedClassValueProductDomainV0 {
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

pub fn summarize_belief_propagation_iteration_v0(
    values: &[AbstractClassValueV0],
) -> BeliefPropagationIterationV0 {
    let source_iteration = iterate_reduced_class_value_product_constraints(values);
    let messages = source_iteration
        .steps
        .iter()
        .map(|step| BeliefPropagationMessageV0 {
            iteration: step.iteration,
            from_factor: step.input_value_kind,
            to_variable: step.result_kind,
            operation: step.operation,
            result_kind: step.result_kind,
            monotone_with_previous: step.monotone_with_previous,
        })
        .collect::<Vec<_>>();

    BeliefPropagationIterationV0 {
        schema_version: "0",
        product: "omena-abstract-value.belief-propagation-iteration",
        algorithm_view: "reducedProductConstraintMessagePassing",
        substrate: "omena-abstract-value.reduced-product-iteration",
        equation_system: "Pr x Su x CI finite-height meet constraints",
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

pub fn summarize_reduced_product_belief_propagation_domain_graph_v0(
    values: &[AbstractClassValueV0],
) -> BeliefPropagationDomainGraphV0 {
    let iteration = summarize_belief_propagation_iteration_v0(values);
    let variables = reduced_product_bp_domain_variables_v0();
    let factors = iteration
        .messages
        .iter()
        .map(|message| BeliefPropagationDomainFactorV0 {
            factor_id: format!("constraint:{}:{}", message.iteration, message.from_factor),
            input_value_kind: message.from_factor,
            operation: message.operation,
            result_kind: message.result_kind,
        })
        .collect::<Vec<_>>();
    let edge_count = factors.len().saturating_mul(2);

    BeliefPropagationDomainGraphV0 {
        schema_version: "0",
        product: "omena-abstract-value.belief-propagation-domain-graph",
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

fn reduced_product_bp_domain_variables_v0() -> Vec<BeliefPropagationDomainVariableV0> {
    vec![
        BeliefPropagationDomainVariableV0 {
            variable_id: "Pr",
            axis: "prefix",
        },
        BeliefPropagationDomainVariableV0 {
            variable_id: "Su",
            axis: "suffix",
        },
        BeliefPropagationDomainVariableV0 {
            variable_id: "CI",
            axis: "charInclusion",
        },
        BeliefPropagationDomainVariableV0 {
            variable_id: "Len",
            axis: "minLength",
        },
    ]
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
    let left = reduce_class_value_product(left)?;
    let right = reduce_class_value_product(right)?;
    join_reduced_class_value_products(&left, &right)
        .map(|facts| facts.into_abstract_value(AbstractClassValueProvenanceV0::CompositeJoin))
}

pub(crate) fn concatenate_reduced_product_class_values(
    left: &AbstractClassValueV0,
    right: &AbstractClassValueV0,
) -> Option<AbstractClassValueV0> {
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
