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
/// Identifies one emitted rule by module instance and source order within that module.
pub struct EmissionOrderKeyV0 {
    pub module_instance: ModuleInstanceKeyV0,
    pub intra_module_ordinal: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Records an order-bearing dependency used to construct a bundle emission plan.
pub struct EmissionDependencyFactV0 {
    pub from_module: ModuleInstanceKeyV0,
    pub to_module: ModuleInstanceKeyV0,
    pub edge_kind: TransformBundleEdgeKind,
    pub import_ordinal: u32,
    pub order_relevance: EdgeOrderRelevanceV0,
    pub order_relevance_reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Classifies a dependency cycle by the edge kinds that participate in it.
pub enum EmissionCycleClassV0 {
    Import,
    Composition,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Selects the deterministic tie-break policy used inside a dependency cycle.
pub enum EmissionCyclePolicyV0 {
    ModuleIdentity,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Selects how linked modules are ordered before their rules are emitted.
pub enum EmissionOrderingPolicyV0 {
    #[default]
    ModuleIdLegacy,
    ImportOrderPreserving,
}

impl EmissionOrderingPolicyV0 {
    /// Returns the stable label serialized by command and adapter surfaces.
    pub const fn as_wire_label(self) -> &'static str {
        match self {
            Self::ModuleIdLegacy => "moduleIdLegacy",
            Self::ImportOrderPreserving => "importOrderPreserving",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Describes a strongly connected dependency group and its deterministic member order.
pub struct EmissionCycleGroupV0 {
    pub members: Vec<ModuleInstanceKeyV0>,
    pub chosen_order: Vec<ModuleInstanceKeyV0>,
    pub class: EmissionCycleClassV0,
    pub policy: EmissionCyclePolicyV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Contains the complete rule order and supporting dependency evidence for a linked bundle.
pub struct EmissionPlanV0 {
    pub policy: EmissionOrderingPolicyV0,
    pub entries: Vec<EmissionOrderKeyV0>,
    pub dependency_facts: Vec<EmissionDependencyFactV0>,
    pub cycle_groups: Vec<EmissionCycleGroupV0>,
}

pub(crate) fn build_emission_plan(
    inputs: &[LinkerInputV0],
    linked_modules: &[ModuleInstanceKeyV0],
    entrypoints: &[ModuleInstanceKeyV0],
    policy: EmissionOrderingPolicyV0,
) -> Result<EmissionPlanV0, TransformBundleLinkErrorV0> {
    let dependency_facts = collect_emission_dependency_facts(inputs, linked_modules)?;
    let cycle_groups = build_cycle_groups(linked_modules, &dependency_facts)?;
    let module_order = match policy {
        EmissionOrderingPolicyV0::ModuleIdLegacy => linked_modules.to_vec(),
        EmissionOrderingPolicyV0::ImportOrderPreserving => import_ordered_modules(
            entrypoints,
            linked_modules,
            &dependency_facts,
            &cycle_groups,
        )?,
    };
    let inputs_by_instance = inputs
        .iter()
        .map(|input| (input.instance.clone(), input))
        .collect::<BTreeMap<_, _>>();
    let mut entries = Vec::new();
    for instance in &module_order {
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
    Ok(EmissionPlanV0 {
        policy,
        entries,
        dependency_facts,
        cycle_groups,
    })
}

fn import_ordered_modules(
    entrypoints: &[ModuleInstanceKeyV0],
    linked_modules: &[ModuleInstanceKeyV0],
    dependency_facts: &[EmissionDependencyFactV0],
    cycle_groups: &[EmissionCycleGroupV0],
) -> Result<Vec<ModuleInstanceKeyV0>, TransformBundleLinkErrorV0> {
    let mut components = cycle_groups
        .iter()
        .map(|group| group.chosen_order.clone())
        .collect::<Vec<_>>();
    let cycle_members = components
        .iter()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>();
    components.extend(
        linked_modules
            .iter()
            .filter(|module| !cycle_members.contains(*module))
            .cloned()
            .map(|module| vec![module]),
    );
    components.sort_by(|left, right| left[0].cmp(&right[0]));

    let mut component_by_module = BTreeMap::new();
    for (component_index, members) in components.iter().enumerate() {
        for member in members {
            component_by_module.insert(member.clone(), component_index);
        }
    }
    if component_by_module.len() != linked_modules.len() {
        return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
            reason: "emission components do not cover closed-world membership".to_string(),
        });
    }

    let mut adjacency = (0..components.len())
        .map(|index| (index, Vec::<(u32, usize)>::new()))
        .collect::<BTreeMap<_, _>>();
    for fact in dependency_facts {
        let Some(from_component) = component_by_module.get(&fact.from_module).copied() else {
            return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: "dependency source is absent from emission components".to_string(),
            });
        };
        let Some(to_component) = component_by_module.get(&fact.to_module).copied() else {
            return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
                reason: "dependency target is absent from emission components".to_string(),
            });
        };
        if from_component != to_component {
            adjacency
                .entry(from_component)
                .or_default()
                .push((fact.import_ordinal, to_component));
        }
    }
    for targets in adjacency.values_mut() {
        targets.sort_by_key(|(ordinal, target)| (*ordinal, *target));
        targets.dedup_by_key(|(_, target)| *target);
    }

    let mut roots = entrypoints
        .iter()
        .filter_map(|entrypoint| component_by_module.get(entrypoint).copied())
        .collect::<Vec<_>>();
    roots.extend(0..components.len());
    let mut visited = BTreeSet::new();
    let mut component_order = Vec::new();
    for root in roots {
        if visited.contains(&root) {
            continue;
        }
        let mut stack = vec![(root, false)];
        while let Some((component, expanded)) = stack.pop() {
            if expanded {
                component_order.push(component);
                continue;
            }
            if !visited.insert(component) {
                continue;
            }
            stack.push((component, true));
            if let Some(targets) = adjacency.get(&component) {
                stack.extend(targets.iter().rev().map(|(_, target)| (*target, false)));
            }
        }
    }

    Ok(component_order
        .into_iter()
        .flat_map(|component| components[component].iter().cloned())
        .collect())
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

fn build_cycle_groups(
    linked_modules: &[ModuleInstanceKeyV0],
    dependency_facts: &[EmissionDependencyFactV0],
) -> Result<Vec<EmissionCycleGroupV0>, TransformBundleLinkErrorV0> {
    let mut adjacency = linked_modules
        .iter()
        .cloned()
        .map(|module| (module, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    let mut reverse = adjacency.clone();
    for fact in dependency_facts {
        adjacency
            .entry(fact.from_module.clone())
            .or_default()
            .insert(fact.to_module.clone());
        reverse
            .entry(fact.to_module.clone())
            .or_default()
            .insert(fact.from_module.clone());
    }

    let finish_order = graph_finish_order(linked_modules, &adjacency);
    let mut assigned = BTreeSet::new();
    let mut groups = Vec::new();
    for root in finish_order.into_iter().rev() {
        if assigned.contains(&root) {
            continue;
        }
        let mut stack = vec![root];
        let mut members = Vec::new();
        while let Some(module) = stack.pop() {
            if !assigned.insert(module.clone()) {
                continue;
            }
            members.push(module.clone());
            if let Some(predecessors) = reverse.get(&module) {
                stack.extend(predecessors.iter().rev().cloned());
            }
        }
        members.sort();
        let has_self_loop = members.len() == 1
            && adjacency
                .get(&members[0])
                .is_some_and(|targets| targets.contains(&members[0]));
        if members.len() < 2 && !has_self_loop {
            continue;
        }

        let member_set = members.iter().cloned().collect::<BTreeSet<_>>();
        let mut has_import = false;
        let mut has_composition = false;
        for fact in dependency_facts.iter().filter(|fact| {
            member_set.contains(&fact.from_module) && member_set.contains(&fact.to_module)
        }) {
            match fact.edge_kind {
                TransformBundleEdgeKind::CssModuleComposesExternal => has_composition = true,
                TransformBundleEdgeKind::CssModuleComposesLocal => {
                    return Err(TransformBundleLinkErrorV0::UnsupportedEmissionCycle {
                        edge_kind: fact.edge_kind,
                    });
                }
                TransformBundleEdgeKind::SassUse
                | TransformBundleEdgeKind::SassForward
                | TransformBundleEdgeKind::SassImport
                | TransformBundleEdgeKind::CssImport
                | TransformBundleEdgeKind::LessImport
                | TransformBundleEdgeKind::CssModuleValueImport
                | TransformBundleEdgeKind::IcssImport => has_import = true,
            }
        }
        let class = match (has_import, has_composition) {
            (true, true) => EmissionCycleClassV0::Mixed,
            (false, true) => EmissionCycleClassV0::Composition,
            (true, false) => EmissionCycleClassV0::Import,
            (false, false) => {
                return Err(TransformBundleLinkErrorV0::InvalidEmissionPlan {
                    reason: "cycle group has no classified order-bearing edge".to_string(),
                });
            }
        };
        groups.push(EmissionCycleGroupV0 {
            chosen_order: members.clone(),
            members,
            class,
            policy: EmissionCyclePolicyV0::ModuleIdentity,
        });
    }
    groups.sort_by(|left, right| left.members.cmp(&right.members));
    Ok(groups)
}

fn graph_finish_order(
    nodes: &[ModuleInstanceKeyV0],
    adjacency: &BTreeMap<ModuleInstanceKeyV0, BTreeSet<ModuleInstanceKeyV0>>,
) -> Vec<ModuleInstanceKeyV0> {
    let mut visited = BTreeSet::new();
    let mut finished = Vec::new();
    for root in nodes {
        if visited.contains(root) {
            continue;
        }
        let mut stack = vec![(root.clone(), false)];
        while let Some((module, expanded)) = stack.pop() {
            if expanded {
                finished.push(module);
                continue;
            }
            if !visited.insert(module.clone()) {
                continue;
            }
            stack.push((module.clone(), true));
            if let Some(targets) = adjacency.get(&module) {
                stack.extend(targets.iter().rev().cloned().map(|target| (target, false)));
            }
        }
    }
    finished
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
