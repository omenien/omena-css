use std::collections::{BTreeMap, VecDeque};

use omena_cascade::{
    CascadeValue, CustomPropertyEnv, resolve_custom_property_env_least_fixed_point,
    substitute_custom_properties,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::helpers::{
    blocks::{at_rule_block_start, rule_block_token_indexes},
    collections::push_unique_string,
    declarations::collect_simple_declarations_in_block,
    identifiers::{is_css_ident_continue, normalize_custom_property_name},
    rules::{
        SimpleRuleSlice, collect_declaration_ordinary_rule_slices,
        collect_top_level_ordinary_rule_slices,
    },
    tokens::{matching_right_brace_index, skip_whitespace_tokens, token_end, token_start},
    values::{
        matching_function_call_end, parse_whole_function_value_arguments,
        split_top_level_value_arguments,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomPropertyRegistrationRule {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) initial_value: Option<String>,
}

pub(crate) fn collect_custom_property_registration_rules(
    tokens: &[omena_parser::LexedToken],
) -> Vec<CustomPropertyRegistrationRule> {
    let mut rules = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@property")
            && let Some((rule, next_index)) = parse_custom_property_registration_rule(tokens, index)
        {
            rules.push(rule);
            index = next_index;
            continue;
        }
        index += 1;
    }

    rules
}

fn parse_custom_property_registration_rule(
    tokens: &[omena_parser::LexedToken],
    at_property_index: usize,
) -> Option<(CustomPropertyRegistrationRule, usize)> {
    let name_index = skip_whitespace_tokens(tokens, at_property_index + 1, tokens.len());
    let name = normalize_custom_property_name(tokens.get(name_index)?.text.as_str())?.to_string();
    let block_start_index = at_rule_block_start(tokens, name_index + 1)?;
    let close_index = matching_right_brace_index(tokens, block_start_index)?;
    let initial_value =
        collect_simple_declarations_in_block(tokens, block_start_index, close_index)
            .into_iter()
            .find(|declaration| declaration.property == "initial-value" && !declaration.important)
            .map(|declaration| declaration.value);

    Some((
        CustomPropertyRegistrationRule {
            name,
            start: token_start(&tokens[at_property_index]),
            end: token_end(&tokens[close_index]),
            initial_value,
        },
        close_index + 1,
    ))
}

pub(crate) fn close_custom_property_dependency_graph(
    roots: Vec<String>,
    dependencies_by_name: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut reachable = Vec::new();
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();

    while let Some(name) = queue.pop_front() {
        if reachable.iter().any(|existing| existing == &name) {
            continue;
        }
        reachable.push(name.clone());
        if let Some(dependencies) = dependencies_by_name.get(&name) {
            for dependency in dependencies {
                queue.push_back(dependency.clone());
            }
        }
    }

    reachable.sort();
    reachable
}

pub(crate) fn collect_custom_property_references_in_value(value: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index = close_index + ')'.len_utf8();
                    continue;
                };
                if let [name, fallback @ ..] = arguments.as_slice()
                    && let Some(name) = normalize_custom_property_name(name)
                {
                    push_unique_string(&mut names, name.to_string());
                    for fallback_value in fallback {
                        for fallback_name in
                            collect_custom_property_references_in_value(fallback_value)
                        {
                            push_unique_string(&mut names, fallback_name);
                        }
                    }
                }
                index = close_index + ')'.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    names
}

pub(crate) fn substitute_static_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let env_rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let env = resolve_custom_property_env_least_fixed_point(
        &collect_static_root_custom_property_env(tokens, &env_rules),
    );
    if env.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = Vec::new();
    for rule in &rules {
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--") {
                continue;
            }
            let Some(resolved_value) =
                substitute_static_custom_property_references_in_value(&declaration.value, &env)
            else {
                continue;
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {resolved_value};", declaration.property),
            ));
        }
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn substitute_static_custom_property_references_in_value(
    value: &str,
    env: &CustomPropertyEnv,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(var_value) = parse_static_var_arguments(&arguments) else {
                    index += ch.len_utf8();
                    continue;
                };
                let resolved_value = substitute_custom_properties(&var_value, env);
                let Some(resolved_value) = render_static_cascade_value(&resolved_value) else {
                    index += ch.len_utf8();
                    continue;
                };
                output.push_str(&value[cursor..index]);
                output.push_str(&resolved_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                changed = true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

pub(crate) fn collect_static_root_custom_property_env(
    tokens: &[omena_parser::LexedToken],
    rules: &[SimpleRuleSlice],
) -> CustomPropertyEnv {
    let mut env = CustomPropertyEnv::new();
    let mut blocked_names = Vec::new();
    let registrations = collect_custom_property_registration_rules(tokens);

    for rule in rules {
        if rule.selector == ":root" {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--")
                && !blocked_names.contains(&declaration.property)
            {
                blocked_names.push(declaration.property);
            }
        }
    }

    for rule in rules {
        if rule.selector != ":root" {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if !declaration.property.starts_with("--") {
                continue;
            }
            if declaration.important {
                env.remove(&declaration.property);
                if !blocked_names.contains(&declaration.property) {
                    blocked_names.push(declaration.property);
                }
                continue;
            }
            if blocked_names.contains(&declaration.property) {
                continue;
            }
            if env.contains_key(&declaration.property) {
                env.remove(&declaration.property);
                blocked_names.push(declaration.property);
                continue;
            }
            let Some(value) = parse_static_custom_property_env_value(&declaration.value) else {
                env.remove(&declaration.property);
                if !blocked_names.contains(&declaration.property) {
                    blocked_names.push(declaration.property);
                }
                continue;
            };
            env.insert(declaration.property, value);
        }
    }

    let mut registration_names = Vec::new();
    for registration in registrations {
        if blocked_names.contains(&registration.name) {
            continue;
        }
        if registration_names.contains(&registration.name) {
            env.remove(&registration.name);
            blocked_names.push(registration.name);
            continue;
        }
        registration_names.push(registration.name.clone());
        if env.contains_key(&registration.name) {
            continue;
        }
        let Some(initial_value) = registration.initial_value else {
            continue;
        };
        let Some(value) = parse_static_custom_property_env_value(&initial_value) else {
            continue;
        };
        env.insert(registration.name, value);
    }

    env
}

pub(crate) fn parse_static_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    if contains_runtime_dependent_css_function(value) {
        return None;
    }
    parse_static_var_value(value)
        .or_else(|| parse_static_composite_custom_property_env_value(value))
}

fn contains_runtime_dependent_css_function(value: &str) -> bool {
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if css_function_name_starts_at(value, index, "env")
                || css_function_name_starts_at(value, index, "attr") =>
            {
                return true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    false
}

fn css_function_name_starts_at(value: &str, index: usize, function_name: &str) -> bool {
    let Some(name) = value.get(index..index + function_name.len()) else {
        return false;
    };
    if !name.eq_ignore_ascii_case(function_name) {
        return false;
    }
    if !value[index + function_name.len()..].starts_with('(') {
        return false;
    }
    if index == 0 {
        return true;
    }
    let Some(previous) = value[..index].chars().next_back() else {
        return true;
    };
    !is_css_ident_continue(previous)
}

fn parse_static_composite_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    let mut parts = Vec::new();
    let mut cursor = 0;
    let mut index = 0;
    let mut quote = None;
    let mut found_var = false;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(next_ch) = value[index..].chars().next() {
                    index += next_ch.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index = close_index + ')'.len_utf8();
                    continue;
                };
                let Some(var_value) = parse_static_var_arguments(&arguments) else {
                    index += ch.len_utf8();
                    continue;
                };
                if cursor < index {
                    parts.push(CascadeValue::Literal(value[cursor..index].to_string()));
                }
                parts.push(var_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                found_var = true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !found_var {
        return Some(CascadeValue::Literal(value.to_string()));
    }
    if cursor < value.len() {
        parts.push(CascadeValue::Literal(value[cursor..].to_string()));
    }
    Some(CascadeValue::Composite(parts))
}

fn parse_static_var_value(value: &str) -> Option<CascadeValue> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    parse_static_var_arguments(&arguments)
}

fn parse_static_var_arguments(arguments: &[String]) -> Option<CascadeValue> {
    match arguments {
        [name] if name.starts_with("--") => Some(CascadeValue::Var {
            name: name.clone(),
            fallback: None,
        }),
        [name, fallback] if name.starts_with("--") => {
            let fallback = parse_static_custom_property_env_value(fallback)?;
            Some(CascadeValue::Var {
                name: name.clone(),
                fallback: Some(Box::new(fallback)),
            })
        }
        _ => None,
    }
}

fn render_static_cascade_value(value: &CascadeValue) -> Option<String> {
    match value {
        CascadeValue::Literal(value) => Some(value.clone()),
        CascadeValue::Composite(parts) => {
            let mut output = String::new();
            for part in parts {
                output.push_str(&render_static_cascade_value(part)?);
            }
            Some(output)
        }
        CascadeValue::Var { .. }
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => None,
    }
}
