use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    domains::number::{
        parse_reducible_abs_value, parse_reducible_calc_value, parse_reducible_clamp_value,
        parse_reducible_hypot_value, parse_reducible_max_value, parse_reducible_min_value,
        parse_reducible_mod_value, parse_reducible_rem_value, parse_reducible_round_value,
        parse_reducible_sign_value,
    },
    helpers::{
        declarations::{
            collect_simple_declarations_in_block, format_replacement_declaration_like_source,
        },
        source_rewrite::replace_source_ranges,
        tokens::matching_right_brace_index,
        values::substitute_static_css_function_references_in_value_until_stable,
    },
};

pub(crate) fn reduce_css_calc_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
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
                let Some(replacement_value) =
                    substitute_static_css_function_references_in_value_until_stable(
                        &declaration.value,
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
                        ],
                    )
                else {
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
            index += 1;
            continue;
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}
