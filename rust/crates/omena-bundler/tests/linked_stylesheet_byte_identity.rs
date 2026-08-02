#![allow(deprecated)]

use std::collections::BTreeSet;

use omena_bundler::{
    EmissionItemKindV0, EmissionOrderingPolicyV0, LinkedStylesheetV0, TransformBundleLinkOptionsV0,
    TransformBundleModuleInputV0, TransformBundleSemanticReachabilityInputV0,
    link_omena_transform_bundle_modules_with_semantic_reachability,
    link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options,
    project_omena_transform_bundle_linker_and_emission_items,
};
use omena_parser::StyleDialect;
use serde_json::json;

const LINKED_STYLESHEET_BYTE_IDENTITY_SNAPSHOT: &str =
    include_str!("snapshots/linked-stylesheet-byte-identity.json");

#[test]
fn linked_stylesheet_output_matches_committed_contract() -> Result<(), String> {
    let (modules, reachability) = linked_stylesheet_inputs();
    let linked = linked_stylesheet_fixture(&modules, &reachability)?;
    assert_linked_stylesheet_fixture_is_non_vacuous(&linked)?;

    let snapshot = json!({
        "schemaVersion": "0",
        "product": "omena-bundler.linked-stylesheet.byte-identity-corpus",
        "fixtureCount": 1,
        "linkedStylesheets": [linked],
    });
    let current = format!(
        "{}\n",
        serde_json::to_string_pretty(&snapshot).map_err(|err| format!("{err:?}"))?
    );

    // FALSIFIER: id=linked-stylesheet-legacy-byte-contract class=placement via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=committed-legacy-entry-bytes
    assert_eq!(current, LINKED_STYLESHEET_BYTE_IDENTITY_SNAPSHOT);
    Ok(())
}

#[test]
fn emission_item_order_covers_non_class_rules_without_widening_legacy_order() -> Result<(), String>
{
    let (modules, reachability) = linked_stylesheet_inputs();
    let projections =
        project_omena_transform_bundle_linker_and_emission_items(&modules, &reachability);
    let linked =
        link_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options(
            &["src/app.module.css"],
            projections.linker_projection(),
            projections.emission_item_projection(),
            &[],
            &[],
            TransformBundleLinkOptionsV0 {
                emission_ordering_policy: EmissionOrderingPolicyV0::ImportOrderPreserving,
            },
        )
        .map_err(|error| format!("{error:?}"))?;

    // FALSIFIER: id=linked-stylesheet-legacy-rule-count class=placement via=--inject-linked-rule-misattribution producer=can-fail owner=linked-stylesheet-byte-contract entry=legacy-rule-order-complete
    assert_eq!(linked.linked_stylesheet.global_rule_order.rules.len(), 5);
    // FALSIFIER: id=linked-stylesheet-legacy-rule-kinds class=placement via=--inject-linked-rule-misattribution producer=can-fail owner=linked-stylesheet-byte-contract entry=legacy-rule-order-class-only
    assert!(
        linked
            .linked_stylesheet
            .global_rule_order
            .rules
            .iter()
            .all(|rule| rule.selector_kind == "class")
    );
    let theme_items = linked
        .emission_item_order
        .items
        .iter()
        .filter(|item| item.module_instance.module().as_str() == "src/theme.css")
        .collect::<Vec<_>>();
    // FALSIFIER: id=linked-stylesheet-theme-pseudo-class class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-stylesheet-byte-contract entry=theme-pseudo-class-emitted
    assert!(theme_items.iter().any(|item| {
        item.kind == EmissionItemKindV0::SelectorPseudoClass && item.name == ":root"
    }));
    // FALSIFIER: id=linked-stylesheet-theme-keyframes class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-stylesheet-byte-contract entry=theme-keyframes-emitted
    assert!(theme_items.iter().any(|item| {
        item.kind == EmissionItemKindV0::KeyframesDeclaration && item.name == "pulse"
    }));
    Ok(())
}

fn linked_stylesheet_inputs() -> (
    Vec<TransformBundleModuleInputV0>,
    Vec<TransformBundleSemanticReachabilityInputV0>,
) {
    let modules = vec![
        TransformBundleModuleInputV0::new(
            "src/app.module.css",
            r#"@import "./theme.css"; @import "./components/card.module.css"; .app { color: var(--brand); } .appAlt { color: blue; }"#,
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/theme.css",
            r#":root { --brand: red; } .theme { color: red; } @keyframes pulse { from { opacity: 0; } to { opacity: 1; } }"#,
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/components/card.module.css",
            r#".card { color: green; } .cardTitle { font-weight: 700; }"#,
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/dead.module.css",
            r#".dead { color: black; }"#,
            StyleDialect::Css,
        ),
    ];
    let mut reachability = TransformBundleSemanticReachabilityInputV0::new("src/app.module.css");
    reachability.class_names.push("app-live".to_string());
    reachability
        .custom_property_names
        .push("--app-token".to_string());

    (modules, vec![reachability])
}

fn linked_stylesheet_fixture(
    modules: &[TransformBundleModuleInputV0],
    reachability: &[TransformBundleSemanticReachabilityInputV0],
) -> Result<LinkedStylesheetV0, String> {
    link_omena_transform_bundle_modules_with_semantic_reachability(
        &["src/app.module.css"],
        modules,
        reachability,
    )
    .map_err(|err| format!("{err:?}"))
}

fn assert_linked_stylesheet_fixture_is_non_vacuous(
    linked: &LinkedStylesheetV0,
) -> Result<(), String> {
    let rule_modules = linked
        .global_rule_order
        .rules
        .iter()
        .map(|rule| rule.module_instance.clone())
        .collect::<BTreeSet<_>>();
    // FALSIFIER: id=linked-stylesheet-nonempty-rule-order class=placement via=--inject-live-declaration-loss producer=can-fail owner=linked-stylesheet-byte-contract entry=multi-rule-linked-output
    assert!(linked.global_rule_order.rules.len() >= 2);
    // FALSIFIER: id=linked-stylesheet-multi-module-rule-order class=placement via=--inject-cross-module-declaration-loss producer=can-fail owner=linked-stylesheet-byte-contract entry=multi-module-linked-output
    assert!(rule_modules.len() >= 2);

    // FALSIFIER: id=linked-stylesheet-dead-module-excluded class=liveness via=--inject-authored-liveness-flip producer=can-fail owner=linked-stylesheet-byte-contract entry=unreachable-module-not-emitted
    assert!(
        !linked
            .module_instances
            .iter()
            .any(|instance| instance.module().as_str() == "src/dead.module.css")
    );

    let reachable_classes = linked.closed_world_bundle.reachability().class_names();
    // FALSIFIER: id=linked-stylesheet-live-class-retained class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-stylesheet-byte-contract entry=authored-live-class-retained
    assert!(reachable_classes.contains(&"app-live".to_string()));
    // FALSIFIER: id=linked-stylesheet-dead-class-excluded class=liveness via=--inject-authored-liveness-flip producer=can-fail owner=linked-stylesheet-byte-contract entry=authored-dead-class-excluded
    assert!(!reachable_classes.contains(&"appAlt".to_string()));
    let dead_instance = linked
        .closed_world_bundle
        .reachability()
        .module_qualified_symbols()
        .iter()
        .find(|symbols| symbols.module_instance().module().as_str() == "src/dead.module.css")
        .ok_or_else(|| {
            "the known dead module should remain part of the qualified symbol universe".to_string()
        })?;
    // FALSIFIER: id=linked-stylesheet-dead-instance-unreachable class=liveness via=--inject-authored-liveness-flip producer=can-fail owner=linked-stylesheet-byte-contract entry=dead-instance-marked-unreachable
    assert!(!dead_instance.is_reachable());
    // FALSIFIER: id=linked-stylesheet-dead-instance-symbols-empty class=liveness via=--inject-authored-liveness-flip producer=can-fail owner=linked-stylesheet-byte-contract entry=dead-instance-symbol-set-empty
    assert!(dead_instance.class_names().is_empty());
    Ok(())
}
