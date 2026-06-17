use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
pub use omena_value_lattice::reduce_static_numeric_expression;
pub(crate) use omena_value_lattice::{
    compress_number_prefix, format_css_number, numeric_prefix_end, parse_numeric_value_with_unit,
    parse_reducible_abs_value, parse_reducible_calc_value, parse_reducible_clamp_value,
    parse_reducible_exp_value, parse_reducible_hypot_value, parse_reducible_log_value,
    parse_reducible_max_value, parse_reducible_min_value, parse_reducible_mod_value,
    parse_reducible_pow_value, parse_reducible_rem_value, parse_reducible_round_value,
    parse_reducible_sign_value, parse_reducible_sqrt_value,
};

use crate::helpers::source_rewrite::rewrite_lexer_tokens;

pub(crate) fn compress_css_numbers_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    rewrite_lexer_tokens(source, dialect, |kind, text| {
        if matches!(
            kind,
            SyntaxKind::Number | SyntaxKind::Percentage | SyntaxKind::Dimension
        ) {
            return omena_value_lattice::compress_numeric_token_text(text);
        }
        None
    })
}
