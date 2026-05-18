use std::collections::{BTreeMap, VecDeque};

use omena_syntax::SyntaxKind;

use crate::helpers::{
    blocks::at_rule_block_start,
    collections::push_unique_string,
    declarations::collect_simple_declarations_in_block,
    identifiers::normalize_custom_property_name,
    tokens::{matching_right_brace_index, skip_whitespace_tokens, token_end, token_start},
    values::{matching_function_call_end, split_top_level_value_arguments},
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

pub(crate) fn collect_custom_property_references_in_value(value: &str) -> Option<Vec<String>> {
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

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
                let close_index = matching_function_call_end(value, left_paren_index)?;
                let arguments =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])?;
                let [name, fallback @ ..] = arguments.as_slice() else {
                    return None;
                };
                let name = normalize_custom_property_name(name)?;
                push_unique_string(&mut names, name.to_string());
                for fallback_value in fallback {
                    for fallback_name in
                        collect_custom_property_references_in_value(fallback_value)?
                    {
                        push_unique_string(&mut names, fallback_name);
                    }
                }
                index = close_index + ')'.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    Some(names)
}
