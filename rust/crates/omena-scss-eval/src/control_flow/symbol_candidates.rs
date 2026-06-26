use cstree::syntax::SyntaxNode;
use omena_parser::{ParsedSassSymbolFact, ParsedSassSymbolFactKind};
use omena_syntax::SyntaxKind;

use crate::abstract_css_value_kind;

use super::{
    analysis_model::ScssCallReturnCandidate,
    arguments::{
        scss_named_value_from_text, split_scss_call_arguments, static_scss_argument_abstract_value,
    },
    model::{
        OmenaScssEvalCallArgumentValueV0, OmenaScssEvalCallLocalBindingV0,
        OmenaScssEvalCallParameterValueV0,
    },
    return_candidates::static_scss_return_abstract_value,
    variables::variable_name_end,
};

pub(super) fn call_return_candidate_from_sass_symbol_cst(
    source: &str,
    symbol: &ParsedSassSymbolFact,
    root: &SyntaxNode<SyntaxKind>,
) -> Option<ScssCallReturnCandidate> {
    let (kind, symbol_kind, role) = sass_symbol_candidate_shape(symbol)?;
    Some(ScssCallReturnCandidate {
        kind,
        symbol_kind,
        role,
        name: Some(symbol.name.clone()),
        namespace: symbol.namespace.clone(),
        parameter_names: scss_declaration_parameter_names_from_cst(source, root, symbol),
        parameter_values: scss_declaration_parameter_values_from_cst(source, root, symbol),
        local_binding_values: scss_declaration_local_bindings_from_cst(source, root, symbol)
            .unwrap_or_default(),
        argument_values: scss_call_argument_values_from_cst(source, root, symbol),
        return_text: None,
        return_value: None,
        body_has_control_flow: scss_declaration_body_has_control_flow_from_cst(root, symbol, false),
        body_has_loop_control_flow: scss_declaration_body_has_control_flow_from_cst(
            root, symbol, true,
        ),
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

fn sass_symbol_candidate_shape(
    symbol: &ParsedSassSymbolFact,
) -> Option<(&'static str, &'static str, &'static str)> {
    match symbol.kind {
        ParsedSassSymbolFactKind::MixinDeclaration => {
            Some(("mixinDeclaration", "mixin", "declaration"))
        }
        ParsedSassSymbolFactKind::MixinInclude => Some(("mixinInclude", "mixin", "call")),
        ParsedSassSymbolFactKind::FunctionDeclaration => {
            Some(("functionDeclaration", "function", "declaration"))
        }
        ParsedSassSymbolFactKind::FunctionCall => Some(("functionCall", "function", "call")),
        ParsedSassSymbolFactKind::VariableDeclaration
        | ParsedSassSymbolFactKind::VariableReference => None,
    }
}

fn scss_call_argument_values_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallArgumentValueV0> {
    let Some(arguments) = scss_call_argument_texts_from_cst(source, root, symbol) else {
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

fn scss_call_argument_texts_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    match symbol.kind {
        ParsedSassSymbolFactKind::FunctionCall => {
            let node = scss_symbol_node(root, SyntaxKind::FunctionCall, symbol)?;
            cst_parenthesized_argument_texts_after_symbol(source, node, symbol)
        }
        ParsedSassSymbolFactKind::MixinInclude => {
            let node = scss_symbol_node(root, SyntaxKind::ScssIncludeRule, symbol)?;
            cst_parenthesized_argument_texts_after_symbol(source, node, symbol)
                .or_else(|| cst_bare_mixin_argument_texts_after_symbol(source, node, symbol))
        }
        _ => None,
    }
}

fn scss_declaration_parameter_texts_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    let node = scss_symbol_declaration_node(root, symbol)?;
    cst_parenthesized_argument_texts_after_symbol(source, node, symbol).or(Some(Vec::new()))
}

fn scss_symbol_declaration_node<'a>(
    root: &'a SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Option<&'a SyntaxNode<SyntaxKind>> {
    let kind = match symbol.kind {
        ParsedSassSymbolFactKind::MixinDeclaration => SyntaxKind::ScssMixinDeclaration,
        ParsedSassSymbolFactKind::FunctionDeclaration => SyntaxKind::ScssFunctionDeclaration,
        _ => return None,
    };
    scss_symbol_node(root, kind, symbol)
}

fn scss_symbol_node<'a>(
    root: &'a SyntaxNode<SyntaxKind>,
    kind: SyntaxKind,
    symbol: &ParsedSassSymbolFact,
) -> Option<&'a SyntaxNode<SyntaxKind>> {
    let symbol_start = u32::from(symbol.range.start()) as usize;
    let symbol_end = u32::from(symbol.range.end()) as usize;
    root.descendants()
        .filter(|node| node.kind() == kind)
        .find(|node| {
            let start = u32::from(node.text_range().start()) as usize;
            let end = u32::from(node.text_range().end()) as usize;
            start <= symbol_start && symbol_end <= end
        })
}

fn cst_parenthesized_argument_texts_after_symbol(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    let symbol_end = u32::from(symbol.range.end()) as usize;
    let tokens = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token());
    let mut paren_depth = 0usize;
    let mut argument_start = None;
    for token in tokens {
        let token_start = u32::from(token.text_range().start()) as usize;
        if token_start < symbol_end {
            continue;
        }
        match token.kind() {
            SyntaxKind::LeftParen if paren_depth == 0 => {
                paren_depth = 1;
                argument_start = Some(u32::from(token.text_range().end()) as usize);
            }
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 {
                    let start = argument_start?;
                    let end = token_start;
                    return split_scss_call_arguments(source.get(start..end)?);
                }
            }
            _ => {}
        }
    }
    None
}

fn cst_bare_mixin_argument_texts_after_symbol(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    let argument_start = u32::from(symbol.range.end()) as usize;
    let argument_end = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| {
            let token_start = u32::from(token.text_range().start()) as usize;
            token_start >= argument_start
                && matches!(
                    token.kind(),
                    SyntaxKind::Semicolon
                        | SyntaxKind::SassOptionalSemicolon
                        | SyntaxKind::SassIndentedNewline
                        | SyntaxKind::LeftBrace
                        | SyntaxKind::RightBrace
                )
        })
        .map(|token| u32::from(token.text_range().start()) as usize)
        .unwrap_or_else(|| u32::from(node.text_range().end()) as usize);
    split_scss_call_arguments(source.get(argument_start..argument_end)?)
}

fn scss_declaration_parameter_names_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Vec<String> {
    let Some(parameters) = scss_declaration_parameter_texts_from_cst(source, root, symbol) else {
        return Vec::new();
    };
    parameters
        .into_iter()
        .filter_map(|parameter| scss_parameter_name_from_text(parameter.as_str()))
        .collect()
}

fn scss_declaration_parameter_values_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallParameterValueV0> {
    let Some(parameters) = scss_declaration_parameter_texts_from_cst(source, root, symbol) else {
        return Vec::new();
    };
    parameters
        .into_iter()
        .filter_map(|parameter| scss_parameter_value_from_text(parameter.as_str()))
        .collect()
}

pub(super) fn scss_parameter_value_from_text(
    parameter: &str,
) -> Option<OmenaScssEvalCallParameterValueV0> {
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

fn scss_declaration_local_bindings_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<OmenaScssEvalCallLocalBindingV0>> {
    if !matches!(symbol.kind, ParsedSassSymbolFactKind::FunctionDeclaration) {
        return Some(Vec::new());
    }
    let declaration = scss_symbol_declaration_node(root, symbol)?;
    let declaration_start = u32::from(declaration.text_range().start()) as usize;
    let declaration_end = u32::from(declaration.text_range().end()) as usize;
    let mut bindings = Vec::new();
    for variable in declaration
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::ScssVariableDeclaration)
    {
        let Some((name, name_start, name_end, value_text)) =
            cst_variable_declaration_parts(source, variable)
        else {
            continue;
        };
        if name_start < declaration_start || name_end > declaration_end {
            continue;
        }
        if value_text.is_empty() {
            continue;
        }
        let value = static_scss_return_abstract_value(value_text.as_str());
        let (scope_span_start, scope_span_end) =
            cst_local_binding_scope_span(variable).unwrap_or((declaration_start, declaration_end));
        bindings.push(OmenaScssEvalCallLocalBindingV0 {
            name,
            source_span_start: name_start,
            source_span_end: name_end,
            scope_span_start,
            scope_span_end,
            value_text,
            value_kind: abstract_css_value_kind(&value),
            value,
        });
    }
    Some(bindings)
}

pub(super) fn cst_variable_declaration_parts(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
) -> Option<(String, usize, usize, String)> {
    let tokens = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .collect::<Vec<_>>();
    let (name_index, name_token) = tokens
        .iter()
        .enumerate()
        .find(|(_, token)| token.kind() == SyntaxKind::ScssVariable)?;
    let colon_index = tokens
        .iter()
        .enumerate()
        .skip(name_index + 1)
        .find(|(_, token)| token.kind() == SyntaxKind::Colon)
        .map(|(index, _)| index)?;
    let value_start = u32::from(tokens[colon_index].text_range().end()) as usize;
    let value_end = tokens
        .iter()
        .skip(colon_index + 1)
        .find(|token| cst_variable_declaration_value_delimiter(token.kind()))
        .map(|token| u32::from(token.text_range().start()) as usize)
        .unwrap_or_else(|| u32::from(node.text_range().end()) as usize);
    let value_text = source.get(value_start..value_end)?.trim().to_string();
    let name_start = u32::from(name_token.text_range().start()) as usize;
    let name_end = u32::from(name_token.text_range().end()) as usize;
    Some((
        source.get(name_start..name_end)?.to_string(),
        name_start,
        name_end,
        value_text,
    ))
}

fn cst_variable_declaration_value_delimiter(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::SassIndentedNewline
            | SyntaxKind::SassDedent
            | SyntaxKind::RightBrace
    )
}

fn cst_local_binding_scope_span(node: &SyntaxNode<SyntaxKind>) -> Option<(usize, usize)> {
    node.ancestors()
        .skip(1)
        .find(|ancestor| cst_control_scope_kind(ancestor.kind()))
        .map(|ancestor| {
            (
                u32::from(ancestor.text_range().start()) as usize,
                u32::from(ancestor.text_range().end()) as usize,
            )
        })
}

fn cst_control_scope_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssControlIf
            | SyntaxKind::ScssControlElse
            | SyntaxKind::ScssControlFor
            | SyntaxKind::ScssControlEach
            | SyntaxKind::ScssControlWhile
    )
}

fn scss_declaration_body_has_control_flow_from_cst(
    root: &SyntaxNode<SyntaxKind>,
    symbol: &ParsedSassSymbolFact,
    loop_only: bool,
) -> bool {
    scss_symbol_declaration_node(root, symbol)
        .map(|node| cst_declaration_body_has_control_flow(node, loop_only))
        .unwrap_or(false)
}

fn cst_declaration_body_has_control_flow(node: &SyntaxNode<SyntaxKind>, loop_only: bool) -> bool {
    node.descendants().any(|candidate| match candidate.kind() {
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile => {
            true
        }
        SyntaxKind::ScssControlIf | SyntaxKind::ScssControlElse => !loop_only,
        _ => false,
    })
}

pub(super) fn scss_parameter_name_from_text(parameter: &str) -> Option<String> {
    let trimmed = parameter.trim();
    if !trimmed.starts_with('$') || trimmed.contains("...") {
        return None;
    }
    let end = variable_name_end(trimmed, '$'.len_utf8());
    (end > '$'.len_utf8())
        .then(|| trimmed.get(..end).map(ToString::to_string))
        .flatten()
}
