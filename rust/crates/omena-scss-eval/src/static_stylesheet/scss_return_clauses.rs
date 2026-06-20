use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::{
    StaticScssFunctionReturnClause, StaticScssLoopHeader,
    static_stylesheet_composite_value_is_safe, static_stylesheet_matching_token_index,
    static_stylesheet_token_end, static_stylesheet_token_start,
    static_stylesheet_value_end_token_until,
};

pub(super) fn collect_static_scss_function_return_clauses(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionReturnClause>> {
    let clauses = collect_static_scss_function_return_clauses_in_range(
        source,
        tokens,
        start,
        end,
        &Vec::new(),
    )?;
    (!clauses.is_empty()).then_some(clauses)
}

fn collect_static_scss_function_return_clauses_in_range(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
    loop_headers: &[StaticScssLoopHeader],
) -> Option<Vec<StaticScssFunctionReturnClause>> {
    let mut clauses = Vec::new();
    let mut branch_conditions = Vec::<String>::new();
    let mut index = start;
    while index < end {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword {
            index += 1;
            continue;
        }
        if token.text.eq_ignore_ascii_case("@return") {
            let value_end_index = static_stylesheet_value_end_token_until(tokens, index + 1, end)?;
            let value = static_scss_return_value_text(source, tokens, index, value_end_index)?;
            clauses.push(StaticScssFunctionReturnClause {
                condition: None,
                value,
                span_start: static_stylesheet_token_start(token),
                loop_headers: loop_headers.to_vec(),
            });
            index = value_end_index + 1;
            branch_conditions.clear();
            continue;
        }
        if token.text.eq_ignore_ascii_case("@if") {
            let (condition, body_open_index, body_close_index) =
                static_scss_control_block_header_and_body(source, tokens, index, end)?;
            let return_clauses = collect_static_scss_function_return_clauses_in_range(
                source,
                tokens,
                body_open_index + 1,
                body_close_index,
                loop_headers,
            )?;
            clauses.extend(
                return_clauses
                    .into_iter()
                    .map(|return_clause| {
                        static_scss_return_clause_with_condition(return_clause, condition.as_str())
                    })
                    .collect::<Vec<_>>(),
            );
            branch_conditions.clear();
            branch_conditions.push(condition);
            index = body_close_index + 1;
            continue;
        }
        if token.text.eq_ignore_ascii_case("@else") {
            let (condition, body_open_index, body_close_index) =
                static_scss_control_block_header_and_body(source, tokens, index, end)?;
            let return_clauses = collect_static_scss_function_return_clauses_in_range(
                source,
                tokens,
                body_open_index + 1,
                body_close_index,
                loop_headers,
            )?;
            let branch_condition = if let Some(else_if_condition) =
                static_scss_else_if_condition(condition.as_str())
            {
                static_scss_branch_chain_condition(branch_conditions.as_slice(), else_if_condition)
            } else {
                static_scss_branch_chain_else_condition(branch_conditions.as_slice())?
            };
            clauses.extend(
                return_clauses
                    .into_iter()
                    .map(|return_clause| {
                        static_scss_return_clause_with_condition(
                            return_clause,
                            branch_condition.as_str(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            if let Some(else_if_condition) = static_scss_else_if_condition(condition.as_str()) {
                branch_conditions.push(else_if_condition.to_string());
            } else {
                branch_conditions.clear();
            }
            index = body_close_index + 1;
            continue;
        }
        if static_scss_loop_at_keyword(token.text.as_str()).is_some() {
            let (header, body_open_index, body_close_index) =
                static_scss_control_block_header_and_body(source, tokens, index, end)?;
            let mut nested_loop_headers = loop_headers.to_vec();
            nested_loop_headers.push(StaticScssLoopHeader {
                text: format!("{} {}", token.text.trim(), header.trim()),
                span_start: static_stylesheet_token_start(token),
                body_start: static_stylesheet_token_end(&tokens[body_open_index]),
                body_end: static_stylesheet_token_start(&tokens[body_close_index]),
            });
            clauses.extend(collect_static_scss_function_return_clauses_in_range(
                source,
                tokens,
                body_open_index + 1,
                body_close_index,
                nested_loop_headers.as_slice(),
            )?);
            branch_conditions.clear();
            index = body_close_index + 1;
            continue;
        }
        index += 1;
    }
    Some(clauses)
}

pub(super) fn static_scss_function_return_clauses_are_safe(
    clauses: &[StaticScssFunctionReturnClause],
) -> bool {
    !clauses.is_empty()
        && clauses.iter().all(|clause| {
            static_stylesheet_composite_value_is_safe(clause.value.as_str())
                && clause
                    .condition
                    .as_deref()
                    .is_none_or(static_stylesheet_composite_value_is_safe)
                && clause
                    .loop_headers
                    .iter()
                    .all(|header| static_stylesheet_composite_value_is_safe(header.text.as_str()))
        })
}

fn static_scss_loop_at_keyword(keyword: &str) -> Option<&'static str> {
    if keyword.eq_ignore_ascii_case("@for") {
        Some("@for")
    } else if keyword.eq_ignore_ascii_case("@each") {
        Some("@each")
    } else if keyword.eq_ignore_ascii_case("@while") {
        Some("@while")
    } else {
        None
    }
}

fn static_scss_return_clause_with_condition(
    mut clause: StaticScssFunctionReturnClause,
    condition: &str,
) -> StaticScssFunctionReturnClause {
    clause.condition = Some(match clause.condition {
        Some(inner_condition) => format!("({condition}) and ({inner_condition})"),
        None => condition.to_string(),
    });
    clause
}

fn static_scss_return_value_text(
    source: &str,
    tokens: &[LexedToken],
    return_index: usize,
    value_end_index: usize,
) -> Option<String> {
    let value_start = static_stylesheet_token_end(&tokens[return_index]);
    let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
    let value = source.get(value_start..value_end)?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn static_scss_control_block_header_and_body(
    source: &str,
    tokens: &[LexedToken],
    control_index: usize,
    end: usize,
) -> Option<(String, usize, usize)> {
    let body_open_index = (control_index + 1..end).find(|index| {
        matches!(
            tokens[*index].kind,
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent
        )
    })?;
    let (body_open_kind, body_close_kind) =
        if tokens[body_open_index].kind == SyntaxKind::SassIndent {
            (SyntaxKind::SassIndent, SyntaxKind::SassDedent)
        } else {
            (SyntaxKind::LeftBrace, SyntaxKind::RightBrace)
        };
    let body_close_index = static_stylesheet_matching_token_index(
        tokens,
        body_open_index,
        body_open_kind,
        body_close_kind,
    )?;
    if body_close_index >= end {
        return None;
    }
    let header_start = static_stylesheet_token_end(&tokens[control_index]);
    let header_end = static_stylesheet_token_start(&tokens[body_open_index]);
    let header = source.get(header_start..header_end)?.trim().to_string();
    Some((header, body_open_index, body_close_index))
}

fn static_scss_else_if_condition(header: &str) -> Option<&str> {
    let trimmed = header.trim();
    let prefix = trimmed.get(..2)?;
    let rest = trimmed.get(2..)?;
    if !prefix.eq_ignore_ascii_case("if") || !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    Some(rest.trim()).filter(|condition| !condition.is_empty())
}

fn static_scss_branch_chain_condition(previous: &[String], current: &str) -> String {
    previous
        .iter()
        .map(|condition| format!("not ({condition})"))
        .chain(std::iter::once(current.to_string()))
        .collect::<Vec<_>>()
        .join(" and ")
}

fn static_scss_branch_chain_else_condition(previous: &[String]) -> Option<String> {
    (!previous.is_empty()).then(|| {
        previous
            .iter()
            .map(|condition| format!("not ({condition})"))
            .collect::<Vec<_>>()
            .join(" and ")
    })
}
