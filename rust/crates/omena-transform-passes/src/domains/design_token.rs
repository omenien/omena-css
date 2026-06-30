use omena_parser::StyleDialect;
use omena_transform_cst::{TransformIrV0, lower_transform_ir_from_source};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    helpers::{
        blocks::at_rule_prelude_end_index,
        declarations::collect_simple_declarations_in_block,
        ir_transaction::{
            TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
            TransformIrSourceReplacementV0, replace_ir_node_spans_in_ir,
        },
        source_rewrite::replace_source_ranges,
        tokens::{matching_right_brace_index, token_end, token_start},
        values::{
            matching_function_call_end, parse_whole_function_value_arguments,
            split_top_level_value_arguments,
        },
    },
    model::TransformDesignTokenRouteV0,
};

pub(crate) fn route_design_token_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> (String, usize) {
    let replacements = collect_design_token_route_replacements(source, dialect, routes);
    let ranges = replacements
        .iter()
        .map(|replacement| {
            (
                replacement.source_span_start,
                replacement.source_span_end,
                replacement.replacement.clone(),
            )
        })
        .collect::<Vec<_>>();
    replace_source_ranges(source, &ranges)
}

pub(crate) fn route_design_token_values_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir = lower_transform_ir_from_source(
        source,
        dialect,
        "omena-transform-passes.design-token-routing",
    );
    route_design_token_values_with_ir_transaction_on_ir(&mut ir, dialect, routes)
}

pub(crate) fn route_design_token_values_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let replacements = collect_design_token_route_replacements(ir.source_text(), dialect, routes);
    let replacements = design_token_route_node_replacements(replacements.as_slice())?;
    replace_ir_node_spans_in_ir(ir, "design-token-routing", replacements.as_slice())
}

fn design_token_route_node_replacements(
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<Vec<TransformIrSourceReplacementV0>, TransformIrSourceReplacementErrorV0> {
    replacements
        .iter()
        .map(|replacement| {
            let kind = match replacement.kind {
                TransformIrReplacementKindV0::AtRule => TransformIrReplacementKindV0::AtRule,
                TransformIrReplacementKindV0::CustomPropertyReference => {
                    TransformIrReplacementKindV0::Declaration
                }
                _ => {
                    return Err(TransformIrSourceReplacementErrorV0::MissingNode {
                        source_span_start: replacement.source_span_start,
                        source_span_end: replacement.source_span_end,
                        kind: replacement.kind,
                        candidate_spans: Vec::new(),
                    });
                }
            };
            Ok(TransformIrSourceReplacementV0 {
                source_span_start: replacement.source_span_start,
                source_span_end: replacement.source_span_end,
                replacement: replacement.replacement.clone(),
                kind,
            })
        })
        .collect()
}

fn collect_design_token_route_replacements(
    source: &str,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == omena_syntax::SyntaxKind::AtKeyword
            && at_rule_prelude_can_route_design_tokens(&tokens[index].text)
            && let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1)
        {
            let prelude_start = token_end(&tokens[index]);
            let prelude_end = token_start(&tokens[prelude_end_index]);
            if let Some(routed_prelude) = route_design_token_references_in_value(
                &source[prelude_start..prelude_end],
                routes,
                None,
            ) {
                replacements.push(TransformIrSourceReplacementV0 {
                    source_span_start: prelude_start,
                    source_span_end: prelude_end,
                    replacement: routed_prelude,
                    kind: TransformIrReplacementKindV0::AtRule,
                });
            }
        }

        let Some(block_end_index) = (tokens[index].kind == omena_syntax::SyntaxKind::LeftBrace)
            .then(|| matching_right_brace_index(tokens, index))
            .flatten()
        else {
            index += 1;
            continue;
        };
        for declaration in collect_simple_declarations_in_block(tokens, index, block_end_index) {
            let declaration_value = if declaration.important {
                let Some(value) = declaration_value_without_important(&declaration.value) else {
                    continue;
                };
                value
            } else {
                declaration.value.as_str()
            };
            let blocked_token_name = declaration
                .property
                .starts_with("--")
                .then(|| normalize_design_token_name(&declaration.property))
                .flatten();
            let Some(routed_value) = route_design_token_references_in_value(
                declaration_value,
                routes,
                blocked_token_name,
            ) else {
                continue;
            };
            let important = if declaration.important {
                "!important"
            } else {
                ""
            };
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: declaration.start,
                source_span_end: declaration.end,
                replacement: format!("{}: {routed_value}{important};", declaration.property),
                kind: TransformIrReplacementKindV0::CustomPropertyReference,
            });
        }
        index += 1;
    }

    replacements
}

fn at_rule_prelude_can_route_design_tokens(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@container" | "@custom-media" | "@media" | "@supports"
    )
}

fn declaration_value_without_important(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.ends_with("!important") {
        let suffix_start = trimmed.len().saturating_sub("!important".len());
        return Some(trimmed[..suffix_start].trim_end());
    }
    if lower.ends_with("! important") {
        let suffix_start = trimmed.rfind('!')?;
        return Some(trimmed[..suffix_start].trim_end());
    }
    None
}

fn route_design_token_references_in_value(
    value: &str,
    routes: &[TransformDesignTokenRouteV0],
    blocked_token_name: Option<&str>,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index = close_index + ')'.len_utf8();
                    continue;
                };
                if let Some(routed_value) = routed_design_token_value_for_var_arguments(
                    &arguments,
                    routes,
                    blocked_token_name,
                    &mut Vec::new(),
                ) {
                    output.push_str(&value[cursor..index]);
                    output.push_str(&routed_value);
                    index = close_index + ')'.len_utf8();
                    cursor = index;
                    changed = true;
                } else {
                    index += ch.len_utf8();
                }
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn routed_design_token_value_for_var_arguments(
    arguments: &[String],
    routes: &[TransformDesignTokenRouteV0],
    blocked_token_name: Option<&str>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    let (token_name, fallback_arguments) = arguments.split_first()?;
    let token_name = normalize_design_token_name(token_name)?;
    if blocked_token_name.is_some_and(|blocked| blocked == token_name) {
        return None;
    }
    let routed_value = design_token_routed_value(token_name, routes)?;
    if !fallback_arguments.is_empty()
        && let Some(routed_token_name) = parse_single_custom_property_var_reference(routed_value)
    {
        let fallback = fallback_arguments.join(", ");
        let routed_fallback =
            route_design_token_references_in_value(&fallback, routes, blocked_token_name)
                .unwrap_or(fallback);
        return Some(format!("var({routed_token_name}, {routed_fallback})"));
    }
    resolve_nested_design_token_route_value(routed_value, routes, blocked_token_name, visiting)
        .or_else(|| Some(routed_value.to_string()))
}

fn resolve_nested_design_token_route_value(
    value: &str,
    routes: &[TransformDesignTokenRouteV0],
    blocked_token_name: Option<&str>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    let Some(routed_token_name) = parse_single_custom_property_var_reference(value) else {
        return route_design_token_references_in_value(value, routes, blocked_token_name);
    };
    if visiting.iter().any(|name| name == &routed_token_name) {
        return None;
    }
    visiting.push(routed_token_name.clone());
    let resolved =
        resolve_design_token_route_name(&routed_token_name, routes, blocked_token_name, visiting);
    visiting.pop();
    resolved
}

fn resolve_design_token_route_name(
    token_name: &str,
    routes: &[TransformDesignTokenRouteV0],
    blocked_token_name: Option<&str>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    if blocked_token_name.is_some_and(|blocked| blocked == token_name) {
        return None;
    }
    let routed_value = design_token_routed_value(token_name, routes)?;
    resolve_nested_design_token_route_value(routed_value, routes, blocked_token_name, visiting)
        .or_else(|| Some(routed_value.to_string()))
}

fn parse_single_custom_property_var_reference(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    let [name] = arguments.as_slice() else {
        return None;
    };
    Some(normalize_design_token_name(name)?.to_string())
}

fn design_token_routed_value<'a>(
    token_name: &str,
    routes: &'a [TransformDesignTokenRouteV0],
) -> Option<&'a str> {
    routes.iter().find_map(|route| {
        let route_name = normalize_design_token_name(&route.token_name)?;
        let routed_value = route.routed_value.trim();
        if routed_value.is_empty() || routed_value.chars().any(|ch| matches!(ch, ';' | '{' | '}')) {
            return None;
        }
        (route_name == token_name).then_some(routed_value)
    })
}

fn normalize_design_token_name(name: &str) -> Option<&str> {
    let name = name.trim();
    if name.starts_with("--") && name.len() > 2 {
        return Some(name);
    }
    None
}
