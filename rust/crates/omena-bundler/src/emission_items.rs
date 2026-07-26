use std::collections::BTreeMap;

use omena_parser::{
    ModuleInstanceKeyV0, ParsedAnimationFactKind, ParsedEmissionSelectorFactKindV0,
    ParsedEmissionSelectorFactsV0, ParsedSelectorFactKind, ParsedStyleFacts,
};
use serde::Serialize;

use crate::{
    GlobalRuleOrderV0, LinkedEmissionMaterializationErrorV0, LinkedStylesheetRuleV0,
    TransformBundleLinkErrorV0, emission_order::EmissionModulePlanV0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "category", content = "nodeKind", rename_all = "camelCase")]
#[non_exhaustive]
pub enum EmissionItemKindV0 {
    SelectorClass,
    SelectorId,
    SelectorPlaceholder,
    SelectorElement,
    SelectorAttribute,
    SelectorUniversal,
    SelectorPseudoClass,
    SelectorPseudoElement,
    UnknownStructuralSelector,
    AtRule { node_kind: u32 },
    UnknownAtRule,
    KeyframesDeclaration,
    AnimationNameReference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EmissionItemV0 {
    pub kind: EmissionItemKindV0,
    pub name: String,
    pub range_start: u32,
    pub range_end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum EmissionItemFactCategoryV0 {
    NamedSelectors,
    EmissionSelectors,
    Variables,
    SassSymbols,
    SassIncludes,
    SassModuleEdges,
    SassPlaceholderDefinitions,
    ExtendTargets,
    Animations,
    CssModuleValues,
    CssModuleValueImportEdges,
    CssModuleValueDefinitionEdges,
    CssModuleComposes,
    CssModuleComposesEdges,
    Icss,
    IcssImportEdges,
    IcssExportEdges,
    AtRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum EmissionItemProjectionDispositionV0 {
    Projected,
    NotProjected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum EmissionItemProjectionReasonV0 {
    SourceOrderedNamedSelector,
    SourceOrderedStructuralSelector,
    SourceOrderedAtRule,
    SourceOrderedAnimation,
    NotAnEmissionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EmissionItemProjectionDisclosureV0 {
    pub category: EmissionItemFactCategoryV0,
    pub disposition: EmissionItemProjectionDispositionV0,
    pub reason: EmissionItemProjectionReasonV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EmissionItemInputV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub items: Vec<EmissionItemV0>,
    pub disclosure: Vec<EmissionItemProjectionDisclosureV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EmissionItemOrderKeyV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub intra_module_ordinal: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EmissionItemPlanV0 {
    pub policy: crate::EmissionOrderingPolicyV0,
    pub entries: Vec<EmissionItemOrderKeyV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LinkedEmissionItemV0 {
    pub global_order_index: u32,
    pub module_instance: ModuleInstanceKeyV0,
    pub kind: EmissionItemKindV0,
    pub name: String,
    pub range_start: u32,
    pub range_end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LinkedEmissionItemOrderV0 {
    pub items: Vec<LinkedEmissionItemV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum LinkedEmissionItemMaterializationErrorV0 {
    LegacyMaterialization {
        error: LinkedEmissionMaterializationErrorV0,
    },
    InvalidItemOrderIndex {
        expected: u32,
        actual: u32,
    },
    UnknownEmissionItemModule {
        module_instance: ModuleInstanceKeyV0,
    },
    MissingEmissionItem {
        module_instance: ModuleInstanceKeyV0,
    },
}

impl From<LinkedEmissionMaterializationErrorV0> for LinkedEmissionItemMaterializationErrorV0 {
    fn from(error: LinkedEmissionMaterializationErrorV0) -> Self {
        Self::LegacyMaterialization { error }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TransformBundleEmissionItemProjectionV0 {
    inputs: Vec<EmissionItemInputV0>,
}

impl TransformBundleEmissionItemProjectionV0 {
    pub fn inputs(&self) -> &[EmissionItemInputV0] {
        self.inputs.as_slice()
    }

    pub(crate) fn new(inputs: Vec<EmissionItemInputV0>) -> Self {
        Self { inputs }
    }
}

pub(crate) fn collect_emission_items(
    facts: &ParsedStyleFacts,
    emission_selectors: &ParsedEmissionSelectorFactsV0,
) -> Vec<EmissionItemV0> {
    let mut items = facts
        .selectors
        .iter()
        .map(|selector| EmissionItemV0 {
            kind: match selector.kind {
                ParsedSelectorFactKind::Class => EmissionItemKindV0::SelectorClass,
                ParsedSelectorFactKind::Id => EmissionItemKindV0::SelectorId,
                ParsedSelectorFactKind::Placeholder => EmissionItemKindV0::SelectorPlaceholder,
            },
            name: selector.name.clone(),
            range_start: u32::from(selector.range.start()),
            range_end: u32::from(selector.range.end()),
        })
        .collect::<Vec<_>>();
    for selector in &emission_selectors.selectors {
        let kind = match selector.kind {
            ParsedEmissionSelectorFactKindV0::Element => EmissionItemKindV0::SelectorElement,
            ParsedEmissionSelectorFactKindV0::Attribute => EmissionItemKindV0::SelectorAttribute,
            ParsedEmissionSelectorFactKindV0::Universal => EmissionItemKindV0::SelectorUniversal,
            ParsedEmissionSelectorFactKindV0::PseudoClass => {
                EmissionItemKindV0::SelectorPseudoClass
            }
            ParsedEmissionSelectorFactKindV0::PseudoElement => {
                EmissionItemKindV0::SelectorPseudoElement
            }
            _ => EmissionItemKindV0::UnknownStructuralSelector,
        };
        items.push(EmissionItemV0 {
            kind,
            name: selector.name.clone(),
            range_start: u32::from(selector.range.start()),
            range_end: u32::from(selector.range.end()),
        });
    }
    items.extend(facts.at_rules.iter().map(|at_rule| {
        EmissionItemV0 {
            kind: at_rule
                .node_kind
                .map_or(EmissionItemKindV0::UnknownAtRule, |node_kind| {
                    EmissionItemKindV0::AtRule {
                        node_kind: node_kind.as_u32(),
                    }
                }),
            name: at_rule.name.clone(),
            range_start: u32::from(at_rule.range.start()),
            range_end: u32::from(at_rule.range.end()),
        }
    }));
    items.extend(facts.animations.iter().map(|animation| EmissionItemV0 {
        kind: match animation.kind {
            ParsedAnimationFactKind::KeyframesDeclaration => {
                EmissionItemKindV0::KeyframesDeclaration
            }
            ParsedAnimationFactKind::AnimationNameReference => {
                EmissionItemKindV0::AnimationNameReference
            }
        },
        name: animation.name.clone(),
        range_start: u32::from(animation.range.start()),
        range_end: u32::from(animation.range.end()),
    }));
    items.sort_by_key(|item| {
        (
            item.range_start,
            item.range_end,
            item.kind,
            item.name.clone(),
        )
    });
    items
}

pub(crate) fn emission_item_projection_disclosure(
    facts: &ParsedStyleFacts,
) -> Vec<EmissionItemProjectionDisclosureV0> {
    let ParsedStyleFacts {
        product: _,
        dialect: _,
        selector_count: _,
        selectors: _,
        variable_count: _,
        variables: _,
        sass_symbol_count: _,
        sass_symbols: _,
        sass_include_count: _,
        sass_includes: _,
        sass_module_edge_count: _,
        sass_module_edges: _,
        sass_placeholder_definition_count: _,
        sass_placeholder_definitions: _,
        extend_target_count: _,
        extend_targets: _,
        animation_count: _,
        animations: _,
        css_module_value_count: _,
        css_module_values: _,
        css_module_value_import_edge_count: _,
        css_module_value_import_edges: _,
        css_module_value_definition_edge_count: _,
        css_module_value_definition_edges: _,
        css_module_composes_count: _,
        css_module_composes: _,
        css_module_composes_edge_count: _,
        css_module_composes_edges: _,
        icss_count: _,
        icss: _,
        icss_import_edge_count: _,
        icss_import_edges: _,
        icss_export_edge_count: _,
        icss_export_edges: _,
        at_rule_count: _,
        at_rules: _,
        error_count: _,
    } = facts;

    use EmissionItemFactCategoryV0 as Category;
    use EmissionItemProjectionDispositionV0 as Disposition;
    use EmissionItemProjectionReasonV0 as Reason;

    [
        (
            Category::NamedSelectors,
            Disposition::Projected,
            Reason::SourceOrderedNamedSelector,
        ),
        (
            Category::EmissionSelectors,
            Disposition::Projected,
            Reason::SourceOrderedStructuralSelector,
        ),
        (
            Category::AtRules,
            Disposition::Projected,
            Reason::SourceOrderedAtRule,
        ),
        (
            Category::Animations,
            Disposition::Projected,
            Reason::SourceOrderedAnimation,
        ),
        (
            Category::Variables,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::SassSymbols,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::SassIncludes,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::SassModuleEdges,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::SassPlaceholderDefinitions,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::ExtendTargets,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::CssModuleValues,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::CssModuleValueImportEdges,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::CssModuleValueDefinitionEdges,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::CssModuleComposes,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::CssModuleComposesEdges,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::Icss,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::IcssImportEdges,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
        (
            Category::IcssExportEdges,
            Disposition::NotProjected,
            Reason::NotAnEmissionBoundary,
        ),
    ]
    .into_iter()
    .map(
        |(category, disposition, reason)| EmissionItemProjectionDisclosureV0 {
            category,
            disposition,
            reason,
        },
    )
    .collect()
}

pub(crate) fn build_emission_item_plan(
    inputs: &[EmissionItemInputV0],
    module_plan: &EmissionModulePlanV0,
) -> Result<EmissionItemPlanV0, TransformBundleLinkErrorV0> {
    let inputs_by_instance = inputs
        .iter()
        .map(|input| (input.module_instance.clone(), input))
        .collect::<BTreeMap<_, _>>();
    if inputs_by_instance.len() != inputs.len() {
        return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
            reason: "emission-item inputs contain a duplicate module instance".to_string(),
        });
    }
    let mut entries = Vec::new();
    for instance in &module_plan.module_order {
        let input = inputs_by_instance.get(instance).ok_or_else(|| {
            TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "reachable module {} has no emission-item input",
                    instance.module().as_str()
                ),
            }
        })?;
        for intra_module_ordinal in 0..input.items.len() {
            entries.push(EmissionItemOrderKeyV0 {
                module_instance: instance.clone(),
                intra_module_ordinal: u32::try_from(intra_module_ordinal).map_err(|_| {
                    TransformBundleLinkErrorV0::InvalidEmissionPlan {
                        reason: format!(
                            "module {} has more emission items than the order key can represent",
                            instance.module().as_str()
                        ),
                    }
                })?,
            });
        }
    }
    Ok(EmissionItemPlanV0 {
        policy: module_plan.policy,
        entries,
    })
}

pub(crate) fn build_linked_emission_item_order(
    inputs: &[EmissionItemInputV0],
    plan: &EmissionItemPlanV0,
) -> Result<LinkedEmissionItemOrderV0, TransformBundleLinkErrorV0> {
    let inputs_by_instance = inputs
        .iter()
        .map(|input| (input.module_instance.clone(), input))
        .collect::<BTreeMap<_, _>>();
    let mut items = Vec::with_capacity(plan.entries.len());
    for (global_order_index, key) in plan.entries.iter().enumerate() {
        let input = inputs_by_instance
            .get(&key.module_instance)
            .ok_or_else(|| TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "emission-item key refers to unknown module {}",
                    key.module_instance.module().as_str()
                ),
            })?;
        let item = input
            .items
            .get(key.intra_module_ordinal as usize)
            .ok_or_else(|| TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "emission-item key refers to missing item {} in {}",
                    key.intra_module_ordinal,
                    key.module_instance.module().as_str()
                ),
            })?;
        items.push(LinkedEmissionItemV0 {
            global_order_index: u32::try_from(global_order_index).map_err(|_| {
                TransformBundleLinkErrorV0::InvalidEmissionPlan {
                    reason: "emission-item plan exceeds the output index range".to_string(),
                }
            })?,
            module_instance: key.module_instance.clone(),
            kind: item.kind,
            name: item.name.clone(),
            range_start: item.range_start,
            range_end: item.range_end,
        });
    }
    Ok(LinkedEmissionItemOrderV0 { items })
}

pub(crate) fn build_global_rule_order_from_emission_items(
    order: &LinkedEmissionItemOrderV0,
) -> Result<GlobalRuleOrderV0, TransformBundleLinkErrorV0> {
    let mut rules = Vec::new();
    for item in &order.items {
        let selector_kind = match item.kind {
            EmissionItemKindV0::SelectorClass => "class",
            EmissionItemKindV0::SelectorId => "id",
            EmissionItemKindV0::SelectorPlaceholder => "placeholder",
            _ => continue,
        };
        rules.push(LinkedStylesheetRuleV0 {
            global_order_index: u32::try_from(rules.len()).map_err(|_| {
                TransformBundleLinkErrorV0::InvalidEmissionPlan {
                    reason: "selector projection exceeds the output index range".to_string(),
                }
            })?,
            module_instance: item.module_instance.clone(),
            selector_name: item.name.clone(),
            selector_kind,
            range_start: item.range_start,
            range_end: item.range_end,
        });
    }
    Ok(GlobalRuleOrderV0 { rules })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use omena_parser::{StyleDialect, collect_style_fact_collection};

    use super::{
        EmissionItemFactCategoryV0, EmissionItemKindV0, collect_emission_items,
        emission_item_projection_disclosure,
    };

    fn items_for(source: &str) -> Vec<super::EmissionItemV0> {
        let collection = collect_style_fact_collection(source, StyleDialect::Css);
        collect_emission_items(&collection.facts, &collection.emission_selectors)
    }

    #[test]
    fn selector_less_stylesheets_contribute_emission_items() {
        for source in [
            "main { color: red; }",
            "@font-face { font-family: Demo; src: url(demo.woff2); }",
            "@layer reset, theme;",
        ] {
            assert!(
                !items_for(source).is_empty(),
                "expected an emission item for {source}"
            );
        }
    }

    #[test]
    fn emission_items_follow_source_order_across_fact_categories() {
        let items = items_for(
            ":root { --brand: red; }\n\
             @layer reset;\n\
             div, .card, [hidden], *::before { color: red; }\n\
             @keyframes pulse { from { opacity: 0; } }",
        );

        assert!(items.windows(2).all(|pair| {
            (pair[0].range_start, pair[0].range_end) <= (pair[1].range_start, pair[1].range_end)
        }));
        for kind in [
            EmissionItemKindV0::SelectorPseudoClass,
            EmissionItemKindV0::SelectorElement,
            EmissionItemKindV0::SelectorClass,
            EmissionItemKindV0::SelectorAttribute,
            EmissionItemKindV0::SelectorUniversal,
            EmissionItemKindV0::SelectorPseudoElement,
            EmissionItemKindV0::KeyframesDeclaration,
        ] {
            assert!(
                items.iter().any(|item| item.kind == kind),
                "missing emission item kind {kind:?}"
            );
        }
        assert!(
            items
                .iter()
                .any(|item| matches!(item.kind, EmissionItemKindV0::AtRule { .. }))
        );
    }

    #[test]
    fn projection_disclosure_is_total_over_fact_categories() {
        let collection = collect_style_fact_collection(".card { color: red; }", StyleDialect::Css);
        let disclosure = emission_item_projection_disclosure(&collection.facts);
        let categories = disclosure
            .iter()
            .map(|entry| entry.category)
            .collect::<BTreeSet<_>>();

        assert_eq!(disclosure.len(), 18);
        assert_eq!(categories.len(), disclosure.len());
        assert!(categories.contains(&EmissionItemFactCategoryV0::NamedSelectors));
        assert!(categories.contains(&EmissionItemFactCategoryV0::EmissionSelectors));
        assert!(categories.contains(&EmissionItemFactCategoryV0::AtRules));
        assert!(categories.contains(&EmissionItemFactCategoryV0::Animations));
    }
}
