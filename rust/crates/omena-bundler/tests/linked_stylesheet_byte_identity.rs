use std::collections::BTreeSet;

use omena_bundler::{
    LinkedStylesheetV0, TransformBundleModuleInputV0, TransformBundleSemanticReachabilityInputV0,
    link_omena_transform_bundle_modules_with_semantic_reachability,
};
use omena_parser::StyleDialect;
use serde_json::json;

const LINKED_STYLESHEET_BYTE_IDENTITY_SNAPSHOT: &str =
    include_str!("snapshots/linked-stylesheet-byte-identity.json");

#[test]
fn linked_stylesheet_output_matches_committed_contract() -> Result<(), String> {
    let linked = linked_stylesheet_fixture()?;
    assert_linked_stylesheet_fixture_is_non_vacuous(&linked);

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

    assert_eq!(current, LINKED_STYLESHEET_BYTE_IDENTITY_SNAPSHOT);
    Ok(())
}

fn linked_stylesheet_fixture() -> Result<LinkedStylesheetV0, String> {
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

    link_omena_transform_bundle_modules_with_semantic_reachability(
        &["src/app.module.css"],
        &modules,
        &[reachability],
    )
    .map_err(|err| format!("{err:?}"))
}

fn assert_linked_stylesheet_fixture_is_non_vacuous(linked: &LinkedStylesheetV0) {
    let rule_modules = linked
        .global_rule_order
        .rules
        .iter()
        .map(|rule| rule.module_instance.clone())
        .collect::<BTreeSet<_>>();
    assert!(linked.global_rule_order.rules.len() >= 2);
    assert!(rule_modules.len() >= 2);

    assert!(
        !linked
            .module_instances
            .iter()
            .any(|instance| instance.module().as_str() == "src/dead.module.css")
    );

    let reachable_classes = linked.closed_world_bundle.reachability().class_names();
    assert!(reachable_classes.contains(&"app-live".to_string()));
    assert!(!reachable_classes.contains(&"appAlt".to_string()));
}
