use std::collections::BTreeMap;

use omena_value_lattice::{
    parse_color_function_value, parse_color_mix_value, parse_numeric_value_with_unit,
    parse_oklab_oklch_value, parse_static_hsl_function_color_with_alpha,
    parse_static_hwb_function_color_with_alpha, parse_static_rgb_function_color_with_alpha,
    parse_static_srgb_color_with_alpha, parse_whole_function_value_arguments,
    parse_whole_function_value_inner,
};

use crate::value_eval::static_scss_literal_truthiness;

use super::{
    StaticLessDetachedRulesetDeclaration, StaticLessMixinRenderContext,
    StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration, find_static_less_detached_ruleset_declaration,
    find_static_less_property_declaration, find_static_less_variable_declaration,
    less_mixin_values::resolve_static_less_mixin_value_with_bindings,
    static_less_variable_name_is_safe, static_stylesheet_property_name_is_safe,
};

pub(super) fn static_less_mixin_guard_matches(
    guard: &str,
    argument_values: &BTreeMap<String, String>,
    call_scope_id: usize,
    call_position: usize,
    render_context: StaticLessMixinRenderContext<'_>,
    default_matches: Option<bool>,
) -> Option<bool> {
    let guard = guard.trim();
    let when = guard.get(.."when".len())?;
    if !when.eq_ignore_ascii_case("when") {
        return None;
    }
    let expression = guard.get("when".len()..)?.trim();
    let context = StaticLessGuardContext {
        argument_values,
        captured_values: render_context.captured_values,
        call_scope_id,
        call_position,
        scopes: render_context.scopes,
        variable_declarations: render_context.variable_declarations,
        property_declarations: render_context.property_declarations,
        detached_ruleset_declarations: render_context.detached_ruleset_declarations,
        default_matches,
    };
    static_less_guard_expression_matches(expression, context)
}

pub(super) fn static_less_value_condition_matches(expression: &str) -> Option<bool> {
    let argument_values = BTreeMap::new();
    let captured_values = BTreeMap::new();
    let variable_declarations = BTreeMap::new();
    let property_declarations = BTreeMap::new();
    let context = StaticLessGuardContext {
        argument_values: &argument_values,
        captured_values: &captured_values,
        call_scope_id: 0,
        call_position: 0,
        scopes: &[],
        variable_declarations: &variable_declarations,
        property_declarations: &property_declarations,
        detached_ruleset_declarations: &[],
        default_matches: Some(false),
    };
    static_less_guard_expression_matches(expression, context)
}

#[derive(Clone, Copy)]
struct StaticLessGuardContext<'a> {
    argument_values: &'a BTreeMap<String, String>,
    captured_values: &'a BTreeMap<String, String>,
    call_scope_id: usize,
    call_position: usize,
    scopes: &'a [StaticStylesheetScope],
    variable_declarations: &'a BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &'a BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &'a [StaticLessDetachedRulesetDeclaration],
    default_matches: Option<bool>,
}

fn static_less_guard_expression_matches(
    expression: &str,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    let expression = expression.trim();
    if expression.is_empty() {
        return None;
    }
    if expression.eq_ignore_ascii_case("true") {
        return Some(true);
    }
    if expression.eq_ignore_ascii_case("false") {
        return Some(false);
    }
    if let Some(inner) = static_less_guard_strip_outer_parens(expression) {
        return static_less_guard_expression_matches(inner, context);
    }
    if let Some(operands) = split_static_less_guard_top_level_separator(expression, ',')? {
        return static_less_guard_or_matches(operands, context);
    }
    if let Some(operands) = split_static_less_guard_top_level_keyword(expression, "and")? {
        return static_less_guard_and_matches(operands, context);
    }
    if expression
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("not"))
        && let Some(operand) = expression.get(3..)
        && (operand.chars().next().is_some_and(char::is_whitespace)
            || operand.trim_start().starts_with('('))
    {
        return static_less_guard_expression_matches(operand.trim(), context).map(|truthy| !truthy);
    }
    static_less_guard_predicate_expression_matches(expression, context)
        .or_else(|| static_less_guard_default_matches(expression, context))
        .or_else(|| static_less_guard_comparison_matches(expression, context))
}

pub(super) fn static_less_mixin_guard_depends_on_default(guard: &str) -> bool {
    guard
        .to_ascii_lowercase()
        .split("when")
        .nth(1)
        .is_some_and(|expression| expression.contains("default("))
}

pub(super) fn static_less_mixin_guard_depends_on_negated_default(guard: &str) -> bool {
    guard
        .to_ascii_lowercase()
        .split("when")
        .nth(1)
        .map(|expression| {
            expression
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>()
        })
        .is_some_and(|expression| expression.contains("not(default("))
}

fn static_less_guard_or_matches(
    operands: Vec<&str>,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_less_guard_expression_matches(operand, context) {
            Some(true) => return Some(true),
            Some(false) => {}
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(false)
}

fn static_less_guard_and_matches(
    operands: Vec<&str>,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_less_guard_expression_matches(operand, context) {
            Some(true) => {}
            Some(false) => return Some(false),
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(true)
}

fn static_less_guard_predicate_expression_matches(
    predicate: &str,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    static_less_mixin_guard_predicate_matches(
        predicate,
        "iscolor",
        context,
        static_less_guard_value_is_color,
    )
    .or_else(|| {
        static_less_mixin_guard_predicate_matches(
            predicate,
            "isnumber",
            context,
            static_less_guard_value_is_number,
        )
    })
    .or_else(|| {
        static_less_mixin_guard_predicate_matches(predicate, "ispixel", context, |value| {
            static_less_guard_value_has_unit(value, "px")
        })
    })
    .or_else(|| {
        static_less_mixin_guard_predicate_matches(predicate, "ispercentage", context, |value| {
            static_less_guard_value_has_unit(value, "%")
        })
    })
    .or_else(|| {
        static_less_mixin_guard_predicate_matches(predicate, "isem", context, |value| {
            static_less_guard_value_has_unit(value, "em")
        })
    })
    .or_else(|| static_less_mixin_guard_isunit_predicate_matches(predicate, context))
    .or_else(|| static_less_mixin_guard_isdefined_predicate_matches(predicate, context))
    .or_else(|| static_less_mixin_guard_isruleset_predicate_matches(predicate, context))
    .or_else(|| {
        static_less_mixin_guard_predicate_matches(
            predicate,
            "isurl",
            context,
            static_less_guard_value_is_url,
        )
    })
    .or_else(|| {
        static_less_mixin_guard_predicate_matches(
            predicate,
            "isstring",
            context,
            static_less_guard_value_is_string,
        )
    })
    .or_else(|| {
        static_less_mixin_guard_predicate_matches(
            predicate,
            "iskeyword",
            context,
            static_less_guard_value_is_keyword,
        )
    })
}

fn static_less_guard_default_matches(
    predicate: &str,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    parse_whole_function_value_inner(predicate, "default")
        .filter(|inner| inner.trim().is_empty())
        .and(context.default_matches)
}

fn static_less_mixin_guard_predicate_matches(
    predicate: &str,
    function_name: &str,
    context: StaticLessGuardContext<'_>,
    matches_value: impl FnOnce(&str) -> bool,
) -> Option<bool> {
    let arguments = parse_whole_function_value_arguments(predicate, function_name)?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let resolved = resolve_static_less_mixin_value_with_bindings(
        value.trim(),
        context.argument_values,
        context.captured_values,
        context.call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        Some(context.call_position),
        context.detached_ruleset_declarations,
    )?;
    Some(matches_value(resolved.trim()))
}

fn static_less_mixin_guard_isunit_predicate_matches(
    predicate: &str,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    let arguments = parse_whole_function_value_arguments(predicate, "isunit")?;
    let [value, unit] = arguments.as_slice() else {
        return None;
    };
    let resolved_value = resolve_static_less_mixin_value_with_bindings(
        value.trim(),
        context.argument_values,
        context.captured_values,
        context.call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        Some(context.call_position),
        context.detached_ruleset_declarations,
    )?;
    let resolved_unit = resolve_static_less_mixin_value_with_bindings(
        unit.trim(),
        context.argument_values,
        context.captured_values,
        context.call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        Some(context.call_position),
        context.detached_ruleset_declarations,
    )?;
    let expected_unit = static_less_guard_unit_text(resolved_unit.trim())?;
    Some(static_less_guard_value_has_unit(
        resolved_value.trim(),
        expected_unit,
    ))
}

fn static_less_mixin_guard_isdefined_predicate_matches(
    predicate: &str,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    let arguments = parse_whole_function_value_arguments(predicate, "isdefined")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with('$') {
        if !value
            .strip_prefix('$')
            .is_some_and(static_stylesheet_property_name_is_safe)
        {
            return None;
        }
        return Some(
            find_static_less_property_declaration(
                value,
                context.call_scope_id,
                context.scopes,
                context.property_declarations,
            )
            .is_some(),
        );
    }
    if !value.starts_with('@') {
        return Some(true);
    }
    if value.starts_with("@@") || !static_less_variable_name_is_safe(value) {
        return None;
    }
    Some(
        context.argument_values.contains_key(value)
            || context.captured_values.contains_key(value)
            || find_static_less_variable_declaration(
                value,
                context.call_scope_id,
                context.scopes,
                context.variable_declarations,
            )
            .is_some()
            || find_static_less_detached_ruleset_declaration(
                value,
                context.call_scope_id,
                context.scopes,
                context.detached_ruleset_declarations,
            )
            .is_some(),
    )
}

fn static_less_mixin_guard_isruleset_predicate_matches(
    predicate: &str,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    let arguments = parse_whole_function_value_arguments(predicate, "isruleset")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let resolved = resolve_static_less_mixin_value_with_bindings(
        value.trim(),
        context.argument_values,
        context.captured_values,
        context.call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        Some(context.call_position),
        context.detached_ruleset_declarations,
    )?;
    Some(static_less_guard_value_is_ruleset(
        resolved.trim(),
        context.call_scope_id,
        context.scopes,
        context.detached_ruleset_declarations,
    ))
}

fn static_less_guard_comparison_matches(
    expression: &str,
    context: StaticLessGuardContext<'_>,
) -> Option<bool> {
    let (left, operator, right) = split_static_less_guard_comparison(expression)?;
    let left = resolve_static_less_mixin_value_with_bindings(
        left,
        context.argument_values,
        context.captured_values,
        context.call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        Some(context.call_position),
        context.detached_ruleset_declarations,
    )?;
    let right = resolve_static_less_mixin_value_with_bindings(
        right,
        context.argument_values,
        context.captured_values,
        context.call_scope_id,
        context.scopes,
        context.variable_declarations,
        context.property_declarations,
        Some(context.call_position),
        context.detached_ruleset_declarations,
    )?;
    static_scss_literal_truthiness(
        format!(
            "{} {} {}",
            left.trim(),
            operator.scss_operator(),
            right.trim()
        )
        .as_str(),
    )
}

#[derive(Clone, Copy)]
enum StaticLessGuardComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl StaticLessGuardComparisonOperator {
    fn scss_operator(self) -> &'static str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
        }
    }
}

fn split_static_less_guard_comparison(
    expression: &str,
) -> Option<(&str, StaticLessGuardComparisonOperator, &str)> {
    let mut comparison = None;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < expression.len() {
        let ch = expression.get(index..)?.chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = expression.get(index..).and_then(|rest| rest.chars().next())
                {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            '=' | '!' | '<' | '>' if paren_depth == 0 && bracket_depth == 0 => {
                let (operator, width) =
                    static_less_guard_comparison_operator_at(expression, index)?;
                let left = expression.get(..index)?.trim();
                let right = expression.get(index + width..)?.trim();
                if left.is_empty() || right.is_empty() || comparison.is_some() {
                    return None;
                }
                comparison = Some((left, operator, right));
                index += width;
                continue;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    comparison
}

fn static_less_guard_comparison_operator_at(
    expression: &str,
    index: usize,
) -> Option<(StaticLessGuardComparisonOperator, usize)> {
    let suffix = expression.get(index..)?;
    if suffix.starts_with("!=") {
        return Some((StaticLessGuardComparisonOperator::NotEqual, 2));
    }
    if suffix.starts_with("==") {
        return Some((StaticLessGuardComparisonOperator::Equal, 2));
    }
    if suffix.starts_with("<=") || suffix.starts_with("=<") {
        return Some((StaticLessGuardComparisonOperator::LessThanOrEqual, 2));
    }
    if suffix.starts_with(">=") || suffix.starts_with("=>") {
        return Some((StaticLessGuardComparisonOperator::GreaterThanOrEqual, 2));
    }
    if suffix.starts_with('=') {
        return Some((StaticLessGuardComparisonOperator::Equal, 1));
    }
    if suffix.starts_with('<') {
        return Some((StaticLessGuardComparisonOperator::LessThan, 1));
    }
    if suffix.starts_with('>') {
        return Some((StaticLessGuardComparisonOperator::GreaterThan, 1));
    }
    None
}

fn static_less_guard_strip_outer_parens(value: &str) -> Option<&str> {
    value.strip_prefix('(')?;
    value.strip_suffix(')')?;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < value.len() {
        let ch = value.get(index..)?.chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value.get(index..).and_then(|rest| rest.chars().next()) {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 && index + ch.len_utf8() != value.len() {
                    return None;
                }
            }
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            _ => {}
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    Some(value.strip_prefix('(')?.strip_suffix(')')?.trim())
}

fn split_static_less_guard_top_level_separator(
    expression: &str,
    separator: char,
) -> Option<Option<Vec<&str>>> {
    let mut operands = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < expression.len() {
        let ch = expression.get(index..)?.chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = expression.get(index..).and_then(|rest| rest.chars().next())
                {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            _ => {}
        }
        if ch == separator && paren_depth == 0 && bracket_depth == 0 {
            let operand = expression.get(cursor..index)?.trim();
            if operand.is_empty() {
                return None;
            }
            operands.push(operand);
            cursor = index + ch.len_utf8();
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    if operands.is_empty() {
        return Some(None);
    }
    let operand = expression.get(cursor..)?.trim();
    if operand.is_empty() {
        return None;
    }
    operands.push(operand);
    Some(Some(operands))
}

fn split_static_less_guard_top_level_keyword<'a>(
    expression: &'a str,
    keyword: &str,
) -> Option<Option<Vec<&'a str>>> {
    let mut operands = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < expression.len() {
        let ch = expression.get(index..)?.chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = expression.get(index..).and_then(|rest| rest.chars().next())
                {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && static_less_guard_keyword_at(expression, index, keyword)
        {
            let operand = expression.get(cursor..index)?.trim();
            if operand.is_empty() {
                return None;
            }
            operands.push(operand);
            index += keyword.len();
            cursor = index;
            continue;
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    if operands.is_empty() {
        return Some(None);
    }
    let operand = expression.get(cursor..)?.trim();
    if operand.is_empty() {
        return None;
    }
    operands.push(operand);
    Some(Some(operands))
}

fn static_less_guard_keyword_at(expression: &str, index: usize, keyword: &str) -> bool {
    if expression
        .get(index..)
        .is_none_or(|suffix| suffix.len() < keyword.len())
    {
        return false;
    }
    if !expression
        .get(index..index + keyword.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(keyword))
    {
        return false;
    }
    let before_ok = expression
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(char::is_whitespace);
    let after_index = index + keyword.len();
    let after_ok = expression
        .get(after_index..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(char::is_whitespace);
    before_ok && after_ok
}

pub(super) fn static_less_guard_value_is_color(value: &str) -> bool {
    parse_static_srgb_color_with_alpha(value).is_some()
        || parse_static_rgb_function_color_with_alpha(value).is_some()
        || parse_static_hsl_function_color_with_alpha(value).is_some()
        || parse_static_hwb_function_color_with_alpha(value).is_some()
        || parse_color_function_value(value).is_some()
        || parse_color_mix_value(value).is_some()
        || parse_oklab_oklch_value(value).is_some()
}

pub(super) fn static_less_guard_value_is_number(value: &str) -> bool {
    parse_numeric_value_with_unit(value).is_some()
}

pub(super) fn static_less_guard_value_has_unit(value: &str, expected_unit: &str) -> bool {
    parse_numeric_value_with_unit(value)
        .is_some_and(|value| value.unit.eq_ignore_ascii_case(expected_unit))
}

pub(super) fn static_less_guard_unit_text(value: &str) -> Option<&str> {
    let value = value.trim();
    if matches!(value, "%") {
        return Some(value);
    }
    if static_less_guard_value_is_string(value) {
        return static_less_guard_quoted_string_inner(value);
    }
    static_stylesheet_property_name_is_safe(value).then_some(value)
}

pub(super) fn static_less_guard_value_is_url(value: &str) -> bool {
    parse_whole_function_value_arguments(value.trim(), "url")
        .is_some_and(|arguments| arguments.len() == 1)
}

pub(super) fn static_less_guard_value_is_string(value: &str) -> bool {
    static_less_guard_quoted_string_end(value.trim(), 0)
        .is_some_and(|end| end == value.trim().len())
}

fn static_less_guard_quoted_string_inner(value: &str) -> Option<&str> {
    let value = value.trim();
    let quote = value.chars().next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    let end = static_less_guard_quoted_string_end(value, 0)?;
    if end != value.len() {
        return None;
    }
    value.get(quote.len_utf8()..value.len().checked_sub(quote.len_utf8())?)
}

pub(super) fn static_less_guard_value_is_keyword(value: &str) -> bool {
    let value = value.trim();
    if !static_stylesheet_property_name_is_safe(value)
        || static_less_guard_value_is_color(value)
        || static_less_guard_value_is_number(value)
    {
        return false;
    }
    !matches!(
        value.to_ascii_lowercase().as_str(),
        "false" | "null" | "true"
    )
}

fn static_less_guard_value_is_ruleset(
    value: &str,
    call_scope_id: usize,
    scopes: &[StaticStylesheetScope],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> bool {
    let value = value.trim();
    value.starts_with('@')
        && find_static_less_detached_ruleset_declaration(
            value,
            call_scope_id,
            scopes,
            detached_ruleset_declarations,
        )
        .is_some()
}

fn static_less_guard_quoted_string_end(source: &str, start: usize) -> Option<usize> {
    let quote = source.get(start..)?.chars().next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    let mut index = start + quote.len_utf8();
    while index < source.len() {
        let ch = source.get(index..)?.chars().next()?;
        index += ch.len_utf8();
        if ch == '\\' {
            if let Some(escaped) = source.get(index..).and_then(|rest| rest.chars().next()) {
                index += escaped.len_utf8();
            }
        } else if ch == quote {
            return Some(index);
        }
    }
    None
}
