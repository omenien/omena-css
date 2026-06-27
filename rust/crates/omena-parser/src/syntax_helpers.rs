use cstree::text::TextRange;
use omena_syntax::{StyleDialect, SyntaxKind};

use crate::{
    lex::Token,
    value_names::{
        CSS_COLOR_FUNCTION_NAMES, CSS_FILTER_FUNCTION_NAMES, CSS_GRADIENT_FUNCTION_NAMES,
        CSS_IMAGE_FUNCTION_NAMES, CSS_SHAPE_FUNCTION_NAMES, CSS_TRANSFORM_FUNCTION_NAMES,
        VALUES_L4_MATH_FUNCTION_NAMES,
    },
};

pub(crate) fn is_interpolation_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
            | SyntaxKind::TemplateInterpolationStart
    )
}

pub(crate) fn is_component_value_atom_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Number
            | SyntaxKind::Percentage
            | SyntaxKind::Dimension
            | SyntaxKind::String
            | SyntaxKind::LessEscapedString
            | SyntaxKind::UnicodeRange
            | SyntaxKind::Hash
            | SyntaxKind::Url
            | SyntaxKind::BadUrl
            | SyntaxKind::BadString
            | SyntaxKind::Important
            | SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::LessPropertyVariableToken
            | SyntaxKind::TemplatePlaceholder
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
            | SyntaxKind::TemplateInterpolationStart
    )
}

pub(crate) fn interpolation_end_kind(start_kind: SyntaxKind) -> Option<SyntaxKind> {
    match start_kind {
        SyntaxKind::ScssInterpolationStart => Some(SyntaxKind::ScssInterpolationEnd),
        SyntaxKind::LessInterpolationStart => Some(SyntaxKind::LessInterpolationEnd),
        SyntaxKind::TemplateInterpolationStart => Some(SyntaxKind::TemplateInterpolationEnd),
        _ => None,
    }
}

pub(crate) fn top_level_token_kind_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected: SyntaxKind,
) -> Option<usize> {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if kind == expected && paren_depth == 0 && bracket_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub(crate) fn top_level_token_text_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected: &str,
) -> Option<usize> {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Ident
                if paren_depth == 0
                    && bracket_depth == 0
                    && tokens[index].text.eq_ignore_ascii_case(expected) =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub(crate) fn previous_non_trivia_token_index(
    tokens: &[Token<'_>],
    mut index: usize,
    start: usize,
) -> Option<usize> {
    while index > start {
        index -= 1;
        if !tokens[index].kind.is_trivia() {
            return Some(index);
        }
    }
    None
}

pub(crate) fn containing_at_rule_header_name<'text>(
    tokens: &'text [Token<'text>],
    index: usize,
) -> Option<&'text str> {
    let mut current = index;
    while current > 0 {
        current -= 1;
        let token = tokens.get(current)?;
        if token.kind.is_trivia() {
            continue;
        }
        if matches!(
            token.kind,
            SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::LeftBrace
                | SyntaxKind::RightBrace
                | SyntaxKind::SassIndent
                | SyntaxKind::SassDedent
        ) {
            return None;
        }
        if token.kind == SyntaxKind::AtKeyword {
            return Some(token.text);
        }
    }
    None
}

pub(crate) fn skip_trivia_tokens(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end && tokens[index].kind.is_trivia() {
        index += 1;
    }
    index
}

pub(crate) fn skip_statement(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end {
        match tokens[index].kind {
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return index + 1,
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => return index,
            _ => index += 1,
        }
    }
    index
}

pub(crate) fn find_block_after_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent => return None,
            SyntaxKind::LeftBrace => {
                let close = matching_right_brace(tokens, index, end)?;
                return Some((index, close));
            }
            SyntaxKind::SassIndent => {
                let close = matching_sass_dedent(tokens, index, end)?;
                return Some((index, close));
            }
            _ => index += 1,
        }
    }
    None
}

pub(crate) fn matching_right_brace(tokens: &[Token<'_>], open: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn matching_sass_dedent(tokens: &[Token<'_>], open: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::SassIndent => depth += 1,
            SyntaxKind::SassDedent => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub(crate) fn style_wrapper_at_rule(name: &str) -> bool {
    matches_ignore_ascii_case(
        name,
        &[
            "@media",
            "@supports",
            "@when",
            "@else",
            "@layer",
            "@scope",
            "@container",
            "@starting-style",
            "@if",
            "@else",
            "@for",
            "@each",
            "@while",
            "@at-root",
            "@include",
        ],
    )
}

pub(crate) fn is_selector_combinator_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Tilde
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::DoublePipe
    )
}

pub(crate) fn selector_component_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Dot
            | SyntaxKind::Hash
            | SyntaxKind::Ident
            | SyntaxKind::Star
            | SyntaxKind::Ampersand
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::LeftBracket
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
    )
}

pub(crate) fn namespace_selector_target_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star
    )
}

pub(crate) fn keyframe_selector_token_is_valid(token: Token<'_>) -> bool {
    token.kind == SyntaxKind::Percentage
        || (token.kind == SyntaxKind::Ident
            && (token.text.eq_ignore_ascii_case("from") || token.text.eq_ignore_ascii_case("to")))
}

pub(crate) fn selector_component_can_end(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Hash
            | SyntaxKind::RightBracket
            | SyntaxKind::RightParen
            | SyntaxKind::Star
    )
}

pub(crate) fn next_non_trivia_token<'text>(
    tokens: &'text [Token<'text>],
    mut index: usize,
) -> Option<Token<'text>> {
    while let Some(token) = tokens.get(index).copied() {
        if !token.kind.is_trivia() {
            return Some(token);
        }
        index += 1;
    }
    None
}

pub(crate) fn next_non_trivia_token_until<'text>(
    tokens: &'text [Token<'text>],
    mut index: usize,
    end: usize,
) -> Option<Token<'text>> {
    while index < end {
        let token = tokens.get(index).copied()?;
        if !token.kind.is_trivia() {
            return Some(token);
        }
        index += 1;
    }
    None
}

pub(crate) fn next_non_trivia_token_index_until(
    tokens: &[Token<'_>],
    mut index: usize,
    end: usize,
) -> Option<usize> {
    while index < end {
        let token = tokens.get(index)?;
        if !token.kind.is_trivia() {
            return Some(index);
        }
        index += 1;
    }
    None
}

pub(crate) fn next_non_trivia_token_after_range<'text>(
    tokens: &'text [Token<'text>],
    range: TextRange,
    end: usize,
) -> Option<Token<'text>> {
    let index = token_index_by_range(tokens, range)?;
    next_non_trivia_token_until(tokens, index + 1, end)
}

pub(crate) fn token_index_by_range(tokens: &[Token<'_>], range: TextRange) -> Option<usize> {
    tokens.iter().position(|token| token.range == range)
}

pub(crate) fn matching_right_paren_from_range(
    tokens: &[Token<'_>],
    open_range: TextRange,
    end: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = token_index_by_range(tokens, open_range)?;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub(crate) fn previous_non_trivia_token<'text>(
    tokens: &'text [Token<'text>],
    start: usize,
    index: usize,
) -> Option<Token<'text>> {
    let mut current = index;
    while current > start {
        current -= 1;
        let token = tokens.get(current).copied()?;
        if !token.kind.is_trivia() {
            return Some(token);
        }
    }
    None
}

pub(crate) fn is_selector_boundary(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Comma
            | SyntaxKind::LeftBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
            | SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
    )
}

pub(crate) fn is_selector_boundary_until(kind: SyntaxKind, recovery: &[SyntaxKind]) -> bool {
    is_selector_boundary(kind) || recovery.contains(&kind)
}

pub(crate) fn is_selector_list_pseudo_class(text: &str) -> bool {
    matches!(text, "is" | "where" | "local" | "global")
}

pub(crate) fn is_nth_pseudo_class(text: &str) -> bool {
    matches!(
        text,
        "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type"
    )
}

pub(crate) fn language_tag_token_can_start(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::Ident | SyntaxKind::String)
}

pub(crate) fn selector_item_token_is_recoverable(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::SassIndentedNewline
            | SyntaxKind::Dot
            | SyntaxKind::Comma
            | SyntaxKind::Hash
            | SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::String
            | SyntaxKind::Number
            | SyntaxKind::Percentage
            | SyntaxKind::Dimension
            | SyntaxKind::Star
            | SyntaxKind::Ampersand
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::LeftBracket
            | SyntaxKind::RightBracket
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
            | SyntaxKind::LeftParen
            | SyntaxKind::RightParen
            | SyntaxKind::Equals
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
            | SyntaxKind::Pipe
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Tilde
            | SyntaxKind::KeywordAnd
            | SyntaxKind::KeywordOr
            | SyntaxKind::KeywordNot
    )
}

pub(crate) fn is_at_rule_prelude_boundary(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
    )
}

pub(crate) fn is_statement_end(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
    )
}

pub(crate) fn function_argument_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::RightParen];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

pub(crate) fn bracketed_value_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::RightBracket];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

pub(crate) fn simple_block_recovery(
    close_kind: SyntaxKind,
    recovery: &[SyntaxKind],
) -> Vec<SyntaxKind> {
    let mut kinds = vec![close_kind];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

pub(crate) fn matching_simple_block_close(open_kind: SyntaxKind) -> Option<SyntaxKind> {
    match open_kind {
        SyntaxKind::LeftBrace => Some(SyntaxKind::RightBrace),
        SyntaxKind::LeftBracket => Some(SyntaxKind::RightBracket),
        SyntaxKind::LeftParen => Some(SyntaxKind::RightParen),
        _ => None,
    }
}

pub(crate) fn value_list_item_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::Comma];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

pub(crate) fn comma_separated_component_value_list_item_recovery(
    recovery: &[SyntaxKind],
) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::Comma];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

pub(crate) fn variable_declaration_node_kind(kind: SyntaxKind, has_colon: bool) -> SyntaxKind {
    if has_colon {
        return kind;
    }
    match kind {
        SyntaxKind::ScssVariableDeclaration => SyntaxKind::BogusScssVariable,
        SyntaxKind::LessVariableDeclaration => SyntaxKind::BogusLessVariable,
        _ => kind,
    }
}

pub(crate) fn is_attribute_matcher(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Equals
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
    )
}

pub(crate) fn attribute_name_token_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star
    )
}

pub(crate) fn attribute_name_token_can_continue(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Star
            | SyntaxKind::Pipe
            | SyntaxKind::ColumnCombinator
    )
}

pub(crate) fn attribute_value_token_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::String
            | SyntaxKind::Hash
            | SyntaxKind::Number
            | SyntaxKind::Dimension
    )
}

pub(crate) fn is_combinator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Tilde
            | SyntaxKind::ColumnCombinator
    )
}

const ADDITIVE_LEFT_BINDING_POWER: u8 = 7;
const ADDITIVE_RIGHT_BINDING_POWER: u8 = 8;
const MULTIPLICATIVE_LEFT_BINDING_POWER: u8 = 9;
const MULTIPLICATIVE_RIGHT_BINDING_POWER: u8 = 10;
const LOGICAL_OR_LEFT_BINDING_POWER: u8 = 1;
const LOGICAL_OR_RIGHT_BINDING_POWER: u8 = 2;
const LOGICAL_AND_LEFT_BINDING_POWER: u8 = 3;
const LOGICAL_AND_RIGHT_BINDING_POWER: u8 = 4;
const COMPARISON_LEFT_BINDING_POWER: u8 = 5;
const COMPARISON_RIGHT_BINDING_POWER: u8 = 6;

pub(crate) const UNARY_PREFIX_RIGHT_BINDING_POWER: u8 = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueInfixOperatorBinding {
    pub(crate) left_binding_power: u8,
    pub(crate) right_binding_power: u8,
    pub(crate) token_count: usize,
}

pub(crate) fn infix_binding_power(kind: SyntaxKind) -> Option<(u8, u8)> {
    match kind {
        SyntaxKind::Plus | SyntaxKind::Minus => {
            Some((ADDITIVE_LEFT_BINDING_POWER, ADDITIVE_RIGHT_BINDING_POWER))
        }
        SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent => Some((
            MULTIPLICATIVE_LEFT_BINDING_POWER,
            MULTIPLICATIVE_RIGHT_BINDING_POWER,
        )),
        _ => None,
    }
}

pub(crate) fn dialect_allows_value_logical_operators(dialect: StyleDialect) -> bool {
    matches!(
        dialect,
        StyleDialect::Scss | StyleDialect::Sass | StyleDialect::Less
    )
}

pub(crate) fn value_infix_operator_binding(
    dialect: StyleDialect,
    kind: SyntaxKind,
    current_text: Option<&str>,
    next_kind: Option<SyntaxKind>,
    adjacent_to_next: bool,
) -> Option<ValueInfixOperatorBinding> {
    let (left_binding_power, right_binding_power, token_count) = match infix_binding_power(kind) {
        Some((left, right)) => (left, right, 1),
        None if dialect_allows_value_logical_operators(dialect) => match kind {
            SyntaxKind::KeywordOr | SyntaxKind::ColumnCombinator => (
                LOGICAL_OR_LEFT_BINDING_POWER,
                LOGICAL_OR_RIGHT_BINDING_POWER,
                1,
            ),
            SyntaxKind::Ident
                if current_text.is_some_and(|text| text.eq_ignore_ascii_case("or")) =>
            {
                (
                    LOGICAL_OR_LEFT_BINDING_POWER,
                    LOGICAL_OR_RIGHT_BINDING_POWER,
                    1,
                )
            }
            SyntaxKind::KeywordAnd | SyntaxKind::DoubleAmpersand => (
                LOGICAL_AND_LEFT_BINDING_POWER,
                LOGICAL_AND_RIGHT_BINDING_POWER,
                1,
            ),
            SyntaxKind::Ident
                if current_text.is_some_and(|text| text.eq_ignore_ascii_case("and")) =>
            {
                (
                    LOGICAL_AND_LEFT_BINDING_POWER,
                    LOGICAL_AND_RIGHT_BINDING_POWER,
                    1,
                )
            }
            SyntaxKind::LessThan | SyntaxKind::GreaterThan => (
                COMPARISON_LEFT_BINDING_POWER,
                COMPARISON_RIGHT_BINDING_POWER,
                if adjacent_to_next && next_kind == Some(SyntaxKind::Equals) {
                    2
                } else {
                    1
                },
            ),
            SyntaxKind::Equals if adjacent_to_next && next_kind == Some(SyntaxKind::Equals) => (
                COMPARISON_LEFT_BINDING_POWER,
                COMPARISON_RIGHT_BINDING_POWER,
                2,
            ),
            SyntaxKind::Delim
                if current_text == Some("!")
                    && adjacent_to_next
                    && next_kind == Some(SyntaxKind::Equals) =>
            {
                (
                    COMPARISON_LEFT_BINDING_POWER,
                    COMPARISON_RIGHT_BINDING_POWER,
                    2,
                )
            }
            _ => return None,
        },
        None => return None,
    };

    Some(ValueInfixOperatorBinding {
        left_binding_power,
        right_binding_power,
        token_count,
    })
}

pub(crate) fn specialized_function_kind(text: &str) -> Option<SyntaxKind> {
    if text.eq_ignore_ascii_case("var") {
        return Some(SyntaxKind::VarFunction);
    }
    if text.eq_ignore_ascii_case("calc") {
        return Some(SyntaxKind::CalcFunction);
    }
    if text.eq_ignore_ascii_case("env") {
        return Some(SyntaxKind::EnvFunction);
    }
    if text.eq_ignore_ascii_case("attr") {
        return Some(SyntaxKind::AttrFunction);
    }
    if text.eq_ignore_ascii_case("if") {
        return Some(SyntaxKind::IfFunction);
    }
    if matches_ignore_ascii_case(text, VALUES_L4_MATH_FUNCTION_NAMES) {
        return Some(SyntaxKind::MathFunction);
    }
    if matches_ignore_ascii_case(text, CSS_COLOR_FUNCTION_NAMES) {
        return Some(SyntaxKind::ColorValue);
    }
    if matches_ignore_ascii_case(text, CSS_GRADIENT_FUNCTION_NAMES) {
        return Some(SyntaxKind::GradientFunction);
    }
    if matches_ignore_ascii_case(text, CSS_TRANSFORM_FUNCTION_NAMES) {
        return Some(SyntaxKind::TransformFunction);
    }
    if matches_ignore_ascii_case(text, CSS_FILTER_FUNCTION_NAMES) {
        return Some(SyntaxKind::FilterFunction);
    }
    if matches_ignore_ascii_case(text, CSS_IMAGE_FUNCTION_NAMES) {
        return Some(SyntaxKind::ImageFunction);
    }
    if matches_ignore_ascii_case(text, CSS_SHAPE_FUNCTION_NAMES) {
        return Some(SyntaxKind::ShapeFunction);
    }
    None
}

pub(crate) fn function_argument_count_is_valid(function_name: &str, argument_count: usize) -> bool {
    if function_name.eq_ignore_ascii_case("calc") {
        return argument_count == 1;
    }
    if matches_ignore_ascii_case(function_name, &["min", "max", "hypot"]) {
        return argument_count >= 1;
    }
    if function_name.eq_ignore_ascii_case("clamp") {
        return argument_count == 3;
    }
    if function_name.eq_ignore_ascii_case("round") {
        return (2..=3).contains(&argument_count);
    }
    if function_name.eq_ignore_ascii_case("log") {
        return (1..=2).contains(&argument_count);
    }
    if matches_ignore_ascii_case(function_name, &["mod", "rem", "pow", "atan2"]) {
        return argument_count == 2;
    }
    if matches_ignore_ascii_case(
        function_name,
        &[
            "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "exp", "abs", "sign",
        ],
    ) {
        return argument_count == 1;
    }
    if function_name.eq_ignore_ascii_case("color-mix") {
        return argument_count == 3;
    }
    if function_name.eq_ignore_ascii_case("light-dark") {
        return argument_count == 2;
    }
    if function_name.eq_ignore_ascii_case("contrast-color") {
        return argument_count == 1;
    }
    true
}

pub(crate) fn function_requires_filled_top_level_arguments(function_name: &str) -> bool {
    function_name.eq_ignore_ascii_case("calc")
        || matches_ignore_ascii_case(function_name, VALUES_L4_MATH_FUNCTION_NAMES)
        || matches_ignore_ascii_case(
            function_name,
            &["color-mix", "light-dark", "contrast-color"],
        )
}

pub(crate) fn at_rule_prelude_head_is_custom_property_name(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::CustomPropertyName || is_interpolation_start(kind)
}

pub(crate) fn at_rule_prelude_head_is_custom_ident(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::Ident || is_interpolation_start(kind)
}

pub(crate) fn is_dynamic_function_argument_head(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    )
}

pub(crate) fn is_scss_module_source_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::ScssInterpolationStart
    )
}

pub(crate) fn is_scss_module_namespace_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::Star | SyntaxKind::ScssInterpolationStart
    )
}

pub(crate) fn is_scss_module_visibility_name_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::ScssVariable
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::ScssInterpolationStart
    )
}

pub(crate) fn is_css_module_from_source_token(kind: SyntaxKind, text: &str) -> bool {
    matches!(
        kind,
        SyntaxKind::String
            | SyntaxKind::Url
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    ) || (kind == SyntaxKind::Ident && text == "global")
}

pub(crate) fn is_scss_control_rule_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssControlIf
            | SyntaxKind::ScssControlElse
            | SyntaxKind::ScssControlEach
            | SyntaxKind::ScssControlFor
            | SyntaxKind::ScssControlWhile
    )
}

pub(crate) fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

pub(crate) fn css_module_scope_function_kind(text: &str) -> Option<SyntaxKind> {
    match text {
        "local" => Some(SyntaxKind::CssModuleLocalBlock),
        "global" => Some(SyntaxKind::CssModuleGlobalBlock),
        _ => None,
    }
}
