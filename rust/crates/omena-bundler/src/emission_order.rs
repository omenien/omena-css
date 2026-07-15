use std::collections::{BTreeMap, BTreeSet};

use omena_cross_file_summary::EdgeOrderRelevanceV0;
use omena_parser::ModuleInstanceKeyV0;
use serde::Serialize;

use crate::{
    GlobalRuleOrderV0, LinkedStylesheetRuleV0, LinkerInputV0, TransformBundleEdgeKind,
    TransformBundleLinkErrorV0, module_instances_by_linker_path, resolve_imported_module_instance,
    selector_kind_label,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionOrderKeyV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub intra_module_ordinal: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionDependencyFactV0 {
    pub from_module: ModuleInstanceKeyV0,
    pub to_module: ModuleInstanceKeyV0,
    pub edge_kind: TransformBundleEdgeKind,
    pub import_ordinal: u32,
    pub order_relevance: EdgeOrderRelevanceV0,
    pub order_relevance_reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionPlanV0 {
    pub entries: Vec<EmissionOrderKeyV0>,
    pub dependency_facts: Vec<EmissionDependencyFactV0>,
}

pub(crate) fn build_module_identity_emission_plan(
    inputs: &[LinkerInputV0],
    linked_modules: &[ModuleInstanceKeyV0],
) -> Result<EmissionPlanV0, TransformBundleLinkErrorV0> {
    let inputs_by_instance = inputs
        .iter()
        .map(|input| (input.instance.clone(), input))
        .collect::<BTreeMap<_, _>>();
    let mut entries = Vec::new();
    for instance in linked_modules {
        let input = inputs_by_instance.get(instance).ok_or_else(|| {
            TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "reachable module {} has no linker input",
                    instance.module().as_str()
                ),
            }
        })?;
        for intra_module_ordinal in 0..input.ordered_rules.len() {
            entries.push(EmissionOrderKeyV0 {
                module_instance: instance.clone(),
                intra_module_ordinal: u32::try_from(intra_module_ordinal).map_err(|_| {
                    TransformBundleLinkErrorV0::InvalidEmissionPlan {
                        reason: format!(
                            "module {} has more rules than the emission key can represent",
                            instance.module().as_str()
                        ),
                    }
                })?,
            });
        }
    }
    let dependency_facts = collect_emission_dependency_facts(inputs, linked_modules)?;
    Ok(EmissionPlanV0 {
        entries,
        dependency_facts,
    })
}

fn collect_emission_dependency_facts(
    inputs: &[LinkerInputV0],
    linked_modules: &[ModuleInstanceKeyV0],
) -> Result<Vec<EmissionDependencyFactV0>, TransformBundleLinkErrorV0> {
    let reachable = linked_modules.iter().cloned().collect::<BTreeSet<_>>();
    let inputs_by_instance = inputs
        .iter()
        .map(|input| (input.instance.clone(), input))
        .collect::<BTreeMap<_, _>>();
    let instances_by_path = module_instances_by_linker_path(inputs);
    let mut facts = Vec::new();
    for from_module in linked_modules {
        let input = inputs_by_instance.get(from_module).ok_or_else(|| {
            TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "reachable module {} has no dependency projection",
                    from_module.module().as_str()
                ),
            }
        })?;
        for edge in &input.dependency_edges {
            let order_relevance = edge.kind.order_relevance();
            if order_relevance == EdgeOrderRelevanceV0::OrderNeutral {
                continue;
            }
            let import_ordinal = edge.import_ordinal.ok_or_else(|| {
                TransformBundleLinkErrorV0::InvalidEmissionPlan {
                    reason: format!(
                        "order-bearing dependency {} in {} has no parser-origin ordinal",
                        edge.import_source,
                        from_module.module().as_str()
                    ),
                }
            })?;
            let to_module = resolve_imported_module_instance(
                input.source_path.as_str(),
                edge.import_source.as_str(),
                &instances_by_path,
            )?
            .ok_or_else(|| TransformBundleLinkErrorV0::MissingDependency {
                source_path: input.source_path.clone(),
                import_source: edge.import_source.clone(),
            })?;
            if !reachable.contains(&to_module) {
                return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
                    reason: format!(
                        "dependency {} from {} is absent from the closed-world membership",
                        to_module.module().as_str(),
                        from_module.module().as_str()
                    ),
                });
            }
            facts.push(EmissionDependencyFactV0 {
                from_module: from_module.clone(),
                to_module,
                edge_kind: edge.kind,
                import_ordinal,
                order_relevance,
                order_relevance_reason: edge.kind.order_relevance_reason(),
            });
        }
    }
    facts.sort_by(|left, right| {
        left.from_module
            .cmp(&right.from_module)
            .then_with(|| left.import_ordinal.cmp(&right.import_ordinal))
            .then_with(|| left.to_module.cmp(&right.to_module))
    });
    Ok(facts)
}

pub(crate) fn build_global_rule_order_from_plan(
    inputs: &[LinkerInputV0],
    plan: &EmissionPlanV0,
) -> Result<GlobalRuleOrderV0, TransformBundleLinkErrorV0> {
    let inputs_by_instance = inputs
        .iter()
        .map(|input| (input.instance.clone(), input))
        .collect::<BTreeMap<_, _>>();
    let mut rules = Vec::with_capacity(plan.entries.len());
    for (global_order_index, key) in plan.entries.iter().enumerate() {
        let input = inputs_by_instance
            .get(&key.module_instance)
            .ok_or_else(|| TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "emission key refers to unknown module {}",
                    key.module_instance.module().as_str()
                ),
            })?;
        let selector = input
            .ordered_rules
            .get(key.intra_module_ordinal as usize)
            .ok_or_else(|| TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: format!(
                    "emission key refers to missing rule {} in {}",
                    key.intra_module_ordinal,
                    key.module_instance.module().as_str()
                ),
            })?;
        rules.push(LinkedStylesheetRuleV0 {
            global_order_index: u32::try_from(global_order_index).map_err(|_| {
                TransformBundleLinkErrorV0::InvalidEmissionPlan {
                    reason: "emission plan has more rules than the output index can represent"
                        .to_string(),
                }
            })?,
            module_instance: key.module_instance.clone(),
            selector_name: selector.selector_name.clone(),
            selector_kind: selector_kind_label(selector.selector_kind),
            range_start: selector.range_start,
            range_end: selector.range_end,
        });
    }
    Ok(GlobalRuleOrderV0 { rules })
}
