use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use crate::helpers::tokens::{skip_whitespace_tokens, token_end, token_start};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticCssModulesValueImportStatement {
    pub(crate) local_names: Vec<String>,
    pub(crate) bindings: Vec<StaticCssModulesValueImportBinding>,
    pub(crate) from_clause: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticCssModulesValueImportBinding {
    pub(crate) local_name: String,
    pub(crate) binding_text: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn collect_static_css_modules_value_import_statements(
    tokens: &[LexedToken],
) -> Vec<StaticCssModulesValueImportStatement> {
    let mut statements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@value") =>
            {
                let Some((statement, next_index)) =
                    parse_static_css_modules_value_import_statement(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                statements.push(statement);
                index = next_index;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    statements
}

fn parse_static_css_modules_value_import_statement(
    tokens: &[LexedToken],
    at_value_index: usize,
) -> Option<(StaticCssModulesValueImportStatement, usize)> {
    let mut end_index = at_value_index + 1;
    while end_index < tokens.len() && tokens[end_index].kind != SyntaxKind::Semicolon {
        if matches!(
            tokens[end_index].kind,
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace
        ) {
            return None;
        }
        end_index += 1;
    }
    if end_index >= tokens.len() {
        return None;
    }

    let from_index = (at_value_index + 1..end_index).find(|index| {
        tokens[*index].kind == SyntaxKind::Ident && tokens[*index].text.eq_ignore_ascii_case("from")
    })?;
    let from_clause = tokens[from_index..end_index]
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>()
        .trim()
        .to_string();
    let mut local_names = Vec::new();
    let mut bindings = Vec::new();
    let mut index = at_value_index + 1;

    while index < from_index {
        index = skip_whitespace_tokens(tokens, index, from_index);
        if index >= from_index {
            break;
        }
        if tokens[index].text == "," {
            index += 1;
            continue;
        }
        if tokens[index].kind != SyntaxKind::Ident {
            index += 1;
            continue;
        }

        let binding_start_index = index;
        let mut local_name = tokens[index].text.clone();
        index += 1;
        let mut binding_end_index = index;
        let maybe_as_index = skip_whitespace_tokens(tokens, index, from_index);
        if maybe_as_index < from_index
            && tokens[maybe_as_index].kind == SyntaxKind::Ident
            && tokens[maybe_as_index].text.eq_ignore_ascii_case("as")
        {
            let alias_index = skip_whitespace_tokens(tokens, maybe_as_index + 1, from_index);
            if alias_index < from_index && tokens[alias_index].kind == SyntaxKind::Ident {
                local_name = tokens[alias_index].text.clone();
                index = alias_index + 1;
                binding_end_index = index;
            }
        }
        if !local_names.iter().any(|name| name == &local_name) {
            let binding_text = tokens[binding_start_index..binding_end_index]
                .iter()
                .map(|token| token.text.as_str())
                .collect::<String>()
                .trim()
                .to_string();
            bindings.push(StaticCssModulesValueImportBinding {
                local_name: local_name.clone(),
                binding_text,
                start: token_start(&tokens[binding_start_index]),
                end: token_end(&tokens[binding_end_index - 1]),
            });
            local_names.push(local_name);
        }
    }

    Some((
        StaticCssModulesValueImportStatement {
            local_names,
            bindings,
            from_clause,
            start: token_start(&tokens[at_value_index]),
            end: token_end(&tokens[end_index]),
        },
        end_index + 1,
    ))
}
