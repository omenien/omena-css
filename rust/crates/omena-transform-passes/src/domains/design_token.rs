use omena_parser::{StyleDialect, lex};

use crate::{
    helpers::{
        blocks::at_rule_prelude_end_index,
        declarations::collect_simple_declarations_in_block,
        source_rewrite::replace_source_ranges,
        tokens::{matching_right_brace_index, token_end, token_start},
        values::{
            matching_function_call_end, parse_whole_function_value_arguments,
            split_top_level_value_arguments,
        },
    },
    model::TransformDesignTokenRouteV0,
};

pub(crate) fn route_design_token_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == omena_syntax::SyntaxKind::AtKeyword
            && at_rule_prelude_can_route_design_tokens(&tokens[index].text)
            && let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1)
        {
            let prelude_start = token_end(&tokens[index]);
            let prelude_end = token_start(&tokens[prelude_end_index]);
            if let Some(routed_prelude) = route_design_token_references_in_value(
                &source[prelude_start..prelude_end],
                routes,
                None,
            ) {
                replacements.push((prelude_start, prelude_end, routed_prelude));
            }
        }

        let Some(block_end_index) = (tokens[index].kind == omena_syntax::SyntaxKind::LeftBrace)
            .then(|| matching_right_brace_index(tokens, index))
            .flatten()
        else {
            index += 1;
            continue;
        };
        for declaration in collect_simple_declarations_in_block(tokens, index, block_end_index) {
            let declaration_value = if declaration.important {
                let Some(value) = declaration_value_without_important(&declaration.value) else {
                    continue;
                };
                value
            } else {
                declaration.value.as_str()
            };
            let blocked_token_name = declaration
                .property
                .starts_with("--")
                .then(|| normalize_design_token_name(&declaration.property))
                .flatten();
            let Some(routed_value) = route_design_token_references_in_value(
                declaration_value,
                routes,
                blocked_token_name,
            ) else {
                continue;
            };
            let important = if declaration.important {
                "!important"
            } else {
                ""
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {routed_value}{important};", declaration.property),
            ));
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn at_rule_prelude_can_route_design_tokens(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@container" | "@custom-media" | "@media" | "@supports"
    )
}

fn declaration_value_without_important(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.ends_with("!important") {
        let suffix_start = trimmed.len().saturating_sub("!important".len());
        return Some(trimmed[..suffix_start].trim_end());
    }
    if lower.ends_with("! important") {
        let suffix_start = trimmed.rfind('!')?;
        return Some(trimmed[..suffix_start].trim_end());
    }
    None
}

fn route_design_token_references_in_value(
    value: &str,
    routes: &[TransformDesignTokenRouteV0],
    blocked_token_name: Option<&str>,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

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
                if let Some(routed_value) = routed_design_token_value_for_var_arguments(
                    &arguments,
                    routes,
                    blocked_token_name,
                ) {
                    output.push_str(&value[cursor..index]);
                    output.push_str(&routed_value);
                    index = close_index + ')'.len_utf8();
                    cursor = index;
                    changed = true;
                } else {
                    index += ch.len_utf8();
                }
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

fn routed_design_token_value_for_var_arguments(
    arguments: &[String],
    routes: &[TransformDesignTokenRouteV0],
    blocked_token_name: Option<&str>,
) -> Option<String> {
    let (token_name, fallback_arguments) = arguments.split_first()?;
    let token_name = normalize_design_token_name(token_name)?;
    if blocked_token_name.is_some_and(|blocked| blocked == token_name) {
        return None;
    }
    let routed_value = design_token_routed_value(token_name, routes)?;
    if !fallback_arguments.is_empty()
        && let Some(routed_token_name) = parse_single_custom_property_var_reference(routed_value)
    {
        let fallback = fallback_arguments.join(", ");
        let routed_fallback =
            route_design_token_references_in_value(&fallback, routes, blocked_token_name)
                .unwrap_or(fallback);
        return Some(format!("var({routed_token_name}, {routed_fallback})"));
    }
    Some(routed_value.to_string())
}

fn parse_single_custom_property_var_reference(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    let [name] = arguments.as_slice() else {
        return None;
    };
    Some(normalize_design_token_name(name)?.to_string())
}

fn design_token_routed_value<'a>(
    token_name: &str,
    routes: &'a [TransformDesignTokenRouteV0],
) -> Option<&'a str> {
    routes.iter().find_map(|route| {
        let route_name = normalize_design_token_name(&route.token_name)?;
        let routed_value = route.routed_value.trim();
        if routed_value.is_empty() || routed_value.chars().any(|ch| matches!(ch, ';' | '{' | '}')) {
            return None;
        }
        (route_name == token_name).then_some(routed_value)
    })
}

fn normalize_design_token_name(name: &str) -> Option<&str> {
    let name = name.trim();
    if name.starts_with("--") && name.len() > 2 {
        return Some(name);
    }
    None
}
