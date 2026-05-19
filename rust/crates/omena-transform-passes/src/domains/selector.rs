use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    domains::keyframes::is_keyframes_at_keyword,
    helpers::{
        blocks::at_rule_block_indexes,
        rules::{collect_ordinary_rule_selector_slices, first_non_trivia_token_start},
        selectors::split_css_selector_list,
        tokens::{matching_right_brace_index, matching_right_paren_index, token_start},
        values::{matching_function_call_end, split_top_level_value_arguments},
    },
};

pub(crate) fn compress_css_is_where_selectors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let (source, function_mutation_count) =
        compress_css_is_where_functions_with_lexer(source, dialect);
    let (source, list_expansion_mutation_count) =
        expand_specificity_safe_is_selector_lists_with_lexer(&source, dialect);
    let (source, selector_list_mutation_count) =
        dedupe_ordinary_selector_lists_with_lexer(&source, dialect);
    let (source, keyframe_selector_mutation_count) =
        normalize_keyframe_selector_aliases_with_lexer(&source, dialect);

    (
        source,
        function_mutation_count
            + list_expansion_mutation_count
            + selector_list_mutation_count
            + keyframe_selector_mutation_count,
    )
}

fn normalize_keyframe_selector_aliases_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && is_keyframes_at_keyword(&tokens[index].text)
            && let Some((block_start_index, block_end_index)) = at_rule_block_indexes(tokens, index)
        {
            collect_keyframe_selector_alias_replacements(
                source,
                tokens,
                block_start_index,
                block_end_index,
                &mut replacements,
            );
            index = block_end_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    replacements.sort_by_key(|(start, _, _)| *start);
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

fn collect_keyframe_selector_alias_replacements(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    keyframes_block_start_index: usize,
    keyframes_block_end_index: usize,
    replacements: &mut Vec<(usize, usize, String)>,
) {
    let mut frame_prelude_start_index = keyframes_block_start_index + 1;
    let mut index = keyframes_block_start_index + 1;

    while index < keyframes_block_end_index {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let Some(frame_prelude_start) =
                    first_non_trivia_token_start(tokens, frame_prelude_start_index, index)
                else {
                    index += 1;
                    continue;
                };
                let frame_prelude_end = token_start(&tokens[index]);
                let frame_prelude = source[frame_prelude_start..frame_prelude_end].trim();
                if let Some(normalized_frame_prelude) =
                    normalize_keyframe_selector_alias_list(frame_prelude)
                    && normalized_frame_prelude != frame_prelude
                {
                    replacements.push((
                        frame_prelude_start,
                        frame_prelude_end,
                        normalized_frame_prelude,
                    ));
                }

                let Some(close_index) = matching_right_brace_index(tokens, index) else {
                    return;
                };
                index = close_index + 1;
                frame_prelude_start_index = index;
                continue;
            }
            SyntaxKind::Semicolon => {
                frame_prelude_start_index = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
}

fn normalize_keyframe_selector_alias_list(selector_list: &str) -> Option<String> {
    let selectors = split_top_level_value_arguments(selector_list)?;
    let mut changed = false;
    let normalized = selectors
        .into_iter()
        .map(
            |selector| match normalize_keyframe_selector_alias(&selector) {
                Some(normalized_selector) => {
                    changed = true;
                    normalized_selector.to_string()
                }
                None => selector,
            },
        )
        .collect::<Vec<_>>();

    changed.then(|| normalized.join(","))
}

fn normalize_keyframe_selector_alias(selector: &str) -> Option<&'static str> {
    match selector.trim().to_ascii_lowercase().as_str() {
        "from" => Some("0%"),
        "100%" | "to" => Some("to"),
        _ => None,
    }
}

fn compress_css_is_where_functions_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;
    let mut index = 0;

    while index < tokens.len() {
        if let Some((replacement, consumed)) = rewrite_is_where_selector_function(tokens, index) {
            output.push_str(&replacement);
            mutation_count += 1;
            index += consumed;
            continue;
        }

        output.push_str(&tokens[index].text);
        index += 1;
    }

    (output, mutation_count)
}

fn dedupe_ordinary_selector_lists_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_ordinary_rule_selector_slices(source, tokens);
    let mut replacements = Vec::new();

    for rule in rules {
        let Some(selectors) = split_css_selector_list(&rule.selector) else {
            continue;
        };
        let deduped = dedupe_selector_arguments(&selectors);
        if deduped.len() != selectors.len() {
            let separator = if source[rule.start..rule.block_start]
                .chars()
                .last()
                .is_some_and(char::is_whitespace)
            {
                " "
            } else {
                ""
            };
            replacements.push((
                rule.start,
                rule.block_start,
                format!("{}{separator}", deduped.join(", ")),
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

fn expand_specificity_safe_is_selector_lists_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_ordinary_rule_selector_slices(source, tokens);
    let mut replacements = Vec::new();

    for rule in rules {
        let Some(selectors) = split_css_selector_list(&rule.selector) else {
            continue;
        };
        let mut expanded_selectors = Vec::new();
        let mut changed = false;
        for selector in selectors {
            if let Some(expanded) = expand_specificity_safe_is_selector(&selector) {
                expanded_selectors.extend(expanded);
                changed = true;
            } else {
                expanded_selectors.push(selector);
            }
        }
        if !changed {
            continue;
        }
        let deduped = dedupe_selector_arguments(&expanded_selectors);
        let separator = if source[rule.start..rule.block_start]
            .chars()
            .last()
            .is_some_and(char::is_whitespace)
        {
            " "
        } else {
            ""
        };
        replacements.push((
            rule.start,
            rule.block_start,
            format!("{}{separator}", deduped.join(", ")),
        ));
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

fn expand_specificity_safe_is_selector(selector: &str) -> Option<Vec<String>> {
    let selector_lower = selector.to_ascii_lowercase();
    let start = selector_lower.find(":is(")?;
    if selector_lower[start + ":is(".len()..].contains(":is(") {
        return None;
    }
    let left_paren_index = start + ":is".len();
    let close_index = matching_function_call_end(selector, left_paren_index)?;
    if selector_lower[close_index + ')'.len_utf8()..].contains(":is(") {
        return None;
    }

    let inner = selector[left_paren_index + '('.len_utf8()..close_index].trim();
    let arguments = split_css_selector_list(inner)?;
    if arguments.len() < 2
        || !arguments
            .iter()
            .all(|argument| is_simple_class_selector(argument))
    {
        return None;
    }

    let prefix = &selector[..start];
    let suffix = &selector[close_index + ')'.len_utf8()..];
    Some(
        arguments
            .into_iter()
            .map(|argument| format!("{prefix}{argument}{suffix}"))
            .collect(),
    )
}

fn is_simple_class_selector(selector: &str) -> bool {
    let Some(class_name) = selector.trim().strip_prefix('.') else {
        return false;
    };
    !class_name.is_empty()
        && class_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

fn rewrite_is_where_selector_function(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<(String, usize)> {
    let colon = tokens.get(index)?;
    let ident = tokens.get(index + 1)?;
    let left_paren = tokens.get(index + 2)?;
    if colon.kind != SyntaxKind::Colon
        || ident.kind != SyntaxKind::Ident
        || left_paren.kind != SyntaxKind::LeftParen
    {
        return None;
    }

    let pseudo_name = ident.text.to_ascii_lowercase();
    if pseudo_name != "is" && pseudo_name != "where" {
        return None;
    }

    let close_index = matching_right_paren_index(tokens, index + 2)?;
    let inner_tokens = &tokens[index + 3..close_index];
    let mut arguments = split_top_level_selector_arguments(inner_tokens)?;
    if arguments.is_empty() {
        return None;
    }

    if pseudo_name == "is" {
        arguments = flatten_nested_is_selector_arguments(&arguments)?;
    } else {
        arguments = flatten_nested_where_selector_arguments(&arguments)?;
    }

    let deduped = dedupe_selector_arguments(&arguments);
    let replacement = if pseudo_name == "is" {
        if deduped.len() == 1 {
            deduped[0].clone()
        } else if deduped.len() != arguments.len() {
            format!(":is({})", deduped.join(","))
        } else {
            return None;
        }
    } else if deduped.len() != arguments.len() {
        format!(":where({})", deduped.join(","))
    } else {
        return None;
    };

    let original = tokens[index..=close_index]
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>();
    (replacement != original).then_some((replacement, close_index - index + 1))
}

fn flatten_nested_is_selector_arguments(arguments: &[String]) -> Option<Vec<String>> {
    let mut flattened = Vec::new();
    for argument in arguments {
        if let Some(inner_arguments) = parse_exact_selector_function_argument(argument, "is")? {
            flattened.extend(inner_arguments);
        } else {
            flattened.push(argument.clone());
        }
    }
    Some(flattened)
}

fn flatten_nested_where_selector_arguments(arguments: &[String]) -> Option<Vec<String>> {
    let mut flattened = Vec::new();
    for argument in arguments {
        if let Some(inner_arguments) = parse_exact_selector_function_argument(argument, "where")? {
            flattened.extend(inner_arguments);
        } else {
            flattened.push(argument.clone());
        }
    }
    Some(flattened)
}

fn parse_exact_selector_function_argument(
    argument: &str,
    function_name: &str,
) -> Option<Option<Vec<String>>> {
    let trimmed = argument.trim();
    let lexed = lex(trimmed, StyleDialect::Css);
    let tokens = lexed.tokens();
    if tokens.len() < 4 {
        return Some(None);
    }

    let colon = tokens.first()?;
    let ident = tokens.get(1)?;
    let left_paren = tokens.get(2)?;
    if colon.kind != SyntaxKind::Colon
        || ident.kind != SyntaxKind::Ident
        || !ident.text.eq_ignore_ascii_case(function_name)
        || left_paren.kind != SyntaxKind::LeftParen
    {
        return Some(None);
    }

    let close_index = matching_right_paren_index(tokens, 2)?;
    if close_index != tokens.len() - 1 {
        return Some(None);
    }

    split_top_level_selector_arguments(&tokens[3..close_index]).map(Some)
}

fn split_top_level_selector_arguments(tokens: &[omena_parser::LexedToken]) -> Option<Vec<String>> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for token in tokens {
        match token.kind {
            SyntaxKind::LeftParen => {
                paren_depth += 1;
                current.push_str(&token.text);
            }
            SyntaxKind::RightParen => {
                paren_depth = paren_depth.checked_sub(1)?;
                current.push_str(&token.text);
            }
            SyntaxKind::LeftBracket => {
                bracket_depth += 1;
                current.push_str(&token.text);
            }
            SyntaxKind::RightBracket => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                current.push_str(&token.text);
            }
            SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                let argument = current.trim().to_string();
                if argument.is_empty() {
                    return None;
                }
                arguments.push(argument);
                current.clear();
            }
            _ => current.push_str(&token.text),
        }
    }

    let argument = current.trim().to_string();
    if argument.is_empty() {
        return None;
    }
    arguments.push(argument);
    Some(arguments)
}

pub(crate) fn dedupe_selector_arguments(arguments: &[String]) -> Vec<String> {
    let mut deduped = Vec::new();
    for argument in arguments {
        if !deduped.contains(argument) {
            deduped.push(argument.clone());
        }
    }
    deduped
}
