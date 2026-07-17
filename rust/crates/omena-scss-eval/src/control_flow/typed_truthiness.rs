use omena_abstract_value::{
    AbstractCssTypedComparisonOperatorV0, AbstractCssTypedScalarValueV0, AbstractCssTypedValueV0,
    AbstractCssValueV0, compare_abstract_css_values_with_typed_payloads,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScssTruthinessConsumer {
    StringLattice,
    TypedPayload,
}

pub(super) const TYPED_PRUNE_CONSUMER_ENABLED: bool = true;

pub(super) fn production_truthiness_consumer() -> ScssTruthinessConsumer {
    if TYPED_PRUNE_CONSUMER_ENABLED {
        ScssTruthinessConsumer::TypedPayload
    } else {
        ScssTruthinessConsumer::StringLattice
    }
}

pub(super) fn typed_truthiness_label(value: &AbstractCssValueV0) -> Option<&'static str> {
    typed_truthiness(value).map(truthiness_label)
}

pub(super) fn typed_comparison_truthiness(
    left: &AbstractCssValueV0,
    operator: AbstractCssTypedComparisonOperatorV0,
    right: &AbstractCssValueV0,
) -> Option<bool> {
    compare_abstract_css_values_with_typed_payloads(left, operator, right)
}

fn typed_truthiness(value: &AbstractCssValueV0) -> Option<bool> {
    match value {
        AbstractCssValueV0::Exact {
            typed: Some(typed), ..
        }
        | AbstractCssValueV0::FiniteSet {
            typed: Some(typed), ..
        } => typed_value_truthiness(typed),
        AbstractCssValueV0::Bottom
        | AbstractCssValueV0::Exact { typed: None, .. }
        | AbstractCssValueV0::FiniteSet { typed: None, .. }
        | AbstractCssValueV0::Raw { .. }
        | AbstractCssValueV0::Top => None,
    }
}

fn typed_value_truthiness(value: &AbstractCssTypedValueV0) -> Option<bool> {
    match value {
        AbstractCssTypedValueV0::Exact { value } => typed_scalar_truthiness(value),
        AbstractCssTypedValueV0::Compound { .. } => None,
        AbstractCssTypedValueV0::FiniteSet { values } => uniform_typed_set_truthiness(values),
        AbstractCssTypedValueV0::Top => None,
    }
}

fn uniform_typed_set_truthiness(values: &[AbstractCssTypedScalarValueV0]) -> Option<bool> {
    let mut decision = None;
    for value in values {
        let next = typed_scalar_truthiness(value)?;
        if decision.is_some_and(|current| current != next) {
            return None;
        }
        decision = Some(next);
    }
    decision
}

fn typed_scalar_truthiness(value: &AbstractCssTypedScalarValueV0) -> Option<bool> {
    match value {
        AbstractCssTypedScalarValueV0::Keyword { value } => Some(keyword_truthiness(value)),
        AbstractCssTypedScalarValueV0::CssWide { .. }
        | AbstractCssTypedScalarValueV0::Dimension { .. }
        | AbstractCssTypedScalarValueV0::Integer { .. }
        | AbstractCssTypedScalarValueV0::Number { .. } => Some(true),
        AbstractCssTypedScalarValueV0::Color { serialized } if !serialized.contains('(') => {
            Some(true)
        }
        AbstractCssTypedScalarValueV0::QuotedString { value } if !value.contains('(') => Some(true),
        AbstractCssTypedScalarValueV0::Color { .. }
        | AbstractCssTypedScalarValueV0::QuotedString { .. }
        | AbstractCssTypedScalarValueV0::Url { .. }
        | AbstractCssTypedScalarValueV0::Image { .. }
        | AbstractCssTypedScalarValueV0::Transform { .. } => None,
    }
}

fn keyword_truthiness(value: &str) -> bool {
    !matches!(value.trim().to_ascii_lowercase().as_str(), "false" | "null")
}

fn truthiness_label(value: bool) -> &'static str {
    if value { "truthy" } else { "falsey" }
}
