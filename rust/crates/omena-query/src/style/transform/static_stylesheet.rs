use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::super::stylesheet_evaluation::{
    canonical_static_scss_variable_name,
    derive_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation, static_scss_variable_names_equal,
};
use super::*;
use omena_syntax::SyntaxKind;
use omena_transform_passes::{
    TransformImportInlineV0, TransformLessInlineLiteralPlaceholderV0, TransformModuleEvaluationV0,
    inline_css_imports, inline_css_imports_for_static_module_evaluation,
    restore_less_inline_literal_placeholders,
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

mod scss_variable_overrides;

pub(super) fn derive_static_stylesheet_module_evaluation_for_transform_context(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
    import_inlines: &[TransformImportInlineV0],
    scss_module_uses: &[StaticScssModuleUseEvaluation],
) -> Option<TransformModuleEvaluationV0> {
    let import_aware_source = derive_import_aware_static_stylesheet_module_evaluation_source(
        style_source,
        dialect,
        import_inlines,
    );
    let evaluation_source = derive_scss_use_aware_static_stylesheet_module_evaluation_source(
        import_aware_source.source.as_ref(),
        dialect,
        scss_module_uses,
    );
    if let Some(evaluation) =
        derive_static_stylesheet_module_evaluation(evaluation_source.as_ref(), dialect)
    {
        return Some(TransformModuleEvaluationV0 {
            evaluator: evaluation.evaluator,
            evaluated_css: restore_less_inline_literal_placeholders(
                evaluation.evaluated_css.as_str(),
                &import_aware_source.less_inline_literal_placeholders,
            ),
        });
    }
    (evaluation_source.as_ref() != style_source).then(|| TransformModuleEvaluationV0 {
        evaluator: static_stylesheet_module_system_evaluator_label(dialect).to_string(),
        evaluated_css: restore_less_inline_literal_placeholders(
            evaluation_source.as_ref(),
            &import_aware_source.less_inline_literal_placeholders,
        ),
    })
}

struct StaticModuleEvaluationSource<'a> {
    source: Cow<'a, str>,
    less_inline_literal_placeholders: Vec<TransformLessInlineLiteralPlaceholderV0>,
}

fn derive_import_aware_static_stylesheet_module_evaluation_source<'a>(
    style_source: &'a str,
    dialect: OmenaParserStyleDialect,
    import_inlines: &[TransformImportInlineV0],
) -> StaticModuleEvaluationSource<'a> {
    if import_inlines.is_empty() {
        return StaticModuleEvaluationSource {
            source: Cow::Borrowed(style_source),
            less_inline_literal_placeholders: Vec::new(),
        };
    }
    let (inlined_source, mutation_count, less_inline_literal_placeholders) = if dialect
        == OmenaParserStyleDialect::Less
    {
        let (inlined_source, mutation_count, placeholders) =
            inline_css_imports_for_static_module_evaluation(style_source, dialect, import_inlines);
        (inlined_source, mutation_count, placeholders)
    } else {
        let (inlined_source, mutation_count) =
            inline_css_imports(style_source, dialect, import_inlines);
        (inlined_source, mutation_count, Vec::new())
    };
    if mutation_count == 0 {
        StaticModuleEvaluationSource {
            source: Cow::Borrowed(style_source),
            less_inline_literal_placeholders,
        }
    } else {
        StaticModuleEvaluationSource {
            source: Cow::Owned(inlined_source),
            less_inline_literal_placeholders,
        }
    }
}

fn static_stylesheet_module_system_evaluator_label(
    dialect: OmenaParserStyleDialect,
) -> &'static str {
    match dialect {
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass => {
            "omena-query-static-scss-module-system-evaluator"
        }
        OmenaParserStyleDialect::Less => "omena-query-static-less-module-system-evaluator",
        OmenaParserStyleDialect::Css => "omena-query-static-css-module-system-evaluator",
    }
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssModuleUseEvaluation {
    source: String,
    module_identity_key: String,
    namespace_kind: Option<&'static str>,
    namespace: Option<String>,
    evaluated_css: String,
    variable_exports: BTreeMap<String, String>,
}

pub(super) fn derive_static_scss_module_use_evaluations_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<StaticScssModuleUseEvaluation> {
    if !matches!(
        omena_parser_dialect_for_style_path(entry.style_path.as_str()),
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let mut emitted_module_identity_keys = BTreeSet::new();
    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassUse")
        .filter(|edge| {
            matches!(
                edge.namespace_kind,
                Some("alias") | Some("default") | Some("wildcard")
            )
        })
        .filter_map(|edge| {
            let resolved = resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
                package_manifests,
            )?;
            let source = source_by_path.get(resolved.as_str())?;
            let variable_overrides = derive_static_scss_module_rule_variable_overrides(
                entry.style_source.as_str(),
                "@use",
                edge.source.as_str(),
            );
            let module_identity_key =
                static_scss_module_instance_identity_key(resolved.as_str(), &variable_overrides);
            let module_context = {
                let mut visited = BTreeSet::new();
                let mut derive_context = StaticScssModuleDeriveContext {
                    available_style_paths,
                    source_by_path,
                    package_manifests,
                    visited: &mut visited,
                    emitted_module_identity_keys: &mut emitted_module_identity_keys,
                };
                derive_static_scss_module_context_for_transform_context(
                    resolved.as_str(),
                    source,
                    &variable_overrides,
                    &mut derive_context,
                )
            };
            let evaluated_css = if emitted_module_identity_keys.insert(module_identity_key.clone())
            {
                module_context.evaluated_css
            } else {
                String::new()
            };
            Some(StaticScssModuleUseEvaluation {
                source: edge.source.clone(),
                module_identity_key,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                evaluated_css,
                variable_exports: module_context.variable_exports,
            })
        })
        .collect()
}

#[derive(Debug, Clone)]
struct StaticScssModuleContext {
    evaluated_css: String,
    variable_exports: BTreeMap<String, String>,
}

struct StaticScssModuleDeriveContext<'a> {
    available_style_paths: &'a BTreeSet<&'a str>,
    source_by_path: &'a BTreeMap<String, String>,
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    visited: &'a mut BTreeSet<String>,
    emitted_module_identity_keys: &'a mut BTreeSet<String>,
}

fn static_scss_module_instance_identity_key(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    let mut key = format!("path:{}:{canonical_path}", canonical_path.len());
    if variable_overrides.is_empty() {
        key.push_str("|with:none");
        return key;
    }
    key.push_str("|with");
    for (name, value) in variable_overrides {
        key.push('|');
        key.push_str(name.len().to_string().as_str());
        key.push(':');
        key.push_str(name);
        key.push('=');
        key.push_str(value.len().to_string().as_str());
        key.push(':');
        key.push_str(value);
    }
    key
}

fn derive_static_scss_module_context_for_transform_context(
    style_path: &str,
    style_source: &str,
    variable_overrides: &BTreeMap<String, String>,
    context: &mut StaticScssModuleDeriveContext<'_>,
) -> StaticScssModuleContext {
    let module_identity_key =
        static_scss_module_instance_identity_key(style_path, variable_overrides);
    if !context.visited.insert(module_identity_key.clone()) {
        return StaticScssModuleContext {
            evaluated_css: String::new(),
            variable_exports: BTreeMap::new(),
        };
    }

    let style_source =
        apply_static_scss_module_variable_overrides(style_source, variable_overrides);
    let style_source = style_source.as_ref();

    let forward_evaluations = derive_static_scss_module_forward_evaluations_for_transform_context(
        style_path,
        style_source,
        context,
    );
    let mut variable_exports = derive_static_scss_stylesheet_module_variable_exports(style_source);
    for forward in &forward_evaluations {
        for (name, value) in &forward.variable_exports {
            variable_exports
                .entry(name.clone())
                .or_insert_with(|| value.clone());
        }
    }

    let (evaluation_source, forward_mutation_count) = inline_static_scss_forward_rules(
        style_source,
        OmenaParserStyleDialect::Scss,
        &forward_evaluations,
        context.emitted_module_identity_keys,
    );
    let evaluated_css = derive_static_stylesheet_module_evaluation(
        evaluation_source.as_str(),
        OmenaParserStyleDialect::Scss,
    )
    .map(|evaluation| evaluation.evaluated_css)
    .unwrap_or_else(|| {
        if forward_mutation_count > 0 {
            evaluation_source
        } else {
            style_source.to_string()
        }
    });

    context.visited.remove(&module_identity_key);
    StaticScssModuleContext {
        evaluated_css,
        variable_exports,
    }
}

#[derive(Debug, Clone)]
struct StaticScssModuleForwardEvaluation {
    source: String,
    module_identity_key: String,
    evaluated_css: String,
    variable_exports: BTreeMap<String, String>,
}

fn derive_static_scss_module_forward_evaluations_for_transform_context(
    style_path: &str,
    style_source: &str,
    context: &mut StaticScssModuleDeriveContext<'_>,
) -> Vec<StaticScssModuleForwardEvaluation> {
    let facts =
        summarize_omena_query_omena_parser_style_facts(style_source, OmenaParserStyleDialect::Scss);

    facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassForward")
        .filter_map(|edge| {
            let resolved = resolve_style_module_source(
                style_path,
                edge.source.as_str(),
                context.available_style_paths,
                context.package_manifests,
            )?;
            let source = context.source_by_path.get(resolved.as_str())?;
            let variable_overrides = derive_static_scss_module_rule_variable_overrides(
                style_source,
                "@forward",
                edge.source.as_str(),
            );
            let module_identity_key =
                static_scss_module_instance_identity_key(resolved.as_str(), &variable_overrides);
            let export_prefix =
                derive_static_scss_forward_export_prefix(style_source, edge.source.as_str());
            let module_context = derive_static_scss_module_context_for_transform_context(
                resolved.as_str(),
                source,
                &variable_overrides,
                context,
            );
            Some(StaticScssModuleForwardEvaluation {
                source: edge.source.clone(),
                module_identity_key,
                evaluated_css: module_context.evaluated_css,
                variable_exports: filter_static_scss_forward_exports(
                    prefix_static_scss_forward_exports(
                        module_context.variable_exports,
                        export_prefix.as_deref(),
                    ),
                    edge.visibility_filter_kind,
                    &edge.visibility_filter_names,
                ),
            })
        })
        .collect()
}

fn filter_static_scss_forward_exports(
    exports: BTreeMap<String, String>,
    filter_kind: Option<&'static str>,
    filter_names: &[String],
) -> BTreeMap<String, String> {
    match filter_kind {
        Some("show") => exports
            .into_iter()
            .filter(|(name, _)| {
                filter_names
                    .iter()
                    .any(|filter| static_scss_variable_names_equal(filter, name))
            })
            .collect(),
        Some("hide") => exports
            .into_iter()
            .filter(|(name, _)| {
                !filter_names
                    .iter()
                    .any(|filter| static_scss_variable_names_equal(filter, name))
            })
            .collect(),
        _ => exports,
    }
}

fn prefix_static_scss_forward_exports(
    exports: BTreeMap<String, String>,
    prefix: Option<&str>,
) -> BTreeMap<String, String> {
    let Some(prefix) = prefix else {
        return exports;
    };
    exports
        .into_iter()
        .map(|(name, value)| (prefix.replace('*', name.as_str()), value))
        .collect()
}

fn apply_static_scss_module_variable_overrides<'a>(
    style_source: &'a str,
    variable_overrides: &BTreeMap<String, String>,
) -> Cow<'a, str> {
    if variable_overrides.is_empty() {
        return Cow::Borrowed(style_source);
    }
    let configurable_names =
        derive_static_scss_stylesheet_module_configurable_variable_names(style_source);
    if !variable_overrides
        .keys()
        .all(|name| configurable_names.contains(name))
    {
        return Cow::Borrowed(style_source);
    }

    let mut source = String::new();
    for (name, value) in variable_overrides {
        source.push('$');
        source.push_str(name);
        source.push_str(": ");
        source.push_str(value);
        source.push_str("; ");
    }
    source.push_str(style_source);
    Cow::Owned(source)
}

fn derive_scss_use_aware_static_stylesheet_module_evaluation_source<'a>(
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

fn static_scss_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
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
                    && let Some(module_use) = scss_module_uses
                        .iter()
                        .find(|module_use| module_use.source == source_name)
                {
                    let replacement = if emitted_module_identity_keys
                        .insert(module_use.module_identity_key.clone())
                    {
                        module_use.evaluated_css.clone()
                    } else {
                        String::new()
                    };
                    replacements.push((start, end, replacement));
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

fn inline_static_scss_forward_rules(
    source: &str,
    dialect: OmenaParserStyleDialect,
    forward_evaluations: &[StaticScssModuleForwardEvaluation],
    emitted_module_identity_keys: &mut BTreeSet<String>,
) -> (String, usize) {
    let lexed = lex_omena_query_omena_parser_style_source(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@forward") =>
            {
                let Some(end_index) = static_scss_use_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                let start = transform_token_start(&tokens[index]);
                let end = transform_token_end(&tokens[end_index]);
                if let Some(source_name) =
                    static_scss_module_rule_source_name(tokens, index + 1, end_index)
                    && let Some(forward) = forward_evaluations
                        .iter()
                        .find(|forward| forward.source == source_name)
                {
                    let replacement = if emitted_module_identity_keys
                        .insert(forward.module_identity_key.clone())
                    {
                        forward.evaluated_css.clone()
                    } else {
                        String::new()
                    };
                    replacements.push((start, end, replacement));
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

fn static_scss_use_rule_semicolon(
    tokens: &[omena_parser::LexedToken],
    at_use_index: usize,
) -> Option<usize> {
    let mut index = at_use_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

fn static_scss_module_rule_source_name(
    tokens: &[omena_parser::LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<String> {
    tokens[start_index..end_index]
        .iter()
        .find(|token| matches!(token.kind, SyntaxKind::String | SyntaxKind::Url))
        .map(|token| token.text.trim_matches('"').trim_matches('\'').to_string())
}

fn derive_static_scss_module_rule_variable_overrides(
    style_source: &str,
    at_keyword: &str,
    use_source: &str,
) -> BTreeMap<String, String> {
    let lexed =
        lex_omena_query_omena_parser_style_source(style_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case(at_keyword) =>
            {
                let Some(end_index) = static_scss_use_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                if static_scss_module_rule_source_name(tokens, index + 1, end_index)
                    .is_some_and(|source_name| source_name == use_source)
                {
                    let start = transform_token_start(&tokens[index]);
                    let end = transform_token_end(&tokens[end_index]);
                    return style_source
                        .get(start..end)
                        .map(parse_static_scss_use_variable_overrides_from_rule)
                        .unwrap_or_default();
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    BTreeMap::new()
}

fn derive_static_scss_forward_export_prefix(
    style_source: &str,
    forward_source: &str,
) -> Option<String> {
    let lexed =
        lex_omena_query_omena_parser_style_source(style_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@forward") =>
            {
                let Some(end_index) = static_scss_use_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                if static_scss_module_rule_source_name(tokens, index + 1, end_index)
                    .is_some_and(|source_name| source_name == forward_source)
                {
                    return parse_static_scss_forward_export_prefix(tokens, index + 1, end_index)
                        .and_then(|(start, end)| style_source.get(start..end))
                        .map(str::trim)
                        .filter(|prefix| static_scss_forward_export_prefix_is_safe(prefix))
                        .map(str::to_string);
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn parse_static_scss_forward_export_prefix(
    tokens: &[omena_parser::LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<(usize, usize)> {
    let source_index = tokens[start_index..end_index]
        .iter()
        .position(|token| matches!(token.kind, SyntaxKind::String | SyntaxKind::Url))
        .map(|offset| start_index + offset)?;
    let as_index = tokens[source_index + 1..end_index]
        .iter()
        .position(|token| token.text.eq_ignore_ascii_case("as"))
        .map(|offset| source_index + 1 + offset)?;
    let prefix_start_index = tokens[as_index + 1..end_index]
        .iter()
        .position(|token| token.kind != SyntaxKind::Whitespace)
        .map(|offset| as_index + 1 + offset)?;
    let prefix_end_index = tokens[prefix_start_index..end_index]
        .iter()
        .position(|token| {
            matches!(
                token.text.to_ascii_lowercase().as_str(),
                "show" | "hide" | "with"
            )
        })
        .map(|offset| prefix_start_index + offset)
        .unwrap_or(end_index);
    Some((
        transform_token_start(&tokens[prefix_start_index]),
        transform_token_start(&tokens[prefix_end_index]),
    ))
}

fn static_scss_forward_export_prefix_is_safe(prefix: &str) -> bool {
    prefix.contains('*')
        && prefix
            .chars()
            .all(|ch| static_scss_identifier_char(ch) || ch == '*')
}

fn parse_static_scss_use_variable_overrides_from_rule(
    rule_source: &str,
) -> BTreeMap<String, String> {
    let lexed =
        lex_omena_query_omena_parser_style_source(rule_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let Some(with_index) = tokens
        .iter()
        .position(|token| token.text.eq_ignore_ascii_case("with"))
    else {
        return BTreeMap::new();
    };
    let Some(left_paren_index) = tokens[with_index + 1..]
        .iter()
        .position(|token| token.kind == SyntaxKind::LeftParen)
        .map(|offset| with_index + 1 + offset)
    else {
        return BTreeMap::new();
    };
    let Some(right_paren_index) =
        scss_variable_overrides::static_scss_matching_right_paren(tokens, left_paren_index)
    else {
        return BTreeMap::new();
    };
    let start = transform_token_end(&tokens[left_paren_index]);
    let end = transform_token_start(&tokens[right_paren_index]);
    rule_source
        .get(start..end)
        .map(scss_variable_overrides::parse_static_scss_use_variable_override_list)
        .unwrap_or_default()
}
