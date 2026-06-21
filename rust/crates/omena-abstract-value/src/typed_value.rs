use omena_value_lattice::{format_css_number, parse_numeric_value_with_unit};

use crate::{
    AbstractCssTypedComparisonOperatorV0, AbstractCssTypedScalarValueV0, AbstractCssTypedValueV0,
    AbstractCssValueV0, DeclaredNumericTypeV0, DeclaredValueKindV0,
    classify_registered_property_declared_value_v0,
};

pub fn abstract_css_typed_value_from_text(value: &str) -> Option<AbstractCssTypedValueV0> {
    abstract_css_typed_scalar_from_text(value).map(|value| AbstractCssTypedValueV0::Exact { value })
}

pub fn abstract_css_typed_scalar_from_text(value: &str) -> Option<AbstractCssTypedScalarValueV0> {
    let trimmed = value.trim();
    let value_kind = classify_registered_property_declared_value_v0(trimmed);
    match value_kind {
        DeclaredValueKindV0::Dimension(numeric_type) => {
            let numeric = parse_numeric_value_with_unit(trimmed)?;
            Some(AbstractCssTypedScalarValueV0::Dimension {
                numeric_type,
                number: format_css_number(numeric.value),
                unit: numeric.unit.to_ascii_lowercase(),
                serialized: trimmed.to_string(),
            })
        }
        DeclaredValueKindV0::Number => {
            let numeric = parse_numeric_value_with_unit(trimmed)?;
            if !numeric.unit.is_empty() {
                return None;
            }
            Some(AbstractCssTypedScalarValueV0::Number {
                number: format_css_number(numeric.value),
                serialized: trimmed.to_string(),
            })
        }
        DeclaredValueKindV0::Integer => {
            let numeric = parse_numeric_value_with_unit(trimmed)?;
            if !numeric.unit.is_empty() || numeric.value.fract() != 0.0 {
                return None;
            }
            Some(AbstractCssTypedScalarValueV0::Integer {
                number: format_css_number(numeric.value),
                serialized: trimmed.to_string(),
            })
        }
        DeclaredValueKindV0::HexColor
        | DeclaredValueKindV0::ColorFunction
        | DeclaredValueKindV0::ColorKeyword(_) => Some(AbstractCssTypedScalarValueV0::Color {
            serialized: trimmed.to_string(),
        }),
        DeclaredValueKindV0::BareIdent(value) => {
            Some(AbstractCssTypedScalarValueV0::Keyword { value })
        }
        DeclaredValueKindV0::QuotedString => Some(AbstractCssTypedScalarValueV0::QuotedString {
            value: trimmed.to_string(),
        }),
        DeclaredValueKindV0::Url => Some(AbstractCssTypedScalarValueV0::Url {
            value: trimmed.to_string(),
        }),
        DeclaredValueKindV0::ImageFunction => Some(AbstractCssTypedScalarValueV0::Image {
            serialized: trimmed.to_string(),
        }),
        DeclaredValueKindV0::TransformFunction => Some(AbstractCssTypedScalarValueV0::Transform {
            serialized: trimmed.to_string(),
        }),
        DeclaredValueKindV0::CssWide => Some(AbstractCssTypedScalarValueV0::CssWide {
            value: trimmed.to_string(),
        }),
        DeclaredValueKindV0::Unknown => None,
    }
}

pub fn join_abstract_css_typed_values(
    left: Option<&AbstractCssTypedValueV0>,
    right: Option<&AbstractCssTypedValueV0>,
) -> Option<AbstractCssTypedValueV0> {
    match (left, right) {
        (Some(left), Some(right)) => Some(join_present_typed_values(left, right)),
        (Some(value), None) | (None, Some(value)) => Some(value.clone()),
        (None, None) => None,
    }
}

pub fn abstract_css_typed_value_kind_label(value: &AbstractCssTypedValueV0) -> &'static str {
    match value {
        AbstractCssTypedValueV0::Exact { value } => typed_scalar_family_label(value),
        AbstractCssTypedValueV0::FiniteSet { .. } => "finiteSet",
        AbstractCssTypedValueV0::Top => "top",
    }
}

pub fn compare_abstract_css_values_with_typed_payloads(
    left: &AbstractCssValueV0,
    operator: AbstractCssTypedComparisonOperatorV0,
    right: &AbstractCssValueV0,
) -> Option<bool> {
    typed_scalar_comparison(
        exact_typed_scalar(left)?,
        operator,
        exact_typed_scalar(right)?,
    )
}

fn join_present_typed_values(
    left: &AbstractCssTypedValueV0,
    right: &AbstractCssTypedValueV0,
) -> AbstractCssTypedValueV0 {
    match (left, right) {
        (AbstractCssTypedValueV0::Top, _) | (_, AbstractCssTypedValueV0::Top) => {
            AbstractCssTypedValueV0::Top
        }
        (
            AbstractCssTypedValueV0::Exact { value: left },
            AbstractCssTypedValueV0::Exact { value: right },
        ) if left == right => AbstractCssTypedValueV0::Exact {
            value: left.clone(),
        },
        (
            AbstractCssTypedValueV0::Exact { value: left },
            AbstractCssTypedValueV0::Exact { value: right },
        ) if typed_scalars_share_family(left, right) => finite_typed_value_set([left, right]),
        (
            AbstractCssTypedValueV0::FiniteSet { values },
            AbstractCssTypedValueV0::Exact { value },
        )
        | (
            AbstractCssTypedValueV0::Exact { value },
            AbstractCssTypedValueV0::FiniteSet { values },
        ) if typed_scalar_is_compatible_with_set(value, values) => {
            finite_typed_value_set_from_iter(values.iter().chain(std::iter::once(value)))
        }
        (
            AbstractCssTypedValueV0::FiniteSet {
                values: left_values,
            },
            AbstractCssTypedValueV0::FiniteSet {
                values: right_values,
            },
        ) if typed_sets_are_compatible(left_values, right_values) => {
            finite_typed_value_set_from_iter(left_values.iter().chain(right_values.iter()))
        }
        _ => AbstractCssTypedValueV0::Top,
    }
}

fn finite_typed_value_set<const N: usize>(
    values: [&AbstractCssTypedScalarValueV0; N],
) -> AbstractCssTypedValueV0 {
    finite_typed_value_set_from_iter(values.into_iter())
}

fn finite_typed_value_set_from_iter<'a>(
    values: impl Iterator<Item = &'a AbstractCssTypedScalarValueV0>,
) -> AbstractCssTypedValueV0 {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(value) {
            unique.push(value.clone());
        }
    }
    if unique.len() == 1 {
        AbstractCssTypedValueV0::Exact {
            value: unique
                .pop()
                .unwrap_or(AbstractCssTypedScalarValueV0::Keyword {
                    value: String::new(),
                }),
        }
    } else {
        AbstractCssTypedValueV0::FiniteSet { values: unique }
    }
}

fn typed_scalar_is_compatible_with_set(
    value: &AbstractCssTypedScalarValueV0,
    values: &[AbstractCssTypedScalarValueV0],
) -> bool {
    values
        .first()
        .is_some_and(|first| typed_scalars_share_family(first, value))
        && values
            .iter()
            .all(|candidate| typed_scalars_share_family(candidate, value))
}

fn typed_sets_are_compatible(
    left: &[AbstractCssTypedScalarValueV0],
    right: &[AbstractCssTypedScalarValueV0],
) -> bool {
    match (left.first(), right.first()) {
        (Some(left_first), Some(right_first))
            if typed_scalars_share_family(left_first, right_first) =>
        {
            left.iter()
                .chain(right.iter())
                .all(|value| typed_scalars_share_family(left_first, value))
        }
        _ => false,
    }
}

fn typed_scalars_share_family(
    left: &AbstractCssTypedScalarValueV0,
    right: &AbstractCssTypedScalarValueV0,
) -> bool {
    typed_scalar_family_label(left) == typed_scalar_family_label(right)
}

fn typed_scalar_family_label(value: &AbstractCssTypedScalarValueV0) -> &'static str {
    match value {
        AbstractCssTypedScalarValueV0::Dimension { numeric_type, .. } => match numeric_type {
            DeclaredNumericTypeV0::Length => "dimension:length",
            DeclaredNumericTypeV0::Percentage => "dimension:percentage",
            DeclaredNumericTypeV0::Angle => "dimension:angle",
            DeclaredNumericTypeV0::Time => "dimension:time",
            DeclaredNumericTypeV0::Resolution => "dimension:resolution",
        },
        AbstractCssTypedScalarValueV0::Number { .. } => "number",
        AbstractCssTypedScalarValueV0::Integer { .. } => "integer",
        AbstractCssTypedScalarValueV0::Color { .. } => "color",
        AbstractCssTypedScalarValueV0::Keyword { .. } => "keyword",
        AbstractCssTypedScalarValueV0::QuotedString { .. } => "quotedString",
        AbstractCssTypedScalarValueV0::Url { .. } => "url",
        AbstractCssTypedScalarValueV0::Image { .. } => "image",
        AbstractCssTypedScalarValueV0::Transform { .. } => "transform",
        AbstractCssTypedScalarValueV0::CssWide { .. } => "cssWide",
    }
}

fn exact_typed_scalar(value: &AbstractCssValueV0) -> Option<&AbstractCssTypedScalarValueV0> {
    let typed = match value {
        AbstractCssValueV0::Exact {
            typed: Some(typed), ..
        } => typed.as_ref(),
        AbstractCssValueV0::Bottom
        | AbstractCssValueV0::Exact { typed: None, .. }
        | AbstractCssValueV0::FiniteSet { .. }
        | AbstractCssValueV0::Raw { .. }
        | AbstractCssValueV0::Top => return None,
    };
    match typed {
        AbstractCssTypedValueV0::Exact { value } => Some(value),
        AbstractCssTypedValueV0::FiniteSet { .. } | AbstractCssTypedValueV0::Top => None,
    }
}

fn typed_scalar_comparison(
    left: &AbstractCssTypedScalarValueV0,
    operator: AbstractCssTypedComparisonOperatorV0,
    right: &AbstractCssTypedScalarValueV0,
) -> Option<bool> {
    let left = typed_scalar_comparable_number(left)?;
    let right = typed_scalar_comparable_number(right)?;
    if left.family != right.family {
        return None;
    }
    Some(match operator {
        AbstractCssTypedComparisonOperatorV0::Equal => css_float_eq(left.value, right.value),
        AbstractCssTypedComparisonOperatorV0::NotEqual => !css_float_eq(left.value, right.value),
        AbstractCssTypedComparisonOperatorV0::LessThan => left.value < right.value,
        AbstractCssTypedComparisonOperatorV0::LessThanOrEqual => {
            left.value < right.value || css_float_eq(left.value, right.value)
        }
        AbstractCssTypedComparisonOperatorV0::GreaterThan => left.value > right.value,
        AbstractCssTypedComparisonOperatorV0::GreaterThanOrEqual => {
            left.value > right.value || css_float_eq(left.value, right.value)
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypedNumericFamily {
    Number,
    Length,
    Percentage,
    Angle,
    Time,
    Resolution,
}

#[derive(Debug, Clone, Copy)]
struct TypedComparableNumber {
    family: TypedNumericFamily,
    value: f64,
}

fn typed_scalar_comparable_number(
    value: &AbstractCssTypedScalarValueV0,
) -> Option<TypedComparableNumber> {
    match value {
        AbstractCssTypedScalarValueV0::Number { number, .. }
        | AbstractCssTypedScalarValueV0::Integer { number, .. } => Some(TypedComparableNumber {
            family: TypedNumericFamily::Number,
            value: number.parse().ok()?,
        }),
        AbstractCssTypedScalarValueV0::Dimension {
            numeric_type,
            number,
            unit,
            ..
        } => {
            let number = number.parse::<f64>().ok()?;
            let unit_factor = typed_dimension_unit_factor(*numeric_type, unit.as_str())?;
            Some(TypedComparableNumber {
                family: typed_numeric_family(*numeric_type),
                value: number * unit_factor,
            })
        }
        AbstractCssTypedScalarValueV0::Color { .. }
        | AbstractCssTypedScalarValueV0::Keyword { .. }
        | AbstractCssTypedScalarValueV0::QuotedString { .. }
        | AbstractCssTypedScalarValueV0::Url { .. }
        | AbstractCssTypedScalarValueV0::Image { .. }
        | AbstractCssTypedScalarValueV0::Transform { .. }
        | AbstractCssTypedScalarValueV0::CssWide { .. } => None,
    }
}

fn typed_numeric_family(numeric_type: DeclaredNumericTypeV0) -> TypedNumericFamily {
    match numeric_type {
        DeclaredNumericTypeV0::Length => TypedNumericFamily::Length,
        DeclaredNumericTypeV0::Percentage => TypedNumericFamily::Percentage,
        DeclaredNumericTypeV0::Angle => TypedNumericFamily::Angle,
        DeclaredNumericTypeV0::Time => TypedNumericFamily::Time,
        DeclaredNumericTypeV0::Resolution => TypedNumericFamily::Resolution,
    }
}

fn typed_dimension_unit_factor(numeric_type: DeclaredNumericTypeV0, unit: &str) -> Option<f64> {
    let unit = unit.to_ascii_lowercase();
    match numeric_type {
        DeclaredNumericTypeV0::Length => match unit.as_str() {
            "px" => Some(1.0),
            "in" => Some(96.0),
            "cm" => Some(96.0 / 2.54),
            "mm" => Some(96.0 / 25.4),
            "q" => Some(96.0 / 101.6),
            "pt" => Some(96.0 / 72.0),
            "pc" => Some(16.0),
            _ => None,
        },
        DeclaredNumericTypeV0::Percentage => (unit == "%").then_some(1.0),
        DeclaredNumericTypeV0::Angle => match unit.as_str() {
            "deg" => Some(1.0),
            "grad" => Some(0.9),
            "rad" => Some(180.0 / std::f64::consts::PI),
            "turn" => Some(360.0),
            _ => None,
        },
        DeclaredNumericTypeV0::Time => match unit.as_str() {
            "s" => Some(1.0),
            "ms" => Some(0.001),
            _ => None,
        },
        DeclaredNumericTypeV0::Resolution => match unit.as_str() {
            "dppx" | "x" => Some(1.0),
            "dpi" => Some(1.0 / 96.0),
            "dpcm" => Some(2.54 / 96.0),
            _ => None,
        },
    }
}

fn css_float_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
