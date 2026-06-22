use omena_abstract_value::{RegisteredPropertySyntaxV0, parse_registered_property_syntax_v0};
use omena_cascade::{
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, StaticSupportsEvalWitnessV0,
    evaluate_static_supports_condition,
};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionSurfaceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub function_count: usize,
    pub parameter_count: usize,
    pub typed_parameter_count: usize,
    pub supported_parameter_syntax_count: usize,
    pub unsupported_parameter_syntax_count: usize,
    pub result_count: usize,
    pub functions: Vec<OmenaScssEvalNativeCssFunctionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionV0 {
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub name_span_start: usize,
    pub name_span_end: usize,
    pub return_syntax_source: Option<String>,
    pub return_syntax: Option<RegisteredPropertySyntaxV0>,
    pub parameter_count: usize,
    pub typed_parameter_count: usize,
    pub result_count: usize,
    pub parameters: Vec<OmenaScssEvalNativeCssFunctionParameterV0>,
    pub results: Vec<OmenaScssEvalNativeCssFunctionResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionParameterV0 {
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub name_span_start: usize,
    pub name_span_end: usize,
    pub syntax_source: Option<String>,
    pub syntax: Option<RegisteredPropertySyntaxV0>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssFunctionResultV0 {
    pub value: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub function_count: usize,
    pub foldable_function_count: usize,
    pub preserved_function_count: usize,
    pub static_supports_branch_count: usize,
    pub runtime_branch_count: usize,
    pub functions: Vec<OmenaScssEvalNativeCssIfFunctionDecisionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssIfFunctionDecisionV0 {
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub branch_count: usize,
    pub decision: &'static str,
    pub reason: &'static str,
    pub selected_branch_index: Option<usize>,
    pub selected_value: Option<String>,
    pub branches: Vec<OmenaScssEvalNativeCssIfFunctionBranchV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalNativeCssIfFunctionBranchV0 {
    pub branch_index: usize,
    pub condition: String,
    pub condition_kind: &'static str,
    pub verdict: &'static str,
    pub value: Option<String>,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub supports_witness: Option<StaticSupportsEvalWitnessV0>,
}

pub fn summarize_native_css_function_surface(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalNativeCssFunctionSurfaceV0> {
    if dialect != StyleDialect::Css {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let functions = collect_native_css_functions(source, tokens);
    let parameter_count = functions
        .iter()
        .map(|function| function.parameter_count)
        .sum();
    let typed_parameter_count = functions
        .iter()
        .map(|function| function.typed_parameter_count)
        .sum();
    let supported_parameter_syntax_count = functions
        .iter()
        .flat_map(|function| function.parameters.iter())
        .filter(|parameter| parameter_syntax_is_supported(parameter.syntax.as_ref()))
        .count();
    let unsupported_parameter_syntax_count = functions
        .iter()
        .flat_map(|function| function.parameters.iter())
        .filter(|parameter| {
            matches!(
                parameter.syntax,
                Some(RegisteredPropertySyntaxV0::Unsupported { .. })
            )
        })
        .count();
    let result_count = functions.iter().map(|function| function.result_count).sum();

    Some(OmenaScssEvalNativeCssFunctionSurfaceV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-function-surface",
        mode: "recognitionOnly",
        dialect: "css",
        function_count: functions.len(),
        parameter_count,
        typed_parameter_count,
        supported_parameter_syntax_count,
        unsupported_parameter_syntax_count,
        result_count,
        functions,
    })
}

pub fn summarize_native_css_if_function_decisions(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0> {
    if dialect != StyleDialect::Css {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let functions = collect_native_css_if_function_decisions(source, tokens);
    let function_count = functions.len();
    let foldable_function_count = functions
        .iter()
        .filter(|function| function.decision == "foldToStaticValue")
        .count();
    let preserved_function_count = function_count.saturating_sub(foldable_function_count);
    let static_supports_branch_count = functions
        .iter()
        .flat_map(|function| function.branches.iter())
        .filter(|branch| branch.condition_kind == "supports")
        .count();
    let runtime_branch_count = functions
        .iter()
        .flat_map(|function| function.branches.iter())
        .filter(|branch| matches!(branch.condition_kind, "media" | "style"))
        .count();

    Some(OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0 {
        schema_version: "0",
        product: "omena-scss-eval.native-css-if-function-decisions",
        mode: "oracleOnlyPruneButKeep",
        dialect: "css",
        function_count,
        foldable_function_count,
        preserved_function_count,
        static_supports_branch_count,
        runtime_branch_count,
        functions,
    })
}

fn collect_native_css_functions(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<OmenaScssEvalNativeCssFunctionV0> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@function"))
                .then(|| collect_native_css_function(source, tokens, index))
                .flatten()
        })
        .collect()
}

fn collect_native_css_if_function_decisions(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<OmenaScssEvalNativeCssIfFunctionDecisionV0> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind == SyntaxKind::Ident && token.text.eq_ignore_ascii_case("if"))
                .then(|| collect_native_css_if_function_decision(source, tokens, index))
                .flatten()
        })
        .collect()
}

fn collect_native_css_if_function_decision(
    source: &str,
    tokens: &[LexedToken],
    if_index: usize,
) -> Option<OmenaScssEvalNativeCssIfFunctionDecisionV0> {
    let left_paren_index = next_non_trivia_token_index(tokens, if_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    let right_paren_index = matching_token_index(
        tokens,
        left_paren_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let branches = collect_native_css_if_function_branches(
        source,
        tokens,
        left_paren_index + 1,
        right_paren_index,
    );
    if branches.is_empty() {
        return None;
    }
    let (decision, reason, selected_branch_index, selected_value) =
        decide_native_css_if_function(&branches);

    Some(OmenaScssEvalNativeCssIfFunctionDecisionV0 {
        source_span_start: token_start(tokens.get(if_index)?),
        source_span_end: token_end(tokens.get(right_paren_index)?),
        branch_count: branches.len(),
        decision,
        reason,
        selected_branch_index,
        selected_value,
        branches,
    })
}

fn collect_native_css_if_function_branches(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Vec<OmenaScssEvalNativeCssIfFunctionBranchV0> {
    split_top_level_ranges(tokens, start_index, end_index, SyntaxKind::Semicolon)
        .into_iter()
        .enumerate()
        .filter_map(|(branch_index, (start, end))| {
            collect_native_css_if_function_branch(source, tokens, branch_index, start, end)
        })
        .collect()
}

fn collect_native_css_if_function_branch(
    source: &str,
    tokens: &[LexedToken],
    branch_index: usize,
    start_index: usize,
    end_index: usize,
) -> Option<OmenaScssEvalNativeCssIfFunctionBranchV0> {
    let condition_end = first_top_level_token_index(tokens, start_index, end_index, |token| {
        token.kind == SyntaxKind::Colon
    })?;
    let condition = trimmed_source_between_tokens(source, tokens, start_index, condition_end)?;
    let value = trimmed_source_between_tokens(source, tokens, condition_end + 1, end_index);
    let (condition_kind, verdict, supports_witness) =
        classify_native_css_if_function_condition(&condition);
    let source_span_start = first_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_start)?;
    let source_span_end = previous_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_end)
        .unwrap_or(source_span_start);

    Some(OmenaScssEvalNativeCssIfFunctionBranchV0 {
        branch_index,
        condition,
        condition_kind,
        verdict,
        value,
        source_span_start,
        source_span_end,
        supports_witness,
    })
}

fn classify_native_css_if_function_condition(
    condition: &str,
) -> (
    &'static str,
    &'static str,
    Option<StaticSupportsEvalWitnessV0>,
) {
    if condition.trim().eq_ignore_ascii_case("else") {
        return ("else", "else", None);
    }
    if let Some(inner) = extract_named_function_inner(condition, "supports") {
        let normalized_condition = normalize_supports_condition_for_if(inner);
        let witness = evaluate_static_supports_condition(
            &normalized_condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        let verdict = static_supports_verdict_label(witness.verdict);
        return ("supports", verdict, Some(witness));
    }
    if extract_named_function_inner(condition, "media").is_some() {
        return ("media", "runtime", None);
    }
    if extract_named_function_inner(condition, "style").is_some() {
        return ("style", "runtime", None);
    }

    ("unknown", "unknown", None)
}

fn decide_native_css_if_function(
    branches: &[OmenaScssEvalNativeCssIfFunctionBranchV0],
) -> (&'static str, &'static str, Option<usize>, Option<String>) {
    for branch in branches {
        match branch.verdict {
            "alwaysFalse" => continue,
            "alwaysTrue" | "else" => {
                let Some(value) = branch.value.as_deref() else {
                    return (
                        "preserveVerbatim",
                        "selected branch has an empty token stream",
                        None,
                        None,
                    );
                };
                if native_css_if_value_is_fully_static(value) {
                    return (
                        "foldToStaticValue",
                        "all earlier branches are statically false and the selected value is static",
                        Some(branch.branch_index),
                        Some(value.to_string()),
                    );
                }
                return (
                    "preserveVerbatim",
                    "selected branch value depends on runtime or cascade state",
                    None,
                    None,
                );
            }
            "runtime" => {
                return (
                    "preserveVerbatim",
                    "encountered runtime condition before a static winner",
                    None,
                    None,
                );
            }
            _ => {
                return (
                    "preserveVerbatim",
                    "encountered unknown condition before a static winner",
                    None,
                    None,
                );
            }
        }
    }

    (
        "preserveVerbatim",
        "no statically selected branch",
        None,
        None,
    )
}

fn collect_native_css_function(
    source: &str,
    tokens: &[LexedToken],
    at_keyword_index: usize,
) -> Option<OmenaScssEvalNativeCssFunctionV0> {
    let name_index = next_non_trivia_token_index(tokens, at_keyword_index + 1)?;
    let name = tokens.get(name_index)?;
    if !name.text.starts_with("--") {
        return None;
    }
    let left_paren_index = next_non_trivia_token_index(tokens, name_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    let right_paren_index = matching_token_index(
        tokens,
        left_paren_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let block_start_index = next_matching_token_index(tokens, right_paren_index + 1, |token| {
        matches!(
            token.kind,
            SyntaxKind::LeftBrace | SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
        )
    })?;
    if tokens.get(block_start_index)?.kind != SyntaxKind::LeftBrace {
        return None;
    }
    let block_end_index = matching_token_index(
        tokens,
        block_start_index,
        SyntaxKind::LeftBrace,
        SyntaxKind::RightBrace,
    )
    .unwrap_or(block_start_index);
    let parameters = collect_native_css_function_parameters(
        source,
        tokens,
        left_paren_index + 1,
        right_paren_index,
    );
    let (return_syntax_source, return_syntax) = collect_native_css_function_return_syntax(
        source,
        tokens,
        right_paren_index,
        block_start_index,
    );
    let results =
        collect_native_css_function_results(source, tokens, block_start_index, block_end_index);
    let source_span_start = token_start(tokens.get(at_keyword_index)?);
    let source_span_end = token_end(tokens.get(block_end_index)?);
    let typed_parameter_count = parameters
        .iter()
        .filter(|parameter| parameter.syntax.is_some())
        .count();

    Some(OmenaScssEvalNativeCssFunctionV0 {
        name: name.text.clone(),
        source_span_start,
        source_span_end,
        name_span_start: token_start(name),
        name_span_end: token_end(name),
        return_syntax_source,
        return_syntax,
        parameter_count: parameters.len(),
        typed_parameter_count,
        result_count: results.len(),
        parameters,
        results,
    })
}

fn collect_native_css_function_parameters(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Vec<OmenaScssEvalNativeCssFunctionParameterV0> {
    split_top_level_ranges(tokens, start_index, end_index, SyntaxKind::Comma)
        .into_iter()
        .filter_map(|(start, end)| {
            collect_native_css_function_parameter(source, tokens, start, end)
        })
        .collect()
}

fn collect_native_css_function_parameter(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<OmenaScssEvalNativeCssFunctionParameterV0> {
    let name_index = next_non_trivia_token_index_until(tokens, start_index, end_index)?;
    let name = tokens.get(name_index)?;
    if !name.text.starts_with("--") {
        return None;
    }
    let separator_index = first_top_level_token_index(tokens, name_index + 1, end_index, |token| {
        matches!(token.kind, SyntaxKind::Colon)
    });
    let syntax_end_index = separator_index.unwrap_or(end_index);
    let syntax_source =
        trimmed_source_between_tokens(source, tokens, name_index + 1, syntax_end_index);
    let syntax = syntax_source
        .as_deref()
        .map(parse_registered_property_syntax_v0);
    let default_value = separator_index
        .and_then(|index| trimmed_source_between_tokens(source, tokens, index + 1, end_index));
    let source_span_start = token_start(name);
    let source_span_end = previous_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_end)
        .unwrap_or_else(|| token_end(name));

    Some(OmenaScssEvalNativeCssFunctionParameterV0 {
        name: name.text.clone(),
        source_span_start,
        source_span_end,
        name_span_start: token_start(name),
        name_span_end: token_end(name),
        syntax_source,
        syntax,
        default_value,
    })
}

fn collect_native_css_function_return_syntax(
    source: &str,
    tokens: &[LexedToken],
    right_paren_index: usize,
    block_start_index: usize,
) -> (Option<String>, Option<RegisteredPropertySyntaxV0>) {
    let Some(returns_index) =
        first_top_level_token_index(tokens, right_paren_index + 1, block_start_index, |token| {
            token.kind == SyntaxKind::Ident && token.text.eq_ignore_ascii_case("returns")
        })
    else {
        return (None, None);
    };
    let source =
        trimmed_source_between_tokens(source, tokens, returns_index + 1, block_start_index);
    let syntax = source.as_deref().map(parse_registered_property_syntax_v0);
    (source, syntax)
}

fn collect_native_css_function_results(
    source: &str,
    tokens: &[LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> Vec<OmenaScssEvalNativeCssFunctionResultV0> {
    let mut results = Vec::new();
    let mut index = block_start_index + 1;
    while index < block_end_index {
        let Some(property_index) =
            next_non_trivia_token_index_until(tokens, index, block_end_index)
        else {
            break;
        };
        let property = &tokens[property_index];
        if property.kind != SyntaxKind::Ident || !property.text.eq_ignore_ascii_case("result") {
            index = property_index.saturating_add(1);
            continue;
        }
        let Some(colon_index) =
            next_non_trivia_token_index_until(tokens, property_index + 1, block_end_index)
        else {
            index = property_index.saturating_add(1);
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            index = property_index.saturating_add(1);
            continue;
        }
        let value_end =
            first_top_level_token_index(tokens, colon_index + 1, block_end_index, |token| {
                matches!(
                    token.kind,
                    SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
                )
            })
            .unwrap_or(block_end_index);
        if let Some(value) =
            trimmed_source_between_tokens(source, tokens, colon_index + 1, value_end)
        {
            let value_span_start =
                first_non_trivia_token_index_until(tokens, colon_index + 1, value_end)
                    .and_then(|index| tokens.get(index))
                    .map(token_start)
                    .unwrap_or_else(|| token_end(&tokens[colon_index]));
            let value_span_end =
                previous_non_trivia_token_index_until(tokens, colon_index + 1, value_end)
                    .and_then(|index| tokens.get(index))
                    .map(token_end)
                    .unwrap_or(value_span_start);
            results.push(OmenaScssEvalNativeCssFunctionResultV0 {
                value,
                source_span_start: value_span_start,
                source_span_end: value_span_end,
            });
        }
        index = value_end.saturating_add(1);
    }
    results
}

fn split_top_level_ranges(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
    separator: SyntaxKind,
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut range_start = start_index;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    for index in start_index..end_index {
        let Some(token) = tokens.get(index) else {
            break;
        };
        match token.kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBrace => brace_depth += 1,
            SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if kind == separator
                && paren_depth == 0
                && brace_depth == 0
                && bracket_depth == 0 =>
            {
                ranges.push((range_start, index));
                range_start = index + 1;
            }
            _ => {}
        }
    }
    if range_start < end_index {
        ranges.push((range_start, end_index));
    }
    ranges
}

fn first_top_level_token_index(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
    predicate: impl Fn(&LexedToken) -> bool,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    for index in start_index..end_index {
        let token = tokens.get(index)?;
        match token.kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBrace => brace_depth += 1,
            SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 && predicate(token) {
            return Some(index);
        }
    }
    None
}

fn matching_token_index(
    tokens: &[LexedToken],
    open_index: usize,
    open_kind: SyntaxKind,
    close_kind: SyntaxKind,
) -> Option<usize> {
    let mut depth = 0usize;
    for index in open_index..tokens.len() {
        match tokens.get(index)?.kind {
            kind if kind == open_kind => depth += 1,
            kind if kind == close_kind => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn next_matching_token_index(
    tokens: &[LexedToken],
    start_index: usize,
    predicate: impl Fn(&LexedToken) -> bool,
) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start_index)
        .find_map(|(index, token)| predicate(token).then_some(index))
}

fn next_non_trivia_token_index(tokens: &[LexedToken], start_index: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start_index)
        .find_map(|(index, token)| (!token.kind.is_trivia()).then_some(index))
}

fn next_non_trivia_token_index_until(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(end_index)
        .skip(start_index)
        .find_map(|(index, token)| (!token.kind.is_trivia()).then_some(index))
}

fn first_non_trivia_token_index_until(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<usize> {
    next_non_trivia_token_index_until(tokens, start_index, end_index)
}

fn previous_non_trivia_token_index_until(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(end_index)
        .skip(start_index)
        .rev()
        .find_map(|(index, token)| (!token.kind.is_trivia()).then_some(index))
}

fn trimmed_source_between_tokens(
    source: &str,
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<String> {
    let start = first_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_start)?;
    let end = previous_non_trivia_token_index_until(tokens, start_index, end_index)
        .and_then(|index| tokens.get(index))
        .map(token_end)?;
    source
        .get(start..end)
        .map(str::trim)
        .and_then(|value| (!value.is_empty()).then(|| value.to_string()))
}

fn token_start(token: &LexedToken) -> usize {
    u32::from(token.range.start()) as usize
}

fn token_end(token: &LexedToken) -> usize {
    u32::from(token.range.end()) as usize
}

fn parameter_syntax_is_supported(syntax: Option<&RegisteredPropertySyntaxV0>) -> bool {
    matches!(
        syntax,
        Some(RegisteredPropertySyntaxV0::Universal | RegisteredPropertySyntaxV0::Supported { .. })
    )
}

fn extract_named_function_inner<'a>(condition: &'a str, name: &str) -> Option<&'a str> {
    let trimmed = condition.trim();
    let prefix = trimmed.get(..name.len())?;
    if !prefix.eq_ignore_ascii_case(name) {
        return None;
    }
    let rest = trimmed[name.len()..].trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let close_index = matching_closing_paren_byte_index(rest)?;
    rest[close_index + 1..]
        .trim()
        .is_empty()
        .then_some(&rest[1..close_index])
}

fn matching_closing_paren_byte_index(value: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn normalize_supports_condition_for_if(condition: &str) -> String {
    let trimmed = condition.trim();
    if trimmed.starts_with('(') {
        trimmed.to_string()
    } else {
        format!("({trimmed})")
    }
}

fn static_supports_verdict_label(verdict: StaticSupportsEvalVerdictV0) -> &'static str {
    match verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => "alwaysTrue",
        StaticSupportsEvalVerdictV0::AlwaysFalse => "alwaysFalse",
        StaticSupportsEvalVerdictV0::Unknown => "unknown",
    }
}

fn native_css_if_value_is_fully_static(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    ![
        "var(",
        "attr(",
        "env(",
        "style(",
        "media(",
        "supports(",
        "if(",
        "--",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

#[cfg(test)]
mod tests {
    use omena_abstract_value::RegisteredPropertySyntaxV0;
    use omena_parser::StyleDialect;

    use super::{
        summarize_native_css_function_surface, summarize_native_css_if_function_decisions,
    };

    #[test]
    fn native_css_function_surface_reuses_registered_property_syntax() {
        let source = "@function --gap(--size <length>: 1rem, --tone <color>) returns <length> { result: var(--size); }";
        let report = summarize_native_css_function_surface(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "recognitionOnly");
        assert_eq!(report.function_count, 1);
        assert_eq!(report.parameter_count, 2);
        assert_eq!(report.typed_parameter_count, 2);
        assert_eq!(report.supported_parameter_syntax_count, 2);
        assert_eq!(report.result_count, 1);
        assert_eq!(report.functions[0].name, "--gap");
        assert_eq!(
            report.functions[0].return_syntax,
            Some(RegisteredPropertySyntaxV0::Supported {
                alternatives: vec![
                    omena_abstract_value::RegisteredPropertySyntaxAlternativeV0::Sequence {
                        components: vec![
                            omena_abstract_value::RegisteredPropertySyntaxComponentV0 {
                                base: omena_abstract_value::RegisteredPropertySyntaxBaseV0::Length,
                                multiplier:
                                    omena_abstract_value::RegisteredPropertySyntaxMultiplierV0::One,
                            },
                        ],
                    },
                ]
            })
        );
        assert_eq!(report.functions[0].parameters[0].name, "--size");
        assert_eq!(
            report.functions[0].parameters[0].syntax_source.as_deref(),
            Some("<length>")
        );
        assert_eq!(
            report.functions[0].parameters[0].default_value.as_deref(),
            Some("1rem")
        );
        assert_eq!(report.functions[0].results[0].value, "var(--size)");
    }

    #[test]
    fn native_css_function_surface_stays_css_dialect_only() {
        let source = "@function gap($size) { @return $size; }";

        assert!(summarize_native_css_function_surface(source, StyleDialect::Scss).is_none());
    }

    #[test]
    fn native_css_if_function_decision_folds_static_supports_branch() {
        let source = ".card { display: if(supports(display: grid): grid; else: block); }";
        let report = summarize_native_css_if_function_decisions(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnlyPruneButKeep");
        assert_eq!(report.function_count, 1);
        assert_eq!(report.foldable_function_count, 1);
        assert_eq!(report.preserved_function_count, 0);
        assert_eq!(report.static_supports_branch_count, 1);
        assert_eq!(report.runtime_branch_count, 0);
        assert_eq!(report.functions[0].decision, "foldToStaticValue");
        assert_eq!(report.functions[0].selected_branch_index, Some(0));
        assert_eq!(report.functions[0].selected_value.as_deref(), Some("grid"));
        assert_eq!(report.functions[0].branches[0].condition_kind, "supports");
        assert_eq!(report.functions[0].branches[0].verdict, "alwaysTrue");
        assert!(
            report.functions[0].branches[0]
                .supports_witness
                .as_ref()
                .is_some_and(|witness| witness.product == "omena-cascade.supports-static-eval")
        );
    }

    #[test]
    fn native_css_if_function_decision_preserves_runtime_media_branch() {
        let source = ".card { margin: if(media(width >= 1px): 1rem; else: 2rem); }";
        let report = summarize_native_css_if_function_decisions(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.function_count, 1);
        assert_eq!(report.foldable_function_count, 0);
        assert_eq!(report.preserved_function_count, 1);
        assert_eq!(report.static_supports_branch_count, 0);
        assert_eq!(report.runtime_branch_count, 1);
        assert_eq!(report.functions[0].decision, "preserveVerbatim");
        assert_eq!(
            report.functions[0].reason,
            "encountered runtime condition before a static winner"
        );
        assert_eq!(report.functions[0].selected_branch_index, None);
        assert_eq!(report.functions[0].branches[0].condition_kind, "media");
        assert_eq!(report.functions[0].branches[0].verdict, "runtime");
    }

    #[test]
    fn native_css_if_function_decision_preserves_runtime_value() {
        let source = ".card { color: if(supports(color: red): var(--accent); else: blue); }";
        let report = summarize_native_css_if_function_decisions(source, StyleDialect::Css);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.function_count, 1);
        assert_eq!(report.functions[0].decision, "preserveVerbatim");
        assert_eq!(
            report.functions[0].reason,
            "selected branch value depends on runtime or cascade state"
        );
        assert_eq!(report.functions[0].selected_value, None);
    }

    #[test]
    fn native_css_if_function_decision_stays_css_dialect_only() {
        let source = ".card { width: if(supports(width: 1px): 1px; else: 2px); }";

        assert!(summarize_native_css_if_function_decisions(source, StyleDialect::Scss).is_none());
    }
}
