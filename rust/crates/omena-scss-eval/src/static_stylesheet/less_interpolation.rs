use std::collections::{BTreeMap, BTreeSet};

use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::{
    StaticLessDetachedRulesetDeclaration, StaticStylesheetEvaluationEdit,
    StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration,
    declarations::static_stylesheet_previous_token_starts_declaration,
    less_strings::static_less_quoted_string_contents,
    less_variables::resolve_static_less_variable_value_in_scope,
    scopes::static_stylesheet_scope_for_position, static_less_variable_name_is_safe,
    static_stylesheet_position_is_inside_ranges, static_stylesheet_property_name_is_safe,
    static_stylesheet_selector_name_part_is_safe, static_stylesheet_skip_trivia_tokens,
    static_stylesheet_token_end, static_stylesheet_token_is_trivia, static_stylesheet_token_start,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_static_less_interpolation_edits(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    mixin_declaration_ranges: &[(usize, usize)],
    detached_ruleset_ranges: &[(usize, usize)],
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let mut edits = Vec::new();
    let declaration_removal_ranges = declarations
        .values()
        .flat_map(|declaration| declaration.removal_spans.iter().copied())
        .collect::<Vec<_>>();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::LessInterpolationStart {
            index += 1;
            continue;
        }
        let interpolation_start = static_stylesheet_token_start(&tokens[index]);
        let ident_index = index + 1;
        let end_index = index + 2;
        let supported_interpolation_shape = tokens
            .get(ident_index)
            .is_some_and(|token| token.kind == SyntaxKind::Ident)
            && tokens
                .get(end_index)
                .is_some_and(|token| token.kind == SyntaxKind::LessInterpolationEnd)
            && !static_stylesheet_position_is_inside_ranges(
                interpolation_start,
                mixin_declaration_ranges,
            )
            && !static_stylesheet_position_is_inside_ranges(
                interpolation_start,
                detached_ruleset_ranges,
            );
        if !supported_interpolation_shape {
            if static_less_position_is_inside_quoted_string(source, interpolation_start) {
                index += 1;
                continue;
            }
            return None;
        }
        let Some(interpolation_kind) = static_less_interpolation_kind(tokens, index) else {
            if static_less_position_is_inside_quoted_string(source, interpolation_start) {
                index = end_index + 1;
                continue;
            }
            return None;
        };
        let reference_scope_id = static_stylesheet_scope_for_position(scopes, interpolation_start)?;
        if interpolation_kind == StaticLessInterpolationKind::DeclarationPropertyName
            && reference_scope_id == 0
        {
            return None;
        }
        let variable_name = format!("@{}", tokens[ident_index].text);
        if !static_less_variable_name_is_safe(variable_name.as_str()) {
            return None;
        }
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_variable_value_in_scope(
            variable_name.as_str(),
            reference_scope_id,
            scopes,
            declarations,
            property_declarations,
            detached_ruleset_declarations,
            &mut stack,
        )?;
        if replacement.escaped || !interpolation_kind.replacement_is_safe(replacement.text.as_str())
        {
            return None;
        }
        edits.push(StaticStylesheetEvaluationEdit {
            start: interpolation_start,
            end: static_stylesheet_token_end(&tokens[end_index]),
            replacement: replacement.text,
        });
        index = end_index + 1;
    }
    edits.extend(collect_static_less_quoted_interpolation_edits(
        source,
        tokens,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
        &declaration_removal_ranges,
        mixin_declaration_ranges,
        detached_ruleset_ranges,
    )?);
    Some(edits)
}

fn static_less_position_is_inside_quoted_string(source: &str, position: usize) -> bool {
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    while index < position && index < source.len() {
        let ch = match source[index..].chars().next() {
            Some(ch) => ch,
            None => return false,
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if source.get(index..index + 2) == Some("/*") {
            let Some(end) = source.get(index + 2..).and_then(|rest| rest.find("*/")) else {
                return false;
            };
            index += end + 4;
            continue;
        }
        if source.get(index..index + 2) == Some("//") {
            index = source
                .get(index + 2..)
                .and_then(|rest| rest.find('\n').map(|offset| index + 2 + offset))
                .unwrap_or(source.len());
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
        }
        index += ch.len_utf8();
    }
    quote.is_some()
}

#[allow(clippy::too_many_arguments)]
fn collect_static_less_quoted_interpolation_edits(
    source: &str,
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    declaration_removal_ranges: &[(usize, usize)],
    mixin_declaration_ranges: &[(usize, usize)],
    detached_ruleset_ranges: &[(usize, usize)],
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let mut edits = Vec::new();
    let mut index = 0usize;

    while index < source.len() {
        if source.get(index..index + 2) == Some("/*") {
            let end = source.get(index + 2..)?.find("*/")?;
            index += end + 4;
            continue;
        }
        if source.get(index..index + 2) == Some("//") {
            index = source
                .get(index + 2..)?
                .find('\n')
                .map(|offset| index + 2 + offset)
                .unwrap_or(source.len());
            continue;
        }

        let ch = source[index..].chars().next()?;
        if !matches!(ch, '"' | '\'') {
            index += ch.len_utf8();
            continue;
        }

        let quote = ch;
        let escaped_string_start =
            (index > 0 && source.get(index - 1..index) == Some("~")).then_some(index - 1);
        let mut escaped_output = String::new();
        let mut escaped_cursor = index + quote.len_utf8();
        let mut escaped_interpolation_count = 0usize;
        let mut cursor = index + quote.len_utf8();
        while cursor < source.len() {
            let quoted_ch = source[cursor..].chars().next()?;
            if matches!(quoted_ch, '\n' | '\r' | '\u{000c}') {
                return None;
            }
            if quoted_ch == '\\' {
                if escaped_string_start.is_some() {
                    return None;
                }
                cursor += quoted_ch.len_utf8();
                let escaped = source[cursor..].chars().next()?;
                if matches!(escaped, '\n' | '\r' | '\u{000c}') {
                    return None;
                }
                cursor += escaped.len_utf8();
                continue;
            }
            if quoted_ch == quote {
                if let Some(escaped_start) = escaped_string_start
                    && escaped_interpolation_count > 0
                {
                    escaped_output.push_str(source.get(escaped_cursor..cursor)?);
                    if !static_less_escaped_interpolation_output_is_safe(escaped_output.as_str()) {
                        return None;
                    }
                    edits.push(StaticStylesheetEvaluationEdit {
                        start: escaped_start,
                        end: cursor + quoted_ch.len_utf8(),
                        replacement: escaped_output,
                    });
                }
                cursor += quoted_ch.len_utf8();
                break;
            }
            if source.get(cursor..cursor + 2) == Some("@{") {
                if static_stylesheet_position_is_inside_ranges(cursor, declaration_removal_ranges)
                    || static_stylesheet_position_is_inside_ranges(cursor, mixin_declaration_ranges)
                    || static_stylesheet_position_is_inside_ranges(cursor, detached_ruleset_ranges)
                {
                    cursor += "@{".len();
                    continue;
                }
                if !static_less_position_is_inside_declaration_value(tokens, cursor) {
                    return None;
                }
                let name_start = cursor + "@{".len();
                let relative_end = source.get(name_start..)?.find('}')?;
                let name_end = name_start + relative_end;
                let name = source.get(name_start..name_end)?;
                if !name
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
                {
                    return None;
                }
                let variable_name = format!("@{name}");
                if !static_less_variable_name_is_safe(variable_name.as_str()) {
                    return None;
                }
                let reference_scope_id = static_stylesheet_scope_for_position(scopes, cursor)?;
                let mut stack = BTreeSet::new();
                let replacement = resolve_static_less_variable_value_in_scope(
                    variable_name.as_str(),
                    reference_scope_id,
                    scopes,
                    declarations,
                    property_declarations,
                    detached_ruleset_declarations,
                    &mut stack,
                )?;
                let replacement =
                    static_less_quoted_interpolation_replacement(replacement.text.as_str(), quote)?;
                let reference_end = name_end + "}".len();
                if escaped_string_start.is_some() {
                    escaped_output.push_str(source.get(escaped_cursor..cursor)?);
                    escaped_output.push_str(replacement.as_str());
                    escaped_cursor = reference_end;
                    escaped_interpolation_count += 1;
                } else {
                    edits.push(StaticStylesheetEvaluationEdit {
                        start: cursor,
                        end: reference_end,
                        replacement,
                    });
                }
                cursor = reference_end;
                continue;
            }
            cursor += quoted_ch.len_utf8();
        }
        index = cursor;
    }

    Some(edits)
}

fn static_less_quoted_interpolation_replacement(value: &str, quote: char) -> Option<String> {
    let trimmed = value.trim();
    let replacement = if let Some(rest) = trimmed.strip_prefix('~') {
        static_less_quoted_string_contents(rest)?
    } else {
        static_less_quoted_string_contents(trimmed).unwrap_or_else(|| trimmed.to_string())
    };
    (!replacement
        .chars()
        .any(|ch| ch == quote || matches!(ch, '\\' | '\n' | '\r' | '\u{000c}')))
    .then_some(replacement)
}

fn static_less_escaped_interpolation_output_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, '{' | '}' | ';' | '\n' | '\r' | '\u{000c}'))
}

fn static_less_position_is_inside_declaration_value(
    tokens: &[LexedToken],
    position: usize,
) -> bool {
    let Some(mut index) = tokens.iter().position(|token| {
        position >= static_stylesheet_token_start(token)
            && position < static_stylesheet_token_end(token)
    }) else {
        return false;
    };
    while index > 0 {
        index -= 1;
        match tokens[index].kind {
            kind if static_stylesheet_token_is_trivia(kind) => {}
            SyntaxKind::Colon => return true,
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace | SyntaxKind::Semicolon => return false,
            _ => {}
        }
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticLessInterpolationKind {
    DeclarationPropertyName,
    SelectorNamePart,
}

impl StaticLessInterpolationKind {
    fn replacement_is_safe(self, replacement: &str) -> bool {
        match self {
            Self::DeclarationPropertyName => static_stylesheet_property_name_is_safe(replacement),
            Self::SelectorNamePart => static_stylesheet_selector_name_part_is_safe(replacement),
        }
    }
}

fn static_less_interpolation_kind(
    tokens: &[LexedToken],
    interpolation_start_index: usize,
) -> Option<StaticLessInterpolationKind> {
    if static_less_interpolation_is_declaration_property_name(tokens, interpolation_start_index) {
        return Some(StaticLessInterpolationKind::DeclarationPropertyName);
    }
    static_less_interpolation_is_selector_name_part(tokens, interpolation_start_index)
        .then_some(StaticLessInterpolationKind::SelectorNamePart)
}

fn static_less_interpolation_is_declaration_property_name(
    tokens: &[LexedToken],
    interpolation_start_index: usize,
) -> bool {
    let interpolation_end_index = interpolation_start_index + 2;
    if !static_stylesheet_previous_token_starts_declaration(tokens, interpolation_start_index) {
        return false;
    }
    let colon_index = static_stylesheet_skip_trivia_tokens(tokens, interpolation_end_index + 1);
    tokens
        .get(colon_index)
        .is_some_and(|token| token.kind == SyntaxKind::Colon)
}

fn static_less_interpolation_is_selector_name_part(
    tokens: &[LexedToken],
    interpolation_start_index: usize,
) -> bool {
    static_less_interpolation_has_selector_name_context(tokens, interpolation_start_index)
        && static_less_interpolation_is_in_selector_header(tokens, interpolation_start_index)
}

fn static_less_interpolation_is_in_selector_header(
    tokens: &[LexedToken],
    interpolation_start_index: usize,
) -> bool {
    let mut index = interpolation_start_index;
    while index > 0 {
        index -= 1;
        match tokens[index].kind {
            kind if static_stylesheet_token_is_trivia(kind) => {}
            SyntaxKind::Colon | SyntaxKind::AtKeyword => return false,
            SyntaxKind::LeftParen
            | SyntaxKind::RightParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::RightBracket => return false,
            SyntaxKind::Semicolon | SyntaxKind::RightBrace | SyntaxKind::LeftBrace => break,
            _ => {}
        }
    }

    let mut index = interpolation_start_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            kind if static_stylesheet_token_is_trivia(kind) => {}
            SyntaxKind::LeftBrace => return true,
            SyntaxKind::LeftParen
            | SyntaxKind::RightParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::RightBracket => return false,
            SyntaxKind::Semicolon | SyntaxKind::RightBrace => return false,
            _ => {}
        }
        index += 1;
    }
    false
}

fn static_less_interpolation_has_selector_name_context(
    tokens: &[LexedToken],
    interpolation_start_index: usize,
) -> bool {
    let mut index = interpolation_start_index;
    while index > 0 {
        index -= 1;
        match tokens[index].kind {
            SyntaxKind::Dot => return true,
            SyntaxKind::Delim if tokens[index].text == "#" => return true,
            kind if static_stylesheet_token_is_trivia(kind) => return true,
            SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Minus
            | SyntaxKind::LessInterpolationStart
            | SyntaxKind::LessInterpolationEnd => {}
            SyntaxKind::Comma
            | SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::Semicolon
            | SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Tilde => return true,
            _ => return false,
        }
    }
    true
}
