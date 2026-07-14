use omena_abstract_value::FactPrecision;
use serde::Serialize;

use crate::{
    SourceClassValueUniverseEntryV0, SourceDomainClassReferenceFactV0, SourceSyntaxIndexV0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleIntelligenceProviderMetadataV0 {
    pub provider_id: &'static str,
    pub version: &'static str,
    pub stability: &'static str,
    pub domains: &'static [&'static str],
    pub owns_surfaces: &'static [&'static str],
    pub import_targets: &'static [&'static str],
    pub utility_targets: &'static [&'static str],
    pub precision: FactPrecision,
    pub precision_backed: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct StyleIntelligenceSnapshotV0<'snapshot> {
    source_syntax_index: &'snapshot SourceSyntaxIndexV0,
}

impl<'snapshot> StyleIntelligenceSnapshotV0<'snapshot> {
    pub const fn new(source_syntax_index: &'snapshot SourceSyntaxIndexV0) -> Self {
        Self {
            source_syntax_index,
        }
    }

    pub const fn source_syntax_index(&self) -> &'snapshot SourceSyntaxIndexV0 {
        self.source_syntax_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleIntelligenceSourceContextV0 {
    pub byte_offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleIntelligenceClassUniverseV0 {
    pub provider_id: &'static str,
    pub entries: Vec<SourceClassValueUniverseEntryV0>,
    pub precision: FactPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleIntelligenceCompletionV0 {
    pub provider_id: &'static str,
    pub label: String,
    pub precision: FactPrecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleIntelligenceHoverV0 {
    pub provider_id: &'static str,
    pub owner_name: String,
    pub domain: &'static str,
    pub axis_name: String,
    pub current_option: Option<String>,
    pub known_options: Vec<String>,
    pub precision: FactPrecision,
}

pub trait StyleIntelligenceProvider: Sync {
    fn metadata(&self) -> &'static StyleIntelligenceProviderMetadataV0;

    fn class_universe(
        &self,
        snapshot: &StyleIntelligenceSnapshotV0<'_>,
    ) -> StyleIntelligenceClassUniverseV0;

    fn completions(
        &self,
        snapshot: &StyleIntelligenceSnapshotV0<'_>,
        context: StyleIntelligenceSourceContextV0,
    ) -> Vec<StyleIntelligenceCompletionV0>;

    fn hover(
        &self,
        snapshot: &StyleIntelligenceSnapshotV0<'_>,
        context: StyleIntelligenceSourceContextV0,
    ) -> Option<StyleIntelligenceHoverV0>;
}

#[derive(Debug, Clone, Copy)]
pub struct BuiltInStyleIntelligenceProviderV0 {
    metadata: &'static StyleIntelligenceProviderMetadataV0,
    pub(crate) binder_summary_visible: bool,
    pub(crate) recipe: Option<BuiltInRecipeProviderConfigV0>,
}

impl StyleIntelligenceProvider for BuiltInStyleIntelligenceProviderV0 {
    fn metadata(&self) -> &'static StyleIntelligenceProviderMetadataV0 {
        self.metadata
    }

    fn class_universe(
        &self,
        snapshot: &StyleIntelligenceSnapshotV0<'_>,
    ) -> StyleIntelligenceClassUniverseV0 {
        StyleIntelligenceClassUniverseV0 {
            provider_id: self.metadata.provider_id,
            entries: snapshot
                .source_syntax_index
                .class_value_universes
                .iter()
                .filter(|entry| entry.plugin_id == self.metadata.provider_id)
                .cloned()
                .collect(),
            precision: self.metadata.precision,
        }
    }

    fn completions(
        &self,
        snapshot: &StyleIntelligenceSnapshotV0<'_>,
        context: StyleIntelligenceSourceContextV0,
    ) -> Vec<StyleIntelligenceCompletionV0> {
        let Some(reference) = reference_at_offset(
            snapshot.source_syntax_index,
            context.byte_offset,
            self.metadata.provider_id,
        ) else {
            return Vec::new();
        };
        let universe = self.class_universe(snapshot);
        let mut labels = universe
            .entries
            .iter()
            .filter(|entry| {
                entry.domain == reference.domain && entry.owner_name == reference.owner_name
            })
            .flat_map(|entry| {
                entry
                    .axes
                    .iter()
                    .filter(|axis| axis.axis_name == reference.axis_name)
                    .flat_map(|axis| axis.values.iter().cloned())
            })
            .collect::<Vec<_>>();
        labels.sort();
        labels.dedup();
        labels
            .into_iter()
            .map(|label| StyleIntelligenceCompletionV0 {
                provider_id: self.metadata.provider_id,
                label,
                precision: universe.precision,
            })
            .collect()
    }

    fn hover(
        &self,
        snapshot: &StyleIntelligenceSnapshotV0<'_>,
        context: StyleIntelligenceSourceContextV0,
    ) -> Option<StyleIntelligenceHoverV0> {
        let reference = reference_at_offset(
            snapshot.source_syntax_index,
            context.byte_offset,
            self.metadata.provider_id,
        )?;
        let known_options = self
            .completions(snapshot, context)
            .into_iter()
            .map(|item| item.label)
            .collect();
        Some(StyleIntelligenceHoverV0 {
            provider_id: self.metadata.provider_id,
            owner_name: reference.owner_name.clone(),
            domain: reference.domain,
            axis_name: reference.axis_name.clone(),
            current_option: reference
                .option_name
                .clone()
                .or_else(|| reference.prefix.clone()),
            known_options,
            precision: self.metadata.precision,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiltInRecipeCallShapeV0 {
    BaseThenConfig,
    ObjectConfig,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BuiltInRecipeProviderConfigV0 {
    pub(crate) plugin_id: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) import_sources: &'static [&'static str],
    pub(crate) import_names: &'static [&'static str],
    pub(crate) call_shape: BuiltInRecipeCallShapeV0,
}

const CSS_MODULES_METADATA: StyleIntelligenceProviderMetadataV0 =
    StyleIntelligenceProviderMetadataV0 {
        provider_id: "css-modules-classnames-bind",
        version: "0",
        stability: "builtIn",
        domains: &["css-modules"],
        owns_surfaces: &[
            "styleImportRecognition",
            "classUtilityRecognition",
            "classReferenceExtraction",
            "sourceExpressionProjection",
        ],
        import_targets: &["*.module.css", "*.module.scss", "*.module.less"],
        utility_targets: &["classnames/bind", "classnames", "clsx", "clsx/lite"],
        precision: FactPrecision::Exact,
        precision_backed: true,
    };

const UTILITY_DOMAIN_METADATA: StyleIntelligenceProviderMetadataV0 =
    StyleIntelligenceProviderMetadataV0 {
        provider_id: "tailwind-uno-utility-domain",
        version: "0",
        stability: "builtIn",
        domains: &["tailwind-utilities", "unocss-utilities"],
        owns_surfaces: &["domainClassReferenceExtraction"],
        import_targets: &[],
        utility_targets: &["class", "className", "classnames", "clsx", "clsx/lite"],
        precision: FactPrecision::Unknown,
        precision_backed: true,
    };

const VANILLA_EXTRACT_METADATA: StyleIntelligenceProviderMetadataV0 =
    StyleIntelligenceProviderMetadataV0 {
        provider_id: "vanilla-extract-recipe-domain",
        version: "0",
        stability: "builtIn",
        domains: &["vanilla-extract-recipes"],
        owns_surfaces: &["domainClassReferenceExtraction"],
        import_targets: &["@vanilla-extract/recipes"],
        utility_targets: &["recipe"],
        precision: FactPrecision::Exact,
        precision_backed: true,
    };

const VUE_STYLE_MODULE_METADATA: StyleIntelligenceProviderMetadataV0 =
    StyleIntelligenceProviderMetadataV0 {
        provider_id: "vue-style-module-domain",
        version: "0",
        stability: "builtIn",
        domains: &["vue-style-modules"],
        owns_surfaces: &["domainClassReferenceExtraction"],
        import_targets: &["*.vue"],
        utility_targets: &["useCssModule"],
        precision: FactPrecision::Exact,
        precision_backed: true,
    };

const CVA_RECIPE_METADATA: StyleIntelligenceProviderMetadataV0 =
    StyleIntelligenceProviderMetadataV0 {
        provider_id: "cva-recipe-domain",
        version: "0",
        stability: "builtIn",
        domains: &["cva-recipe"],
        owns_surfaces: &["domainClassReferenceExtraction"],
        import_targets: &["class-variance-authority", "cva"],
        utility_targets: &["cva"],
        precision: FactPrecision::Exact,
        precision_backed: true,
    };

const VANILLA_EXTRACT_RECIPE_CONFIG: BuiltInRecipeProviderConfigV0 =
    BuiltInRecipeProviderConfigV0 {
        plugin_id: "vanilla-extract-recipe-domain",
        domain: "vanilla-extract-recipe",
        import_sources: &["@vanilla-extract/recipes"],
        import_names: &["recipe"],
        call_shape: BuiltInRecipeCallShapeV0::ObjectConfig,
    };

const CVA_RECIPE_CONFIG: BuiltInRecipeProviderConfigV0 = BuiltInRecipeProviderConfigV0 {
    plugin_id: "cva-recipe-domain",
    domain: "cva-recipe",
    import_sources: &["class-variance-authority", "cva"],
    import_names: &["cva"],
    call_shape: BuiltInRecipeCallShapeV0::BaseThenConfig,
};

const BUILT_IN_STYLE_INTELLIGENCE_PROVIDERS: [BuiltInStyleIntelligenceProviderV0; 5] = [
    BuiltInStyleIntelligenceProviderV0 {
        metadata: &CSS_MODULES_METADATA,
        binder_summary_visible: true,
        recipe: None,
    },
    BuiltInStyleIntelligenceProviderV0 {
        metadata: &UTILITY_DOMAIN_METADATA,
        binder_summary_visible: true,
        recipe: None,
    },
    BuiltInStyleIntelligenceProviderV0 {
        metadata: &VANILLA_EXTRACT_METADATA,
        binder_summary_visible: true,
        recipe: Some(VANILLA_EXTRACT_RECIPE_CONFIG),
    },
    BuiltInStyleIntelligenceProviderV0 {
        metadata: &VUE_STYLE_MODULE_METADATA,
        binder_summary_visible: true,
        recipe: None,
    },
    BuiltInStyleIntelligenceProviderV0 {
        metadata: &CVA_RECIPE_METADATA,
        binder_summary_visible: false,
        recipe: Some(CVA_RECIPE_CONFIG),
    },
];

pub fn built_in_style_intelligence_providers() -> &'static [BuiltInStyleIntelligenceProviderV0] {
    &BUILT_IN_STYLE_INTELLIGENCE_PROVIDERS
}

pub fn built_in_style_intelligence_provider(
    provider_id: &str,
) -> Option<&'static BuiltInStyleIntelligenceProviderV0> {
    BUILT_IN_STYLE_INTELLIGENCE_PROVIDERS
        .iter()
        .find(|provider| provider.metadata.provider_id == provider_id)
}

pub(crate) fn built_in_recipe_provider_configs() -> Vec<BuiltInRecipeProviderConfigV0> {
    let mut configs = BUILT_IN_STYLE_INTELLIGENCE_PROVIDERS
        .iter()
        .filter_map(|provider| provider.recipe)
        .collect::<Vec<_>>();
    configs.sort_by_key(|config| match config.plugin_id {
        "cva-recipe-domain" => 0,
        _ => 1,
    });
    configs
}

pub fn style_intelligence_completions_at_offset(
    snapshot: &StyleIntelligenceSnapshotV0<'_>,
    byte_offset: usize,
) -> Vec<StyleIntelligenceCompletionV0> {
    let Some(reference) = snapshot
        .source_syntax_index
        .domain_class_references
        .iter()
        .find(|reference| {
            byte_offset >= reference.byte_span.start && byte_offset <= reference.byte_span.end
        })
    else {
        return Vec::new();
    };
    built_in_style_intelligence_provider(reference.plugin_id).map_or_else(Vec::new, |provider| {
        provider.completions(snapshot, StyleIntelligenceSourceContextV0 { byte_offset })
    })
}

pub fn style_intelligence_hover_at_offset(
    snapshot: &StyleIntelligenceSnapshotV0<'_>,
    byte_offset: usize,
) -> Option<StyleIntelligenceHoverV0> {
    let reference = snapshot
        .source_syntax_index
        .domain_class_references
        .iter()
        .find(|reference| {
            byte_offset >= reference.byte_span.start && byte_offset <= reference.byte_span.end
        })?;
    built_in_style_intelligence_provider(reference.plugin_id)?
        .hover(snapshot, StyleIntelligenceSourceContextV0 { byte_offset })
}

fn reference_at_offset<'snapshot>(
    index: &'snapshot SourceSyntaxIndexV0,
    byte_offset: usize,
    provider_id: &str,
) -> Option<&'snapshot SourceDomainClassReferenceFactV0> {
    index.domain_class_references.iter().find(|reference| {
        reference.plugin_id == provider_id
            && byte_offset >= reference.byte_span.start
            && byte_offset <= reference.byte_span.end
    })
}

#[cfg(test)]
mod tests {
    use omena_abstract_value::FactPrecision;

    use super::*;
    use crate::summarize_omena_bridge_source_syntax_index;

    #[test]
    fn provider_registry_supersedes_binder_and_recipe_catalogs() {
        let providers = built_in_style_intelligence_providers();
        assert_eq!(providers.len(), 5);
        assert_eq!(
            providers
                .iter()
                .filter(|provider| provider.binder_summary_visible)
                .count(),
            4
        );
        assert!(
            providers
                .iter()
                .all(|provider| provider.metadata.precision_backed)
        );
        assert!(built_in_style_intelligence_provider("cva-recipe-domain").is_some());
    }

    #[test]
    fn provider_projects_recipe_completions_and_hover_from_source_facts() -> Result<(), &'static str>
    {
        let source = r#"import { cva } from "class-variance-authority";
const button = cva("btn", { variants: { intent: { primary: "a", secondary: "b" } } });
const value = button({ intent: "pri" });"#;
        let index = summarize_omena_bridge_source_syntax_index(source, Vec::new(), Vec::new());
        let reference = index
            .domain_class_references
            .first()
            .ok_or("CVA reference should be indexed")?;
        let snapshot = StyleIntelligenceSnapshotV0::new(&index);
        let completions =
            style_intelligence_completions_at_offset(&snapshot, reference.byte_span.start);
        assert_eq!(
            completions
                .iter()
                .map(|item| item.label.as_str())
                .collect::<Vec<_>>(),
            vec!["primary", "secondary"]
        );
        assert!(
            completions
                .iter()
                .all(|item| item.precision == FactPrecision::Exact)
        );

        let hover = style_intelligence_hover_at_offset(&snapshot, reference.byte_span.start)
            .ok_or("CVA hover should be projected")?;
        assert_eq!(hover.provider_id, "cva-recipe-domain");
        assert_eq!(hover.known_options, vec!["primary", "secondary"]);
        assert_eq!(hover.precision, FactPrecision::Exact);
        Ok(())
    }
}
