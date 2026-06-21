use omena_abstract_value::{
    AbstractCssTypedComparisonOperatorV0, abstract_css_value_from_text,
    compare_abstract_css_values_with_typed_payloads,
};
use omena_value_lattice::{
    number::{
        compress_number_prefix, numeric_prefix_end, parse_reducible_abs_value,
        parse_reducible_calc_value, parse_reducible_clamp_value, parse_reducible_exp_value,
        parse_reducible_hypot_value, parse_reducible_log_value, parse_reducible_max_value,
        parse_reducible_min_value, parse_reducible_mod_value, parse_reducible_pow_value,
        parse_reducible_rem_value, parse_reducible_round_value, parse_reducible_sign_value,
        parse_reducible_sqrt_value, reduce_static_numeric_expression,
    },
    parse_numeric_value_with_unit, substitute_static_css_function_references_in_value_until_stable,
};

pub(crate) fn reduce_static_numeric_value(value: String) -> String {
    let trimmed = value.trim();
    if let Some(reduced) = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("calc", parse_reducible_calc_value),
            ("min", parse_reducible_min_value),
            ("max", parse_reducible_max_value),
            ("clamp", parse_reducible_clamp_value),
            ("abs", parse_reducible_abs_value),
            ("sign", parse_reducible_sign_value),
            ("round", parse_reducible_round_value),
            ("mod", parse_reducible_mod_value),
            ("rem", parse_reducible_rem_value),
            ("hypot", parse_reducible_hypot_value),
            ("sqrt", parse_reducible_sqrt_value),
            ("pow", parse_reducible_pow_value),
            ("exp", parse_reducible_exp_value),
            ("log", parse_reducible_log_value),
        ],
    ) {
        return reduced;
    }
    if let Some(reduced) = reduce_static_numeric_expression(trimmed) {
        return reduced;
    }
    let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|without_left| without_left.strip_suffix(')'))
    else {
        return value;
    };
    reduce_static_numeric_expression(inner.trim()).unwrap_or(value)
}

pub(crate) fn static_scss_typed_advisory_numeric_comparison(
    left: &str,
    operator: AbstractCssTypedComparisonOperatorV0,
    right: &str,
) -> Option<bool> {
    compare_abstract_css_values_with_typed_payloads(
        &abstract_css_value_from_text(left),
        operator,
        &abstract_css_value_from_text(right),
    )
}

pub(crate) fn reduce_static_less_numeric_value(value: String) -> String {
    let trimmed = value.trim();
    // Less intentionally exposes a narrower math function surface than CSS/Sass.
    // Preserve unsupported CSS math calls byte-for-byte instead of over-reducing.
    if let Some(reduced) = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("min", parse_reducible_min_value),
            ("max", parse_reducible_max_value),
            ("abs", parse_reducible_abs_value),
            ("mod", parse_reducible_mod_value),
            ("sqrt", parse_reducible_sqrt_value),
            ("pow", parse_reducible_pow_value),
        ],
    ) {
        return reduced;
    }
    if let Some(reduced) = reduce_static_less_standalone_numeric_literal(trimmed) {
        return reduced;
    }
    if let Some(reduced) = reduce_static_numeric_expression(trimmed) {
        return reduced;
    }
    let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|without_left| without_left.strip_suffix(')'))
    else {
        return value;
    };
    reduce_static_numeric_expression(inner.trim()).unwrap_or(value)
}

fn reduce_static_less_standalone_numeric_literal(value: &str) -> Option<String> {
    parse_numeric_value_with_unit(value)?;
    let number_end = numeric_prefix_end(value)?;
    let (number, unit) = value.split_at(number_end);
    let mut number = compress_number_prefix(number);
    if let Some(fraction) = number.strip_prefix('.') {
        number = format!("0.{fraction}");
    } else if let Some(fraction) = number.strip_prefix("-.") {
        number = format!("-0.{fraction}");
    }
    Some(format!("{number}{unit}"))
}
