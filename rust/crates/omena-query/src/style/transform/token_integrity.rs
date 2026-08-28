use super::{LinkedModuleExecutionV0, css_identifier_names_match};
use crate::style::{
    OmenaQueryStyleFactEntry, collect_omena_query_style_fact_entry,
    transform::derive_class_name_rewrites_for_module_instance,
};
use crate::types::normalize_omena_query_style_path;
use crate::{OmenaQueryBundleEmissionPathV0, OmenaQueryTransformExecutionContextV0};
use omena_query_transform_runner::{
    CssModuleTokenCollisionPathScopeV0, CssModuleTokenCollisionV0,
    CssModuleTokenInterfaceMismatchV0, CssModuleTokenOwnershipCensusV0, CssModuleTokenOwnershipV0,
    LinkedStylesheetWithEmissionItemsV0, TransformClassNameRewriteV0,
};
use omena_syntax::ident::{
    CanonicalClassKeyV0, ClassNameV0, is_css_name_continue, is_css_name_start,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssModuleTokenPreimageV0 {
    module_instance: omena_parser::ModuleInstanceKeyV0,
    module_path: String,
    raw_name: String,
    default_token: String,
    linked_token: String,
    interface_token: String,
}

#[allow(clippy::too_many_arguments)]
#[allow(deprecated)]
pub(super) fn summarize_css_module_token_ownership(
    target_style_path: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    linked: &LinkedStylesheetWithEmissionItemsV0,
    default_context: &OmenaQueryTransformExecutionContextV0,
    linked_module_executions: Option<&[LinkedModuleExecutionV0]>,
    module_identity_root: Option<&str>,
    emission_path: OmenaQueryBundleEmissionPathV0,
    emitted_css: &str,
) -> Result<CssModuleTokenOwnershipCensusV0, String> {
    let preimages = collect_token_preimages(
        target_style_path,
        style_fact_entries,
        linked,
        default_context,
        linked_module_executions,
        module_identity_root,
    )?;
    let emitted_names = emitted_class_names(emitted_css);
    let default_collisions =
        collision_groups(&preimages, |preimage| preimage.default_token.as_str());
    let linked_collisions = collision_groups(&preimages, |preimage| preimage.linked_token.as_str());
    let selected_collisions = match emission_path {
        OmenaQueryBundleEmissionPathV0::ImportInlineLegacy => &default_collisions,
        OmenaQueryBundleEmissionPathV0::LinkedOrder => &linked_collisions,
    };
    let selected_by_token = group_preimages_by_token(&preimages, |preimage| match emission_path {
        OmenaQueryBundleEmissionPathV0::ImportInlineLegacy => preimage.default_token.as_str(),
        OmenaQueryBundleEmissionPathV0::LinkedOrder => preimage.linked_token.as_str(),
    });
    let token_ownerships = selected_by_token
        .iter()
        .filter(|(token, _)| emitted_names.contains_key(&canonical_name(token)))
        .map(|(token, owners)| ownership_record(token.as_str(), owners))
        .collect::<Vec<_>>();
    let modeled_emitted_names = selected_by_token
        .keys()
        .map(|token| canonical_name(token))
        .collect::<BTreeSet<_>>();
    let unattributed_emitted_tokens = emitted_names
        .iter()
        .filter(|(name, _)| !modeled_emitted_names.contains(*name))
        .map(|(_, name)| name.clone())
        .collect::<Vec<_>>();
    let mut module_token_collisions = Vec::new();
    for (token, collision) in selected_collisions {
        if !emitted_names.contains_key(&canonical_name(token)) {
            continue;
        }
        let (path_scope, observed_emission_paths) =
            collision_path_observation(token, collision, &default_collisions, &linked_collisions)?;
        let ownership = ownership_record(token.as_str(), collision);
        module_token_collisions.push(CssModuleTokenCollisionV0::new(
            ownership,
            observed_emission_paths,
            path_scope,
        ));
    }

    let interface_mismatches =
        interface_mismatches_for_emitted_names(&preimages, emission_path, &emitted_names);

    Ok(CssModuleTokenOwnershipCensusV0::new(
        emission_path.as_wire_label(),
        preimages.len(),
        token_ownerships,
        module_token_collisions,
        unattributed_emitted_tokens,
        interface_mismatches,
    ))
}

#[allow(deprecated)]
fn interface_mismatches_for_emitted_names(
    preimages: &[CssModuleTokenPreimageV0],
    emission_path: OmenaQueryBundleEmissionPathV0,
    emitted_names: &BTreeMap<CanonicalClassKeyV0, String>,
) -> Vec<CssModuleTokenInterfaceMismatchV0> {
    let mut interface_mismatches = Vec::new();
    for preimage in preimages {
        if !scanner_can_rewrite(preimage.raw_name.as_str()) {
            continue;
        }
        let selected_token = match emission_path {
            OmenaQueryBundleEmissionPathV0::ImportInlineLegacy => preimage.default_token.as_str(),
            OmenaQueryBundleEmissionPathV0::LinkedOrder => preimage.linked_token.as_str(),
        };
        let declaration_is_emitted = emitted_names.contains_key(&canonical_name(selected_token))
            || emitted_names.contains_key(&canonical_name(preimage.raw_name.as_str()));
        if declaration_is_emitted
            && !emitted_names.contains_key(&canonical_name(preimage.interface_token.as_str()))
        {
            interface_mismatches.push(CssModuleTokenInterfaceMismatchV0::new(
                preimage.module_instance.clone(),
                preimage.module_path.clone(),
                preimage.raw_name.clone(),
                preimage.interface_token.clone(),
                selected_token,
            ));
        }
    }
    interface_mismatches
}

pub(super) fn unavailable_css_module_token_ownership_census(
    emission_path: OmenaQueryBundleEmissionPathV0,
    reason: impl Into<String>,
) -> CssModuleTokenOwnershipCensusV0 {
    CssModuleTokenOwnershipCensusV0::unavailable(emission_path.as_wire_label(), reason)
}

pub(super) fn validate_css_module_token_integrity(
    census: &CssModuleTokenOwnershipCensusV0,
) -> Result<(), String> {
    if !census.complete {
        return Err(format!(
            "CSS Modules emitted-token integrity could not attribute every emitted token: unavailable reasons {:?}, unattributed tokens {:?}; build stopped",
            census.unavailable_reasons, census.unattributed_emitted_tokens,
        ));
    }
    if let Some(collision) = census.module_token_collisions.first() {
        let modules = collision
            .module_paths
            .iter()
            .map(|module_path| format!("{module_path:?}"))
            .collect::<Vec<_>>();
        let names = collision
            .original_names
            .iter()
            .map(|name| format!("{name:?}"))
            .collect::<Vec<_>>();
        return Err(format!(
            "CSS Modules emitted-token collision (pathScope={}): modules {} map raw class name(s) {} to emitted token {:?}; build stopped",
            collision.path_scope.as_wire_label(),
            modules.join(", "),
            names.join(", "),
            collision.emitted_token,
        ));
    }
    if let Some(mismatch) = census.interface_mismatches.first() {
        return Err(format!(
            "CSS Modules interface/byte mismatch: module {:?} raw class name {:?} promises token {:?}, but the selected emission model produced {:?}; build stopped",
            mismatch.module_path,
            mismatch.original_name,
            mismatch.promised_token,
            mismatch.emitted_token,
        ));
    }
    Ok(())
}

fn collect_token_preimages(
    target_style_path: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    linked: &LinkedStylesheetWithEmissionItemsV0,
    default_context: &OmenaQueryTransformExecutionContextV0,
    linked_module_executions: Option<&[LinkedModuleExecutionV0]>,
    module_identity_root: Option<&str>,
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
    let mut preimages =
        BTreeMap::<(omena_parser::ModuleInstanceKeyV0, String), CssModuleTokenPreimageV0>::new();

    let planned_modules = linked
        .emission_item_order
        .items
        .iter()
        .map(|item| item.module_instance.clone())
        .collect::<BTreeSet<_>>();
    for module_instance in planned_modules {
        let token_module_instance = module_identity_root.map_or_else(
            || Ok(module_instance.clone()),
            |root| super::css_modules::module_instance_key_relative_to_root(&module_instance, root),
        )?;
        let module_path = normalize_omena_query_style_path(module_instance.module().as_str());
        let Some(entry) = entries_by_path.get(module_path.as_str()).copied() else {
            return Err(format!(
                "CSS Modules emitted-token integrity could not find module {:?} in the existing style-fact set",
                module_path
            ));
        };
        let derived_rewrites =
            derive_class_name_rewrites_for_module_instance(entry, &token_module_instance);
        for raw_name in &entry.facts.class_selector_names {
            let selected_module_rewrites = linked_module_executions
                .and_then(|executions| {
                    executions
                        .iter()
                        .find(|execution| execution.module_instance == module_instance)
                })
                .map(|execution| execution.class_name_rewrites.as_slice());
            let interface_rewrites =
                selected_module_rewrites.unwrap_or(derived_rewrites.as_slice());
            let module_rewrites = selected_module_rewrites.unwrap_or(derived_rewrites.as_slice());
            let default_rewrites = if linked_module_executions.is_some() {
                module_rewrites
            } else {
                default_rewrites
            };
            let default_token = rewritten_name(default_rewrites, raw_name.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string();
            let linked_token = rewritten_name(module_rewrites, raw_name.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string();
            let interface_token = rewritten_name(interface_rewrites, raw_name.as_str())
                .unwrap_or(raw_name.as_str())
                .to_string();
            preimages
                .entry((token_module_instance.clone(), raw_name.clone()))
                .or_insert(CssModuleTokenPreimageV0 {
                    module_instance: token_module_instance.clone(),
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
        .find(|rewrite| rewrite.original_name == raw_name)
        .or_else(|| {
            rewrites.iter().find(|rewrite| {
                css_identifier_names_match(rewrite.original_name.as_str(), raw_name)
            })
        })
        .map(|rewrite| rewrite.rewritten_name.as_str())
}

fn collision_groups(
    preimages: &[CssModuleTokenPreimageV0],
    token: impl Fn(&CssModuleTokenPreimageV0) -> &str,
) -> BTreeMap<String, Vec<&CssModuleTokenPreimageV0>> {
    let mut by_token = group_preimages_by_token(preimages, token);
    by_token.retain(|_, candidates| {
        candidates
            .iter()
            .map(|preimage| {
                (
                    &preimage.module_instance,
                    canonical_name(preimage.raw_name.as_str()),
                )
            })
            .collect::<BTreeSet<_>>()
            .len()
            > 1
    });
    by_token
}

fn collision_signature(
    collision: &[&CssModuleTokenPreimageV0],
) -> BTreeSet<(String, CanonicalClassKeyV0)> {
    collision
        .iter()
        .map(|preimage| {
            (
                format!(
                    "{}#{}",
                    preimage.module_instance.module().as_str(),
                    preimage.module_instance.configuration().as_str()
                ),
                canonical_name(preimage.raw_name.as_str()),
            )
        })
        .collect()
}

#[allow(deprecated)]
fn collision_path_observation(
    token: &str,
    collision: &[&CssModuleTokenPreimageV0],
    default_collisions: &BTreeMap<String, Vec<&CssModuleTokenPreimageV0>>,
    linked_collisions: &BTreeMap<String, Vec<&CssModuleTokenPreimageV0>>,
) -> Result<(CssModuleTokenCollisionPathScopeV0, Vec<&'static str>), String> {
    let signature = collision_signature(collision);
    let on_default = default_collisions
        .get(token)
        .is_some_and(|candidate| collision_signature(candidate) == signature);
    let on_linked = linked_collisions
        .get(token)
        .is_some_and(|candidate| collision_signature(candidate) == signature);
    let path_scope = match (on_default, on_linked) {
        (true, true) => CssModuleTokenCollisionPathScopeV0::BothPaths,
        (true, false) => CssModuleTokenCollisionPathScopeV0::ImportInlineLegacyOnly,
        (false, true) => CssModuleTokenCollisionPathScopeV0::LinkedOrderOnly,
        (false, false) => {
            return Err(
                "CSS Modules emitted-token collision lost both emission-path owners".to_string(),
            );
        }
    };
    let mut observed_emission_paths = Vec::with_capacity(2);
    if on_default {
        observed_emission_paths
            .push(OmenaQueryBundleEmissionPathV0::ImportInlineLegacy.as_wire_label());
    }
    if on_linked {
        observed_emission_paths.push(OmenaQueryBundleEmissionPathV0::LinkedOrder.as_wire_label());
    }
    Ok((path_scope, observed_emission_paths))
}

fn group_preimages_by_token(
    preimages: &[CssModuleTokenPreimageV0],
    token: impl Fn(&CssModuleTokenPreimageV0) -> &str,
) -> BTreeMap<String, Vec<&CssModuleTokenPreimageV0>> {
    let mut by_token = BTreeMap::<String, Vec<&CssModuleTokenPreimageV0>>::new();
    for preimage in preimages {
        by_token
            .entry(token(preimage).to_string())
            .or_default()
            .push(preimage);
    }
    by_token
}

fn ownership_record(
    token: &str,
    preimages: &[&CssModuleTokenPreimageV0],
) -> CssModuleTokenOwnershipV0 {
    CssModuleTokenOwnershipV0::new(
        token,
        preimages
            .iter()
            .map(|preimage| preimage.module_instance.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        preimages
            .iter()
            .map(|preimage| preimage.module_path.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        preimages
            .iter()
            .map(|preimage| preimage.raw_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    )
}

fn emitted_class_names(css: &str) -> BTreeMap<CanonicalClassKeyV0, String> {
    collect_omena_query_style_fact_entry("<emitted>.css", css)
        .facts
        .class_selector_names
        .iter()
        .map(|name| {
            (
                canonical_name(name),
                ClassNameV0::new(name).decoded().to_string(),
            )
        })
        .collect()
}

fn canonical_name(name: &str) -> CanonicalClassKeyV0 {
    ClassNameV0::new(name).canonical_key()
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
    #![allow(deprecated)]

    use super::*;

    fn collision_preimage(
        module_path: &str,
        default_token: &str,
        linked_token: &str,
    ) -> CssModuleTokenPreimageV0 {
        let module_instance = omena_parser::ModuleInstanceKeyV0::unconfigured(
            omena_parser::ModuleIdV0::new(module_path),
        );
        CssModuleTokenPreimageV0 {
            module_instance,
            module_path: module_path.to_string(),
            raw_name: "shared".to_string(),
            default_token: default_token.to_string(),
            linked_token: linked_token.to_string(),
            interface_token: default_token.to_string(),
        }
    }

    #[test]
    fn collision_path_scope_tracks_the_same_token_on_each_emission_path() -> Result<(), String> {
        let preimages = vec![
            collision_preimage("src/a.module.css", "_default", "_linked"),
            collision_preimage("src/b.module.css", "_default", "_linked"),
        ];
        let default_collisions =
            collision_groups(&preimages, |preimage| preimage.default_token.as_str());
        let linked_collisions =
            collision_groups(&preimages, |preimage| preimage.linked_token.as_str());
        let default_collision = default_collisions
            .get("_default")
            .ok_or_else(|| "missing default-path collision".to_string())?;

        // FALSIFIER: STRUCTURAL owner=CSS-Modules-token-integrity;
        // current CSS Modules execution converges both rewrite paths. Re-enter
        // when a path-specific producer can emit equal owners under distinct tokens.
        // At that boundary, an owner-only comparison would mislabel this as bothPaths.
        assert_eq!(
            collision_path_observation(
                "_default",
                default_collision,
                &default_collisions,
                &linked_collisions,
            )?,
            (
                CssModuleTokenCollisionPathScopeV0::ImportInlineLegacyOnly,
                vec![OmenaQueryBundleEmissionPathV0::ImportInlineLegacy.as_wire_label()],
            )
        );
        Ok(())
    }

    #[test]
    fn scanner_scope_follows_the_shared_identifier_predicate() {
        assert!(scanner_can_rewrite("card"));
        assert!(scanner_can_rewrite("카드"));
        assert!(!scanner_can_rewrite(r"a\20 b"));
    }

    #[test]
    fn selected_interface_token_detects_unrewritten_output() {
        let module_instance = omena_parser::ModuleInstanceKeyV0::unconfigured(
            omena_parser::ModuleIdV0::new("src/card.module.css"),
        );
        let preimages = vec![CssModuleTokenPreimageV0 {
            module_instance,
            module_path: "src/card.module.css".to_string(),
            raw_name: "card".to_string(),
            default_token: "_Ab1cdE_card".to_string(),
            linked_token: "_Ab1cdE_card".to_string(),
            interface_token: "_Ab1cdE_card".to_string(),
        }];
        let emitted_names = BTreeMap::from([(canonical_name("card"), "card".to_string())]);

        let mismatches = interface_mismatches_for_emitted_names(
            &preimages,
            OmenaQueryBundleEmissionPathV0::ImportInlineLegacy,
            &emitted_names,
        );
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].promised_token, "_Ab1cdE_card");
        assert_eq!(mismatches[0].emitted_token, "_Ab1cdE_card");
    }

    #[test]
    fn ownership_census_zero_owner_path_is_complete_and_empty() -> Result<(), String> {
        let census = CssModuleTokenOwnershipCensusV0::new(
            OmenaQueryBundleEmissionPathV0::ImportInlineLegacy.as_wire_label(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(census.complete);
        assert_eq!(census.emitted_token_count, 0);
        assert!(census.token_ownerships.is_empty());
        validate_css_module_token_integrity(&census)
    }

    #[test]
    fn ownership_census_analysis_unavailable_path_names_its_reason() {
        let census = CssModuleTokenOwnershipCensusV0::unavailable(
            OmenaQueryBundleEmissionPathV0::ImportInlineLegacy.as_wire_label(),
            "analysis unavailable control",
        );
        assert!(!census.complete);
        assert_eq!(census.unavailable_reasons, ["analysis unavailable control"]);
        assert!(
            validate_css_module_token_integrity(&census)
                .is_err_and(|error| error.contains("analysis unavailable control"))
        );
    }

    #[test]
    fn ownership_census_incomplete_attribution_path_names_the_unowned_token() {
        let census = CssModuleTokenOwnershipCensusV0::new(
            OmenaQueryBundleEmissionPathV0::ImportInlineLegacy.as_wire_label(),
            0,
            Vec::new(),
            Vec::new(),
            vec!["unowned-token".to_string()],
            Vec::new(),
        );
        assert!(!census.complete);
        assert!(census.unavailable_reasons.is_empty());
        assert!(
            validate_css_module_token_integrity(&census)
                .is_err_and(|error| error.contains("unowned-token"))
        );
    }

    #[test]
    fn equivalent_workspace_relative_identity_produces_equal_tokens() -> Result<(), String> {
        let entry_a =
            collect_omena_query_style_fact_entry("/filesystem/card.module.css", ".card {}");
        let entry_b = collect_omena_query_style_fact_entry("memory/card.module.css", ".card {}");
        let module_instance_a = super::super::css_modules::module_instance_key_relative_to_root(
            &omena_parser::ModuleInstanceKeyV0::unconfigured(omena_parser::ModuleIdV0::new(
                "/workspace-a/src/card.module.css",
            )),
            "/workspace-a",
        )?;
        let module_instance_b = super::super::css_modules::module_instance_key_relative_to_root(
            &omena_parser::ModuleInstanceKeyV0::unconfigured(omena_parser::ModuleIdV0::new(
                "/workspace-b/src/card.module.css",
            )),
            "/workspace-b",
        )?;
        assert_eq!(module_instance_a, module_instance_b);
        assert_eq!(
            derive_class_name_rewrites_for_module_instance(&entry_a, &module_instance_a),
            derive_class_name_rewrites_for_module_instance(&entry_b, &module_instance_b)
        );
        assert!(
            super::super::css_modules::module_instance_key_relative_to_root(
                &omena_parser::ModuleInstanceKeyV0::unconfigured(omena_parser::ModuleIdV0::new(
                    "/outside/card.module.css"
                ),),
                "/workspace-a",
            )
            .is_err()
        );
        Ok(())
    }
}
