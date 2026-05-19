use std::collections::{BTreeMap, VecDeque};

use omena_cascade::{
    CascadeValue, CustomPropertyEnv, resolve_custom_property_env_least_fixed_point,
    substitute_custom_properties,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::domains::{
    css_module_global::{CssModuleScopeBlock, collect_css_module_scope_blocks},
    css_modules_values::at_rule_block_has_reachable_ordinary_rule,
    custom_property_icss::{
        collect_static_custom_property_icss_export_rules, custom_property_icss_export_is_reachable,
    },
    keyframes::{
        KeyframesRuleSlice, collect_referenced_keyframe_names, collect_top_level_keyframes_rules,
        keyframe_name_is_reachable,
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
    let mut export_removal_ranges = Vec::new();
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
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let keyframes = collect_top_level_keyframes_rules(tokens);
    let reachable_keyframe_names = collect_reachable_keyframe_names(
        source,
        tokens,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let export_rules = collect_static_custom_property_icss_export_rules(source, tokens);
    for rule in &export_rules {
        let unreachable_exports = rule
            .declarations
            .iter()
            .filter(|declaration| {
                !custom_property_icss_export_is_reachable(
                    &declaration.export_name,
                    reachable_custom_property_names,
                )
            })
            .collect::<Vec<_>>();
        if unreachable_exports.is_empty() {
            continue;
        }
        if unreachable_exports.len() == rule.declarations.len() {
            export_removal_ranges.push((rule.start, rule.end));
        } else {
            export_removal_ranges.extend(
                unreachable_exports
                    .iter()
                    .map(|declaration| (declaration.start, declaration.end)),
            );
        }
        removals.extend(
            unreachable_exports
                .iter()
                .map(|declaration| TransformSemanticRemovalCandidate {
                    symbol_kind: "customPropertyIcssExport",
                    name: declaration.export_name.clone(),
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    reason: "ICSS export declaration was absent from the closed-style-world reachable custom-property export set",
                }),
        );
    }
    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        let rule_is_reachable = custom_property_rule_is_reachable(
            &rule,
            &scope_blocks,
            &keyframes,
            reachable_keyframe_names.as_deref(),
            reachable_class_names,
        );
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
            let name_is_referenced = referenced_names
                .iter()
                .any(|name| name == &declaration.property);
            if !rule_is_reachable || !name_is_referenced {
                removals.push(TransformSemanticRemovalCandidate {
                    symbol_kind: "customProperty",
                    name: declaration.property,
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    reason: if rule_is_reachable {
                        "custom property declaration was absent from transitive var() references and the closed-style-world reachable custom-property set"
                    } else {
                        "custom property declaration belonged to an unreachable closed-style-world rule"
                    },
                });
            }
        }
    }

    let mut ranges = removals
        .iter()
        .filter(|removal| removal.symbol_kind != "customPropertyIcssExport")
        .map(|removal| (removal.source_span_start, removal.source_span_end))
        .collect::<Vec<_>>();
    ranges.extend(export_removal_ranges);
    let (output, _) = remove_source_ranges(source, &ranges);
    (output, removals)
}

fn custom_property_rule_is_reachable(
    rule: &SimpleRuleSlice,
    scope_blocks: &[CssModuleScopeBlock],
    keyframes: &[KeyframesRuleSlice],
    reachable_keyframe_names: Option<&[String]>,
    reachable_class_names: &[String],
) -> bool {
    if let Some(keyframe_name) = enclosing_keyframe_name_for_rule(rule, keyframes)
        && let Some(reachable_keyframe_names) = reachable_keyframe_names
    {
        return keyframe_name_is_reachable(keyframe_name, reachable_keyframe_names);
    }

    rule_slice_matches_reachable_class_context(rule, scope_blocks, reachable_class_names)
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
    for name in collect_custom_property_roots_from_reachable_at_rule_preludes(
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
    for name in collect_custom_property_roots_from_descriptor_at_rules(tokens) {
        push_unique_string(&mut root_names, name);
    }
    for rule in collect_static_custom_property_icss_export_rules(source, tokens) {
        for declaration in rule.declarations {
            if !custom_property_icss_export_is_reachable(&declaration.export_name, external_roots) {
                continue;
            }
            for name in collect_custom_property_references_in_value(&declaration.value) {
                push_unique_string(&mut root_names, name);
            }
        }
    }

    for registration in collect_custom_property_registration_rules(tokens) {
        let Some(initial_value) = registration.initial_value else {
            continue;
        };
        let dependencies = dependencies_by_name.entry(registration.name).or_default();
        for name in collect_custom_property_references_in_value(&initial_value) {
            push_unique_string(dependencies, name);
        }
    }

    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        if rule.selector.trim().eq_ignore_ascii_case(":export") {
            continue;
        }
        if let Some(keyframe_name) = enclosing_keyframe_name_for_rule(&rule, &keyframes)
            && let Some(reachable_keyframe_names) = reachable_keyframe_names.as_ref()
            && !keyframe_name_is_reachable(keyframe_name, reachable_keyframe_names)
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

fn collect_custom_property_roots_from_reachable_at_rule_preludes(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    mut block_is_reachable: impl FnMut(usize, usize) -> bool,
) -> Vec<String> {
    let mut roots = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !at_rule_prelude_can_reference_custom_properties(&tokens[index].text)
        {
            index += 1;
            continue;
        }
        let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        let prelude_can_keep_roots = match tokens[prelude_end_index].kind {
            SyntaxKind::LeftBrace => matching_right_brace_index(tokens, prelude_end_index)
                .is_some_and(|close_index| block_is_reachable(prelude_end_index, close_index)),
            SyntaxKind::Semicolon => true,
            _ => false,
        };
        if prelude_can_keep_roots {
            let prelude_start = token_end(&tokens[index]);
            let prelude_end = token_start(&tokens[prelude_end_index]);
            for name in
                collect_custom_property_references_in_value(&source[prelude_start..prelude_end])
            {
                push_unique_string(&mut roots, name);
            }
        }
        index = prelude_end_index.saturating_add(1);
    }

    roots
}

fn collect_custom_property_roots_from_descriptor_at_rules(
    tokens: &[omena_parser::LexedToken],
) -> Vec<String> {
    let mut roots = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !descriptor_at_rule_can_reference_custom_properties(&tokens[index].text)
        {
            index += 1;
            continue;
        }
        let Some(block_start_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        if tokens[block_start_index].kind != SyntaxKind::LeftBrace {
            index = block_start_index.saturating_add(1);
            continue;
        }
        let Some(block_end_index) = matching_right_brace_index(tokens, block_start_index) else {
            break;
        };

        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            for name in collect_custom_property_references_in_value(&declaration.value) {
                push_unique_string(&mut roots, name);
            }
        }
        index = block_end_index + 1;
    }

    roots
}

fn descriptor_at_rule_can_reference_custom_properties(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@color-profile" | "@counter-style" | "@font-face" | "@font-palette-values" | "@page"
    )
}

fn at_rule_prelude_can_reference_custom_properties(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@media" | "@supports" | "@container" | "@custom-media" | "@scope"
    )
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
    replacements.extend(collect_static_custom_property_at_rule_prelude_replacements(
        source, tokens, &env,
    ));
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

fn collect_static_custom_property_at_rule_prelude_replacements(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    env: &CustomPropertyEnv,
) -> Vec<(usize, usize, String)> {
    let mut replacements = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !at_rule_prelude_can_reference_custom_properties(&tokens[index].text)
        {
            index += 1;
            continue;
        }
        let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1) else {
            break;
        };
        if !matches!(
            tokens[prelude_end_index].kind,
            SyntaxKind::LeftBrace | SyntaxKind::Semicolon
        ) {
            index = prelude_end_index.saturating_add(1);
            continue;
        }
        let start = token_end(&tokens[index]);
        let end = token_start(&tokens[prelude_end_index]);
        if start < end
            && let Some(resolved) =
                substitute_static_custom_property_references_in_value(&source[start..end], env)
        {
            replacements.push((start, end, resolved));
        }
        index = prelude_end_index.saturating_add(1);
    }

    replacements
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
    if let Some(value) = parse_css_wide_custom_property_env_value(value) {
        return Some(value);
    }
    if contains_runtime_dependent_css_function(value) {
        return None;
    }
    parse_static_var_value(value)
        .or_else(|| parse_static_composite_custom_property_env_value(value))
}

fn parse_css_wide_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "initial" => Some(CascadeValue::Initial),
        "inherit" | "unset" | "revert" | "revert-layer" => Some(CascadeValue::Inherit),
        _ => None,
    }
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
    let (name, fallback_arguments) = arguments.split_first()?;
    if !name.starts_with("--") {
        return None;
    }
    if fallback_arguments.is_empty() {
        return Some(CascadeValue::Var {
            name: name.to_string(),
            fallback: None,
        });
    }

    let fallback = parse_static_custom_property_env_value(&fallback_arguments.join(", "))?;
    Some(CascadeValue::Var {
        name: name.to_string(),
        fallback: Some(Box::new(fallback)),
    })
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
