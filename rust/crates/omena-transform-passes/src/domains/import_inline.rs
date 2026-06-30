use std::collections::BTreeSet;

use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::{
    IrNodeIdV0, IrNodeKindV0, IrNodeV0, TransformIrV0, lower_transform_ir_from_source,
};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    TransformImportInlineV0, TransformLessInlineLiteralPlaceholderV0,
    helpers::{
        ascii::strip_ascii_prefix_ignore_case,
        ir_transaction::{
            TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
            TransformIrSourceReplacementV0, delete_ir_nodes_in_ir, replace_ir_nodes_in_ir,
        },
        source_rewrite::replace_source_ranges,
        tokens::{token_end, token_start},
    },
};

pub(crate) fn inline_css_imports_with_lexer(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> (String, usize) {
    inline_css_imports_with_lexer_mode(source, dialect, inlines, None)
}

pub(crate) fn inline_css_imports_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir =
        lower_transform_ir_from_source(source, dialect, "omena-transform-passes.import-inline");
    inline_css_imports_with_ir_transaction_on_ir(&mut ir, dialect, inlines)
}

pub(crate) fn inline_css_imports_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let replacements = collect_inline_css_import_replacements_from_ir(ir, dialect, inlines, None);
    let (deletions, replacements): (Vec<_>, Vec<_>) = replacements
        .into_iter()
        .partition(|replacement| replacement.replacement.is_empty());
    let deletion_node_ids = import_inline_deletion_node_ids(ir, deletions.as_slice())?;
    let (output, replacement_count) =
        replace_ir_nodes_in_ir(ir, "import-inline", replacements.as_slice())?;
    let (output, deletion_count) = if deletion_node_ids.is_empty() {
        (output, 0)
    } else {
        delete_ir_nodes_in_ir(ir, "import-inline", deletion_node_ids.as_slice())?
    };
    let mutation_count = replacement_count + deletion_count;
    if mutation_count > 0 {
        let source_id = ir.source_id.clone();
        *ir = lower_transform_ir_from_source(output.as_str(), dialect, source_id);
    }
    Ok((output, mutation_count))
}

pub(crate) fn inline_css_imports_for_static_module_evaluation_with_lexer(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> (String, usize, Vec<TransformLessInlineLiteralPlaceholderV0>) {
    let mut placeholders = Vec::new();
    let (output, mutation_count) =
        inline_css_imports_with_lexer_mode(source, dialect, inlines, Some(&mut placeholders));
    (output, mutation_count, placeholders)
}

pub(crate) fn restore_less_inline_literal_placeholders(
    source: &str,
    placeholders: &[TransformLessInlineLiteralPlaceholderV0],
) -> String {
    placeholders
        .iter()
        .fold(source.to_string(), |output, placeholder| {
            output.replace(&placeholder.placeholder, &placeholder.literal_css)
        })
}

fn inline_css_imports_with_lexer_mode(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
    inline_literal_placeholders: Option<&mut Vec<TransformLessInlineLiteralPlaceholderV0>>,
) -> (String, usize) {
    let replacements = collect_inline_css_import_replacements(
        source,
        dialect,
        inlines,
        inline_literal_placeholders,
    );
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

fn collect_inline_css_import_replacements(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
    mut inline_literal_placeholders: Option<&mut Vec<TransformLessInlineLiteralPlaceholderV0>>,
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut emitted_less_import_sources = BTreeSet::<String>::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@import") =>
            {
                let Some(end_index) = find_import_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                let start = token_start(&tokens[index]);
                let end = token_end(&tokens[end_index]);
                let rule_text = &source[start..end];
                let Some(import_rule) = parse_css_import_rule(rule_text) else {
                    index = end_index + 1;
                    continue;
                };
                if dialect == StyleDialect::Less && import_rule.css_passthrough {
                    let replacement = normalize_css_passthrough_import_rule(&import_rule);
                    replacements.push(TransformIrSourceReplacementV0 {
                        source_span_start: start,
                        source_span_end: end,
                        replacement,
                        kind: TransformIrReplacementKindV0::AtRule,
                    });
                    index = end_index + 1;
                    continue;
                }
                if let Some(replacement_css) =
                    inline_replacement_for_import_source(&import_rule.source, inlines)
                {
                    let mut replacement_css = replacement_css.to_string();
                    if dialect == StyleDialect::Less && import_rule.reference_only {
                        replacement_css =
                            filter_less_reference_import_replacement(&replacement_css);
                    }
                    if dialect == StyleDialect::Less
                        && import_rule.inline_literal
                        && let Some(placeholders) = inline_literal_placeholders.as_deref_mut()
                    {
                        let placeholder =
                            format!("/*__OMENA_LESS_INLINE_LITERAL_{}__*/", placeholders.len());
                        placeholders.push(TransformLessInlineLiteralPlaceholderV0 {
                            placeholder: placeholder.clone(),
                            literal_css: replacement_css,
                        });
                        replacement_css = placeholder;
                    }
                    if dialect == StyleDialect::Less
                        && !import_rule.allow_duplicate
                        && !emitted_less_import_sources.insert(import_rule.source.clone())
                    {
                        replacement_css.clear();
                    }
                    replacements.push(TransformIrSourceReplacementV0 {
                        source_span_start: start,
                        source_span_end: end,
                        replacement: wrap_import_replacement(&import_rule, &replacement_css),
                        kind: TransformIrReplacementKindV0::AtRule,
                    });
                } else if dialect == StyleDialect::Less && import_rule.optional {
                    replacements.push(TransformIrSourceReplacementV0 {
                        source_span_start: start,
                        source_span_end: end,
                        replacement: String::new(),
                        kind: TransformIrReplacementKindV0::AtRule,
                    });
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    replacements
}

fn collect_inline_css_import_replacements_from_ir(
    ir: &TransformIrV0,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
    mut inline_literal_placeholders: Option<&mut Vec<TransformLessInlineLiteralPlaceholderV0>>,
) -> Vec<TransformIrSourceReplacementV0> {
    let mut import_rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.parent.is_none() && node.kind == IrNodeKindV0::AtRule)
        .filter_map(|node| import_rule_from_ir(ir, node))
        .collect::<Vec<_>>();
    import_rules.sort_by_key(|rule| (rule.source_span_start, rule.source_span_end));

    let mut replacements = Vec::new();
    let mut emitted_less_import_sources = BTreeSet::<String>::new();
    for import_rule in import_rules {
        if dialect == StyleDialect::Less && import_rule.rule.css_passthrough {
            let replacement = normalize_css_passthrough_import_rule(&import_rule.rule);
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: import_rule.source_span_start,
                source_span_end: import_rule.source_span_end,
                replacement,
                kind: TransformIrReplacementKindV0::AtRule,
            });
            continue;
        }
        if let Some(replacement_css) =
            inline_replacement_for_import_source(&import_rule.rule.source, inlines)
        {
            let mut replacement_css = replacement_css.to_string();
            if dialect == StyleDialect::Less && import_rule.rule.reference_only {
                replacement_css = filter_less_reference_import_replacement(&replacement_css);
            }
            if dialect == StyleDialect::Less
                && import_rule.rule.inline_literal
                && let Some(placeholders) = inline_literal_placeholders.as_deref_mut()
            {
                let placeholder =
                    format!("/*__OMENA_LESS_INLINE_LITERAL_{}__*/", placeholders.len());
                placeholders.push(TransformLessInlineLiteralPlaceholderV0 {
                    placeholder: placeholder.clone(),
                    literal_css: replacement_css,
                });
                replacement_css = placeholder;
            }
            if dialect == StyleDialect::Less
                && !import_rule.rule.allow_duplicate
                && !emitted_less_import_sources.insert(import_rule.rule.source.clone())
            {
                replacement_css.clear();
            }
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: import_rule.source_span_start,
                source_span_end: import_rule.source_span_end,
                replacement: wrap_import_replacement(&import_rule.rule, &replacement_css),
                kind: TransformIrReplacementKindV0::AtRule,
            });
        } else if dialect == StyleDialect::Less && import_rule.rule.optional {
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: import_rule.source_span_start,
                source_span_end: import_rule.source_span_end,
                replacement: String::new(),
                kind: TransformIrReplacementKindV0::AtRule,
            });
        }
    }

    replacements
}

struct InlineImportIrRuleV0 {
    source_span_start: usize,
    source_span_end: usize,
    rule: CssImportRule,
}

fn import_rule_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> Option<InlineImportIrRuleV0> {
    let rule_text = ir
        .source_text()
        .get(node.source_span_start..node.source_span_end)?;
    let trimmed = rule_text.trim_start();
    if !trimmed
        .get(.."@import".len())
        .is_some_and(|keyword| keyword.eq_ignore_ascii_case("@import"))
    {
        return None;
    }
    Some(InlineImportIrRuleV0 {
        source_span_start: node.source_span_start,
        source_span_end: node.source_span_end,
        rule: parse_css_import_rule(rule_text)?,
    })
}

fn import_inline_deletion_node_ids(
    ir: &TransformIrV0,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<Vec<IrNodeIdV0>, TransformIrSourceReplacementErrorV0> {
    replacements
        .iter()
        .map(|replacement| import_inline_deletion_node_id(ir, replacement))
        .collect()
}

fn import_inline_deletion_node_id(
    ir: &TransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<IrNodeIdV0, TransformIrSourceReplacementErrorV0> {
    ir.nodes
        .iter()
        .find(|node| {
            !node.deleted
                && node.kind == IrNodeKindV0::AtRule
                && node.source_span_start == replacement.source_span_start
                && node.source_span_end == replacement.source_span_end
        })
        .map(|node| node.node_id)
        .ok_or_else(|| TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: TransformIrReplacementKindV0::AtRule,
            candidate_spans: ir
                .nodes
                .iter()
                .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
                .map(|node| (node.source_span_start, node.source_span_end))
                .collect(),
        })
}

fn find_import_rule_semicolon(
    tokens: &[omena_parser::LexedToken],
    at_import_index: usize,
) -> Option<usize> {
    let mut index = at_import_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssImportRule {
    source: String,
    layer_name: Option<String>,
    supports_condition: Option<String>,
    media_query: Option<String>,
    allow_duplicate: bool,
    reference_only: bool,
    optional: bool,
    inline_literal: bool,
    css_passthrough: bool,
}

fn parse_css_import_rule(rule_text: &str) -> Option<CssImportRule> {
    let rest = strip_ascii_prefix_ignore_case(rule_text.trim(), "@import")?;
    let rest = rest.trim().trim_end_matches(';').trim();
    if rest.is_empty() {
        return None;
    }
    let (rest, less_options) = strip_leading_less_import_options(rest);
    let (source, rest) = parse_css_import_source_prefix(rest)?;
    let mut rest = rest.trim();
    let mut layer_name = None;
    let mut supports_condition = None;

    loop {
        if let Some((layer, next_rest)) = parse_layer_import_option(rest) {
            layer_name = Some(layer);
            rest = next_rest.trim();
            continue;
        }
        if let Some((supports, next_rest)) = parse_function_prefix(rest, "supports") {
            supports_condition = Some(format!("({})", supports.trim()));
            rest = next_rest.trim();
            continue;
        }
        break;
    }

    Some(CssImportRule {
        source,
        layer_name,
        supports_condition,
        media_query: (!rest.is_empty()).then(|| rest.to_string()),
        allow_duplicate: less_options.allow_duplicate,
        reference_only: less_options.reference_only,
        optional: less_options.optional,
        inline_literal: less_options.inline_literal,
        css_passthrough: less_options.css_passthrough,
    })
}

fn parse_css_import_source_prefix(text: &str) -> Option<(String, &str)> {
    parse_quoted_css_string_prefix(text).or_else(|| parse_url_import_source_prefix(text))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LessImportOptions {
    allow_duplicate: bool,
    reference_only: bool,
    optional: bool,
    inline_literal: bool,
    css_passthrough: bool,
}

fn strip_leading_less_import_options(mut text: &str) -> (&str, LessImportOptions) {
    let mut options = LessImportOptions::default();
    loop {
        let rest = text.trim_start();
        let Some(after_left_paren) = rest.strip_prefix('(') else {
            return (rest, options);
        };
        let Some(close_index) = matching_function_close_index(after_left_paren) else {
            return (rest, options);
        };
        let option = after_left_paren[..close_index].trim();
        if option.is_empty()
            || !less_import_option_list_is_safe(option)
            || !less_import_option_list_is_known(option)
        {
            return (rest, options);
        }
        options.allow_duplicate |= less_import_option_list_contains(option, "multiple");
        options.reference_only |= less_import_option_list_contains(option, "reference");
        options.optional |= less_import_option_list_contains(option, "optional");
        options.inline_literal |= less_import_option_list_contains(option, "inline");
        options.css_passthrough |= less_import_option_list_contains(option, "css");
        text = &after_left_paren[close_index + 1..];
    }
}

fn less_import_option_list_is_safe(option: &str) -> bool {
    option.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | ',')
    })
}

fn less_import_option_list_is_known(option: &str) -> bool {
    option.split(',').map(str::trim).all(|entry| {
        matches!(
            entry.to_ascii_lowercase().as_str(),
            "css" | "inline" | "less" | "multiple" | "once" | "optional" | "reference"
        )
    })
}

fn less_import_option_list_contains(option: &str, expected: &str) -> bool {
    option
        .split(',')
        .map(str::trim)
        .any(|entry| entry.eq_ignore_ascii_case(expected))
}

fn filter_less_reference_import_replacement(source: &str) -> String {
    let mut output = String::new();
    let mut statement_start = 0usize;
    let mut depth = 0usize;
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => {
                let statement = source[statement_start..=index].trim();
                if less_reference_import_statement_is_static_binding(statement) {
                    if !output.is_empty() {
                        output.push(' ');
                    }
                    output.push_str(statement);
                }
                statement_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    output
}

fn less_reference_import_statement_is_static_binding(statement: &str) -> bool {
    let statement = statement.trim();
    if statement.is_empty() || statement.contains('{') || statement.contains('}') {
        return false;
    }
    let Some(first_char) = statement.chars().next() else {
        return false;
    };
    matches!(first_char, '@' | '$') && top_level_colon_index(statement).is_some()
}

fn top_level_colon_index(content: &str) -> Option<usize> {
    let mut delimiter_stack = Vec::<char>::new();
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, ch) in content.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => delimiter_stack.push(ch),
            ')' if delimiter_stack.last() == Some(&'(') => {
                delimiter_stack.pop();
            }
            ']' if delimiter_stack.last() == Some(&'[') => {
                delimiter_stack.pop();
            }
            ':' if delimiter_stack.is_empty() => return Some(index),
            _ => {}
        }
    }
    None
}

fn parse_quoted_css_string_prefix(text: &str) -> Option<(String, &str)> {
    let mut chars = text.char_indices();
    let (_, quote) = chars.next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    let mut escaped = false;
    let mut output = String::new();
    for (index, ch) in chars {
        if escaped {
            output.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            let end = index + ch.len_utf8();
            return Some((output, &text[end..]));
        }
        output.push(ch);
    }
    None
}

fn parse_url_import_source_prefix(text: &str) -> Option<(String, &str)> {
    let rest = strip_ascii_prefix_ignore_case(text, "url(")?;
    let close_index = matching_function_close_index(rest)?;
    let inner = rest[..close_index].trim();
    if let Some((source, trailing)) = parse_quoted_css_string_prefix(inner)
        && trailing.trim().is_empty()
    {
        return Some((source, &rest[close_index + 1..]));
    }
    if inner.is_empty()
        || inner
            .chars()
            .any(|ch| ch.is_ascii_whitespace() || matches!(ch, '"' | '\'' | '(' | ')'))
    {
        return None;
    }
    Some((inner.to_string(), &rest[close_index + 1..]))
}

fn parse_layer_import_option(text: &str) -> Option<(String, &str)> {
    if let Some((layer, rest)) = parse_function_prefix(text, "layer") {
        return Some((layer.trim().to_string(), rest));
    }
    let rest = strip_ascii_prefix_ignore_case(text, "layer")?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some((String::new(), rest))
}

fn parse_function_prefix<'a>(text: &'a str, name: &str) -> Option<(String, &'a str)> {
    let rest = strip_ascii_prefix_ignore_case(text.trim_start(), name)?;
    let rest = rest.strip_prefix('(')?;
    let close_index = matching_function_close_index(rest)?;
    Some((rest[..close_index].to_string(), &rest[close_index + 1..]))
}

fn matching_function_close_index(text: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
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

fn wrap_import_replacement(import_rule: &CssImportRule, replacement_css: &str) -> String {
    let mut output = replacement_css.to_string();
    if let Some(layer_name) = &import_rule.layer_name {
        output = if layer_name.is_empty() {
            format!("@layer {{ {output} }}")
        } else {
            format!("@layer {layer_name} {{ {output} }}")
        };
    }
    if let Some(supports_condition) = &import_rule.supports_condition {
        output = format!("@supports {supports_condition} {{ {output} }}");
    }
    if let Some(media_query) = &import_rule.media_query {
        output = format!("@media {media_query} {{ {output} }}");
    }
    output
}

fn normalize_css_passthrough_import_rule(import_rule: &CssImportRule) -> String {
    let mut output = format!("@import {}", quote_css_import_source(&import_rule.source));
    if let Some(layer_name) = &import_rule.layer_name {
        if layer_name.is_empty() {
            output.push_str(" layer");
        } else {
            output.push_str(" layer(");
            output.push_str(layer_name);
            output.push(')');
        }
    }
    if let Some(supports_condition) = &import_rule.supports_condition {
        output.push_str(" supports");
        output.push_str(supports_condition);
    }
    if let Some(media_query) = &import_rule.media_query {
        output.push(' ');
        output.push_str(media_query);
    }
    output.push(';');
    output
}

fn quote_css_import_source(source: &str) -> String {
    let mut output = String::from("\"");
    for ch in source.chars() {
        if matches!(ch, '"' | '\\') {
            output.push('\\');
        }
        output.push(ch);
    }
    output.push('"');
    output
}

fn inline_replacement_for_import_source<'a>(
    import_source: &str,
    inlines: &'a [TransformImportInlineV0],
) -> Option<&'a str> {
    inlines
        .iter()
        .find(|inline| inline.import_source == import_source)
        .map(|inline| inline.replacement_css.as_str())
}
