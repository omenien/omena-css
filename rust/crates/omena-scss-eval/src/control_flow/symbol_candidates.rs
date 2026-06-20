use omena_parser::{LexedToken, ParsedSassSymbolFact, ParsedSassSymbolFactKind};
use omena_syntax::SyntaxKind;

use crate::abstract_css_value_kind;

use super::{
    analysis_model::{ScssCallLocalBindingScope, ScssCallReturnCandidate},
    arguments::{
        scss_named_value_from_text, split_scss_call_arguments, static_scss_argument_abstract_value,
    },
    model::{
        OmenaScssEvalCallArgumentValueV0, OmenaScssEvalCallLocalBindingV0,
        OmenaScssEvalCallParameterValueV0,
    },
    return_candidates::static_scss_return_abstract_value,
    tokens::{
        declaration_end_token_index, matching_block_end_token_index,
        matching_right_paren_token_index, next_block_start_token_index,
        next_non_trivia_token_index, token_range_end, token_range_start,
    },
    variables::variable_name_end,
};

pub(super) fn call_return_candidate_from_sass_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<ScssCallReturnCandidate> {
    let (kind, symbol_kind, role) = match symbol.kind {
        ParsedSassSymbolFactKind::MixinDeclaration => ("mixinDeclaration", "mixin", "declaration"),
        ParsedSassSymbolFactKind::MixinInclude => ("mixinInclude", "mixin", "call"),
        ParsedSassSymbolFactKind::FunctionDeclaration => {
            ("functionDeclaration", "function", "declaration")
        }
        ParsedSassSymbolFactKind::FunctionCall => ("functionCall", "function", "call"),
        ParsedSassSymbolFactKind::VariableDeclaration
        | ParsedSassSymbolFactKind::VariableReference => return None,
    };
    Some(ScssCallReturnCandidate {
        kind,
        symbol_kind,
        role,
        name: Some(symbol.name.clone()),
        namespace: symbol.namespace.clone(),
        parameter_names: scss_declaration_parameter_names_from_symbol(source, tokens, symbol),
        parameter_values: scss_declaration_parameter_values_from_symbol(source, tokens, symbol),
        local_binding_values: scss_declaration_local_bindings_from_symbol(source, tokens, symbol),
        argument_values: scss_call_argument_values_from_symbol(source, tokens, symbol),
        return_text: None,
        return_value: None,
        body_has_control_flow: scss_declaration_body_has_control_flow(tokens, symbol),
        body_has_loop_control_flow: scss_declaration_body_has_loop_control_flow(tokens, symbol),
        return_inside_loop_control_flow: false,
        return_loop_header_text: None,
        return_loop_header_texts: Vec::new(),
        return_loop_body_texts: Vec::new(),
        return_condition_text: None,
        return_negated_condition_texts: Vec::new(),
        source_span_start: symbol.range.start().into(),
        source_span_end: symbol.range.end().into(),
    })
}

fn scss_call_argument_values_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallArgumentValueV0> {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionCall | ParsedSassSymbolFactKind::MixinInclude
    ) {
        return Vec::new();
    }
    let Some(arguments) = scss_call_argument_texts_from_symbol(source, tokens, symbol) else {
        return Vec::new();
    };
    arguments
        .into_iter()
        .filter_map(|text| scss_call_argument_value_from_text(text.as_str()))
        .collect()
}

pub(super) fn scss_call_argument_value_from_text(
    text: &str,
) -> Option<OmenaScssEvalCallArgumentValueV0> {
    let (name, text) = match scss_named_value_from_text(text)? {
        Some((name, value)) => (Some(name), value),
        None => (None, text.to_string()),
    };
    let value = static_scss_argument_abstract_value(text.as_str());
    Some(OmenaScssEvalCallArgumentValueV0 {
        name,
        value_kind: abstract_css_value_kind(&value),
        text,
        value,
    })
}

fn scss_call_argument_texts_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    let token_index = token_index_for_symbol_range(tokens, symbol)?;
    match symbol.kind {
        ParsedSassSymbolFactKind::FunctionCall => {
            let left_paren_index = next_non_trivia_token_index(tokens, token_index + 1)?;
            if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
                return None;
            }
            let right_paren_index = matching_right_paren_token_index(tokens, left_paren_index)?;
            split_scss_call_arguments(source.get(
                token_range_end(&tokens[left_paren_index])
                    ..token_range_start(&tokens[right_paren_index]),
            )?)
        }
        ParsedSassSymbolFactKind::MixinInclude => {
            let next_index = next_non_trivia_token_index(tokens, token_index + 1)?;
            if tokens.get(next_index)?.kind == SyntaxKind::LeftParen {
                let right_paren_index = matching_right_paren_token_index(tokens, next_index)?;
                return split_scss_call_arguments(source.get(
                    token_range_end(&tokens[next_index])
                        ..token_range_start(&tokens[right_paren_index]),
                )?);
            }
            let argument_start = token_range_end(&tokens[token_index]);
            let argument_end = tokens
                .iter()
                .skip(token_index + 1)
                .find(|candidate| {
                    matches!(
                        candidate.kind,
                        SyntaxKind::Semicolon
                            | SyntaxKind::SassOptionalSemicolon
                            | SyntaxKind::SassIndentedNewline
                            | SyntaxKind::LeftBrace
                            | SyntaxKind::RightBrace
                    )
                })
                .map(token_range_start)
                .unwrap_or(argument_start);
            split_scss_call_arguments(source.get(argument_start..argument_end)?)
        }
        _ => None,
    }
}

fn scss_declaration_parameter_names_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<String> {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionDeclaration | ParsedSassSymbolFactKind::MixinDeclaration
    ) {
        return Vec::new();
    }
    let Some(parameters) = scss_declaration_parameter_texts_from_symbol(source, tokens, symbol)
    else {
        return Vec::new();
    };
    parameters
        .into_iter()
        .filter_map(|parameter| scss_parameter_name_from_text(parameter.as_str()))
        .collect()
}

fn scss_declaration_parameter_values_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallParameterValueV0> {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionDeclaration | ParsedSassSymbolFactKind::MixinDeclaration
    ) {
        return Vec::new();
    }
    let Some(parameters) = scss_declaration_parameter_texts_from_symbol(source, tokens, symbol)
    else {
        return Vec::new();
    };
    parameters
        .into_iter()
        .filter_map(|parameter| scss_parameter_value_from_text(parameter.as_str()))
        .collect()
}

fn scss_parameter_value_from_text(parameter: &str) -> Option<OmenaScssEvalCallParameterValueV0> {
    let name = scss_parameter_name_from_text(parameter)?;
    let default_value_text = scss_named_value_from_text(parameter)
        .flatten()
        .map(|(_, value)| value);
    let default_value = default_value_text
        .as_deref()
        .map(static_scss_argument_abstract_value);
    let default_value_kind = default_value.as_ref().map(abstract_css_value_kind);
    Some(OmenaScssEvalCallParameterValueV0 {
        name,
        default_value_text,
        default_value,
        default_value_kind,
    })
}

fn scss_declaration_local_bindings_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallLocalBindingV0> {
    if !matches!(symbol.kind, ParsedSassSymbolFactKind::FunctionDeclaration) {
        return Vec::new();
    }
    let Some((body_start, body_end)) = scss_declaration_body_token_range(tokens, symbol) else {
        return Vec::new();
    };
    let mut bindings = Vec::new();
    let mut scope_stack = Vec::<ScssCallLocalBindingScope>::new();
    let Some(function_scope_start) = tokens
        .get(body_start)
        .map(token_range_start)
        .or_else(|| tokens.get(body_end).map(token_range_start))
    else {
        return Vec::new();
    };
    let function_scope_end = tokens
        .get(body_end)
        .map(token_range_start)
        .unwrap_or(function_scope_start);
    let mut index = body_start;
    while index < body_end {
        while scope_stack
            .last()
            .is_some_and(|scope| index > scope.end_index)
        {
            scope_stack.pop();
        }
        let Some(token) = tokens.get(index) else {
            break;
        };
        match token.kind {
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent => {
                let Some(scope_end_index) = matching_block_end_token_index(tokens, index) else {
                    index += 1;
                    continue;
                };
                scope_stack.push(ScssCallLocalBindingScope {
                    end_index: scope_end_index,
                    span_start: token_range_end(token),
                    span_end: token_range_start(&tokens[scope_end_index]),
                });
                index += 1;
                continue;
            }
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                if scope_stack
                    .last()
                    .is_some_and(|scope| scope.end_index == index)
                {
                    scope_stack.pop();
                }
                index += 1;
                continue;
            }
            SyntaxKind::ScssVariable => {
                let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
                    index += 1;
                    continue;
                };
                if tokens.get(colon_index).map(|token| token.kind) != Some(SyntaxKind::Colon) {
                    index += 1;
                    continue;
                }
                let Some(end_index) = declaration_end_token_index(tokens, colon_index + 1) else {
                    index += 1;
                    continue;
                };
                if end_index >= body_end {
                    break;
                }
                let value_start = token_range_end(&tokens[colon_index]);
                let value_end = token_range_start(&tokens[end_index]);
                if let Some(value_text) = source.get(value_start..value_end).map(str::trim)
                    && !value_text.is_empty()
                {
                    let value = static_scss_return_abstract_value(value_text);
                    let (scope_span_start, scope_span_end) = scope_stack
                        .last()
                        .map(|scope| (scope.span_start, scope.span_end))
                        .unwrap_or((function_scope_start, function_scope_end));
                    bindings.push(OmenaScssEvalCallLocalBindingV0 {
                        name: token.text.clone(),
                        source_span_start: token.range.start().into(),
                        source_span_end: token.range.end().into(),
                        scope_span_start,
                        scope_span_end,
                        value_text: value_text.to_string(),
                        value_kind: abstract_css_value_kind(&value),
                        value,
                    });
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    bindings
}

fn scss_declaration_body_token_range(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<(usize, usize)> {
    let token_index = token_index_for_symbol_range(tokens, symbol)?;
    let block_start_index = next_block_start_token_index(tokens, token_index + 1)?;
    let block_end_index = matching_block_end_token_index(tokens, block_start_index)?;
    Some((block_start_index + 1, block_end_index))
}

fn scss_declaration_body_has_control_flow(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> bool {
    scss_declaration_body_has_matching_control_flow(tokens, symbol, |name| {
        matches!(name, "@if" | "@else" | "@for" | "@each" | "@while")
    })
}

fn scss_declaration_body_has_loop_control_flow(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> bool {
    scss_declaration_body_has_matching_control_flow(tokens, symbol, |name| {
        matches!(name, "@for" | "@each" | "@while")
    })
}

fn scss_declaration_body_has_matching_control_flow(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
    matches_name: impl Fn(&str) -> bool,
) -> bool {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionDeclaration | ParsedSassSymbolFactKind::MixinDeclaration
    ) {
        return false;
    }
    let Some((body_start, body_end)) = scss_declaration_body_token_range(tokens, symbol) else {
        return false;
    };
    tokens
        .iter()
        .skip(body_start)
        .take(body_end.saturating_sub(body_start))
        .any(|token| {
            token.kind == SyntaxKind::AtKeyword
                && matches_name(token.text.to_ascii_lowercase().as_str())
        })
}

fn scss_declaration_parameter_texts_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    let token_index = token_index_for_symbol_range(tokens, symbol)?;
    let left_paren_index = next_non_trivia_token_index(tokens, token_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return Some(Vec::new());
    }
    let right_paren_index = matching_right_paren_token_index(tokens, left_paren_index)?;
    split_scss_call_arguments(source.get(
        token_range_end(&tokens[left_paren_index])..token_range_start(&tokens[right_paren_index]),
    )?)
}

fn scss_parameter_name_from_text(parameter: &str) -> Option<String> {
    let trimmed = parameter.trim();
    if !trimmed.starts_with('$') || trimmed.contains("...") {
        return None;
    }
    let end = variable_name_end(trimmed, '$'.len_utf8());
    (end > '$'.len_utf8())
        .then(|| trimmed.get(..end).map(ToString::to_string))
        .flatten()
}

fn token_index_for_symbol_range(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<usize> {
    let start: usize = symbol.range.start().into();
    let end: usize = symbol.range.end().into();
    tokens
        .iter()
        .enumerate()
        .find_map(|(index, token)| {
            (token_range_start(token) == start && token_range_end(token) == end).then_some(index)
        })
        .or_else(|| {
            tokens.iter().enumerate().find_map(|(index, token)| {
                (token_range_start(token) <= start
                    && start < token_range_end(token)
                    && token.text.ends_with(symbol.name.as_str()))
                .then_some(index)
            })
        })
        .or_else(|| {
            tokens.iter().enumerate().find_map(|(index, token)| {
                (token_range_start(token) >= start
                    && token_range_end(token) <= end
                    && token.text.ends_with(symbol.name.as_str()))
                .then_some(index)
            })
        })
}
