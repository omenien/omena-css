use super::{LinkedModuleExecutionV0, css_identifier_names_match};
use crate::style::{
    OmenaQueryStyleFactEntry, collect_omena_query_style_fact_entry,
    transform::derive_class_name_rewrites_for_transform_context,
};
use crate::types::normalize_omena_query_style_path;
use crate::{OmenaQueryBundleEmissionPathV0, OmenaQueryTransformExecutionContextV0};
use omena_query_transform_runner::{
    LinkedStylesheetWithEmissionItemsV0, TransformClassNameRewriteV0,
};
use omena_syntax::ident::{ClassNameV0, is_css_name_continue, is_css_name_start};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssModuleTokenCollisionPathScopeV0 {
    BothPaths,
    ImportInlineLegacyOnly,
    LinkedOrderOnly,
}

impl CssModuleTokenCollisionPathScopeV0 {
    const fn as_wire_label(self) -> &'static str {
        match self {
            Self::BothPaths => "bothPaths",
            Self::ImportInlineLegacyOnly => "importInlineLegacyOnly",
            Self::LinkedOrderOnly => "linkedOrderOnly",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssModuleTokenPreimageV0 {
    module_path: String,
    raw_name: String,
    default_token: String,
    linked_token: String,
    interface_token: String,
}

pub(super) fn validate_css_module_token_integrity(
    target_style_path: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    linked: &LinkedStylesheetWithEmissionItemsV0,
    default_context: &OmenaQueryTransformExecutionContextV0,
    linked_module_executions: Option<&[LinkedModuleExecutionV0]>,
    emission_path: OmenaQueryBundleEmissionPathV0,
    emitted_css: &str,
) -> Result<(), String> {
    let preimages = collect_token_preimages(
        target_style_path,
        style_fact_entries,
        linked,
        default_context,
        linked_module_executions,
    )?;
    let emitted_names = emitted_class_names(emitted_css);
    let default_collisions =
        collision_groups(&preimages, |preimage| preimage.default_token.as_str());
    let linked_collisions = collision_groups(&preimages, |preimage| preimage.linked_token.as_str());
    let selected_collisions = match emission_path {
        OmenaQueryBundleEmissionPathV0::ImportInlineLegacy => &default_collisions,
        OmenaQueryBundleEmissionPathV0::LinkedOrder => &linked_collisions,
    };

    for (token, collision) in selected_collisions {
        if !emitted_names.contains(canonical_name(token).as_str()) {
            continue;
        }
        let signature = collision_signature(collision);
        let on_default = default_collisions
            .values()
            .any(|candidate| collision_signature(candidate) == signature);
        let on_linked = linked_collisions
            .values()
            .any(|candidate| collision_signature(candidate) == signature);
        let path_scope = match (on_default, on_linked) {
            (true, true) => CssModuleTokenCollisionPathScopeV0::BothPaths,
            (true, false) => CssModuleTokenCollisionPathScopeV0::ImportInlineLegacyOnly,
            (false, true) => CssModuleTokenCollisionPathScopeV0::LinkedOrderOnly,
            (false, false) => {
                return Err(
                    "CSS Modules emitted-token collision lost both emission-path owners"
                        .to_string(),
                );
            }
        };
        let modules = collision
            .iter()
            .map(|preimage| format!("{:?}", preimage.module_path))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let names = collision
            .iter()
            .map(|preimage| format!("{:?}", preimage.raw_name))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        return Err(format!(
            "CSS Modules emitted-token collision (pathScope={}): modules {} map raw class name(s) {} to emitted token {:?}; build stopped",
            path_scope.as_wire_label(),
            modules.join(", "),
            names.join(", "),
            token,
        ));
    }

    for preimage in &preimages {
        if !scanner_can_rewrite(preimage.raw_name.as_str()) {
            continue;
        }
        let selected_token = match emission_path {
            OmenaQueryBundleEmissionPathV0::ImportInlineLegacy => preimage.default_token.as_str(),
            OmenaQueryBundleEmissionPathV0::LinkedOrder => preimage.linked_token.as_str(),
        };
        let declaration_is_emitted = emitted_names
            .contains(canonical_name(selected_token).as_str())
            || emitted_names.contains(canonical_name(preimage.raw_name.as_str()).as_str());
        if declaration_is_emitted
            && !emitted_names.contains(canonical_name(preimage.interface_token.as_str()).as_str())
        {
            return Err(format!(
                "CSS Modules interface/byte mismatch: module {:?} raw class name {:?} promises token {:?}, but the selected emission model produced {:?}; build stopped",
                preimage.module_path, preimage.raw_name, preimage.interface_token, selected_token,
            ));
        }
    }

    Ok(())
}

fn collect_token_preimages(
    target_style_path: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    linked: &LinkedStylesheetWithEmissionItemsV0,
    default_context: &OmenaQueryTransformExecutionContextV0,
    linked_module_executions: Option<&[LinkedModuleExecutionV0]>,
) -> Result<Vec<CssModuleTokenPreimageV0>, String> {
    let target_path = normalize_omena_query_style_path(target_style_path);
    let default_rewrites = default_context.class_name_rewrites.as_slice();
    let entries_by_path = style_fact_entries
        .iter()
        .map(|entry| {
            (
                normalize_omena_query_style_path(entry.style_path.as_str()),
                entry,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut preimages = BTreeMap::<(String, String), CssModuleTokenPreimageV0>::new();

    let planned_modules = linked
        .emission_item_order
        .items
        .iter()
        .map(|item| normalize_omena_query_style_path(item.module_instance.module().as_str()))
        .collect::<BTreeSet<_>>();
    for module_path in planned_modules {
        let Some(entry) = entries_by_path.get(module_path.as_str()).copied() else {
            return Err(format!(
                "CSS Modules emitted-token integrity could not find module {:?} in the existing style-fact set",
                module_path
            ));
        };
        let interface_rewrites = derive_class_name_rewrites_for_transform_context(entry);
        for raw_name in &entry.facts.class_selector_names {
            let canonical_raw_name = canonical_name(raw_name.as_str());
            let linked_rewrites = linked_module_executions
                .and_then(|executions| {
                    executions.iter().find(|execution| {
                        normalize_omena_query_style_path(
                            execution.module_instance.module().as_str(),
                        ) == module_path
                    })
                })
                .map_or(interface_rewrites.as_slice(), |execution| {
                    execution.class_name_rewrites.as_slice()
                });
            let default_token = rewritten_name(default_rewrites, raw_name.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string();
            let linked_token = rewritten_name(linked_rewrites, raw_name.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string();
            let interface_token = rewritten_name(interface_rewrites.as_slice(), raw_name.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string();
            preimages
                .entry((module_path.clone(), canonical_raw_name))
                .or_insert(CssModuleTokenPreimageV0 {
                    module_path: module_path.clone(),
                    raw_name: raw_name.clone(),
                    default_token,
                    linked_token,
                    interface_token,
                });
        }
    }

    if !entries_by_path.contains_key(target_path.as_str()) {
        return Err(format!(
            "CSS Modules emitted-token integrity could not find target module {:?} in the existing style-fact set",
            target_path
        ));
    }
    Ok(preimages.into_values().collect())
}

fn rewritten_name<'a>(
    rewrites: &'a [TransformClassNameRewriteV0],
    raw_name: &str,
) -> Option<&'a str> {
    rewrites
        .iter()
        .find(|rewrite| css_identifier_names_match(rewrite.original_name.as_str(), raw_name))
        .map(|rewrite| rewrite.rewritten_name.as_str())
}

fn collision_groups<'a>(
    preimages: &'a [CssModuleTokenPreimageV0],
    token: impl Fn(&CssModuleTokenPreimageV0) -> &str,
) -> BTreeMap<String, Vec<&'a CssModuleTokenPreimageV0>> {
    let mut by_token = BTreeMap::<String, Vec<&CssModuleTokenPreimageV0>>::new();
    for preimage in preimages {
        by_token
            .entry(token(preimage).to_string())
            .or_default()
            .push(preimage);
    }
    by_token.retain(|_, candidates| {
        candidates
            .iter()
            .map(|preimage| preimage.module_path.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            > 1
    });
    by_token
}

fn collision_signature(collision: &[&CssModuleTokenPreimageV0]) -> BTreeSet<(String, String)> {
    collision
        .iter()
        .map(|preimage| {
            (
                preimage.module_path.clone(),
                canonical_name(preimage.raw_name.as_str()),
            )
        })
        .collect()
}

fn emitted_class_names(css: &str) -> BTreeSet<String> {
    collect_omena_query_style_fact_entry("<emitted>.css", css)
        .facts
        .class_selector_names
        .iter()
        .map(|name| canonical_name(name))
        .collect()
}

fn canonical_name(name: &str) -> String {
    ClassNameV0::new(name).canonical_key().as_str().to_string()
}

fn scanner_can_rewrite(raw_name: &str) -> bool {
    let decoded = ClassNameV0::new(raw_name);
    let mut characters = decoded.decoded().chars();
    let Some(first) = characters.next() else {
        return false;
    };
    is_css_name_start(first) && characters.all(is_css_name_continue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_scope_follows_the_shared_identifier_predicate() {
        assert!(scanner_can_rewrite("card"));
        assert!(scanner_can_rewrite("카드"));
        assert!(!scanner_can_rewrite(r"a\20 b"));
    }

    #[test]
    fn filesystem_and_memory_labels_are_vacuously_equal_before_path_identity_lands() {
        // Token generation is currently path-independent. The CSS Modules identity
        // maintainer owns this declared vacuity and must replace it when a
        // caller-supplied workspace root becomes a token input.
        let entry_a =
            collect_omena_query_style_fact_entry("/filesystem/card.module.css", ".card {}");
        let entry_b = collect_omena_query_style_fact_entry("memory/card.module.css", ".card {}");
        assert_eq!(
            derive_class_name_rewrites_for_transform_context(&entry_a),
            derive_class_name_rewrites_for_transform_context(&entry_b)
        );
    }
}
