use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::domains::{
    css_module_global::{
        CssModuleScopeBlockKind, collect_css_module_scope_blocks, css_module_scope_kind_for_range,
    },
    reachability::{normalize_reachable_class_name, selector_list_class_tree_shake_plan},
};
use crate::helpers::{
    ascii::{ascii_css_identifier_end, starts_with_ascii_case_insensitive},
    declarations::collect_simple_declarations_in_block,
    identifiers::css_identifier_text_is_plain,
    rules::collect_declaration_ordinary_rule_slices,
    selectors::{
        global_pseudo_function_end, local_pseudo_function_end, simple_class_selector_names,
    },
    source_rewrite::{remove_source_ranges, replace_source_ranges},
    tokens::{matching_right_brace_index, token_end, token_start},
    values::matching_function_end,
};
use crate::model::{
    TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
    TransformSemanticRemovalCandidate,
};

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

fn css_module_composes_resolution_exists(
    class_name: &str,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> bool {
    resolutions.iter().any(|resolution| {
        !resolution.exported_class_names.is_empty()
            && normalize_reachable_class_name(&resolution.local_class_name)
                .is_some_and(|resolved_name| resolved_name == class_name)
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
                let name_end = ascii_css_identifier_end(selector, name_start);
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

fn rewrite_local_composes_value(
    value: &str,
    rewrites: &[TransformClassNameRewriteV0],
) -> Option<String> {
    if value
        .split_whitespace()
        .any(|part| matches!(part, "from" | "global"))
        || value.contains(',')
    {
        return None;
    }
    let mut changed = false;
    let mut parts = Vec::new();
    for part in value.split_whitespace() {
        if let Some(global_name) = parse_global_composes_part(part) {
            changed = true;
            parts.push(global_name.to_string());
            continue;
        }
        if !css_identifier_text_is_plain(part) {
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

fn rewritten_class_name_for<'a>(
    class_name: &str,
    rewrites: &'a [TransformClassNameRewriteV0],
) -> Option<&'a str> {
    rewrites.iter().find_map(|rewrite| {
        let original_name = normalize_reachable_class_name(&rewrite.original_name)?;
        let rewritten_name = normalize_reachable_class_name(&rewrite.rewritten_name)?;
        (original_name == class_name).then_some(rewritten_name)
    })
}
