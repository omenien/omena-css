use super::super::super::parser_facade::summarize_omena_query_omena_parser_style_facts;
use super::super::super::stylesheet_evaluation::{
    derive_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation,
};
use super::super::TransformResolutionContext;
use super::{
    evaluation_source::static_stylesheet_module_output_css_from_evaluation,
    scss_forwarding::{StaticScssModuleForwardEvaluation, inline_static_scss_forward_rules},
    scss_variable_overrides,
};
use crate::OmenaParserStyleDialect;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub(super) struct StaticScssModuleContext {
    pub(super) module_output_css: String,
    pub(super) variable_exports: BTreeMap<String, String>,
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

pub(in crate::style::transform) fn derive_static_scss_module_configurable_variable_names_for_transform_context(
    style_path: &str,
    style_source: &str,
    available_style_paths: &BTreeSet<&str>,
    source_by_path: &BTreeMap<String, String>,
    resolution_context: TransformResolutionContext<'_>,
) -> BTreeSet<String> {
    let resolver = StaticScssModuleConfigurableNamesResolver { resolution_context };
    omena_semantic::derive_sass_module_configurable_variable_names(
        style_path,
        style_source,
        available_style_paths,
        source_by_path,
        &resolver,
    )
}

struct StaticScssModuleConfigurableNamesResolver<'a> {
    resolution_context: TransformResolutionContext<'a>,
}

impl omena_semantic::SassModuleConfigurableNamesResolverV0
    for StaticScssModuleConfigurableNamesResolver<'_>
{
    fn local_configurable_names(&self, _style_path: &str, style_source: &str) -> BTreeSet<String> {
        derive_static_scss_stylesheet_module_configurable_variable_names(style_source)
    }

    fn resolve_module_source(
        &self,
        from_style_path: &str,
        source: &str,
        available_style_paths: &BTreeSet<&str>,
    ) -> Option<String> {
        self.resolution_context.resolve_style_module_source(
            from_style_path,
            source,
            available_style_paths,
        )
    }
}

pub(super) struct StaticScssModuleContextRequest<'a> {
    pub(super) style_path: &'a str,
    pub(super) style_source: &'a str,
    pub(super) variable_overrides: &'a BTreeMap<String, String>,
    pub(super) available_style_paths: &'a BTreeSet<&'a str>,
    pub(super) source_by_path: &'a BTreeMap<String, String>,
    pub(super) resolution_context: TransformResolutionContext<'a>,
}

pub(super) fn derive_static_scss_module_context_for_transform_context(
    request: StaticScssModuleContextRequest<'_>,
    emitted_module_identity_keys: &mut BTreeSet<String>,
    loaded_module_overrides_by_path: &mut BTreeMap<String, BTreeMap<String, String>>,
) -> Option<StaticScssModuleContext> {
    let mut visited = BTreeSet::new();
    let mut context = StaticScssModuleDeriveContext {
        available_style_paths: request.available_style_paths,
        source_by_path: request.source_by_path,
        resolution_context: request.resolution_context,
        visited: &mut visited,
        emitted_module_identity_keys,
        loaded_module_overrides_by_path,
    };
    derive_static_scss_module_context_inner(
        request.style_path,
        request.style_source,
        request.variable_overrides,
        &mut context,
    )
}

fn derive_static_scss_module_context_inner(
    style_path: &str,
    style_source: &str,
    variable_overrides: &BTreeMap<String, String>,
    context: &mut StaticScssModuleDeriveContext<'_>,
) -> Option<StaticScssModuleContext> {
    let variable_overrides = omena_semantic::resolve_sass_module_effective_variable_overrides(
        style_path,
        variable_overrides,
        context.loaded_module_overrides_by_path,
    )?;
    let module_identity_key = omena_semantic::summarize_sass_module_instance_identity_key(
        style_path,
        &variable_overrides,
    );
    if !context.visited.insert(module_identity_key.clone()) {
        return Some(StaticScssModuleContext {
            module_output_css: String::new(),
            variable_exports: BTreeMap::new(),
            configurable_variable_names: BTreeSet::new(),
        });
    }

    let mut configurable_variable_names =
        derive_static_scss_stylesheet_module_configurable_variable_names(style_source);
    let style_source = scss_variable_overrides::apply_static_scss_module_variable_overrides(
        style_source,
        &variable_overrides,
    );
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
            omena_semantic::derive_sass_module_forward_variable_overrides_at_ordinal(
                style_source,
                forward_rule_ordinal,
            );
        let export_prefix = omena_semantic::derive_sass_forward_export_prefix_at_ordinal(
            style_source,
            forward_rule_ordinal,
        );
        let configurable_variable_names =
            derive_static_scss_module_configurable_variable_names_for_transform_context(
                resolved.as_str(),
                source,
                context.available_style_paths,
                context.source_by_path,
                context.resolution_context,
            );
        let variable_overrides = omena_semantic::derive_sass_forward_effective_variable_overrides(
            &explicit_variable_overrides,
            inherited_variable_overrides,
            export_prefix.as_deref(),
            edge.visibility_filter_kind,
            &edge.visibility_filter_names,
            &configurable_variable_names,
        );
        if !omena_semantic::sass_module_configuration_variables_are_valid(
            &variable_overrides,
            &configurable_variable_names,
        ) {
            continue;
        }
        let variable_overrides = omena_semantic::resolve_sass_module_effective_variable_overrides(
            resolved.as_str(),
            &variable_overrides,
            context.loaded_module_overrides_by_path,
        )?;
        let module_identity_key = omena_semantic::summarize_sass_module_instance_identity_key(
            resolved.as_str(),
            &variable_overrides,
        );
        let module_context = derive_static_scss_module_context_inner(
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
            variable_exports: omena_semantic::filter_sass_forward_exports(
                omena_semantic::prefix_sass_forward_exports(
                    module_context.variable_exports,
                    export_prefix.as_deref(),
                ),
                edge.visibility_filter_kind,
                &edge.visibility_filter_names,
            ),
            configurable_variable_names:
                omena_semantic::filter_sass_forward_configurable_variable_names(
                    module_context.configurable_variable_names,
                    export_prefix.as_deref(),
                    edge.visibility_filter_kind,
                    &edge.visibility_filter_names,
                ),
        });
    }

    Some(evaluations)
}
