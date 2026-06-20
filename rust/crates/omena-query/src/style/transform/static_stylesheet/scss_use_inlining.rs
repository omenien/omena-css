use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::super::stylesheet_evaluation::canonical_static_scss_variable_name;
use super::super::{
    apply_transform_source_replacements, transform_token_end, transform_token_start,
};
use super::{
    StaticScssModuleUseEvaluation, static_scss_identifier_char,
    static_scss_module_rule_source_name, static_scss_use_rule_semicolon,
};
use crate::OmenaParserStyleDialect;
use omena_syntax::SyntaxKind;
use std::{borrow::Cow, collections::BTreeSet};

pub(in crate::style::transform) fn derive_scss_use_aware_static_stylesheet_module_evaluation_source<
    'a,
>(
    style_source: &'a str,
    dialect: OmenaParserStyleDialect,
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> Cow<'a, str> {
    if !matches!(
        dialect,
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) || scss_module_uses.is_empty()
    {
        return Cow::Borrowed(style_source);
    }
    let source = replace_static_scss_namespaced_module_variables(style_source, scss_module_uses);
    let (source, mutation_count) = inline_static_scss_use_rules(&source, dialect, scss_module_uses);
    if mutation_count == 0 && source == style_source {
        Cow::Borrowed(style_source)
    } else {
        Cow::Owned(source)
    }
}

fn replace_static_scss_namespaced_module_variables(
    source: &str,
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> String {
    let mut output = source.to_string();
    for module_use in scss_module_uses {
        match module_use.namespace_kind {
            Some("alias") | Some("default") => {
                let Some(namespace) = module_use.namespace.as_deref() else {
                    continue;
                };
                for (name, value) in &module_use.variable_exports {
                    output = replace_static_scss_namespaced_variable_reference(
                        &output, namespace, name, value,
                    );
                }
            }
            Some("wildcard") => {
                for (name, value) in &module_use.variable_exports {
                    output = replace_static_scss_wildcard_variable_reference(&output, name, value);
                }
            }
            _ => {}
        }
    }
    output
}

fn replace_static_scss_namespaced_variable_reference(
    source: &str,
    namespace: &str,
    name: &str,
    value: &str,
) -> String {
    let needle = format!("{namespace}.$");
    if !source.contains(needle.as_str()) {
        return source.to_string();
    }
    let expected_name = canonical_static_scss_variable_name(name);

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0usize;
    while let Some(offset) = source[cursor..].find(needle.as_str()) {
        let start = cursor + offset;
        let name_start = start + needle.len();
        let end = static_scss_variable_reference_name_end(source, name_start);
        if end > name_start
            && canonical_static_scss_variable_name(&source[name_start..end]) == expected_name
            && static_scss_reference_boundary_is_safe(source, start, end)
        {
            output.push_str(&source[cursor..start]);
            output.push_str(value);
            cursor = end;
        } else {
            output.push_str(&source[cursor..name_start]);
            cursor = name_start;
        }
    }
    output.push_str(&source[cursor..]);
    output
}

fn static_scss_reference_boundary_is_safe(source: &str, start: usize, end: usize) -> bool {
    let before_safe = source[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| !static_scss_identifier_char(ch));
    let after_safe = source[end..]
        .chars()
        .next()
        .is_none_or(|ch| !static_scss_identifier_char(ch));
    before_safe && after_safe
}

fn replace_static_scss_wildcard_variable_reference(
    source: &str,
    name: &str,
    value: &str,
) -> String {
    let expected_name = canonical_static_scss_variable_name(name);

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0usize;
    while let Some(offset) = source[cursor..].find('$') {
        let start = cursor + offset;
        let name_start = start + '$'.len_utf8();
        let end = static_scss_variable_reference_name_end(source, name_start);
        if end > name_start
            && canonical_static_scss_variable_name(&source[name_start..end]) == expected_name
            && static_scss_reference_boundary_is_safe(source, start, end)
            && !static_scss_reference_has_namespace_prefix(source, start)
            && !static_scss_reference_is_declaration(source, end)
        {
            output.push_str(&source[cursor..start]);
            output.push_str(value);
            cursor = end;
        } else {
            output.push_str(&source[cursor..name_start]);
            cursor = name_start;
        }
    }
    output.push_str(&source[cursor..]);
    output
}

fn static_scss_variable_reference_name_end(source: &str, mut index: usize) -> usize {
    while index < source.len() {
        let Some(ch) = source[index..].chars().next() else {
            break;
        };
        if !static_scss_identifier_char(ch) {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn static_scss_reference_has_namespace_prefix(source: &str, start: usize) -> bool {
    source[..start]
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| ch == '.')
}

fn static_scss_reference_is_declaration(source: &str, end: usize) -> bool {
    source[end..]
        .chars()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| ch == ':')
}

fn inline_static_scss_use_rules(
    source: &str,
    dialect: OmenaParserStyleDialect,
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> (String, usize) {
    let lexed = lex_omena_query_omena_parser_style_source(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut emitted_module_identity_keys = BTreeSet::<String>::new();
    let mut depth = 0usize;
    let mut use_rule_ordinal = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@use") =>
            {
                let Some(end_index) = static_scss_use_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                let start = transform_token_start(&tokens[index]);
                let end = transform_token_end(&tokens[end_index]);
                if let Some(source_name) =
                    static_scss_module_rule_source_name(tokens, index + 1, end_index)
                {
                    let matching_module_use = scss_module_uses.iter().find(|module_use| {
                        module_use.use_rule_ordinal == use_rule_ordinal
                            && module_use.source == source_name
                    });
                    use_rule_ordinal += 1;
                    if let Some(module_use) = matching_module_use {
                        let replacement = if emitted_module_identity_keys
                            .insert(module_use.module_identity_key.clone())
                        {
                            module_use.module_output_css.clone()
                        } else {
                            String::new()
                        };
                        replacements.push((start, end, replacement));
                    }
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    apply_transform_source_replacements(source, replacements)
}
