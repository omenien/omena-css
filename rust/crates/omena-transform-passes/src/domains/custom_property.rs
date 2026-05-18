use std::collections::{BTreeMap, VecDeque};

use omena_cascade::{
    CascadeValue, CustomPropertyEnv, resolve_custom_property_env_least_fixed_point,
    substitute_custom_properties,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::domains::{
    css_module_global::collect_css_module_scope_blocks,
    css_modules_values::at_rule_block_has_reachable_ordinary_rule,
    keyframes::{
        KeyframesRuleSlice, collect_referenced_keyframe_names, collect_top_level_keyframes_rules,
    },
    reachability::rule_slice_matches_reachable_class_context,
};
use crate::helpers::{
    blocks::{at_rule_block_start, at_rule_prelude_end_index, rule_block_token_indexes},
    collections::push_unique_string,
    declarations::collect_simple_declarations_in_block,
    identifiers::{is_css_ident_continue, normalize_custom_property_name},
    rules::{
        SimpleRuleSlice, collect_declaration_ordinary_rule_slices,
        collect_top_level_ordinary_rule_slices,
    },
    source_rewrite::{remove_source_ranges, replace_source_ranges},
    tokens::{matching_right_brace_index, skip_whitespace_tokens, token_end, token_start},
    values::{
        matching_function_call_end, parse_whole_function_value_arguments,
        split_top_level_value_arguments,
    },
};
use crate::model::TransformSemanticRemovalCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CustomPropertyRegistrationRule {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) initial_value: Option<String>,
}

pub(crate) fn collect_custom_property_registration_rules(
    tokens: &[omena_parser::LexedToken],
) -> Vec<CustomPropertyRegistrationRule> {
    let mut rules = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@property")
            && let Some((rule, next_index)) = parse_custom_property_registration_rule(tokens, index)
        {
            rules.push(rule);
            index = next_index;
            continue;
        }
        index += 1;
    }

    rules
}

fn parse_custom_property_registration_rule(
    tokens: &[omena_parser::LexedToken],
    at_property_index: usize,
) -> Option<(CustomPropertyRegistrationRule, usize)> {
    let name_index = skip_whitespace_tokens(tokens, at_property_index + 1, tokens.len());
    let name = normalize_custom_property_name(tokens.get(name_index)?.text.as_str())?.to_string();
    let block_start_index = at_rule_block_start(tokens, name_index + 1)?;
    let close_index = matching_right_brace_index(tokens, block_start_index)?;
    let initial_value =
        collect_simple_declarations_in_block(tokens, block_start_index, close_index)
            .into_iter()
            .find(|declaration| declaration.property == "initial-value" && !declaration.important)
            .map(|declaration| declaration.value);

    Some((
        CustomPropertyRegistrationRule {
            name,
            start: token_start(&tokens[at_property_index]),
            end: token_end(&tokens[close_index]),
            initial_value,
        },
        close_index + 1,
    ))
}

pub(crate) fn close_custom_property_dependency_graph(
    roots: Vec<String>,
    dependencies_by_name: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut reachable = Vec::new();
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();

    while let Some(name) = queue.pop_front() {
        if reachable.iter().any(|existing| existing == &name) {
            continue;
        }
        reachable.push(name.clone());
        if let Some(dependencies) = dependencies_by_name.get(&name) {
            for dependency in dependencies {
                queue.push_back(dependency.clone());
            }
        }
    }

    reachable.sort();
    reachable
}

pub(crate) fn collect_custom_property_references_in_value(value: &str) -> Vec<String> {
    let mut names = Vec::new();
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
                if let [name, fallback @ ..] = arguments.as_slice()
                    && let Some(name) = normalize_custom_property_name(name)
                {
                    push_unique_string(&mut names, name.to_string());
                    for fallback_value in fallback {
                        for fallback_name in
                            collect_custom_property_references_in_value(fallback_value)
                        {
                            push_unique_string(&mut names, fallback_name);
                        }
                    }
                }
                index = close_index + ')'.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    names
}

pub(crate) fn collect_custom_property_references_in_container_style_query_prelude(
    prelude: &str,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < prelude.len() {
        let Some(ch) = prelude[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = prelude[index..].chars().next() {
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
            _ if css_function_name_starts_at(prelude, index, "style") => {
                let left_paren_index = index + "style".len();
                let Some(close_index) = matching_function_call_end(prelude, left_paren_index)
                else {
                    index += ch.len_utf8();
                    continue;
                };
                collect_custom_property_names_in_style_query(
                    &prelude[left_paren_index + 1..close_index],
                    &mut names,
                );
                index = close_index + ')'.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    names
}

pub(crate) fn collect_custom_property_roots_from_container_style_query_preludes(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    mut block_is_reachable: impl FnMut(usize, usize) -> bool,
) -> Vec<String> {
    let mut roots = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@container")
        {
            index += 1;
            continue;
        }
        let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        let block_is_reachable = tokens[prelude_end_index].kind == SyntaxKind::LeftBrace
            && matching_right_brace_index(tokens, prelude_end_index)
                .is_some_and(|close_index| block_is_reachable(prelude_end_index, close_index));
        if block_is_reachable {
            let prelude_start = token_end(&tokens[index]);
            let prelude_end = token_start(&tokens[prelude_end_index]);
            for name in collect_custom_property_references_in_container_style_query_prelude(
                &source[prelude_start..prelude_end],
            ) {
                push_unique_string(&mut roots, name);
            }
        }
        index = prelude_end_index.saturating_add(1);
    }

    roots
}

pub(crate) fn tree_shake_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let Some(referenced_names) = collect_reachable_custom_property_names(
        source,
        tokens,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    ) else {
        return (source.to_string(), Vec::new());
    };

    let mut removals = Vec::new();
    for registration in collect_custom_property_registration_rules(tokens) {
        if !referenced_names
            .iter()
            .any(|name| name == &registration.name)
        {
            removals.push(TransformSemanticRemovalCandidate {
                symbol_kind: "customPropertyRegistration",
                name: registration.name,
                source_span_start: registration.start,
                source_span_end: registration.end,
                reason: "custom-property registration was absent from the closed-style-world reachable custom-property set",
            });
        }
    }
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                if declaration.property.starts_with("--")
                    && !referenced_names
                        .iter()
                        .any(|name| name == &declaration.property)
                {
                    removals.push(TransformSemanticRemovalCandidate {
                        symbol_kind: "customProperty",
                        name: declaration.property,
                        source_span_start: declaration.start,
                        source_span_end: declaration.end,
                        reason: "custom property declaration was absent from transitive var() references and the closed-style-world reachable custom-property set",
                    });
                }
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    let ranges = removals
        .iter()
        .map(|removal| (removal.source_span_start, removal.source_span_end))
        .collect::<Vec<_>>();
    let (output, _) = remove_source_ranges(source, &ranges);
    (output, removals)
}

fn collect_reachable_custom_property_names(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    external_roots: &[String],
    external_keyframe_roots: &[String],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut root_names = Vec::new();
    let mut dependencies_by_name = BTreeMap::<String, Vec<String>>::new();
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let keyframes = collect_top_level_keyframes_rules(tokens);
    let reachable_keyframe_names = collect_reachable_keyframe_names(
        source,
        tokens,
        external_keyframe_roots,
        reachable_class_names,
    );

    for name in external_roots {
        if let Some(name) = normalize_custom_property_name(name) {
            push_unique_string(&mut root_names, name.to_string());
        }
    }
    for name in collect_custom_property_roots_from_container_style_query_preludes(
        source,
        tokens,
        |block_start_index, block_end_index| {
            at_rule_block_has_reachable_ordinary_rule(
                source,
                tokens,
                block_start_index,
                block_end_index,
                reachable_class_names,
                &scope_blocks,
            )
        },
    ) {
        push_unique_string(&mut root_names, name);
    }

    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        if let Some(keyframe_name) = enclosing_keyframe_name_for_rule(&rule, &keyframes)
            && let Some(reachable_keyframe_names) = reachable_keyframe_names.as_ref()
            && !reachable_keyframe_names
                .iter()
                .any(|name| name == keyframe_name)
        {
            continue;
        }
        let rule_is_reachable =
            rule_slice_matches_reachable_class_context(&rule, &scope_blocks, reachable_class_names);
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--") {
                if !rule_is_reachable {
                    continue;
                }
                let referenced_names =
                    collect_custom_property_references_in_value(&declaration.value);
                let dependencies = dependencies_by_name
                    .entry(declaration.property)
                    .or_default();
                for name in referenced_names {
                    push_unique_string(dependencies, name);
                }
            } else if rule_is_reachable {
                let referenced_names =
                    collect_custom_property_references_in_value(&declaration.value);
                for name in referenced_names {
                    push_unique_string(&mut root_names, name);
                }
            }
        }
    }

    Some(close_custom_property_dependency_graph(
        root_names,
        &dependencies_by_name,
    ))
}

fn collect_reachable_keyframe_names(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    external_roots: &[String],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut names = collect_referenced_keyframe_names(source, tokens, reachable_class_names)?;
    for name in external_roots {
        push_unique_string(&mut names, name.clone());
    }
    Some(names)
}

fn enclosing_keyframe_name_for_rule<'a>(
    rule: &SimpleRuleSlice,
    keyframes: &'a [KeyframesRuleSlice],
) -> Option<&'a str> {
    keyframes
        .iter()
        .find(|keyframe| rule.start >= keyframe.start && rule.end <= keyframe.end)
        .map(|keyframe| keyframe.name.as_str())
}

fn collect_custom_property_names_in_style_query(query: &str, names: &mut Vec<String>) {
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < query.len() {
        let Some(ch) = query[index..].chars().next() else {
            break;
        };

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = query[index..].chars().next() {
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
            '-' if query[index..].starts_with("--") => {
                let name_end = custom_property_name_end(query, index + "--".len());
                if name_end > index + "--".len()
                    && let Some(name) = normalize_custom_property_name(&query[index..name_end])
                {
                    push_unique_string(names, name.to_string());
                }
                index = name_end;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }
}

fn custom_property_name_end(value: &str, mut index: usize) -> usize {
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if !is_css_ident_continue(ch) {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

pub(crate) fn substitute_static_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let env_rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let env = resolve_custom_property_env_least_fixed_point(
        &collect_static_root_custom_property_env(tokens, &env_rules),
    );
    if env.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let Some(close_index) = (tokens[index].kind == SyntaxKind::LeftBrace)
            .then(|| matching_right_brace_index(tokens, index))
            .flatten()
        else {
            index += 1;
            continue;
        };
        for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
            if declaration.property.starts_with("--") {
                continue;
            }
            let Some(resolved_value) =
                substitute_static_custom_property_references_in_value(&declaration.value, &env)
            else {
                continue;
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {resolved_value};", declaration.property),
            ));
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn substitute_static_custom_property_references_in_value(
    value: &str,
    env: &CustomPropertyEnv,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
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
                    index += ch.len_utf8();
                    continue;
                };
                let Some(var_value) = parse_static_var_arguments(&arguments) else {
                    index += ch.len_utf8();
                    continue;
                };
                let resolved_value = substitute_custom_properties(&var_value, env);
                let Some(resolved_value) = render_static_cascade_value(&resolved_value) else {
                    index += ch.len_utf8();
                    continue;
                };
                output.push_str(&value[cursor..index]);
                output.push_str(&resolved_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                changed = true;
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

pub(crate) fn collect_static_root_custom_property_env(
    tokens: &[omena_parser::LexedToken],
    rules: &[SimpleRuleSlice],
) -> CustomPropertyEnv {
    let mut env = CustomPropertyEnv::new();
    let mut blocked_names = Vec::new();
    let registrations = collect_custom_property_registration_rules(tokens);

    for rule in rules {
        if rule.selector == ":root" {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--")
                && !blocked_names.contains(&declaration.property)
            {
                blocked_names.push(declaration.property);
            }
        }
    }

    for rule in rules {
        if rule.selector != ":root" {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if !declaration.property.starts_with("--") {
                continue;
            }
            if declaration.important {
                env.remove(&declaration.property);
                if !blocked_names.contains(&declaration.property) {
                    blocked_names.push(declaration.property);
                }
                continue;
            }
            if blocked_names.contains(&declaration.property) {
                continue;
            }
            if env.contains_key(&declaration.property) {
                env.remove(&declaration.property);
                blocked_names.push(declaration.property);
                continue;
            }
            let Some(value) = parse_static_custom_property_env_value(&declaration.value) else {
                env.remove(&declaration.property);
                if !blocked_names.contains(&declaration.property) {
                    blocked_names.push(declaration.property);
                }
                continue;
            };
            env.insert(declaration.property, value);
        }
    }

    let mut registration_names = Vec::new();
    for registration in registrations {
        if blocked_names.contains(&registration.name) {
            continue;
        }
        if registration_names.contains(&registration.name) {
            env.remove(&registration.name);
            blocked_names.push(registration.name);
            continue;
        }
        registration_names.push(registration.name.clone());
        if env.contains_key(&registration.name) {
            continue;
        }
        let Some(initial_value) = registration.initial_value else {
            continue;
        };
        let Some(value) = parse_static_custom_property_env_value(&initial_value) else {
            continue;
        };
        env.insert(registration.name, value);
    }

    env
}

pub(crate) fn parse_static_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    if contains_runtime_dependent_css_function(value) {
        return None;
    }
    parse_static_var_value(value)
        .or_else(|| parse_static_composite_custom_property_env_value(value))
}

fn contains_runtime_dependent_css_function(value: &str) -> bool {
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
            _ if css_function_name_starts_at(value, index, "env")
                || css_function_name_starts_at(value, index, "attr") =>
            {
                return true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    false
}

fn css_function_name_starts_at(value: &str, index: usize, function_name: &str) -> bool {
    let Some(name) = value.get(index..index + function_name.len()) else {
        return false;
    };
    if !name.eq_ignore_ascii_case(function_name) {
        return false;
    }
    if !value[index + function_name.len()..].starts_with('(') {
        return false;
    }
    if index == 0 {
        return true;
    }
    let Some(previous) = value[..index].chars().next_back() else {
        return true;
    };
    !is_css_ident_continue(previous)
}

fn parse_static_composite_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    let mut parts = Vec::new();
    let mut cursor = 0;
    let mut index = 0;
    let mut quote = None;
    let mut found_var = false;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(next_ch) = value[index..].chars().next() {
                    index += next_ch.len_utf8();
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
                let Some(var_value) = parse_static_var_arguments(&arguments) else {
                    index += ch.len_utf8();
                    continue;
                };
                if cursor < index {
                    parts.push(CascadeValue::Literal(value[cursor..index].to_string()));
                }
                parts.push(var_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                found_var = true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !found_var {
        return Some(CascadeValue::Literal(value.to_string()));
    }
    if cursor < value.len() {
        parts.push(CascadeValue::Literal(value[cursor..].to_string()));
    }
    Some(CascadeValue::Composite(parts))
}

fn parse_static_var_value(value: &str) -> Option<CascadeValue> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    parse_static_var_arguments(&arguments)
}

fn parse_static_var_arguments(arguments: &[String]) -> Option<CascadeValue> {
    match arguments {
        [name] if name.starts_with("--") => Some(CascadeValue::Var {
            name: name.clone(),
            fallback: None,
        }),
        [name, fallback] if name.starts_with("--") => {
            let fallback = parse_static_custom_property_env_value(fallback)?;
            Some(CascadeValue::Var {
                name: name.clone(),
                fallback: Some(Box::new(fallback)),
            })
        }
        _ => None,
    }
}

fn render_static_cascade_value(value: &CascadeValue) -> Option<String> {
    match value {
        CascadeValue::Literal(value) => Some(value.clone()),
        CascadeValue::Composite(parts) => {
            let mut output = String::new();
            for part in parts {
                output.push_str(&render_static_cascade_value(part)?);
            }
            Some(output)
        }
        CascadeValue::Var { .. }
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => None,
    }
}
