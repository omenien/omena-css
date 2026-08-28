#![allow(deprecated)]

use std::collections::{BTreeMap, BTreeSet};

use omena_bundler::{
    BundleResolutionAuthorityV0, EmissionItemKindV0, EmissionOrderingPolicyV0, LinkedStylesheetV0,
    LinkedStylesheetWithEmissionItemsV0, TransformBundleDependencyResolutionV0,
    TransformBundleEdgeKind, TransformBundleLinkErrorV0, TransformBundleLinkOptionsV0,
    TransformBundleModuleInputV0, TransformBundleResolvedDependencyV0,
    TransformBundleSemanticReachabilityInputV0, link_legacy_path_inferred_bundle,
    link_omena_transform_bundle_modules, link_omena_transform_bundle_modules_with_options,
    link_omena_transform_bundle_modules_with_semantic_reachability,
    link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata,
    link_omena_transform_bundle_projection_with_emission_items, link_resolved_bundle,
    project_omena_transform_bundle_linker_and_emission_items,
};
use omena_parser::StyleDialect;
use omena_syntax::ident::AuthoredPropertyTextV0;
use serde_json::{Value, json};

const LINKED_STYLESHEET_BYTE_IDENTITY_SNAPSHOT: &str =
    include_str!("snapshots/linked-stylesheet-byte-identity.json");
const LINKED_STYLESHEET_EMISSION_ITEM_EXISTING_FIELDS_SNAPSHOT: &str =
    include_str!("snapshots/linked-stylesheet-emission-item-existing-fields.json");

#[test]
fn linked_stylesheet_output_matches_committed_contract() -> Result<(), String> {
    let (modules, reachability) = linked_stylesheet_inputs();
    let linked = linked_stylesheet_fixture(&modules, &reachability)?;
    assert_linked_stylesheet_fixture_is_non_vacuous(&linked)?;
    let extensionless = extensionless_fixture_linked_stylesheet()?;
    let two_candidate = two_candidate_fixture_linked_stylesheet()?;
    let current_by_key = keyed_stylesheets([
        serde_json::to_value(&linked).map_err(|error| format!("{error:?}"))?,
        serde_json::to_value(&extensionless).map_err(|error| format!("{error:?}"))?,
        serde_json::to_value(&two_candidate).map_err(|error| format!("{error:?}"))?,
    ])?;
    let committed = serde_json::from_str::<Value>(LINKED_STYLESHEET_BYTE_IDENTITY_SNAPSHOT)
        .map_err(|error| format!("{error:?}"))?;
    let committed_entries = committed
        .get("linkedStylesheets")
        .and_then(Value::as_array)
        .ok_or_else(|| "committed byte contract has no linkedStylesheets array".to_string())?;
    let committed_by_key = keyed_stylesheets(committed_entries.iter().cloned())?;
    let committed_count = committed
        .get("fixtureCount")
        .and_then(Value::as_u64)
        .ok_or_else(|| "committed byte contract has no numeric fixtureCount".to_string())?
        as usize;

    if std::env::var_os("OMENA_UPDATE_LINKED_STYLESHEET_BYTE_IDENTITY").is_some() {
        let mut updated = committed.clone();
        let updated_entries = updated
            .get_mut("linkedStylesheets")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| {
                "committed byte contract has no mutable linkedStylesheets array".to_string()
            })?;
        for entry in updated_entries {
            let key = linked_stylesheet_entry_key(entry)?;
            *entry = current_by_key.get(key.as_str()).cloned().ok_or_else(|| {
                format!("current linked stylesheet is missing baseline key {key}")
            })?;
        }
        let output = serde_json::to_string_pretty(&updated)
            .map_err(|error| format!("failed to serialize linked stylesheet baseline: {error}"))?;
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/snapshots/linked-stylesheet-byte-identity.json"
            ),
            format!("{output}\n"),
        )
        .map_err(|error| format!("failed to write linked stylesheet baseline: {error}"))?;
        println!("updated linked stylesheet byte-identity baseline");
        return Ok(());
    }

    // F16-a: this assertion owns the legacy LinkedStylesheetV0 bytes. Fixture-count growth is
    // additive; every entry that existed before the new fallback fixtures remains byte-identical.
    // FALSIFIER: id=linked-stylesheet-legacy-fixture-count class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=committed-entry-count-is-keyed
    assert_eq!(committed_count, committed_by_key.len());
    // FALSIFIER: id=linked-stylesheet-additive-fixture-growth class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=new-fixtures-do-not-remove-baseline
    assert!(current_by_key.len() >= committed_count);
    for (key, expected) in &committed_by_key {
        // FALSIFIER: id=linked-stylesheet-legacy-byte-contract class=placement via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=committed-legacy-entry-bytes
        assert_eq!(
            current_by_key.get(key),
            Some(expected),
            "legacy LinkedStylesheetV0 entry changed at {key}"
        );
    }

    let existing_with_disclosures = legacy_linked_stylesheet_with_emission_items(
        &["src/app.module.css"],
        &modules,
        &reachability,
        &[],
        EmissionOrderingPolicyV0::ModuleIdLegacy,
    )?;
    let existing_key = linked_stylesheet_entry_key(
        &serde_json::to_value(&existing_with_disclosures.linked_stylesheet)
            .map_err(|error| format!("{error:?}"))?,
    )?;
    // F16-a and F16-c have disjoint jurisdictions: the old serialized entry stays exact while
    // the sibling product artifact reports the two fallback-resolved dependency edges.
    // FALSIFIER: id=linked-stylesheet-bytes-with-disclosures class=placement via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=legacy-bytes-stable-with-nonempty-provenance
    assert_eq!(
        (
            serde_json::to_value(&existing_with_disclosures.linked_stylesheet)
                .map_err(|error| format!("{error:?}"))?,
            existing_with_disclosures
                .dependency_resolution_disclosures
                .len(),
        ),
        (
            committed_by_key
                .get(existing_key.as_str())
                .cloned()
                .ok_or_else(|| format!("missing committed entry {existing_key}"))?,
            2,
        )
    );
    Ok(())
}

#[test]
fn legacy_entry_points_preserve_the_same_serialized_stylesheet() -> Result<(), String> {
    let (modules, _) = linked_stylesheet_inputs();
    let direct = link_omena_transform_bundle_modules(&["src/app.module.css"], &modules)
        .map_err(|error| format!("{error:?}"))?;
    let semantic = link_omena_transform_bundle_modules_with_semantic_reachability(
        &["src/app.module.css"],
        &modules,
        &[],
    )
    .map_err(|error| format!("{error:?}"))?;
    let metadata = link_omena_transform_bundle_modules_with_semantic_reachability_and_metadata(
        &["src/app.module.css"],
        &modules,
        &[],
        &[],
    )
    .map_err(|error| format!("{error:?}"))?;
    let options = link_omena_transform_bundle_modules_with_options(
        &["src/app.module.css"],
        &modules,
        &[],
        &[],
        TransformBundleLinkOptionsV0::default(),
    )
    .map_err(|error| format!("{error:?}"))?;
    let serialized = [&direct, &semantic, &metadata, &options]
        .into_iter()
        .map(serde_json::to_vec)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{error:?}"))?;
    for pair in serialized.windows(2) {
        // F16-a: all four published legacy wrappers retain the same LinkedStylesheetV0 bytes.
        // FALSIFIER: id=linked-stylesheet-legacy-entrypoint-bytes class=placement via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=four-legacy-wrappers-agree
        assert_eq!(pair[0], pair[1]);
    }
    Ok(())
}

#[test]
fn emission_item_provenance_is_additive_to_the_existing_json_shape() -> Result<(), String> {
    let (modules, reachability) = linked_stylesheet_inputs();
    let linked = legacy_linked_stylesheet_with_emission_items(
        &["src/app.module.css"],
        &modules,
        &reachability,
        &[],
        EmissionOrderingPolicyV0::ImportOrderPreserving,
    )?;
    assert_emission_item_json_preserves_existing_keys(&linked)?;
    assert_emission_item_existing_fields_match_committed_baseline(&linked)
}

#[test]
fn extensionless_fallback_is_disclosed_and_strict_mode_rejects_it() -> Result<(), String> {
    let modules = extensionless_fixture_inputs();
    let legacy = legacy_linked_stylesheet_with_emission_items(
        &["src/extensionless.css"],
        &modules,
        &[],
        &[],
        EmissionOrderingPolicyV0::ImportOrderPreserving,
    )?;
    assert_emission_item_json_preserves_existing_keys(&legacy)?;
    let inferred = legacy
        .dependency_resolution_disclosures
        .iter()
        .filter(|disclosure| {
            disclosure.authority == BundleResolutionAuthorityV0::LegacyPathInferred
        })
        .collect::<Vec<_>>();
    // F16-c: the extension-less edge has no producer record, so a consumer-end legacy disclosure
    // proves that importer-relative candidate expansion actually supplied the target.
    // FALSIFIER: id=linked-stylesheet-extensionless-fallback-disclosure class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=extensionless-edge-has-consumer-provenance
    assert_eq!(
        inferred
            .iter()
            .map(|disclosure| {
                (
                    disclosure.source_instance.module().as_str(),
                    disclosure.import_source.as_str(),
                    disclosure.import_ordinal,
                )
            })
            .collect::<Vec<_>>(),
        vec![("src/extensionless.css", "./theme", Some(0))]
    );

    let projections = project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
    let strict = link_resolved_bundle(
        &["src/extensionless.css"],
        projections.linker_projection(),
        projections.emission_item_projection(),
        &[],
        &[],
        EmissionOrderingPolicyV0::ImportOrderPreserving,
    );
    // FALSIFIER: id=linked-stylesheet-extensionless-strict-rejection class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=resolved-mode-never-guesses
    assert_eq!(
        strict,
        Err(TransformBundleLinkErrorV0::UnresolvedDependencyEdge {
            source_path: "src/extensionless.css".to_string(),
            import_source: "./theme".to_string(),
            import_ordinal: Some(0),
        })
    );
    Ok(())
}

#[test]
fn two_candidate_fallback_names_the_selected_and_dropped_modules() -> Result<(), String> {
    const SELECTED_CANDIDATE: &str = "src/theme.css";
    const DROPPED_CANDIDATE: &str = "src/theme.scss";

    let modules = two_candidate_fixture_inputs();
    let legacy = legacy_linked_stylesheet_with_emission_items(
        &["src/candidates.css"],
        &modules,
        &[],
        &[],
        EmissionOrderingPolicyV0::ImportOrderPreserving,
    )?;
    assert_emission_item_json_preserves_existing_keys(&legacy)?;
    let linked_modules = legacy
        .linked_stylesheet
        .module_instances
        .iter()
        .map(|instance| instance.module().as_str())
        .collect::<BTreeSet<_>>();
    let inferred = legacy
        .dependency_resolution_disclosures
        .iter()
        .filter(|disclosure| {
            disclosure.authority == BundleResolutionAuthorityV0::LegacyPathInferred
                && disclosure.import_source == "./theme"
                && disclosure.import_ordinal == Some(0)
        })
        .count();
    // F16-c: the prescribed disclosure identifies the edge, while this fixture names both the
    // deterministic .css winner and the silently dropped .scss candidate.
    // FALSIFIER: id=linked-stylesheet-two-candidate-fallback class=placement via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=winner-and-loser-are-explicit
    assert_eq!(
        (
            linked_modules.contains(SELECTED_CANDIDATE),
            linked_modules.contains(DROPPED_CANDIDATE),
            inferred,
        ),
        (true, false, 1),
        "selected candidate {SELECTED_CANDIDATE}; dropped candidate {DROPPED_CANDIDATE}"
    );
    Ok(())
}

#[test]
fn partial_resolution_supply_discloses_only_the_unmatched_edge() -> Result<(), String> {
    let modules = partial_supply_fixture_inputs();
    let projections = project_omena_transform_bundle_linker_and_emission_items(&modules, &[]);
    let resolved_tokens = TransformBundleResolvedDependencyV0::new(
        modules[0].module_instance_key(),
        TransformBundleEdgeKind::CssImport,
        "./tokens.css",
        Some(0),
        TransformBundleDependencyResolutionV0::attempted(
            vec!["fileRelativeOrAbsolute"],
            "fileRelative",
            1,
            Some(modules[1].module_instance_key()),
        ),
    );
    let legacy = link_legacy_path_inferred_bundle(
        &["src/partial.css"],
        projections.linker_projection(),
        projections.emission_item_projection(),
        std::slice::from_ref(&resolved_tokens),
        &[],
        EmissionOrderingPolicyV0::ImportOrderPreserving,
    )
    .map_err(|error| format!("{error:?}"))?;
    assert_emission_item_json_preserves_existing_keys(&legacy)?;
    let authority_by_edge = legacy
        .dependency_resolution_disclosures
        .iter()
        .map(|disclosure| {
            (
                disclosure.import_source.as_str(),
                disclosure.import_ordinal,
                disclosure.authority,
            )
        })
        .collect::<Vec<_>>();
    // F16-c: a partially supplied call keeps the supplied sibling Resolved and identifies exactly
    // the edge that fell back. Counting only the call-level mode would not satisfy this assertion.
    // FALSIFIER: id=linked-stylesheet-partial-supply-authority class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=authority-is-per-edge
    assert_eq!(
        authority_by_edge,
        vec![
            (
                "./theme",
                Some(1),
                BundleResolutionAuthorityV0::LegacyPathInferred,
            ),
            (
                "./tokens.css",
                Some(0),
                BundleResolutionAuthorityV0::Resolved,
            ),
        ]
    );
    Ok(())
}

#[test]
fn emission_item_order_covers_non_class_rules_without_widening_legacy_order() -> Result<(), String>
{
    let (modules, reachability) = linked_stylesheet_inputs();
    let projections =
        project_omena_transform_bundle_linker_and_emission_items(&modules, &reachability);
    let linked = link_omena_transform_bundle_projection_with_emission_items(
        &["src/app.module.css"],
        projections.linker_projection(),
        projections.emission_item_projection(),
        &[],
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

fn keyed_stylesheets(
    entries: impl IntoIterator<Item = Value>,
) -> Result<BTreeMap<String, Value>, String> {
    let mut keyed = BTreeMap::new();
    for entry in entries {
        let key = linked_stylesheet_entry_key(&entry)?;
        if keyed.insert(key.clone(), entry).is_some() {
            // FALSIFIER: id=linked-stylesheet-entry-key-unique class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=one-baseline-entry-per-entrypoint-instance
            return Err(format!("duplicate linked stylesheet entry key: {key}"));
        }
    }
    Ok(keyed)
}

fn linked_stylesheet_entry_key(entry: &Value) -> Result<String, String> {
    let entrypoint = entry
        .get("entrypoints")
        .and_then(Value::as_array)
        .and_then(|entrypoints| entrypoints.first())
        .ok_or_else(|| "linked stylesheet entry has no first entrypoint".to_string())?;
    let module = entrypoint
        .get("module")
        .and_then(Value::as_str)
        .ok_or_else(|| "linked stylesheet entrypoint has no module".to_string())?;
    let configuration = entrypoint
        .get("configuration")
        .and_then(Value::as_str)
        .ok_or_else(|| "linked stylesheet entrypoint has no configuration".to_string())?;
    Ok(format!("{module}#{configuration}"))
}

fn legacy_linked_stylesheet_with_emission_items(
    entrypoints: &[&str],
    modules: &[TransformBundleModuleInputV0],
    reachability: &[TransformBundleSemanticReachabilityInputV0],
    resolved_dependencies: &[TransformBundleResolvedDependencyV0],
    emission_ordering_policy: EmissionOrderingPolicyV0,
) -> Result<LinkedStylesheetWithEmissionItemsV0, String> {
    let projections =
        project_omena_transform_bundle_linker_and_emission_items(modules, reachability);
    link_legacy_path_inferred_bundle(
        entrypoints,
        projections.linker_projection(),
        projections.emission_item_projection(),
        resolved_dependencies,
        &[],
        emission_ordering_policy,
    )
    .map_err(|error| format!("{error:?}"))
}

fn assert_emission_item_json_preserves_existing_keys(
    linked: &LinkedStylesheetWithEmissionItemsV0,
) -> Result<(), String> {
    let actual_value = serde_json::to_value(linked).map_err(|error| format!("{error:?}"))?;
    let actual = actual_value
        .as_object()
        .ok_or_else(|| "emission-item artifact is not a JSON object".to_string())?;
    let actual_keys = actual.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_keys = BTreeSet::from([
        "linkedStylesheet",
        "emissionItemPlan",
        "emissionItemOrder",
        "projectionDisclosures",
    ]);

    // The shape arm remains additive; cardinality is owned by the committed baseline below.
    // FALSIFIER: id=linked-stylesheet-emission-item-key-superset class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=existing-json-keys-remain
    assert!(expected_keys.is_subset(&actual_keys));
    // FALSIFIER: id=linked-stylesheet-disclosure-array-shape class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=new-provenance-member-is-additive-array
    assert!(
        actual
            .get("dependencyResolutionDisclosures")
            .is_some_and(Value::is_array)
    );
    Ok(())
}

fn assert_emission_item_existing_fields_match_committed_baseline(
    linked: &LinkedStylesheetWithEmissionItemsV0,
) -> Result<(), String> {
    let actual = serde_json::to_value(linked).map_err(|error| format!("{error:?}"))?;
    let actual_summary = json!({
        "schemaVersion": "0",
        "product": "omena-bundler.linked-stylesheet.emission-item-existing-fields",
        "linkedStylesheet": {
            "entrypointCount": actual["linkedStylesheet"]["entrypoints"]
                .as_array()
                .map_or(0, Vec::len),
            "moduleInstanceCount": actual["linkedStylesheet"]["moduleInstances"]
                .as_array()
                .map_or(0, Vec::len),
            "globalRuleCount": actual["linkedStylesheet"]["globalRuleOrder"]["rules"]
                .as_array()
                .map_or(0, Vec::len),
        },
        "emissionItemPlan": {
            "policy": actual["emissionItemPlan"]["policy"].clone(),
            "entryCount": actual["emissionItemPlan"]["entries"]
                .as_array()
                .map_or(0, Vec::len),
        },
        "emissionItemOrder": {
            "itemCount": actual["emissionItemOrder"]["items"]
                .as_array()
                .map_or(0, Vec::len),
        },
        "projectionDisclosures": {
            "itemCount": actual["projectionDisclosures"]
                .as_array()
                .map_or(0, Vec::len),
        },
    });
    let expected =
        serde_json::from_str::<Value>(LINKED_STYLESHEET_EMISSION_ITEM_EXISTING_FIELDS_SNAPSHOT)
            .map_err(|error| format!("{error:?}"))?;
    // F16-b uses a committed expected side; truncating the prior eighteen-row public array is RED.
    // FALSIFIER: id=linked-stylesheet-emission-item-existing-values class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-stylesheet-byte-contract entry=existing-json-cardinality-matches-committed-baseline
    assert_eq!(actual_summary, expected);
    Ok(())
}

fn extensionless_fixture_inputs() -> Vec<TransformBundleModuleInputV0> {
    vec![
        TransformBundleModuleInputV0::new(
            "src/extensionless.css",
            r#"@import "./theme"; .entry { color: green; }"#,
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/theme.css",
            ".theme { color: purple; }",
            StyleDialect::Css,
        ),
    ]
}

fn two_candidate_fixture_inputs() -> Vec<TransformBundleModuleInputV0> {
    vec![
        TransformBundleModuleInputV0::new(
            "src/candidates.css",
            r#"@import "./theme"; .entry { color: green; }"#,
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/theme.css",
            ".css-theme { color: purple; }",
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/theme.scss",
            ".scss-theme { color: orange; }",
            StyleDialect::Scss,
        ),
    ]
}

fn partial_supply_fixture_inputs() -> Vec<TransformBundleModuleInputV0> {
    vec![
        TransformBundleModuleInputV0::new(
            "src/partial.css",
            r#"@import "./tokens.css"; @import "./theme"; .entry { color: green; }"#,
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/tokens.css",
            ".token { color: rebeccapurple; }",
            StyleDialect::Css,
        ),
        TransformBundleModuleInputV0::new(
            "src/theme.scss",
            ".theme { color: purple; }",
            StyleDialect::Scss,
        ),
    ]
}

fn extensionless_fixture_linked_stylesheet() -> Result<LinkedStylesheetV0, String> {
    link_omena_transform_bundle_modules(&["src/extensionless.css"], &extensionless_fixture_inputs())
        .map_err(|error| format!("{error:?}"))
}

fn two_candidate_fixture_linked_stylesheet() -> Result<LinkedStylesheetV0, String> {
    link_omena_transform_bundle_modules(&["src/candidates.css"], &two_candidate_fixture_inputs())
        .map_err(|error| format!("{error:?}"))
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
        .push(AuthoredPropertyTextV0::new("--app-token"));

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
