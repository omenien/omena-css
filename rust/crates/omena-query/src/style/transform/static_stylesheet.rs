use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::super::stylesheet_evaluation::{
    canonical_static_scss_variable_name,
    derive_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation, static_scss_variable_names_equal,
};
use super::*;
use omena_query_transform_runner::{
    TransformImportInlineV0, TransformLessInlineLiteralPlaceholderV0, TransformModuleEvaluationV0,
    inline_css_imports, inline_css_imports_for_static_module_evaluation,
    restore_less_inline_literal_placeholders,
};
use omena_syntax::SyntaxKind;
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

mod scss_variable_overrides;

use scss_variable_overrides::StaticScssModuleVariableOverride;

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
    (evaluation_source.as_ref() != style_source).then(|| TransformModuleEvaluationV0 {
        evaluator: static_stylesheet_module_system_evaluator_label(dialect).to_string(),
        evaluated_css: restore_less_inline_literal_placeholders(
            evaluation_source.as_ref(),
            &import_aware_source.less_inline_literal_placeholders,
        ),
        native_edit_output: None,
        native_replacements: Vec::new(),
        native_edits: Vec::new(),
        oracle: None,
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
    use_rule_ordinal: usize,
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
            let evaluated_css = if emitted_module_identity_keys.insert(module_identity_key.clone())
            {
                module_context.evaluated_css
            } else {
                String::new()
            };
            Some(StaticScssModuleUseEvaluation {
                source: edge.source.clone(),
                use_rule_ordinal,
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

pub(super) fn static_scss_module_instance_identity_key(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    let mut key = format!("path:{}:{canonical_path}", canonical_path.len());
    key.push('|');
    key.push_str(static_scss_module_configuration_signature(variable_overrides).as_str());
    key
}

pub(super) fn static_scss_module_configuration_signature(
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    if variable_overrides.is_empty() {
        return "with:none".to_string();
    }
    let mut key = String::from("with");
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

fn resolve_static_scss_module_effective_variable_overrides(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
    loaded_module_overrides_by_path: &mut BTreeMap<String, BTreeMap<String, String>>,
) -> Option<BTreeMap<String, String>> {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    match loaded_module_overrides_by_path.get(canonical_path.as_str()) {
        Some(existing_overrides) if variable_overrides.is_empty() => {
            Some(existing_overrides.clone())
        }
        Some(existing_overrides) => {
            (existing_overrides == variable_overrides).then(|| variable_overrides.clone())
        }
        None => {
            loaded_module_overrides_by_path.insert(canonical_path, variable_overrides.clone());
            Some(variable_overrides.clone())
        }
    }
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
            evaluated_css: String::new(),
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
    let evaluated_css = derive_static_stylesheet_module_evaluation(
        evaluation_source.as_str(),
        OmenaParserStyleDialect::Scss,
    )
    .map(|evaluation| {
        evaluation
            .native_edit_output
            .unwrap_or(evaluation.evaluated_css)
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
        evaluated_css,
        variable_exports,
        configurable_variable_names,
    })
}

#[derive(Debug, Clone)]
struct StaticScssModuleForwardEvaluation {
    source: String,
    forward_rule_ordinal: usize,
    module_identity_key: String,
    evaluated_css: String,
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
            evaluated_css: module_context.evaluated_css,
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

fn derive_static_scss_forward_effective_variable_overrides(
    explicit_variable_overrides: &BTreeMap<String, StaticScssModuleVariableOverride>,
    inherited_variable_overrides: &BTreeMap<String, String>,
    export_prefix: Option<&str>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
    configurable_names: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let mut variable_overrides = explicit_variable_overrides
        .iter()
        .filter(|(_, override_entry)| override_entry.is_default)
        .map(|(name, override_entry)| (name.clone(), override_entry.value.clone()))
        .collect::<BTreeMap<_, _>>();
    variable_overrides.extend(
        inherited_variable_overrides
            .iter()
            .filter_map(|(name, value)| {
                let internal_name = static_scss_forward_internal_variable_name_for_exposed_name(
                    name.as_str(),
                    export_prefix,
                )?;
                static_scss_forward_exposed_variable_is_visible(
                    name.as_str(),
                    visibility_filter_kind,
                    visibility_filter_names,
                )
                .then_some((internal_name, value.clone()))
            })
            .filter(|(name, _)| configurable_names.contains(name))
            .collect::<BTreeMap<_, _>>(),
    );
    variable_overrides.extend(
        explicit_variable_overrides
            .iter()
            .filter(|(_, override_entry)| !override_entry.is_default)
            .map(|(name, override_entry)| (name.clone(), override_entry.value.clone())),
    );
    variable_overrides
}

pub(super) fn derive_static_scss_module_forward_effective_variable_override_values_for_resolution_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
    inherited_variable_overrides: &BTreeMap<String, String>,
    export_prefix: Option<&str>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
    configurable_names: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let explicit_variable_overrides =
        derive_static_scss_module_forward_variable_overrides_at_ordinal(
            style_source,
            forward_rule_ordinal,
        );
    derive_static_scss_forward_effective_variable_overrides(
        &explicit_variable_overrides,
        inherited_variable_overrides,
        export_prefix,
        visibility_filter_kind,
        visibility_filter_names,
        configurable_names,
    )
}

fn static_scss_module_configuration_variables_are_valid(
    variable_overrides: &BTreeMap<String, String>,
    configurable_names: &BTreeSet<String>,
) -> bool {
    variable_overrides
        .keys()
        .all(|name| configurable_names.contains(name))
}

fn filter_static_scss_forward_configurable_variable_names(
    names: BTreeSet<String>,
    prefix: Option<&str>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
) -> BTreeSet<String> {
    names
        .into_iter()
        .filter_map(|name| {
            let exposed_name = prefix
                .map(|prefix| prefix.replace('*', name.as_str()))
                .unwrap_or(name);
            static_scss_forward_exposed_variable_is_visible(
                exposed_name.as_str(),
                visibility_filter_kind,
                visibility_filter_names,
            )
            .then(|| canonical_static_scss_variable_name(exposed_name.as_str()))
        })
        .collect()
}

fn static_scss_forward_exposed_variable_is_visible(
    exposed_name: &str,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: &[String],
) -> bool {
    match visibility_filter_kind {
        Some("show") => visibility_filter_names
            .iter()
            .any(|filter| static_scss_variable_names_equal(filter, exposed_name)),
        Some("hide") => !visibility_filter_names
            .iter()
            .any(|filter| static_scss_variable_names_equal(filter, exposed_name)),
        _ => true,
    }
}

fn static_scss_forward_internal_variable_name_for_exposed_name(
    exposed_name: &str,
    export_prefix: Option<&str>,
) -> Option<String> {
    let exposed_name = canonical_static_scss_variable_name(exposed_name);
    let Some(export_prefix) = export_prefix else {
        return Some(exposed_name);
    };
    let star_offset = export_prefix.find('*')?;
    let prefix_before_star = canonical_static_scss_variable_name(&export_prefix[..star_offset]);
    let prefix_after_star =
        canonical_static_scss_variable_name(&export_prefix[star_offset + '*'.len_utf8()..]);
    let without_prefix = exposed_name.strip_prefix(prefix_before_star.as_str())?;
    let without_suffix = if prefix_after_star.is_empty() {
        without_prefix
    } else {
        without_prefix.strip_suffix(prefix_after_star.as_str())?
    };
    (!without_suffix.is_empty()).then(|| canonical_static_scss_variable_name(without_suffix))
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
                            module_use.evaluated_css.clone()
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
                            forward.evaluated_css.clone()
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

fn derive_static_scss_module_forward_variable_overrides_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> BTreeMap<String, StaticScssModuleVariableOverride> {
    static_scss_module_rule_source_at_ordinal(style_source, "@forward", forward_rule_ordinal)
        .map(parse_static_scss_forward_variable_overrides_from_rule)
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

fn static_scss_module_rule_source_at_ordinal<'a>(
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

fn derive_static_scss_forward_export_prefix_at_ordinal(
    style_source: &str,
    forward_rule_ordinal: usize,
) -> Option<String> {
    let lexed =
        lex_omena_query_omena_parser_style_source(style_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut depth = 0usize;
    let mut index = 0usize;
    let mut current_forward_rule_ordinal = 0usize;

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
                if static_scss_module_rule_source_name(tokens, index + 1, end_index).is_some() {
                    if current_forward_rule_ordinal == forward_rule_ordinal {
                        return parse_static_scss_forward_export_prefix(
                            tokens,
                            index + 1,
                            end_index,
                        )
                        .and_then(|(start, end)| style_source.get(start..end))
                        .map(str::trim)
                        .filter(|prefix| static_scss_forward_export_prefix_is_safe(prefix))
                        .map(str::to_string);
                    }
                    current_forward_rule_ordinal += 1;
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

fn parse_static_scss_forward_variable_overrides_from_rule(
    rule_source: &str,
) -> BTreeMap<String, StaticScssModuleVariableOverride> {
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
        .map(scss_variable_overrides::parse_static_scss_forward_variable_override_list)
        .unwrap_or_default()
}
