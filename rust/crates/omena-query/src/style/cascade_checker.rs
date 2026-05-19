use std::collections::{BTreeMap, BTreeSet};

use omena_checker::{
    OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeInputV0,
    OmenaCheckerCustomPropertyInputV0, evaluate_omena_checker_cascade_rules,
};

use super::{
    OmenaQueryStyleDiagnosticV0, ParserByteSpanV0, ParserRangeV0,
    omena_parser_dialect_for_style_path, parser_range_for_byte_span,
    summarize_static_css_custom_property_fixed_point_from_source,
};

const LSP_DIAGNOSTIC_TAG_UNNECESSARY: u8 = 1;

pub(super) fn summarize_query_cascade_checker_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let (checker_input, declaration_ranges, custom_property_ranges) =
        collect_query_checker_cascade_input(style_uri, source);
    let mut diagnostics = Vec::new();

    for evaluation in evaluate_omena_checker_cascade_rules(checker_input) {
        if evaluation.rule_code_name == "iacvt-prone"
            && evaluation
                .custom_property_names
                .iter()
                .all(|name| !custom_property_ranges.contains_key(name))
        {
            continue;
        }
        let range = evaluation
            .declaration_ids
            .iter()
            .find_map(|declaration_id| declaration_ranges.get(declaration_id).copied())
            .or_else(|| {
                evaluation
                    .custom_property_names
                    .iter()
                    .find_map(|name| custom_property_ranges.get(name).copied())
            })
            .unwrap_or_else(|| {
                parser_range_for_byte_span(
                    source,
                    ParserByteSpanV0 {
                        start: 0,
                        end: source.len(),
                    },
                )
            });
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: query_cascade_checker_code(evaluation.rule_code_name),
            range,
            message: evaluation.message,
            tags: query_cascade_checker_diagnostic_tags(evaluation.rule_code_name),
            create_custom_property: None,
        });
    }

    diagnostics
}

fn query_cascade_checker_code(code: &'static str) -> &'static str {
    match code {
        "unreachable-declaration" => "unreachableDeclaration",
        "dead-cascade-layer" => "deadCascadeLayer",
        "iacvt-prone" => "iacvtProne",
        "circular-var" => "circularVar",
        "unspecified-cascade-tie" => "unspecifiedCascadeTie",
        _ => "cascadeAware",
    }
}

fn query_cascade_checker_diagnostic_tags(code: &'static str) -> Vec<u8> {
    match code {
        "unreachable-declaration" | "dead-cascade-layer" => {
            vec![LSP_DIAGNOSTIC_TAG_UNNECESSARY]
        }
        _ => Vec::new(),
    }
}

fn collect_query_checker_cascade_input(
    style_uri: &str,
    source: &str,
) -> (
    OmenaCheckerCascadeInputV0,
    BTreeMap<String, ParserRangeV0>,
    BTreeMap<String, ParserRangeV0>,
) {
    let declarations = collect_query_checker_cascade_declarations(source);
    let declaration_ranges = declarations
        .iter()
        .map(|declaration| {
            (
                declaration.input.declaration_id.clone(),
                parser_range_for_byte_span(source, declaration.byte_span),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let guaranteed_invalid_custom_properties =
        summarize_static_css_custom_property_fixed_point_from_source(source, dialect)
            .entries
            .into_iter()
            .filter(|entry| entry.guaranteed_invalid)
            .map(|entry| entry.name)
            .collect::<BTreeSet<_>>();
    let mut custom_properties_by_name =
        BTreeMap::<String, (BTreeSet<String>, bool, ParserByteSpanV0)>::new();

    for declaration in &declarations {
        if !declaration.input.property.starts_with("--") {
            continue;
        }
        let entry = custom_properties_by_name
            .entry(declaration.input.property.clone())
            .or_insert_with(|| {
                (
                    BTreeSet::new(),
                    guaranteed_invalid_custom_properties.contains(&declaration.input.property),
                    declaration.byte_span,
                )
            });
        entry.1 |= guaranteed_invalid_custom_properties.contains(&declaration.input.property);
        for dependency in collect_query_var_references_in_value(&declaration.input.value) {
            entry.0.insert(dependency);
        }
    }

    let custom_property_ranges = custom_properties_by_name
        .iter()
        .map(|(name, (_, _, byte_span))| {
            (name.clone(), parser_range_for_byte_span(source, *byte_span))
        })
        .collect::<BTreeMap<_, _>>();
    let custom_properties = custom_properties_by_name
        .into_iter()
        .map(
            |(name, (dependencies, guaranteed_invalid, _))| OmenaCheckerCustomPropertyInputV0 {
                name,
                dependencies: dependencies.into_iter().collect(),
                guaranteed_invalid,
            },
        )
        .collect::<Vec<_>>();

    (
        OmenaCheckerCascadeInputV0 {
            declarations: declarations
                .into_iter()
                .map(|declaration| declaration.input)
                .collect(),
            custom_properties,
        },
        declaration_ranges,
        custom_property_ranges,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryCheckerCascadeDeclaration {
    input: OmenaCheckerCascadeDeclarationInputV0,
    byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueryCheckerCascadeScope {
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
}

fn collect_query_checker_cascade_declarations(source: &str) -> Vec<QueryCheckerCascadeDeclaration> {
    let mut declarations = Vec::new();
    let mut layer_orders = BTreeMap::new();
    let mut next_layer_order = 0i32;
    collect_query_checker_cascade_blocks(
        source,
        0,
        source.len(),
        Vec::new(),
        None,
        None,
        &mut layer_orders,
        &mut next_layer_order,
        &mut declarations,
    );
    declarations
}

#[allow(clippy::too_many_arguments)]
fn collect_query_checker_cascade_blocks(
    source: &str,
    start: usize,
    end: usize,
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
    layer_orders: &mut BTreeMap<String, i32>,
    next_layer_order: &mut i32,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let mut index = start;
    while let Some(open_index) = find_query_top_level_byte(source, index, end, b'{') {
        let Some(close_index) = matching_query_block_end(source, open_index, end) else {
            break;
        };
        let prelude_start = query_prelude_start(source, start, open_index);
        let prelude = source[prelude_start..open_index].trim();
        let body_start = open_index + 1;

        if let Some(layer) = query_layer_name_from_prelude(prelude) {
            let order = *layer_orders.entry(layer.clone()).or_insert_with(|| {
                let order = *next_layer_order;
                *next_layer_order += 1;
                order
            });
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                condition_context.clone(),
                Some(layer),
                Some(order),
                layer_orders,
                next_layer_order,
                declarations,
            );
        } else if prelude.starts_with('@') {
            let mut nested_condition_context = condition_context.clone();
            nested_condition_context.push(normalize_query_condition_prelude(prelude));
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                nested_condition_context,
                layer_name.clone(),
                layer_order,
                layer_orders,
                next_layer_order,
                declarations,
            );
        } else if !prelude.is_empty() {
            collect_query_checker_direct_declarations(
                source,
                body_start,
                close_index,
                prelude,
                QueryCheckerCascadeScope {
                    condition_context: condition_context.clone(),
                    layer_name: layer_name.clone(),
                    layer_order,
                },
                declarations,
            );
            collect_query_checker_cascade_blocks(
                source,
                body_start,
                close_index,
                condition_context.clone(),
                layer_name.clone(),
                layer_order,
                layer_orders,
                next_layer_order,
                declarations,
            );
        }

        index = close_index + 1;
    }
}

fn collect_query_checker_direct_declarations(
    source: &str,
    body_start: usize,
    body_end: usize,
    selector: &str,
    scope: QueryCheckerCascadeScope,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let mut statement_start = body_start;
    let mut index = body_start;
    while index < body_end {
        if let Some(open_index) = find_query_top_level_byte(source, index, body_end, b'{') {
            while let Some(semicolon_index) =
                find_query_top_level_byte(source, index, open_index, b';')
            {
                push_query_checker_declaration(
                    source,
                    statement_start,
                    semicolon_index,
                    selector,
                    &scope,
                    declarations,
                );
                statement_start = semicolon_index + 1;
                index = statement_start;
            }
            let Some(close_index) = matching_query_block_end(source, open_index, body_end) else {
                break;
            };
            statement_start = close_index + 1;
            index = statement_start;
            continue;
        }

        while let Some(semicolon_index) = find_query_top_level_byte(source, index, body_end, b';') {
            push_query_checker_declaration(
                source,
                statement_start,
                semicolon_index,
                selector,
                &scope,
                declarations,
            );
            statement_start = semicolon_index + 1;
            index = statement_start;
        }
        break;
    }

    push_query_checker_declaration(
        source,
        statement_start,
        body_end,
        selector,
        &scope,
        declarations,
    );
}

fn push_query_checker_declaration(
    source: &str,
    start: usize,
    end: usize,
    selector: &str,
    scope: &QueryCheckerCascadeScope,
    declarations: &mut Vec<QueryCheckerCascadeDeclaration>,
) {
    let Some((trimmed_start, trimmed_end)) = trimmed_query_span(source, start, end) else {
        return;
    };
    let statement = &source[trimmed_start..trimmed_end];
    let Some(colon_offset) = find_query_top_level_colon(statement) else {
        return;
    };
    let property = statement[..colon_offset].trim();
    if property.is_empty()
        || property.starts_with('@')
        || property.contains(char::is_whitespace)
        || property.contains('{')
        || property.contains('}')
    {
        return;
    }
    let mut value = statement[colon_offset + 1..].trim().to_string();
    let important = query_value_has_important_suffix(&value);
    if important {
        value = value
            .trim_end()
            .trim_end_matches(|ch: char| ch.is_ascii_whitespace())
            .trim_end_matches("!important")
            .trim_end()
            .to_string();
    }
    let source_order = declarations.len();
    let declaration_id = format!("decl-{source_order}");
    declarations.push(QueryCheckerCascadeDeclaration {
        input: OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id,
            selector: selector.to_string(),
            property: property.to_string(),
            value: value.clone(),
            source_order: source_order.min(u32::MAX as usize) as u32,
            condition_context: scope.condition_context.clone(),
            layer_name: scope.layer_name.clone(),
            layer_order: scope.layer_order,
            important,
            var_references: collect_query_var_references_in_value(&value),
        },
        byte_span: ParserByteSpanV0 {
            start: trimmed_start,
            end: trimmed_end,
        },
    });
}

fn query_value_has_important_suffix(value: &str) -> bool {
    value
        .trim_end()
        .to_ascii_lowercase()
        .ends_with("!important")
}

fn trimmed_query_span(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let mut trimmed_start = start;
    let mut trimmed_end = end;
    while trimmed_start < trimmed_end
        && source[trimmed_start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
    {
        trimmed_start += source[trimmed_start..].chars().next()?.len_utf8();
    }
    while trimmed_end > trimmed_start
        && source[..trimmed_end]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    {
        trimmed_end -= source[..trimmed_end].chars().next_back()?.len_utf8();
    }
    (trimmed_start < trimmed_end).then_some((trimmed_start, trimmed_end))
}

fn find_query_top_level_colon(statement: &str) -> Option<usize> {
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;

    while index < statement.len() {
        let ch = statement[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = statement[index..].chars().next() {
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
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            ':' if paren_depth == 0 => return Some(index),
            _ => index += ch.len_utf8(),
        }
    }
    None
}

fn query_prelude_start(source: &str, search_start: usize, open_index: usize) -> usize {
    source[search_start..open_index]
        .rfind(['{', '}', ';'])
        .map(|offset| search_start + offset + 1)
        .unwrap_or(search_start)
}

fn query_layer_name_from_prelude(prelude: &str) -> Option<String> {
    let rest = prelude.trim_start().strip_prefix("@layer")?.trim();
    let name = rest
        .split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, ',' | '{' | ';'))
        .next()
        .unwrap_or_default()
        .trim_matches(['"', '\'']);
    if name.is_empty() {
        Some("(anonymous-layer)".to_string())
    } else {
        Some(name.to_string())
    }
}

fn normalize_query_condition_prelude(prelude: &str) -> String {
    prelude.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn collect_query_var_references_in_value(value: &str) -> Vec<String> {
    let mut refs = BTreeSet::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;
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
            _ if query_function_name_starts_at(value, index, "var") => {
                let open_index = index + "var".len();
                let Some(close_index) = matching_query_paren_end(value, open_index, value.len())
                else {
                    index += ch.len_utf8();
                    continue;
                };
                collect_query_var_references_from_arguments(
                    &value[open_index + 1..close_index],
                    &mut refs,
                );
                index = close_index + 1;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }
    refs.into_iter().collect()
}

fn collect_query_var_references_from_arguments(arguments: &str, refs: &mut BTreeSet<String>) {
    let parts = split_query_top_level_arguments(arguments);
    let Some(first_argument) = parts.first().map(|part| part.trim()) else {
        return;
    };
    if first_argument.starts_with("--") {
        refs.insert(first_argument.to_string());
    }
    for fallback in parts.iter().skip(1) {
        for reference in collect_query_var_references_in_value(fallback) {
            refs.insert(reference);
        }
    }
}

fn split_query_top_level_arguments(arguments: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;

    while index < arguments.len() {
        let Some(ch) = arguments[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
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
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            ',' if paren_depth == 0 => {
                parts.push(&arguments[start..index]);
                index += ch.len_utf8();
                start = index;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }
    parts.push(&arguments[start..]);
    parts
}

fn query_function_name_starts_at(value: &str, index: usize, function_name: &str) -> bool {
    value
        .get(index..index + function_name.len())
        .is_some_and(|name| name.eq_ignore_ascii_case(function_name))
        && value[index + function_name.len()..].starts_with('(')
}

fn find_query_top_level_byte(source: &str, start: usize, end: usize, needle: u8) -> Option<usize> {
    let mut index = start;
    let mut quote: Option<char> = None;
    while index < end {
        let ch = source[index..].chars().next()?;
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
        if source[index..].starts_with("/*")
            && let Some(close_offset) = source[index + 2..end].find("*/")
        {
            index += close_offset + 4;
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if ch.len_utf8() == 1 && source.as_bytes()[index] == needle => return Some(index),
            _ => index += ch.len_utf8(),
        }
    }
    None
}

fn matching_query_block_end(source: &str, open_index: usize, end: usize) -> Option<usize> {
    matching_query_delimiter_end(source, open_index, end, b'{', b'}')
}

fn matching_query_paren_end(source: &str, open_index: usize, end: usize) -> Option<usize> {
    matching_query_delimiter_end(source, open_index, end, b'(', b')')
}

fn matching_query_delimiter_end(
    source: &str,
    open_index: usize,
    end: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    if source.as_bytes().get(open_index).copied()? != open {
        return None;
    }
    let mut index = open_index + 1;
    let mut depth = 1usize;
    let mut quote: Option<char> = None;

    while index < end {
        let ch = source[index..].chars().next()?;
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
        if source[index..].starts_with("/*")
            && let Some(close_offset) = source[index + 2..end].find("*/")
        {
            index += close_offset + 4;
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if ch.len_utf8() == 1 && source.as_bytes()[index] == open => {
                depth += 1;
                index += 1;
            }
            _ if ch.len_utf8() == 1 && source.as_bytes()[index] == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}
