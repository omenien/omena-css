use std::collections::BTreeSet;

use super::{
    model::{StaticScssFunctionArgument, StaticScssFunctionParameter},
    scss_arguments::static_scss_top_level_colon_index,
    static_less_mixin_argument_value_is_safe, static_less_variable_name_is_safe,
};

pub(super) fn collect_static_less_mixin_parameters(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionParameter>> {
    let parameter_start = tokens
        .get(start)
        .map(super::static_stylesheet_token_start)?;
    let parameter_end = tokens
        .get(end)
        .map(super::static_stylesheet_token_start)
        .unwrap_or(parameter_start);
    let parameter_text = source.get(parameter_start..parameter_end)?.trim();
    let arguments = split_static_less_mixin_parameter_arguments(parameter_text)?;
    let mut parameters = Vec::new();
    let mut names = BTreeSet::new();
    let mut saw_default = false;
    let argument_count = arguments.len();
    for (index, argument) in arguments.into_iter().enumerate() {
        let parameter = parse_static_less_mixin_parameter(argument)?;
        if parameter.variadic && index + 1 != argument_count {
            return None;
        }
        if parameter.default_value.is_some() {
            saw_default = true;
        } else if saw_default && !parameter.variadic {
            return None;
        }
        if parameter.pattern_value.is_none() && !names.insert(parameter.name.clone()) {
            return None;
        }
        parameters.push(parameter);
    }
    Some(parameters)
}

pub(super) fn split_static_less_mixin_arguments(
    arguments: &str,
) -> Option<Vec<StaticScssFunctionArgument>> {
    split_static_less_mixin_arguments_with_options(arguments, false)
}

pub(super) fn static_less_mixin_pattern_argument_matches(
    pattern_value: &str,
    argument_value: &str,
) -> bool {
    pattern_value.trim() == argument_value.trim()
}

pub(super) fn static_less_mixin_parameter_patterns_match(
    parameters: &[StaticScssFunctionParameter],
    arguments: &[StaticScssFunctionArgument],
) -> bool {
    let mut positional_index = 0usize;
    let mut saw_named_argument = false;
    for argument in arguments {
        if argument.name.is_some() {
            saw_named_argument = true;
            continue;
        }
        if saw_named_argument {
            return true;
        }
        let Some(parameter) = parameters.get(positional_index) else {
            return true;
        };
        if let Some(pattern_value) = parameter.pattern_value.as_deref()
            && !static_less_mixin_pattern_argument_matches(pattern_value, argument.value.as_str())
        {
            return false;
        }
        positional_index += 1;
        if parameter.variadic {
            return true;
        }
    }
    parameters
        .iter()
        .enumerate()
        .all(|(index, parameter)| parameter.pattern_value.is_none() || index < positional_index)
}

fn split_static_less_mixin_parameter_arguments(
    arguments: &str,
) -> Option<Vec<StaticScssFunctionArgument>> {
    split_static_less_mixin_arguments_with_options(arguments, true)
}

fn parse_static_less_mixin_parameter(
    argument: StaticScssFunctionArgument,
) -> Option<StaticScssFunctionParameter> {
    if let Some(name) = argument.name {
        return Some(StaticScssFunctionParameter {
            name,
            default_value: Some(argument.value),
            variadic: false,
            pattern_value: None,
        });
    }
    let (name, variadic) = if let Some(name) = argument.value.strip_suffix("...") {
        (name.trim(), true)
    } else {
        (argument.value.as_str(), false)
    };
    if static_less_variable_name_is_safe(name) {
        return Some(StaticScssFunctionParameter {
            name: name.to_string(),
            default_value: None,
            variadic,
            pattern_value: None,
        });
    }
    (!variadic && static_less_mixin_argument_value_is_safe(argument.value.as_str())).then(|| {
        StaticScssFunctionParameter {
            name: String::new(),
            default_value: None,
            variadic: false,
            pattern_value: Some(argument.value),
        }
    })
}

fn split_static_less_mixin_arguments_with_options(
    arguments: &str,
    allow_rest_parameter: bool,
) -> Option<Vec<StaticScssFunctionArgument>> {
    let arguments = arguments.trim();
    if arguments.is_empty() {
        return Some(Vec::new());
    }
    let separator = if static_less_mixin_arguments_have_top_level_separator(arguments, ';')? {
        ';'
    } else {
        ','
    };
    split_static_less_mixin_arguments_with_separator(arguments, separator, allow_rest_parameter)
}

fn split_static_less_mixin_arguments_with_separator(
    arguments: &str,
    separator: char,
    allow_rest_parameter: bool,
) -> Option<Vec<StaticScssFunctionArgument>> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < arguments.len() {
        let ch = arguments[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
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
            ch if ch == separator && paren_depth == 0 && bracket_depth == 0 => {
                values.push(parse_static_less_mixin_argument(
                    arguments.get(cursor..index)?.trim(),
                    allow_rest_parameter,
                )?);
                cursor = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    values.push(parse_static_less_mixin_argument(
        arguments.get(cursor..)?.trim(),
        allow_rest_parameter,
    )?);
    Some(values)
}

fn static_less_mixin_arguments_have_top_level_separator(
    arguments: &str,
    separator: char,
) -> Option<bool> {
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < arguments.len() {
        let ch = arguments[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
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
            ch if ch == separator && paren_depth == 0 && bracket_depth == 0 => return Some(true),
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0).then_some(false)
}

fn parse_static_less_mixin_argument(
    value: &str,
    allow_rest_parameter: bool,
) -> Option<StaticScssFunctionArgument> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(colon_index) = static_scss_top_level_colon_index(value)? {
        let name = value.get(..colon_index)?.trim();
        let argument_value = value.get(colon_index + ':'.len_utf8()..)?.trim();
        if !static_less_variable_name_is_safe(name)
            || !static_less_mixin_argument_value_is_safe(argument_value)
        {
            return None;
        }
        return Some(StaticScssFunctionArgument {
            name: Some(name.to_string()),
            value: argument_value.to_string(),
        });
    }
    if allow_rest_parameter
        && let Some(rest_name) = value.strip_suffix("...")
        && static_less_variable_name_is_safe(rest_name.trim())
    {
        return Some(StaticScssFunctionArgument {
            name: None,
            value: value.to_string(),
        });
    }
    static_less_mixin_argument_value_is_safe(value).then_some(StaticScssFunctionArgument {
        name: None,
        value: value.to_string(),
    })
}
