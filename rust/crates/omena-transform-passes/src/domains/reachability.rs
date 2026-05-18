use crate::{
    domains::css_module_global::{
        CssModuleScopeBlock, CssModuleScopeBlockKind, css_module_scope_kind_for_range,
    },
    helpers::{
        collections::push_unique_string, rules::SimpleRuleSlice,
        selectors::selector_branch_owner_class_name, values::split_top_level_value_arguments,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorListClassTreeShakePlan {
    pub(crate) reachable_selector: Option<String>,
    pub(crate) unreachable_owner_class_names: Vec<String>,
}

pub(crate) fn selector_list_class_tree_shake_plan(
    selector: &str,
    reachable_class_names: &[String],
) -> Option<SelectorListClassTreeShakePlan> {
    let branches = split_top_level_value_arguments(selector)?;
    if branches.is_empty() {
        return None;
    }
    let mut owner_class_names = Vec::new();
    let mut reachable_branches = Vec::new();
    for branch in branches {
        let class_name = selector_branch_owner_class_name(&branch)?;
        if class_name_is_reachable(&class_name, reachable_class_names) {
            reachable_branches.push(branch);
        } else {
            push_unique_string(&mut owner_class_names, class_name);
        }
    }
    if owner_class_names.is_empty() {
        return None;
    }

    Some(SelectorListClassTreeShakePlan {
        reachable_selector: (!reachable_branches.is_empty()).then(|| reachable_branches.join(", ")),
        unreachable_owner_class_names: owner_class_names,
    })
}

pub(crate) fn rule_matches_reachable_class_context(
    selector: &str,
    reachable_class_names: &[String],
) -> bool {
    if reachable_class_names.is_empty() {
        return true;
    }

    !matches!(
        selector_list_class_tree_shake_plan(selector, reachable_class_names),
        Some(SelectorListClassTreeShakePlan {
            reachable_selector: None,
            ..
        })
    )
}

pub(crate) fn rule_slice_matches_reachable_class_context(
    rule: &SimpleRuleSlice,
    scope_blocks: &[CssModuleScopeBlock],
    reachable_class_names: &[String],
) -> bool {
    if css_module_scope_kind_for_range(rule.start, rule.end, scope_blocks)
        == Some(CssModuleScopeBlockKind::Global)
    {
        return true;
    }
    rule_matches_reachable_class_context(&rule.selector, reachable_class_names)
}

pub(crate) fn class_name_is_reachable(class_name: &str, reachable_class_names: &[String]) -> bool {
    reachable_class_names
        .iter()
        .filter_map(|name| normalize_reachable_class_name(name))
        .any(|name| name == class_name)
}

pub(crate) fn normalize_reachable_class_name(name: &str) -> Option<&str> {
    let name = name.trim();
    let name = name.strip_prefix('.').unwrap_or(name);
    if name.is_empty() {
        return None;
    }
    Some(name)
}
