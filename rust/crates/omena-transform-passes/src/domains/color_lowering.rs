use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_value_lattice::parse_relative_color_value;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::color::{
        is_static_color_reference_property, parse_color_function_value, parse_color_mix_value,
        parse_oklab_oklch_value,
    },
    helpers::{
        blocks::rule_block_token_indexes,
        declarations::{
            collect_simple_declarations_in_block, format_replacement_declaration_like_source,
        },
        rules::collect_declaration_ordinary_rule_slices,
        tokens::matching_right_brace_index,
        values::{
            StaticCssFunctionSpec, matching_function_call_end,
            parse_whole_function_value_arguments,
            substitute_static_css_function_references_in_value,
        },
    },
};

pub(crate) fn lower_css_light_dark_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut insertions = Vec::new();

    for rule in &rules {
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        let declarations =
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index);
        for declaration in declarations {
            if !is_static_color_reference_property(&declaration.property) {
                continue;
            }
            let Some((light_value, dark_value)) =
                substitute_light_dark_references_in_value(&declaration.value)
            else {
                continue;
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {light_value};", declaration.property),
            ));
            insertions.push((
                rule.end,
                format!(
                    " @media (prefers-color-scheme: dark) {{ {} {{ {}: {dark_value}; }} }}",
                    rule.selector, declaration.property
                ),
            ));
        }
    }

    if replacements.is_empty() && insertions.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut insertion_index = 0;
    for (start, end, replacement) in &replacements {
        while insertion_index < insertions.len() && insertions[insertion_index].0 <= *start {
            let (position, insertion) = &insertions[insertion_index];
            if *position > cursor {
                output.push_str(&source[cursor..*position]);
                cursor = *position;
            }
            output.push_str(insertion);
            insertion_index += 1;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    while insertion_index < insertions.len() {
        let (position, insertion) = &insertions[insertion_index];
        if *position > cursor {
            output.push_str(&source[cursor..*position]);
            cursor = *position;
        }
        output.push_str(insertion);
        insertion_index += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

pub(crate) fn lower_css_color_mix_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    lower_static_color_function_references_with_lexer(
        source,
        dialect,
        &[("color-mix", parse_color_mix_value)],
        StaticColorLoweringTraversal::NestedBlocks,
    )
}

pub(crate) fn lower_css_oklab_oklch_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    lower_static_color_function_references_with_lexer(
        source,
        dialect,
        &[
            ("oklab", parse_oklab_oklch_value),
            ("oklch", parse_oklab_oklch_value),
        ],
        StaticColorLoweringTraversal::NestedBlocks,
    )
}

pub(crate) fn lower_css_color_function_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    lower_static_color_function_references_with_lexer(
        source,
        dialect,
        &[("color", parse_color_function_value)],
        StaticColorLoweringTraversal::SkipBlock,
    )
}

pub(crate) fn lower_relative_color_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    lower_static_color_function_references_with_lexer(
        source,
        dialect,
        &[
            ("rgb", parse_relative_color_value),
            ("rgba", parse_relative_color_value),
        ],
        StaticColorLoweringTraversal::SkipBlock,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticColorLoweringTraversal {
    NestedBlocks,
    SkipBlock,
}

fn lower_static_color_function_references_with_lexer(
    source: &str,
    dialect: StyleDialect,
    functions: &[StaticCssFunctionSpec<'_>],
    traversal: StaticColorLoweringTraversal,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in declarations {
                if !is_static_color_reference_property(&declaration.property) {
                    continue;
                }
                let Some(replacement_value) = substitute_static_css_function_references_in_value(
                    &declaration.value,
                    functions,
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format_replacement_declaration_like_source(
                        source,
                        &declaration,
                        &replacement_value,
                    ),
                ));
            }
            index = match traversal {
                StaticColorLoweringTraversal::NestedBlocks => index + 1,
                StaticColorLoweringTraversal::SkipBlock => close_index + 1,
            };
            continue;
        }
        index += 1;
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

fn parse_light_dark_value(value: &str) -> Option<(String, String)> {
    let arguments = parse_whole_function_value_arguments(value, "light-dark")?;
    let [light, dark] = arguments.as_slice() else {
        return None;
    };
    if light.is_empty() || dark.is_empty() {
        return None;
    }
    Some((light.clone(), dark.clone()))
}

fn substitute_light_dark_references_in_value(value: &str) -> Option<(String, String)> {
    let mut light_output = String::with_capacity(value.len());
    let mut dark_output = String::with_capacity(value.len());
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
                .get(.."light-dark(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("light-dark(")) =>
            {
                let left_paren_index = index + "light-dark".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let function_value = &value[index..close_index + ')'.len_utf8()];
                let Some((light_value, dark_value)) = parse_light_dark_value(function_value) else {
                    index += ch.len_utf8();
                    continue;
                };
                light_output.push_str(&value[cursor..index]);
                dark_output.push_str(&value[cursor..index]);
                light_output.push_str(&light_value);
                dark_output.push_str(&dark_value);
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
    light_output.push_str(&value[cursor..]);
    dark_output.push_str(&value[cursor..]);
    Some((light_output, dark_output))
}
