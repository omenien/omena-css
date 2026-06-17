use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::domains::{
    css_module_global::{
        CssModuleScopeBlockKind, collect_css_module_scope_blocks, css_module_scope_kind_for_range,
    },
    reachability::{
        class_name_is_reachable, normalize_reachable_class_name,
        selector_list_class_tree_shake_plan,
    },
};
use crate::helpers::{
    ascii::starts_with_ascii_case_insensitive,
    blocks::at_rule_prelude_end_index,
    collections::push_unique_string,
    declarations::collect_simple_declarations_in_block,
    identifiers::{css_identifier_names_match, css_identifier_text_is_plain},
    rules::collect_declaration_ordinary_rule_slices,
    selectors::{
        css_class_selector_name_end, global_pseudo_function_end, local_pseudo_function_end,
        simple_class_selector_names,
    },
    source_rewrite::{remove_source_ranges, replace_source_ranges},
    tokens::{matching_right_brace_index, token_end, token_start},
    values::matching_function_end,
};
use crate::model::{
    TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
    TransformSemanticRemovalCandidate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalCssModuleComposesEdge {
    owner_class_name: String,
    local_target_class_names: Vec<String>,
    exported_class_names: Vec<String>,
}

pub(crate) fn tree_shake_css_class_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let mut removals = Vec::new();
    let mut replacements = Vec::new();

    for rule in &rules {
        if css_module_scope_kind_for_range(rule.start, rule.end, &scope_blocks)
            == Some(CssModuleScopeBlockKind::Global)
        {
            continue;
        }
        let Some(plan) = selector_list_class_tree_shake_plan(&rule.selector, reachable_class_names)
        else {
            continue;
        };
        removals.push(TransformSemanticRemovalCandidate {
            symbol_kind: "class",
            name: plan.unreachable_owner_class_names.join(","),
            source_span_start: rule.start,
            source_span_end: rule.end,
            reason: "selector owner classes were absent from the closed-style-world reachable class set",
        });
        if let Some(reachable_selector) = plan.reachable_selector {
            replacements.push((
                rule.start,
                rule.block_start,
                format!("{reachable_selector} "),
            ));
        } else {
            replacements.push((rule.start, rule.end, String::new()));
        }
    }

    let (output, _) = replace_source_ranges(source, &replacements);
    (output, removals)
}

pub(crate) fn strip_resolved_css_module_composes_with_lexer(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let mut ranges = Vec::new();

    for rule in &rules {
        if css_module_scope_kind_for_range(rule.start, rule.end, &scope_blocks)
            == Some(CssModuleScopeBlockKind::Global)
        {
            continue;
        }
        let Some(class_names) = simple_class_selector_names(&rule.selector) else {
            continue;
        };
        if !class_names
            .iter()
            .all(|class_name| css_module_composes_resolution_exists(class_name, resolutions))
        {
            continue;
        }
        let Some(block_start_index) = tokens.iter().position(|token| {
            token.kind == SyntaxKind::LeftBrace && token_start(token) == rule.block_start
        }) else {
            continue;
        };
        let Some(block_end_index) = matching_right_brace_index(tokens, block_start_index) else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property == "composes" {
                ranges.push((declaration.start, declaration.end));
            }
        }
    }

    remove_source_ranges(source, &ranges)
}

pub(crate) fn rewrite_css_module_class_names_with_lexer(
    source: &str,
    dialect: StyleDialect,
    rewrites: &[TransformClassNameRewriteV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let mut replacements = Vec::new();

    for block in &scope_blocks {
        replacements.push((block.start, block.body_start, String::new()));
        replacements.push((block.body_end, block.end, String::new()));
    }

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && (tokens[index].text.eq_ignore_ascii_case("@scope")
                || tokens[index].text.eq_ignore_ascii_case("@supports"))
            && let Some(prelude_end_index) = at_rule_prelude_end_index(tokens, index + 1)
        {
            let prelude_start = token_end(&tokens[index]);
            let prelude_end = token_start(&tokens[prelude_end_index]);
            let prelude = &source[prelude_start..prelude_end];
            let rewritten_prelude = if tokens[index].text.eq_ignore_ascii_case("@scope") {
                rewrite_class_selectors_in_selector(prelude, rewrites)
            } else {
                rewrite_supports_selector_functions(prelude, rewrites)
            };
            if css_module_scope_kind_for_range(prelude_start, prelude_end, &scope_blocks)
                != Some(CssModuleScopeBlockKind::Global)
                && let Some(rewritten_prelude) = rewritten_prelude
            {
                replacements.push((prelude_start, prelude_end, rewritten_prelude));
            }
            index = prelude_end_index;
            continue;
        }
        index += 1;
    }

    for rule in &rules {
        if css_module_scope_kind_for_range(rule.start, rule.end, &scope_blocks)
            == Some(CssModuleScopeBlockKind::Global)
        {
            continue;
        }
        let Some(rewritten_selector) =
            rewrite_class_selectors_in_selector(&rule.selector, rewrites)
        else {
            continue;
        };
        replacements.push((rule.start, rule.block_start, rewritten_selector));
    }

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            if css_module_scope_kind_for_range(
                token_start(&tokens[index]),
                token_end(&tokens[close_index]),
                &scope_blocks,
            ) == Some(CssModuleScopeBlockKind::Global)
            {
                index = close_index + 1;
                continue;
            }
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                if declaration.property != "composes" {
                    continue;
                }
                let Some(rewritten_value) =
                    rewrite_local_composes_value(&declaration.value, rewrites)
                else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("composes: {rewritten_value};"),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

pub(crate) fn reachable_class_names_with_local_composes(
    source: &str,
    dialect: StyleDialect,
    reachable_class_names: &[String],
) -> Vec<String> {
    let edges = collect_local_css_module_composes_edges(source, dialect);

    let mut expanded = reachable_class_names.to_vec();
    let mut changed = true;
    while changed {
        changed = false;
        for edge in &edges {
            if !class_name_is_reachable(&edge.owner_class_name, &expanded) {
                continue;
            }
            for target_class_name in &edge.local_target_class_names {
                if !class_name_is_reachable(target_class_name, &expanded) {
                    expanded.push(target_class_name.clone());
                    changed = true;
                }
            }
        }
    }

    expanded.sort();
    expanded.dedup();
    expanded
}

pub(crate) fn local_css_module_composes_resolutions_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformCssModuleComposesResolutionV0> {
    let edges = collect_local_css_module_composes_edges(source, dialect);
    let graph = edges
        .iter()
        .map(|edge| (edge.owner_class_name.clone(), edge.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut resolutions = Vec::new();

    for owner_class_name in graph.keys() {
        let mut exported_class_names = Vec::<String>::new();
        let mut visited_class_names = BTreeSet::<String>::new();
        let mut queue = VecDeque::from([owner_class_name.clone()]);
        while let Some(class_name) = queue.pop_front() {
            if !visited_class_names.insert(class_name.clone()) {
                continue;
            }
            push_unique_string(&mut exported_class_names, class_name.clone());
            let Some(edge) = graph.get(&class_name) else {
                continue;
            };
            for exported_class_name in &edge.exported_class_names {
                push_unique_string(&mut exported_class_names, exported_class_name.clone());
            }
            for target_class_name in &edge.local_target_class_names {
                queue.push_back(target_class_name.clone());
            }
        }
        if exported_class_names.len() <= 1 {
            continue;
        }
        resolutions.push(TransformCssModuleComposesResolutionV0 {
            local_class_name: owner_class_name.clone(),
            exported_class_names,
        });
    }

    resolutions
}

fn css_module_composes_resolution_exists(
    class_name: &str,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> bool {
    resolutions.iter().any(|resolution| {
        !resolution.exported_class_names.is_empty()
            && normalize_reachable_class_name(&resolution.local_class_name)
                .is_some_and(|resolved_name| css_identifier_names_match(resolved_name, class_name))
            && resolution
                .exported_class_names
                .iter()
                .all(|name| normalize_reachable_class_name(name).is_some())
    })
}

fn rewrite_class_selectors_in_selector(
    selector: &str,
    rewrites: &[TransformClassNameRewriteV0],
) -> Option<String> {
    let mut output = String::with_capacity(selector.len());
    let mut index = 0usize;
    let mut changed = false;
    let mut quote: Option<char> = None;
    let mut bracket_depth = 0usize;

    while index < selector.len() {
        let ch = selector[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            output.push(ch);
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = selector[index..].chars().next() {
                    output.push(escaped);
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if bracket_depth == 0
            && let Some(global_end) = global_pseudo_function_end(selector, index)
        {
            let inner_start = index + ":global(".len();
            let inner_end = global_end.saturating_sub(1);
            output.push_str(&selector[inner_start..inner_end]);
            index = global_end;
            changed = true;
            continue;
        }
        if bracket_depth == 0
            && let Some(local_end) = local_pseudo_function_end(selector, index)
        {
            let inner_start = index + ":local(".len();
            let inner_end = local_end.saturating_sub(1);
            let inner = &selector[inner_start..inner_end];
            if let Some(rewritten_inner) = rewrite_class_selectors_in_selector(inner, rewrites) {
                output.push_str(&rewritten_inner);
            } else {
                output.push_str(inner);
            }
            index = local_end;
            changed = true;
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                output.push(ch);
                index += ch.len_utf8();
            }
            '[' => {
                bracket_depth += 1;
                output.push(ch);
                index += ch.len_utf8();
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                output.push(ch);
                index += ch.len_utf8();
            }
            '.' if bracket_depth == 0 => {
                let name_start = index + ch.len_utf8();
                let name_end = css_class_selector_name_end(selector, name_start);
                if name_end == name_start {
                    output.push(ch);
                    index += ch.len_utf8();
                    continue;
                }
                let class_name = &selector[name_start..name_end];
                if let Some(rewritten_name) = rewritten_class_name_for(class_name, rewrites) {
                    output.push('.');
                    output.push_str(rewritten_name);
                    index = name_end;
                    changed = true;
                } else {
                    output.push_str(&selector[index..name_end]);
                    index = name_end;
                }
            }
            _ => {
                output.push(ch);
                index += ch.len_utf8();
            }
        }
    }

    changed.then_some(output)
}

fn rewrite_supports_selector_functions(
    prelude: &str,
    rewrites: &[TransformClassNameRewriteV0],
) -> Option<String> {
    let mut output = String::with_capacity(prelude.len());
    let mut index = 0usize;
    let mut changed = false;
    let mut quote: Option<char> = None;

    while index < prelude.len() {
        let ch = prelude[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            output.push(ch);
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = prelude[index..].chars().next() {
                    output.push(escaped);
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }

        if starts_with_css_function_name(prelude, index, "selector") {
            let open_paren_index = index + "selector".len();
            let function_end = matching_function_end(prelude, open_paren_index)?;
            let inner_start = open_paren_index + 1;
            let inner_end = function_end.saturating_sub(1);
            output.push_str(&prelude[index..inner_start]);
            let inner = &prelude[inner_start..inner_end];
            if let Some(rewritten_inner) = rewrite_class_selectors_in_selector(inner, rewrites) {
                output.push_str(&rewritten_inner);
                changed = true;
            } else {
                output.push_str(inner);
            }
            output.push(')');
            index = function_end;
            continue;
        }

        output.push(ch);
        index += ch.len_utf8();
    }

    changed.then_some(output)
}

fn starts_with_css_function_name(text: &str, index: usize, name: &str) -> bool {
    if index > 0
        && let Some(previous) = text[..index].chars().next_back()
        && css_function_name_codepoint(previous)
    {
        return false;
    }
    let Some(candidate) = text.get(index..index + name.len()) else {
        return false;
    };
    candidate.eq_ignore_ascii_case(name) && text[index + name.len()..].starts_with('(')
}

fn css_function_name_codepoint(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

fn rewrite_local_composes_value(
    value: &str,
    rewrites: &[TransformClassNameRewriteV0],
) -> Option<String> {
    if value
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case("from") || part.eq_ignore_ascii_case("global"))
        || value.contains(',')
    {
        return None;
    }
    let mut changed = false;
    let mut parts = Vec::new();
    for part in value.split_whitespace() {
        if parse_global_composes_part(part).is_some() {
            parts.push(part.to_string());
            continue;
        }
        if !css_identifier_text_is_plain(part) && !part.contains('\\') {
            return None;
        }
        if let Some(rewritten_name) = rewritten_class_name_for(part, rewrites) {
            changed = true;
            parts.push(rewritten_name.to_string());
        } else {
            parts.push(part.to_string());
        }
    }
    changed.then(|| parts.join(" "))
}

fn parse_global_composes_part(part: &str) -> Option<&str> {
    const GLOBAL_PREFIX: &str = "global(";
    if !starts_with_ascii_case_insensitive(part, GLOBAL_PREFIX) {
        return None;
    }
    let end = matching_function_end(part, GLOBAL_PREFIX.len() - 1)?;
    if end != part.len() {
        return None;
    }
    let inner = part[GLOBAL_PREFIX.len()..end.saturating_sub(1)].trim();
    let class_name = normalize_reachable_class_name(inner)?;
    css_identifier_text_is_plain(class_name).then_some(class_name)
}

fn local_composes_target_names(value: &str) -> Vec<String> {
    local_composes_names(value, false)
}

fn local_composes_export_names(value: &str) -> Vec<String> {
    local_composes_names(value, true)
}

fn local_composes_names(value: &str, include_global_function_names: bool) -> Vec<String> {
    if value.contains(',') {
        return Vec::new();
    }
    let parts = value.split_whitespace().collect::<Vec<_>>();
    if parts
        .iter()
        .any(|part| part.eq_ignore_ascii_case("from") || part.eq_ignore_ascii_case("global"))
    {
        return Vec::new();
    }

    let mut names = Vec::new();
    for part in parts {
        if let Some(global_name) = parse_global_composes_part(part) {
            if include_global_function_names {
                push_unique_string(&mut names, global_name.to_string());
            }
            continue;
        }
        if !css_identifier_text_is_plain(part) && !part.contains('\\') {
            return Vec::new();
        }
        push_unique_string(&mut names, part.to_string());
    }
    names
}

fn collect_local_css_module_composes_edges(
    source: &str,
    dialect: StyleDialect,
) -> Vec<LocalCssModuleComposesEdge> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let mut edges = Vec::new();

    for rule in &rules {
        if css_module_scope_kind_for_range(rule.start, rule.end, &scope_blocks)
            == Some(CssModuleScopeBlockKind::Global)
        {
            continue;
        }
        let Some(owner_class_names) = simple_class_selector_names(&rule.selector) else {
            continue;
        };
        let Some(block_start_index) = tokens.iter().position(|token| {
            token.kind == SyntaxKind::LeftBrace && token_start(token) == rule.block_start
        }) else {
            continue;
        };
        let Some(block_end_index) = matching_right_brace_index(tokens, block_start_index) else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property != "composes" {
                continue;
            }
            let local_target_class_names = local_composes_target_names(&declaration.value);
            let exported_target_class_names = local_composes_export_names(&declaration.value);
            if local_target_class_names.is_empty() && exported_target_class_names.is_empty() {
                continue;
            }
            for owner_class_name in &owner_class_names {
                let mut exported_class_names = vec![owner_class_name.clone()];
                for target_class_name in &exported_target_class_names {
                    push_unique_string(&mut exported_class_names, target_class_name.clone());
                }
                edges.push(LocalCssModuleComposesEdge {
                    owner_class_name: owner_class_name.clone(),
                    local_target_class_names: local_target_class_names.clone(),
                    exported_class_names,
                });
            }
        }
    }

    edges
}

fn rewritten_class_name_for<'a>(
    class_name: &str,
    rewrites: &'a [TransformClassNameRewriteV0],
) -> Option<&'a str> {
    rewrites.iter().find_map(|rewrite| {
        let original_name = normalize_reachable_class_name(&rewrite.original_name)?;
        let rewritten_name = normalize_reachable_class_name(&rewrite.rewritten_name)?;
        css_identifier_names_match(original_name, class_name).then_some(rewritten_name)
    })
}
