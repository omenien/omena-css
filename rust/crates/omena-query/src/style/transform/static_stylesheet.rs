use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::super::stylesheet_evaluation::{
    canonical_static_scss_variable_name,
    derive_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation,
};
use super::*;
use omena_query_transform_runner::{
    TransformImportInlineV0, TransformModuleEvaluationV0, restore_less_inline_literal_placeholders,
};
use omena_syntax::SyntaxKind;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

mod evaluation_source;
mod scss_forwarding;
mod scss_module_identity;
mod scss_variable_overrides;

use evaluation_source::{
    derive_import_aware_static_stylesheet_module_evaluation_source,
    static_stylesheet_module_output_css_from_evaluation,
    static_stylesheet_module_system_evaluator_label,
};
pub(super) use scss_forwarding::derive_static_scss_module_forward_effective_variable_override_values_for_resolution_at_ordinal;
use scss_forwarding::{
    derive_static_scss_forward_effective_variable_overrides,
    derive_static_scss_forward_export_prefix_at_ordinal,
    derive_static_scss_module_forward_variable_overrides_at_ordinal,
    filter_static_scss_forward_configurable_variable_names, filter_static_scss_forward_exports,
    prefix_static_scss_forward_exports,
};
use scss_module_identity::{
    resolve_static_scss_module_effective_variable_overrides,
    static_scss_module_configuration_variables_are_valid,
};
pub(super) use scss_module_identity::{
    static_scss_module_configuration_signature, static_scss_module_instance_identity_key,
};

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
        let native_edit_output = evaluation.native_edit_output.as_deref().map(|output| {
            restore_less_inline_literal_placeholders(
                output,
                &import_aware_source.less_inline_literal_placeholders,
            )
        });
        let oracle = evaluation.oracle;
        return Some(TransformModuleEvaluationV0 {
            evaluator: evaluation.evaluator,
            product_output_source: evaluation.product_output_source,
            evaluated_css: restore_less_inline_literal_placeholders(
                evaluation.evaluated_css.as_str(),
                &import_aware_source.less_inline_literal_placeholders,
            ),
            native_edit_output,
            native_replacements: evaluation.native_replacements,
            native_edits: evaluation.native_edits,
            oracle,
        });
    }
    (evaluation_source.as_ref() != style_source).then(|| {
        let output = restore_less_inline_literal_placeholders(
            evaluation_source.as_ref(),
            &import_aware_source.less_inline_literal_placeholders,
        );
        TransformModuleEvaluationV0 {
            evaluator: static_stylesheet_module_system_evaluator_label(dialect).to_string(),
            product_output_source: Some("nativeEditOutput".to_string()),
            evaluated_css: output.clone(),
            native_edit_output: Some(output),
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
            oracle: None,
        }
    })
}

#[derive(Debug, Clone)]
pub(super) struct StaticScssModuleUseEvaluation {
    source: String,
    use_rule_ordinal: usize,
    module_identity_key: String,
    namespace_kind: Option<&'static str>,
    namespace: Option<String>,
    module_output_css: String,
    variable_exports: BTreeMap<String, String>,
}

pub(super) fn derive_static_scss_module_use_evaluations_for_transform_context(
    entry: &OmenaQueryStyleFactEntry,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
) -> Vec<StaticScssModuleUseEvaluation> {
    if !matches!(
        omena_parser_dialect_for_style_path(entry.style_path.as_str()),
        OmenaParserStyleDialect::Scss | OmenaParserStyleDialect::Sass
    ) {
        return Vec::new();
    }

    let mut emitted_module_identity_keys = BTreeSet::new();
    let mut loaded_module_overrides_by_path = BTreeMap::new();
    entry
        .facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassUse")
        .enumerate()
        .filter(|(_, edge)| {
            matches!(
                edge.namespace_kind,
                Some("alias") | Some("default") | Some("wildcard")
            )
        })
        .filter_map(|(use_rule_ordinal, edge)| {
            let resolved = resolution_context.resolve_style_module_source(
                entry.style_path.as_str(),
                edge.source.as_str(),
                available_style_paths,
            )?;
            let source = source_by_path.get(resolved.as_str())?;
            let variable_overrides = derive_static_scss_module_rule_variable_overrides_at_ordinal(
                entry.style_source.as_str(),
                "@use",
                use_rule_ordinal,
            );
            let configurable_variable_names =
                derive_static_scss_module_configurable_variable_names_for_transform_context(
                    resolved.as_str(),
                    source,
                    available_style_paths,
                    source_by_path,
                    resolution_context,
                );
            if !static_scss_module_configuration_variables_are_valid(
                &variable_overrides,
                &configurable_variable_names,
            ) {
                return None;
            }
            let variable_overrides = resolve_static_scss_module_effective_variable_overrides(
                resolved.as_str(),
                &variable_overrides,
                &mut loaded_module_overrides_by_path,
            )?;
            let module_identity_key =
                static_scss_module_instance_identity_key(resolved.as_str(), &variable_overrides);
            let module_context = {
                let mut visited = BTreeSet::new();
                let mut derive_context = StaticScssModuleDeriveContext {
                    available_style_paths,
                    source_by_path,
                    resolution_context,
                    visited: &mut visited,
                    emitted_module_identity_keys: &mut emitted_module_identity_keys,
                    loaded_module_overrides_by_path: &mut loaded_module_overrides_by_path,
                };
                derive_static_scss_module_context_for_transform_context(
                    resolved.as_str(),
                    source,
                    &variable_overrides,
                    &mut derive_context,
                )?
            };
            let module_output_css =
                if emitted_module_identity_keys.insert(module_identity_key.clone()) {
                    module_context.module_output_css
                } else {
                    String::new()
                };
            Some(StaticScssModuleUseEvaluation {
                source: edge.source.clone(),
                use_rule_ordinal,
                module_identity_key,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                module_output_css,
                variable_exports: module_context.variable_exports,
            })
        })
        .collect()
}

#[derive(Debug, Clone)]
struct StaticScssModuleContext {
    module_output_css: String,
    variable_exports: BTreeMap<String, String>,
    configurable_variable_names: BTreeSet<String>,
}

struct StaticScssModuleDeriveContext<'a> {
    available_style_paths: &'a BTreeSet<&'a str>,
    source_by_path: &'a BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'a>,
    visited: &'a mut BTreeSet<String>,
    emitted_module_identity_keys: &'a mut BTreeSet<String>,
    loaded_module_overrides_by_path: &'a mut BTreeMap<String, BTreeMap<String, String>>,
}

pub(super) fn derive_static_scss_module_configurable_variable_names_for_transform_context(
    style_path: &str,
    style_source: &str,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
) -> BTreeSet<String> {
    let mut visiting = BTreeSet::new();
    derive_static_scss_module_configurable_variable_names_for_transform_context_inner(
        style_path,
        style_source,
        available_style_paths,
        source_by_path,
        resolution_context,
        &mut visiting,
    )
}

fn derive_static_scss_module_configurable_variable_names_for_transform_context_inner(
    style_path: &str,
    style_source: &str,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let identity_path = canonicalize_omena_resolver_style_identity_path(style_path);
    if !visiting.insert(identity_path.clone()) {
        return BTreeSet::new();
    }

    let mut names = derive_static_scss_stylesheet_module_configurable_variable_names(style_source);
    let facts =
        summarize_omena_query_omena_parser_style_facts(style_source, OmenaParserStyleDialect::Scss);
    for (forward_rule_ordinal, edge) in facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassForward")
        .enumerate()
    {
        let Some(resolved) = resolution_context.resolve_style_module_source(
            style_path,
            edge.source.as_str(),
            available_style_paths,
        ) else {
            continue;
        };
        let Some(source) = source_by_path.get(resolved.as_str()) else {
            continue;
        };
        let child_names =
            derive_static_scss_module_configurable_variable_names_for_transform_context_inner(
                resolved.as_str(),
                source,
                available_style_paths,
                source_by_path,
                resolution_context,
                visiting,
            );
        let non_default_forward_overrides =
            derive_static_scss_module_forward_variable_overrides_at_ordinal(
                style_source,
                forward_rule_ordinal,
            )
            .into_iter()
            .filter_map(|(name, override_entry)| (!override_entry.is_default).then_some(name))
            .collect::<BTreeSet<_>>();
        let child_names = child_names
            .into_iter()
            .filter(|name| !non_default_forward_overrides.contains(name))
            .collect::<BTreeSet<_>>();
        let export_prefix =
            derive_static_scss_forward_export_prefix_at_ordinal(style_source, forward_rule_ordinal);
        names.extend(filter_static_scss_forward_configurable_variable_names(
            child_names,
            export_prefix.as_deref(),
            edge.visibility_filter_kind,
            &edge.visibility_filter_names,
        ));
    }

    visiting.remove(identity_path.as_str());
    names
}

fn derive_static_scss_module_context_for_transform_context(
    style_path: &str,
    style_source: &str,
    variable_overrides: &BTreeMap<String, String>,
    context: &mut StaticScssModuleDeriveContext<'_>,
) -> Option<StaticScssModuleContext> {
    let variable_overrides = resolve_static_scss_module_effective_variable_overrides(
        style_path,
        variable_overrides,
        context.loaded_module_overrides_by_path,
    )?;
    let module_identity_key =
        static_scss_module_instance_identity_key(style_path, &variable_overrides);
    if !context.visited.insert(module_identity_key.clone()) {
        return Some(StaticScssModuleContext {
            module_output_css: String::new(),
            variable_exports: BTreeMap::new(),
            configurable_variable_names: BTreeSet::new(),
        });
    }

    let mut configurable_variable_names =
        derive_static_scss_stylesheet_module_configurable_variable_names(style_source);
    let style_source =
        apply_static_scss_module_variable_overrides(style_source, &variable_overrides);
    let style_source = style_source.as_ref();

    let forward_evaluations = derive_static_scss_module_forward_evaluations_for_transform_context(
        style_path,
        style_source,
        &variable_overrides,
        context,
    )?;
    let mut variable_exports = derive_static_scss_stylesheet_module_variable_exports(style_source);
    for forward in &forward_evaluations {
        for (name, value) in &forward.variable_exports {
            variable_exports
                .entry(name.clone())
                .or_insert_with(|| value.clone());
        }
        configurable_variable_names.extend(forward.configurable_variable_names.iter().cloned());
    }

    let (evaluation_source, forward_mutation_count) = inline_static_scss_forward_rules(
        style_source,
        OmenaParserStyleDialect::Scss,
        &forward_evaluations,
        context.emitted_module_identity_keys,
    );
    let module_output_css = derive_static_stylesheet_module_evaluation(
        evaluation_source.as_str(),
        OmenaParserStyleDialect::Scss,
    )
    .and_then(|evaluation| {
        static_stylesheet_module_output_css_from_evaluation(evaluation_source.as_ref(), evaluation)
    })
    .unwrap_or_else(|| {
        if forward_mutation_count > 0 {
            evaluation_source
        } else {
            style_source.to_string()
        }
    });

    context.visited.remove(&module_identity_key);
    Some(StaticScssModuleContext {
        module_output_css,
        variable_exports,
        configurable_variable_names,
    })
}

#[derive(Debug, Clone)]
struct StaticScssModuleForwardEvaluation {
    source: String,
    forward_rule_ordinal: usize,
    module_identity_key: String,
    module_output_css: String,
    variable_exports: BTreeMap<String, String>,
    configurable_variable_names: BTreeSet<String>,
}

fn derive_static_scss_module_forward_evaluations_for_transform_context(
    style_path: &str,
    style_source: &str,
    inherited_variable_overrides: &BTreeMap<String, String>,
    context: &mut StaticScssModuleDeriveContext<'_>,
) -> Option<Vec<StaticScssModuleForwardEvaluation>> {
    let facts =
        summarize_omena_query_omena_parser_style_facts(style_source, OmenaParserStyleDialect::Scss);

    let mut evaluations = Vec::new();
    for (forward_rule_ordinal, edge) in facts
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassForward")
        .enumerate()
    {
        let Some(resolved) = context.resolution_context.resolve_style_module_source(
            style_path,
            edge.source.as_str(),
            context.available_style_paths,
        ) else {
            continue;
        };
        let Some(source) = context.source_by_path.get(resolved.as_str()) else {
            continue;
        };
        let explicit_variable_overrides =
            derive_static_scss_module_forward_variable_overrides_at_ordinal(
                style_source,
                forward_rule_ordinal,
            );
        let export_prefix =
            derive_static_scss_forward_export_prefix_at_ordinal(style_source, forward_rule_ordinal);
        let configurable_variable_names =
            derive_static_scss_module_configurable_variable_names_for_transform_context(
                resolved.as_str(),
                source,
                context.available_style_paths,
                context.source_by_path,
                context.resolution_context,
            );
        let variable_overrides = derive_static_scss_forward_effective_variable_overrides(
            &explicit_variable_overrides,
            inherited_variable_overrides,
            export_prefix.as_deref(),
            edge.visibility_filter_kind,
            &edge.visibility_filter_names,
            &configurable_variable_names,
        );
        if !static_scss_module_configuration_variables_are_valid(
            &variable_overrides,
            &configurable_variable_names,
        ) {
            continue;
        }
        let variable_overrides = resolve_static_scss_module_effective_variable_overrides(
            resolved.as_str(),
            &variable_overrides,
            context.loaded_module_overrides_by_path,
        )?;
        let module_identity_key =
            static_scss_module_instance_identity_key(resolved.as_str(), &variable_overrides);
        let module_context = derive_static_scss_module_context_for_transform_context(
            resolved.as_str(),
            source,
            &variable_overrides,
            context,
        )?;
        evaluations.push(StaticScssModuleForwardEvaluation {
            source: edge.source.clone(),
            forward_rule_ordinal,
            module_identity_key,
            module_output_css: module_context.module_output_css,
            variable_exports: filter_static_scss_forward_exports(
                prefix_static_scss_forward_exports(
                    module_context.variable_exports,
                    export_prefix.as_deref(),
                ),
                edge.visibility_filter_kind,
                &edge.visibility_filter_names,
            ),
            configurable_variable_names: filter_static_scss_forward_configurable_variable_names(
                module_context.configurable_variable_names,
                export_prefix.as_deref(),
                edge.visibility_filter_kind,
                &edge.visibility_filter_names,
            ),
        });
    }

    Some(evaluations)
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

pub(super) fn derive_scss_use_aware_static_stylesheet_module_evaluation_source<'a>(
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
    let mut forward_rule_ordinal = 0usize;
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
                {
                    let matching_forward = forward_evaluations.iter().find(|forward| {
                        forward.forward_rule_ordinal == forward_rule_ordinal
                            && forward.source == source_name
                    });
                    forward_rule_ordinal += 1;
                    if let Some(forward) = matching_forward {
                        let replacement = if emitted_module_identity_keys
                            .insert(forward.module_identity_key.clone())
                        {
                            forward.module_output_css.clone()
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

pub(super) fn derive_static_scss_module_rule_variable_overrides_at_ordinal(
    style_source: &str,
    at_keyword: &str,
    rule_ordinal: usize,
) -> BTreeMap<String, String> {
    static_scss_module_rule_source_at_ordinal(style_source, at_keyword, rule_ordinal)
        .map(parse_static_scss_use_variable_overrides_from_rule)
        .unwrap_or_default()
}

pub(super) fn derive_static_scss_module_forward_variable_override_values_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> BTreeMap<String, String> {
    derive_static_scss_module_forward_variable_overrides_at_ordinal(
        style_source,
        forward_rule_ordinal,
    )
    .into_iter()
    .map(|(name, override_entry)| (name, override_entry.value))
    .collect()
}

pub(super) fn static_scss_module_rule_source_at_ordinal<'a>(
    style_source: &'a str,
    at_keyword: &str,
    rule_ordinal: usize,
) -> Option<&'a str> {
    let lexed =
        lex_omena_query_omena_parser_style_source(style_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut depth = 0usize;
    let mut index = 0usize;
    let mut current_rule_ordinal = 0usize;

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
                if static_scss_module_rule_source_name(tokens, index + 1, end_index).is_some() {
                    if current_rule_ordinal == rule_ordinal {
                        let start = transform_token_start(&tokens[index]);
                        let end = transform_token_end(&tokens[end_index]);
                        return style_source.get(start..end);
                    }
                    current_rule_ordinal += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use omena_query_transform_runner::TransformModuleEvaluationNativeEditV0;

    #[test]
    fn static_module_output_rejects_blind_legacy_css_for_native_product_source() {
        let evaluation = test_transform_module_evaluation(Some("nativeEditOutput"), None, None);

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_declared_legacy_product_source_without_oracle() {
        let evaluation = test_transform_module_evaluation(Some("evaluatedCss"), None, None);

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_preserved_oracle_legacy_output() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            None,
            Some(
                omena_query_transform_runner::TransformModuleEvaluationOracleV0 {
                    mode: "oracleOnly".to_string(),
                    product_output_source: "legacyEvaluatedCss".to_string(),
                    all_legacy_declaration_values_preserved: true,
                    ..omena_query_transform_runner::TransformModuleEvaluationOracleV0::default()
                },
            ),
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_divergent_oracle_legacy_output() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            None,
            Some(
                omena_query_transform_runner::TransformModuleEvaluationOracleV0 {
                    mode: "oracleOnly".to_string(),
                    product_output_source: "legacyEvaluatedCss".to_string(),
                    divergence_count: 1,
                    all_legacy_declaration_values_preserved: true,
                    ..omena_query_transform_runner::TransformModuleEvaluationOracleV0::default()
                },
            ),
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_prefers_native_edit_output() {
        let evaluation = test_transform_module_evaluation(
            Some("nativeEditOutput"),
            Some(".native { color: red; }".to_string()),
            None,
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            Some(".native { color: red; }".to_string())
        );
    }

    #[test]
    fn static_module_output_rejects_native_edit_output_without_native_marker() {
        let evaluation = test_transform_module_evaluation(
            Some("legacyEvaluatedCss"),
            Some(".native { color: red; }".to_string()),
            None,
        );

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation("", evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_materializes_matching_native_edits() {
        let input_css = ".button { color: red; }";
        let start = ".button { color: ".len();
        let end = start + "red".len();
        let mut evaluation = test_transform_module_evaluation(Some("nativeEditOutput"), None, None);
        evaluation.evaluated_css = ".button { color: blue; }".to_string();
        evaluation
            .native_edits
            .push(TransformModuleEvaluationNativeEditV0 {
                start,
                end,
                replacement: "blue".to_string(),
                edit_kind: "value".to_string(),
                abstract_value: None,
                abstract_value_kind: None,
            });

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation(input_css, evaluation),
            Some(".button { color: blue; }".to_string())
        );
    }

    #[test]
    fn static_module_output_rejects_matching_native_edits_without_native_marker() {
        let input_css = ".button { color: red; }";
        let start = ".button { color: ".len();
        let end = start + "red".len();
        let mut evaluation =
            test_transform_module_evaluation(Some("legacyEvaluatedCss"), None, None);
        evaluation.evaluated_css = ".button { color: blue; }".to_string();
        evaluation
            .native_edits
            .push(TransformModuleEvaluationNativeEditV0 {
                start,
                end,
                replacement: "blue".to_string(),
                edit_kind: "value".to_string(),
                abstract_value: None,
                abstract_value_kind: None,
            });

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation(input_css, evaluation),
            None
        );
    }

    #[test]
    fn static_module_output_rejects_mismatched_native_edits_without_oracle() {
        let input_css = ".button { color: red; }";
        let start = ".button { color: ".len();
        let end = start + "red".len();
        let mut evaluation = test_transform_module_evaluation(Some("nativeEditOutput"), None, None);
        evaluation.evaluated_css = ".button { color: green; }".to_string();
        evaluation
            .native_edits
            .push(TransformModuleEvaluationNativeEditV0 {
                start,
                end,
                replacement: "blue".to_string(),
                edit_kind: "value".to_string(),
                abstract_value: None,
                abstract_value_kind: None,
            });

        assert_eq!(
            static_stylesheet_module_output_css_from_evaluation(input_css, evaluation),
            None
        );
    }

    fn test_transform_module_evaluation(
        product_output_source: Option<&str>,
        native_edit_output: Option<String>,
        oracle: Option<omena_query_transform_runner::TransformModuleEvaluationOracleV0>,
    ) -> TransformModuleEvaluationV0 {
        TransformModuleEvaluationV0 {
            evaluator: "test".to_string(),
            product_output_source: product_output_source.map(str::to_string),
            evaluated_css: ".legacy { color: red; }".to_string(),
            native_edit_output,
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
            oracle,
        }
    }
}
