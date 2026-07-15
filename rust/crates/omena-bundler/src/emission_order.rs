use std::collections::BTreeMap;

use omena_parser::ModuleInstanceKeyV0;
use serde::Serialize;

use crate::{
    GlobalRuleOrderV0, LinkedStylesheetRuleV0, LinkerInputV0, TransformBundleLinkErrorV0,
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
pub struct EmissionPlanV0 {
    pub entries: Vec<EmissionOrderKeyV0>,
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
    Ok(EmissionPlanV0 { entries })
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
