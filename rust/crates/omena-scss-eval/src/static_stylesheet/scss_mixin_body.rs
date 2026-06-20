use omena_parser::{ParsedVariableFactKind, StyleDialect};

use super::{
    declarations::extract_static_stylesheet_variable_declaration,
    model::{StaticScssMixinBodyLocalDeclaration, StaticStylesheetVariableKind},
    safety::static_stylesheet_scss_declaration_value_is_removal_safe,
    scss_arguments::static_scss_top_level_colon_index,
    tokens::parser_text_size_to_usize,
    variable_references::static_stylesheet_variable_reference_is_named_argument_label,
};

pub(super) fn collect_static_scss_mixin_body_local_declarations(
    body: &str,
    dialect: StyleDialect,
) -> Option<Vec<StaticScssMixinBodyLocalDeclaration>> {
    let facts = omena_parser::collect_style_facts(body, dialect);
    let mut declarations = Vec::new();
    for fact in facts
        .variables
        .iter()
        .filter(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
    {
        let start = parser_text_size_to_usize(fact.range.start().into());
        let end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(body, start, end) {
            continue;
        }
        let declaration = extract_static_stylesheet_variable_declaration(
            body,
            start,
            end,
            dialect,
            StaticStylesheetVariableKind::Scss,
        )?;
        if !static_stylesheet_scss_declaration_value_is_removal_safe(&declaration.value) {
            return None;
        }
        declarations.push(StaticScssMixinBodyLocalDeclaration {
            name: fact.name.clone(),
            declaration,
        });
    }
    declarations.sort_by_key(|declaration| declaration.declaration.span_start);
    Some(declarations)
}

pub(super) fn collect_static_scss_mixin_body_declaration_value_ranges(
    body: &str,
    dialect: StyleDialect,
) -> Option<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    let mut statement_start = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;

    while index < body.len() {
        let ch = body[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = body[index..].chars().next() {
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
            ';' if paren_depth == 0 && bracket_depth == 0 => {
                collect_static_scss_mixin_body_statement_value_range(
                    body,
                    statement_start,
                    index,
                    &mut ranges,
                )?;
                statement_start = index + ch.len_utf8();
            }
            '\n' if dialect == StyleDialect::Sass && paren_depth == 0 && bracket_depth == 0 => {
                collect_static_scss_mixin_body_statement_value_range(
                    body,
                    statement_start,
                    index,
                    &mut ranges,
                )?;
                statement_start = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    let trailing = body.get(statement_start..)?;
    if trailing.trim().is_empty() {
        return Some(ranges);
    }
    if dialect == StyleDialect::Sass {
        collect_static_scss_mixin_body_statement_value_range(
            body,
            statement_start,
            body.len(),
            &mut ranges,
        )?;
        return Some(ranges);
    }
    None
}

fn collect_static_scss_mixin_body_statement_value_range(
    body: &str,
    statement_start: usize,
    statement_end: usize,
    ranges: &mut Vec<(usize, usize)>,
) -> Option<()> {
    let statement = body.get(statement_start..statement_end)?;
    if statement.trim().is_empty() {
        return Some(());
    }
    let colon_index = static_scss_top_level_colon_index(statement)??;
    let mut value_start = statement_start + colon_index + ':'.len_utf8();
    let mut value_end = statement_end;
    while value_start < value_end {
        let ch = body[value_start..].chars().next()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        value_start += ch.len_utf8();
    }
    while value_start < value_end {
        let ch = body[..value_end].chars().next_back()?;
        if !ch.is_ascii_whitespace() {
            break;
        }
        value_end -= ch.len_utf8();
    }
    if value_start >= value_end {
        return None;
    }
    ranges.push((value_start, value_end));
    Some(())
}
