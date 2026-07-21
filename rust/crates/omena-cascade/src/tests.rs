use super::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn declaration(id: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    declaration_with_specificity_exactness(id, value, key, SpecificityExactnessV0::Exact)
}

fn declaration_with_specificity_exactness(
    id: &str,
    value: &str,
    key: CascadeKey,
    specificity_exactness: SpecificityExactnessV0,
) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: "color".to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key,
        specificity_exactness,
    }
}

fn property_declaration(
    id: &str,
    property: &str,
    value: CascadeValue,
    source_order: u32,
) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: property.to_string(),
        value,
        key: key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            source_order,
        ),
        specificity_exactness: SpecificityExactnessV0::Exact,
    }
}

fn key(
    level: CascadeLevel,
    layer_rank: i32,
    scope_proximity: u32,
    specificity: Specificity,
    source_order: u32,
) -> CascadeKey {
    CascadeKey::new(
        level,
        LayerRank(layer_rank),
        scope_proximity,
        specificity,
        ModuleRank::ZERO,
        source_order,
    )
}

#[test]
fn orders_specificity_lexicographically() {
    assert!(Specificity::new(1, 0, 0) > Specificity::new(0, 99, 99));
    assert!(Specificity::new(0, 2, 0) > Specificity::new(0, 1, 99));
    assert!(Specificity::new(0, 0, 2) > Specificity::new(0, 0, 1));
}

#[test]
fn origin_inputs_drive_every_non_temporal_cascade_level() {
    let driven_levels = cascade_origin_driver_catalog_v0()
        .into_iter()
        .map(|driver| driver.level)
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        CascadeLevel::UserAgentNormal,
        CascadeLevel::UserNormal,
        CascadeLevel::AuthorNormal,
        CascadeLevel::InlineNormal,
        CascadeLevel::AuthorImportant,
        CascadeLevel::UserImportant,
        CascadeLevel::UserAgentImportant,
    ]);

    assert_eq!(driven_levels, expected);
    assert_eq!(cascade_level_catalog_v0().len(), 9);
    assert_eq!(driven_levels.len(), 7);
    assert_eq!(
        cascade_level_for_origin(CascadeOriginV0::Inline, true),
        CascadeLevel::AuthorImportant
    );
}

#[test]
fn derives_scope_proximity_from_the_nearest_matching_ancestor() {
    let target = ElementIdentityV0 {
        source_path: "Child.tsx".to_string(),
        byte_start: 1,
        byte_end: 2,
    };
    let near = ElementIdentityV0 {
        source_path: "Parent.tsx".to_string(),
        byte_start: 3,
        byte_end: 4,
    };
    let far = ElementIdentityV0 {
        source_path: "Root.tsx".to_string(),
        byte_start: 5,
        byte_end: 6,
    };
    let result = scope_proximity_from_ancestor_signatures(
        ".scope-root",
        &[
            (
                target,
                ElementSignature::concrete(Some("span"), None::<String>, [] as [&str; 0]),
            ),
            (
                near.clone(),
                ElementSignature::concrete(Some("section"), None::<String>, ["scope-root"]),
            ),
            (
                far,
                ElementSignature::concrete(Some("main"), None::<String>, ["scope-root"]),
            ),
        ],
        true,
    );

    assert_eq!(result.status, ScopeProximityStatusV0::Known);
    assert_eq!(result.distance, Some(1));
    assert_eq!(result.matched_root, Some(near));
}

#[test]
fn nearer_derived_scope_root_wins_between_equal_declarations() {
    let element = |source_path: &str, byte_start: usize| ElementIdentityV0 {
        source_path: source_path.to_string(),
        byte_start,
        byte_end: byte_start + 1,
    };
    let signature = |classes: &[&str]| {
        ElementSignature::concrete(Some("div"), None::<String>, classes.iter().copied())
    };
    let near = scope_proximity_from_ancestor_signatures(
        ".scope-root",
        &[
            (element("Near.tsx", 1), signature(&[])),
            (element("Near.tsx", 3), signature(&["scope-root"])),
        ],
        true,
    );
    let far = scope_proximity_from_ancestor_signatures(
        ".scope-root",
        &[
            (element("Far.tsx", 1), signature(&[])),
            (element("Far.tsx", 3), signature(&[])),
            (element("Far.tsx", 5), signature(&["scope-root"])),
        ],
        true,
    );
    assert_eq!(near.distance, Some(1));
    assert_eq!(far.distance, Some(2));
    let near_distance = near.distance.unwrap_or(u32::MAX);
    let far_distance = far.distance.unwrap_or(u32::MAX);

    let outcome = cascade_property(
        [
            declaration(
                "far-scope",
                "red",
                key(
                    CascadeLevel::AuthorNormal,
                    0,
                    far_distance,
                    Specificity::new(0, 1, 0),
                    1,
                ),
            ),
            declaration(
                "near-scope",
                "blue",
                key(
                    CascadeLevel::AuthorNormal,
                    0,
                    near_distance,
                    Specificity::new(0, 1, 0),
                    1,
                ),
            ),
        ],
        "color",
    );

    assert!(matches!(
        outcome,
        CascadeOutcome::Definite { ref winner, .. } if winner.id == "near-scope"
    ));
}

#[test]
fn keeps_scope_proximity_unknown_for_inexact_dynamic_classes() {
    let mut signature = ElementSignature::at_least_classes(Vec::<String>::new());
    signature.tag = Some("section".to_string());
    signature.tag_is_exact = true;
    let result = scope_proximity_from_ancestor_signatures(
        ".scope-root",
        &[(
            ElementIdentityV0 {
                source_path: "View.tsx".to_string(),
                byte_start: 1,
                byte_end: 2,
            },
            signature,
        )],
        true,
    );

    assert_eq!(
        result.status,
        ScopeProximityStatusV0::UnsupportedRootSelector
    );
    assert_eq!(result.distance, None);
}

#[test]
fn orders_cascade_keys_by_level_layer_scope_specificity_and_source() {
    let base = key(
        CascadeLevel::AuthorNormal,
        0,
        3,
        Specificity::new(0, 1, 0),
        1,
    );
    assert!(
        key(
            CascadeLevel::AuthorImportant,
            0,
            3,
            Specificity::new(0, 1, 0),
            1,
        ) > base
    );
    assert!(
        key(
            CascadeLevel::AuthorNormal,
            1,
            3,
            Specificity::new(0, 1, 0),
            1,
        ) > base
    );
    assert!(
        key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            1,
        ) > base
    );
    assert!(
        key(
            CascadeLevel::AuthorNormal,
            0,
            3,
            Specificity::new(0, 2, 0),
            1,
        ) > base
    );
    assert!(
        key(
            CascadeLevel::AuthorNormal,
            0,
            3,
            Specificity::new(0, 1, 0),
            2,
        ) > base
    );
}

#[test]
fn carries_module_rank_without_using_it_as_an_exact_order_axis() {
    let css_specificity_winner = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::new(0, 2, 0),
        ModuleRank::ZERO,
        1,
    );
    let module_rank_winner = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::new(0, 1, 0),
        ModuleRank::new(u32::MAX, u32::MAX, u32::MAX),
        2,
    );
    assert!(
        css_specificity_winner > module_rank_winner,
        "real CSS specificity must outrank import-graph provenance evidence"
    );

    let earlier_module = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::ZERO,
        ModuleRank::new(u32::MAX, u32::MAX, u32::MAX),
        1,
    );
    let later_module = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::ZERO,
        ModuleRank::ZERO,
        2,
    );
    assert!(
        later_module > earlier_module,
        "exact cascade ordering uses source order, not module provenance rank"
    );

    let first = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::ZERO,
        ModuleRank::ZERO,
        1,
    );
    let same_exact_key_different_module_rank = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::ZERO,
        ModuleRank::new(1, 2, 3),
        1,
    );
    assert_eq!(
        first.cmp(&same_exact_key_different_module_rank),
        std::cmp::Ordering::Equal
    );
}

#[test]
fn open_world_ambiguity_returns_ranked_set_with_module_rank_hint() {
    let weaker_module_hint = declaration(
        "weaker-module-hint",
        "red",
        CascadeKey::new(
            CascadeLevel::AuthorNormal,
            LayerRank(0),
            1,
            Specificity::ZERO,
            ModuleRank::ZERO,
            1,
        ),
    );
    let stronger_module_hint = declaration(
        "stronger-module-hint",
        "blue",
        CascadeKey::new(
            CascadeLevel::AuthorNormal,
            LayerRank(0),
            1,
            Specificity::ZERO,
            ModuleRank::new(u32::MAX, u32::MAX, u32::MAX),
            1,
        ),
    );

    let outcome = cascade_property_open_world([weaker_module_hint, stronger_module_hint], "color");

    assert!(
        matches!(outcome, CascadeOutcome::RankedSet(_)),
        "open-world ambiguity must not fabricate a definite winner"
    );
    let CascadeOutcome::RankedSet(ranked) = outcome else {
        return;
    };
    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].id, "stronger-module-hint");
    assert_eq!(ranked[1].id, "weaker-module-hint");
}

#[test]
fn open_world_strict_cascade_level_dominance_returns_definite() {
    let normal = declaration(
        "author-normal",
        "red",
        key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(1, 0, 0),
            99,
        ),
    );
    let important = declaration(
        "author-important",
        "blue",
        key(CascadeLevel::AuthorImportant, 0, 1, Specificity::ZERO, 1),
    );

    for declarations in [
        [normal.clone(), important.clone()],
        [important.clone(), normal.clone()],
    ] {
        let outcome = cascade_property_open_world(declarations, "color");
        assert!(
            matches!(&outcome, CascadeOutcome::Definite { .. }),
            "strict cascade-level dominance must select a definite winner"
        );
        if let CascadeOutcome::Definite {
            winner,
            also_considered,
            ..
        } = outcome
        {
            assert_eq!(winner.id, "author-important");
            assert_eq!(also_considered.len(), 1);
            assert_eq!(also_considered[0].id, "author-normal");
        }
    }
}

#[test]
fn open_world_strict_scope_dominance_uses_nearer_scope() {
    let farther = declaration(
        "farther-scope",
        "red",
        key(CascadeLevel::AuthorNormal, 0, 3, Specificity::ZERO, 1),
    );
    let nearer = declaration(
        "nearer-scope",
        "blue",
        key(CascadeLevel::AuthorNormal, 0, 1, Specificity::ZERO, 1),
    );

    let outcome = cascade_property_open_world([farther, nearer], "color");
    assert!(matches!(
        outcome,
        CascadeOutcome::Definite { ref winner, .. } if winner.id == "nearer-scope"
    ));
}

#[test]
fn selects_definite_winner_with_proof() {
    let earlier = declaration(
        "earlier",
        "red",
        key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            1,
        ),
    );
    let later = declaration(
        "later",
        "blue",
        key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            2,
        ),
    );

    let outcome = cascade_property([earlier, later], "color");

    assert!(matches!(outcome, CascadeOutcome::Definite { .. }));
    if let CascadeOutcome::Definite {
        winner,
        proof,
        also_considered,
    } = &outcome
    {
        assert_eq!(winner.id, "later");
        assert_eq!(proof.declaration_id, "later");
        assert_eq!(also_considered.len(), 1);
    }

    let margin = cascade_margin_for_outcome(&outcome);
    assert!(margin.is_some(), "definite outcome has margin");
    let Some(margin) = margin else {
        return;
    };
    assert_eq!(margin.product, "omena-cascade.margin");
    assert_eq!(margin.margin_kind, "lexicographicCascadeKeyDelta");
    assert_eq!(margin.winner_declaration_id, "later");
    assert_eq!(margin.challenger_declaration_id.as_deref(), Some("earlier"));
    assert_eq!(margin.dominant_axis, "sourceOrder");
    assert_eq!(margin.signed_distance, 1);
    assert!(!margin.public_safety_claim_ready);
}

#[test]
fn selects_generic_winner_with_same_cascade_ordering() {
    let ranked = select_cascade_winner(["earlier", "later"], |item| match *item {
        "earlier" => key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            1,
        ),
        _ => key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            2,
        ),
    });

    let Some((winner, also_considered)) = ranked else {
        unreachable!("test input contains candidates")
    };
    assert_eq!(winner, "later");
    assert_eq!(also_considered, vec!["earlier"]);
}

#[test]
fn cascade_margin_schema_is_substrate_only_until_calibrated() {
    let schema = summarize_cascade_margin_schema_v0();

    assert_eq!(schema.schema_version, "0");
    assert_eq!(schema.product, "omena-cascade.margin-schema");
    assert_eq!(schema.margin_kind, "lexicographicCascadeKeyDelta");
    assert_eq!(
        schema.axis_order,
        vec![
            "level",
            "layerRank",
            "scopeProximity",
            "specificityIds",
            "specificityClasses",
            "specificityElements",
            "sourceOrder",
        ]
    );
    assert_eq!(schema.calibration_stage, "schemaOnlyUncalibrated");
    assert!(!schema.public_safety_claim_ready);
}

#[test]
fn computes_values_through_var_substitution() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        "--brand".to_string(),
        CascadeValue::Literal("red".to_string()),
    );

    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            "color-decl",
            "color",
            CascadeValue::Var {
                name: "--brand".to_string(),
                fallback: None,
            },
            1,
        )],
        custom_property_env: env,
        parent_computed_value: Some(CascadeValue::Literal("blue".to_string())),
        registered_custom_property: None,
    });

    assert_eq!(result.product, "omena-cascade.computed-value");
    assert_eq!(result.status, ComputedCascadeValueStatusV0::Resolved);
    assert_eq!(result.value, CascadeValue::Literal("red".to_string()));
    assert_eq!(result.winner_declaration_id.as_deref(), Some("color-decl"));
    assert!(!result.inherited);
    assert!(!result.used_initial_value);
    assert!(!result.invalid_at_computed_value_time);
    assert!(result.derivation_steps.contains(&"computedValueResolved"));
}

#[test]
fn resolves_inheritance_initial_and_unset_keywords() {
    let inherited = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("purple".to_string())),
        registered_custom_property: None,
    });
    assert_eq!(inherited.status, ComputedCascadeValueStatusV0::Inherited);
    assert_eq!(inherited.value, CascadeValue::Literal("purple".to_string()));
    assert!(inherited.inherited);

    let initial = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "opacity".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("0.5".to_string())),
        registered_custom_property: None,
    });
    assert_eq!(initial.status, ComputedCascadeValueStatusV0::Initial);
    assert_eq!(initial.value, CascadeValue::Literal("1".to_string()));
    assert!(initial.used_initial_value);

    let unset_inherited = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            "unset-color",
            "color",
            CascadeValue::Unset,
            1,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("green".to_string())),
        registered_custom_property: None,
    });
    assert_eq!(
        unset_inherited.status,
        ComputedCascadeValueStatusV0::Inherited
    );
    assert_eq!(
        unset_inherited.value,
        CascadeValue::Literal("green".to_string())
    );

    let unset_initial = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "opacity".to_string(),
        declarations: vec![property_declaration(
            "unset-opacity",
            "opacity",
            CascadeValue::Unset,
            1,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("0.5".to_string())),
        registered_custom_property: None,
    });
    assert_eq!(unset_initial.status, ComputedCascadeValueStatusV0::Initial);
    assert_eq!(unset_initial.value, CascadeValue::Literal("1".to_string()));
}

#[test]
fn property_metadata_db_preserves_seed_inheritance_and_initial_values() {
    assert!(CSS_PROPERTY_METADATA_RECORDS_V1.len() > 29);
    assert_eq!(
        css_property_is_inherited("color"),
        CssPropertyInheritanceV0::Inherited
    );
    assert_eq!(
        css_property_is_inherited("font"),
        CssPropertyInheritanceV0::Inherited
    );
    assert_eq!(
        css_property_is_inherited("--brand"),
        CssPropertyInheritanceV0::Inherited
    );
    assert_eq!(
        css_property_is_inherited("opacity"),
        CssPropertyInheritanceV0::NotInherited
    );
    assert_eq!(
        css_property_is_inherited("unknown-property"),
        CssPropertyInheritanceV0::Unknown
    );
    assert_eq!(
        css_property_is_inherited("fill"),
        CssPropertyInheritanceV0::Inherited
    );

    assert_eq!(
        css_property_initial_value("color"),
        CssPropertyInitialValueV0::Literal("canvastext")
    );
    assert_eq!(
        css_property_initial_value("opacity"),
        CssPropertyInitialValueV0::Literal("1")
    );
    assert_eq!(
        css_property_initial_value("direction"),
        CssPropertyInitialValueV0::Literal("initial")
    );
    assert_eq!(
        css_property_initial_value("--brand"),
        CssPropertyInitialValueV0::GuaranteedInvalid
    );
    assert_eq!(
        css_property_initial_value("fill"),
        CssPropertyInitialValueV0::Literal("black")
    );
    assert_eq!(
        css_property_initial_value("future-property"),
        CssPropertyInitialValueV0::Unknown
    );
}

#[test]
fn registered_custom_properties_drive_inheritance_initial_values_and_syntax_fallback() {
    let registration =
        |inherits: bool, verdicts: BTreeMap<String, CascadeRegisteredValueVerdictV0>| {
            CascadeRegisteredCustomPropertyV0 {
                name: "--gap".to_string(),
                inherits,
                initial_value: CascadeValue::Literal("8px".to_string()),
                declaration_value_verdicts: verdicts,
            }
        };

    let initial = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "--gap".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("16px".to_string())),
        registered_custom_property: Some(registration(false, BTreeMap::new())),
    });
    assert_eq!(initial.status, ComputedCascadeValueStatusV0::Initial);
    assert_eq!(initial.value, CascadeValue::Literal("8px".to_string()));
    assert!(!initial.inherited);

    let inherited = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "--gap".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("16px".to_string())),
        registered_custom_property: Some(registration(true, BTreeMap::new())),
    });
    assert_eq!(inherited.status, ComputedCascadeValueStatusV0::Inherited);
    assert_eq!(inherited.value, CascadeValue::Literal("16px".to_string()));

    let invalid_declaration = property_declaration(
        "invalid-gap",
        "--gap",
        CascadeValue::Literal("red".to_string()),
        1,
    );
    let invalid = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "--gap".to_string(),
        declarations: vec![invalid_declaration],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("16px".to_string())),
        registered_custom_property: Some(registration(
            false,
            BTreeMap::from([(
                "invalid-gap".to_string(),
                CascadeRegisteredValueVerdictV0::Unmatched,
            )]),
        )),
    });
    assert_eq!(
        invalid.status,
        ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
    );
    assert_eq!(invalid.value, CascadeValue::Literal("8px".to_string()));
    assert!(invalid.used_initial_value);
    assert!(invalid.invalid_at_computed_value_time);

    let valid_declaration = property_declaration(
        "valid-gap",
        "--gap",
        CascadeValue::Literal("12px".to_string()),
        1,
    );
    let valid = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "--gap".to_string(),
        declarations: vec![valid_declaration],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("16px".to_string())),
        registered_custom_property: Some(registration(
            false,
            BTreeMap::from([(
                "valid-gap".to_string(),
                CascadeRegisteredValueVerdictV0::Matched,
            )]),
        )),
    });
    assert_eq!(valid.status, ComputedCascadeValueStatusV0::Resolved);
    assert_eq!(valid.value, CascadeValue::Literal("12px".to_string()));
}

#[test]
fn unregistered_custom_property_keeps_the_inherited_computed_value_contract() {
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "--gap".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("16px".to_string())),
        registered_custom_property: None,
    });

    assert_eq!(result.status, ComputedCascadeValueStatusV0::Inherited);
    assert_eq!(result.value, CascadeValue::Literal("16px".to_string()));
    assert_eq!(
        result.derivation_steps,
        vec![
            "noCascadeWinner",
            "inheritanceOrInitialSelected",
            "inheritKeywordResolved",
            "parentComputedValueUsed",
        ]
    );
}

#[test]
fn property_metadata_lookup_respects_the_supplied_sorted_registry() {
    let prefix = &CSS_PROPERTY_METADATA_RECORDS_V1[..64];
    let first_name = prefix[0].canonical_name;
    let outside_name = CSS_PROPERTY_METADATA_RECORDS_V1[64].canonical_name;

    assert_eq!(
        css_property_metadata_for_property_in_records(first_name, prefix)
            .map(|record| record.canonical_name),
        Some(first_name)
    );
    assert!(css_property_metadata_for_property_in_records(outside_name, prefix).is_none());
    assert_eq!(
        css_property_metadata_for_property_in_records(
            outside_name,
            CSS_PROPERTY_METADATA_RECORDS_V1
        )
        .map(|record| record.canonical_name),
        Some(outside_name)
    );
}

#[test]
fn unknown_property_metadata_is_typed_as_indeterminate() {
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "future-property".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
    });
    assert_eq!(result.status, ComputedCascadeValueStatusV0::Indeterminate);
    assert_eq!(result.value, CascadeValue::Indeterminate);
    assert!(!result.invalid_at_computed_value_time);
    assert_eq!(
        result.indeterminate_reason,
        Some(ComputedCascadeIndeterminateReasonV0::PropertyInheritanceMetadataUnavailable)
    );
    assert!(
        result
            .derivation_steps
            .contains(&"propertyInheritanceMetadataUnavailable")
    );
}

#[test]
fn every_computed_value_indeterminate_reason_has_a_typed_fixture() {
    let cascade_outcome = crate::computed_value::computed_value_from_indeterminate_cascade_outcome(
        "color",
        &CascadeOutcome::RankedSet(Vec::new()),
    );
    assert!(cascade_outcome.is_some());
    let Some(cascade_outcome) = cascade_outcome else {
        return;
    };

    let unknown_inheritance = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "future-property".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
    });

    let unknown_initial_value = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "background".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
    });

    let unknown_declaration = property_declaration(
        "unknown-gap",
        "--gap",
        CascadeValue::Literal("12px".to_string()),
        1,
    );
    let unknown_registered_syntax = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "--gap".to_string(),
        declarations: vec![unknown_declaration],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: Some(CascadeRegisteredCustomPropertyV0 {
            name: "--gap".to_string(),
            inherits: false,
            initial_value: CascadeValue::Literal("8px".to_string()),
            declaration_value_verdicts: BTreeMap::from([(
                "unknown-gap".to_string(),
                CascadeRegisteredValueVerdictV0::Unknown,
            )]),
        }),
    });

    let inherited_from_indeterminate =
        compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: Vec::new(),
            custom_property_env: CustomPropertyEnv::new(),
            parent_computed_value: Some(CascadeValue::Indeterminate),
            registered_custom_property: None,
        });

    let fixtures = [
        cascade_outcome,
        unknown_inheritance,
        unknown_initial_value,
        unknown_registered_syntax,
        inherited_from_indeterminate,
    ];
    for fixture in &fixtures {
        assert_eq!(fixture.status, ComputedCascadeValueStatusV0::Indeterminate);
        assert_eq!(fixture.value, CascadeValue::Indeterminate);
        assert!(!fixture.invalid_at_computed_value_time);
        assert!(fixture.indeterminate_reason.is_some());
    }

    let observed = fixtures
        .iter()
        .filter_map(|fixture| fixture.indeterminate_reason)
        .collect::<BTreeSet<_>>();
    let expected = ComputedCascadeIndeterminateReasonV0::ALL
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, expected);
}

#[test]
fn genuine_substitution_failure_survives_unknown_metadata_fallbacks() {
    for property in ["future-prop", "background"] {
        let mut env = CustomPropertyEnv::new();
        env.insert(
            "--cyclic".to_string(),
            CascadeValue::Var {
                name: "--cyclic".to_string(),
                fallback: None,
            },
        );
        let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: property.to_string(),
            declarations: vec![property_declaration(
                "cyclic-value",
                property,
                CascadeValue::Var {
                    name: "--cyclic".to_string(),
                    fallback: None,
                },
                1,
            )],
            custom_property_env: env,
            parent_computed_value: None,
            registered_custom_property: None,
        });

        assert_eq!(
            result.status,
            ComputedCascadeValueStatusV0::InvalidAtComputedValueTime,
            "{property}"
        );
        assert_eq!(result.value, CascadeValue::GuaranteedInvalid, "{property}");
        assert!(result.invalid_at_computed_value_time, "{property}");
        assert_eq!(result.indeterminate_reason, None, "{property}");
    }
}

#[test]
fn treats_guaranteed_invalid_var_substitution_as_iacvt_unset() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        "--a".to_string(),
        CascadeValue::Var {
            name: "--b".to_string(),
            fallback: None,
        },
    );
    env.insert(
        "--b".to_string(),
        CascadeValue::Var {
            name: "--a".to_string(),
            fallback: None,
        },
    );

    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            "cycle-color",
            "color",
            CascadeValue::Var {
                name: "--a".to_string(),
                fallback: None,
            },
            1,
        )],
        custom_property_env: env,
        parent_computed_value: Some(CascadeValue::Literal("canvas".to_string())),
        registered_custom_property: None,
    });

    assert_eq!(
        result.status,
        ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
    );
    assert_eq!(result.value, CascadeValue::Literal("canvas".to_string()));
    assert!(result.inherited);
    assert!(result.invalid_at_computed_value_time);
    assert!(
        result
            .derivation_steps
            .contains(&"invalidAtComputedValueTimeFallsBackAsUnset")
    );
}

#[test]
fn proves_adjacent_box_longhands_can_combine_to_shorthand() {
    let proof = prove_box_shorthand_combination(
        "margin",
        &[
            BoxLonghandInputV0 {
                property: "margin-top".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 1,
            },
            BoxLonghandInputV0 {
                property: "margin-right".to_string(),
                value: "2px".to_string(),
                important: false,
                source_order: 2,
            },
            BoxLonghandInputV0 {
                property: "margin-bottom".to_string(),
                value: "3px".to_string(),
                important: false,
                source_order: 3,
            },
            BoxLonghandInputV0 {
                property: "margin-left".to_string(),
                value: "4px".to_string(),
                important: false,
                source_order: 4,
            },
        ],
    );

    assert_eq!(proof.product, "omena-cascade.shorthand-combination-proof");
    assert!(proof.accepted);
    assert_eq!(proof.blocked_reason, None);
    assert!(proof.provenance_preserved);
    assert!(proof.cascade_safe_witness.contains("canonical merge order"));

    let border_proof = prove_box_shorthand_combination(
        "border-color",
        &[
            BoxLonghandInputV0 {
                property: "border-top-color".to_string(),
                value: "red".to_string(),
                important: false,
                source_order: 1,
            },
            BoxLonghandInputV0 {
                property: "border-right-color".to_string(),
                value: "blue".to_string(),
                important: false,
                source_order: 2,
            },
            BoxLonghandInputV0 {
                property: "border-bottom-color".to_string(),
                value: "red".to_string(),
                important: false,
                source_order: 3,
            },
            BoxLonghandInputV0 {
                property: "border-left-color".to_string(),
                value: "blue".to_string(),
                important: false,
                source_order: 4,
            },
        ],
    );
    assert!(border_proof.accepted);
    assert!(border_proof.provenance_preserved);

    let scroll_proof = prove_box_shorthand_combination(
        "scroll-margin",
        &[
            BoxLonghandInputV0 {
                property: "scroll-margin-top".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 1,
            },
            BoxLonghandInputV0 {
                property: "scroll-margin-right".to_string(),
                value: "2px".to_string(),
                important: false,
                source_order: 2,
            },
            BoxLonghandInputV0 {
                property: "scroll-margin-bottom".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 3,
            },
            BoxLonghandInputV0 {
                property: "scroll-margin-left".to_string(),
                value: "2px".to_string(),
                important: false,
                source_order: 4,
            },
        ],
    );
    assert!(scroll_proof.accepted);
    assert!(scroll_proof.provenance_preserved);
}

#[test]
fn proves_generic_longhand_merge_with_canonical_order_contract() {
    let proof = prove_longhand_merge(
        "place-content",
        &["align-content", "justify-content"],
        &[
            LonghandMergeInputV0 {
                property: "align-content".to_string(),
                value: "center".to_string(),
                important: false,
                source_order: 10,
            },
            LonghandMergeInputV0 {
                property: "justify-content".to_string(),
                value: "space-between".to_string(),
                important: false,
                source_order: 11,
            },
        ],
    );

    assert!(proof.accepted);
    assert_eq!(
        proof.ordered_longhand_properties,
        vec!["align-content".to_string(), "justify-content".to_string()]
    );

    let rejected = prove_longhand_merge(
        "place-content",
        &["align-content", "justify-content"],
        &[
            LonghandMergeInputV0 {
                property: "justify-content".to_string(),
                value: "space-between".to_string(),
                important: false,
                source_order: 10,
            },
            LonghandMergeInputV0 {
                property: "align-content".to_string(),
                value: "center".to_string(),
                important: false,
                source_order: 11,
            },
        ],
    );

    assert!(!rejected.accepted);
    assert_eq!(
        rejected.blocked_reason,
        Some("longhands are not in canonical merge order")
    );
}

#[test]
fn blocks_box_shorthand_combination_when_intervening_order_is_possible() {
    let proof = prove_box_shorthand_combination(
        "padding",
        &[
            BoxLonghandInputV0 {
                property: "padding-top".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 1,
            },
            BoxLonghandInputV0 {
                property: "padding-right".to_string(),
                value: "2px".to_string(),
                important: false,
                source_order: 3,
            },
            BoxLonghandInputV0 {
                property: "padding-bottom".to_string(),
                value: "3px".to_string(),
                important: false,
                source_order: 4,
            },
            BoxLonghandInputV0 {
                property: "padding-left".to_string(),
                value: "4px".to_string(),
                important: false,
                source_order: 5,
            },
        ],
    );

    assert!(!proof.accepted);
    assert_eq!(
        proof.blocked_reason,
        Some("intervening declaration may change cascade outcome")
    );
    assert!(!proof.provenance_preserved);
}

#[test]
fn evaluates_simple_supports_conditions_under_modern_browser_assumption() {
    let positive = evaluate_static_supports_condition(
        "(display: grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(positive.product, "omena-cascade.supports-static-eval");
    assert_eq!(positive.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
    assert!(positive.provenance_preserved);

    let negative = evaluate_static_supports_condition(
        "not (display: grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(negative.verdict, StaticSupportsEvalVerdictV0::AlwaysFalse);
    assert!(negative.provenance_preserved);

    let conjunction = evaluate_static_supports_condition(
        "(display: grid) and (color: red)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(conjunction.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
    assert!(conjunction.provenance_preserved);

    let disjunction = evaluate_static_supports_condition(
        "(display: grid) or (selector(:has(*)))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(disjunction.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
    assert!(disjunction.provenance_preserved);

    let selector = evaluate_static_supports_condition(
        "selector(:has(*))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(selector.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
    assert!(selector.provenance_preserved);

    let obsolete_selector = evaluate_static_supports_condition(
        "selector(:-ms-input-placeholder)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        obsolete_selector.verdict,
        StaticSupportsEvalVerdictV0::AlwaysFalse
    );
    assert!(obsolete_selector.provenance_preserved);

    let negated_selector = evaluate_static_supports_condition(
        "not selector(:has(*))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        negated_selector.verdict,
        StaticSupportsEvalVerdictV0::AlwaysFalse
    );
    assert!(negated_selector.provenance_preserved);

    let font_tech = evaluate_static_supports_condition(
        "font-tech(color-COLRv1)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(font_tech.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
    assert!(font_tech.provenance_preserved);

    let font_format = evaluate_static_supports_condition(
        "font-format(woff2)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(font_format.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
    assert!(font_format.provenance_preserved);

    let obsolete_font_format = evaluate_static_supports_condition(
        "font-format(embedded-opentype)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        obsolete_font_format.verdict,
        StaticSupportsEvalVerdictV0::AlwaysFalse
    );
    assert!(obsolete_font_format.provenance_preserved);

    let unknown_font_tech = evaluate_static_supports_condition(
        "font-tech(unknown-thing)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        unknown_font_tech.verdict,
        StaticSupportsEvalVerdictV0::Unknown
    );
    assert!(!unknown_font_tech.provenance_preserved);

    let color_function = evaluate_static_supports_condition(
        "(color: color(display-p3 1 0 0))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        color_function.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(color_function.provenance_preserved);

    let gradient_function = evaluate_static_supports_condition(
        "(background-image: linear-gradient(red, blue))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        gradient_function.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(gradient_function.provenance_preserved);

    let malformed_function = evaluate_static_supports_condition(
        "(color: color(display-p3 1 0 0)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        malformed_function.verdict,
        StaticSupportsEvalVerdictV0::Unknown
    );
    assert!(!malformed_function.provenance_preserved);

    let grouped_disjunction = evaluate_static_supports_condition(
        "((display: grid) or (display: -ms-grid))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        grouped_disjunction.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(grouped_disjunction.provenance_preserved);

    let grouped_conjunction = evaluate_static_supports_condition(
        "((display: grid) or (display: -ms-grid)) and (color: red)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        grouped_conjunction.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(grouped_conjunction.provenance_preserved);

    let obsolete_disjunction = evaluate_static_supports_condition(
        "(display: -ms-grid) or (-ms-ime-align: auto)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        obsolete_disjunction.verdict,
        StaticSupportsEvalVerdictV0::AlwaysFalse
    );
    assert!(obsolete_disjunction.provenance_preserved);

    let obsolete = evaluate_static_supports_condition(
        "(display: -ms-grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(obsolete.verdict, StaticSupportsEvalVerdictV0::AlwaysFalse);
    assert!(obsolete.provenance_preserved);

    let negated_obsolete = evaluate_static_supports_condition(
        "not (display: -ms-grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        negated_obsolete.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(negated_obsolete.provenance_preserved);

    let uppercase_negated_obsolete = evaluate_static_supports_condition(
        "NOT (display: -MS-grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        uppercase_negated_obsolete.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(uppercase_negated_obsolete.provenance_preserved);

    let uppercase_logical_selector = evaluate_static_supports_condition(
        "SELECTOR(:-MS-input-placeholder) OR (display: grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        uppercase_logical_selector.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(uppercase_logical_selector.provenance_preserved);

    let uppercase_font_tech = evaluate_static_supports_condition(
        "FONT-TECH(COLOR-COLRv1)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        uppercase_font_tech.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(uppercase_font_tech.provenance_preserved);

    let negated_grouped_obsolete = evaluate_static_supports_condition(
        "not ((display: -ms-grid) or (-ms-ime-align: auto))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        negated_grouped_obsolete.verdict,
        StaticSupportsEvalVerdictV0::AlwaysTrue
    );
    assert!(negated_grouped_obsolete.provenance_preserved);

    let negated_grouped_supported = evaluate_static_supports_condition(
        "not ((display: grid) or (display: -ms-grid))",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    assert_eq!(
        negated_grouped_supported.verdict,
        StaticSupportsEvalVerdictV0::AlwaysFalse
    );
    assert!(negated_grouped_supported.provenance_preserved);
}

#[test]
fn supports_target_capability_downgrades_unsupported_feature_to_unknown() {
    let capability = SupportsTargetCapabilityV0 {
        supports_light_dark: false,
        ..SupportsTargetCapabilityV0::all_supported()
    };
    let witness = evaluate_static_supports_condition(
        "(color: light-dark(#000, #fff))",
        StaticSupportsAssumptionV0::TargetCapability(capability),
    );

    assert_eq!(witness.verdict, StaticSupportsEvalVerdictV0::Unknown);
    assert!(!witness.provenance_preserved);
}

#[test]
fn supports_target_capability_accepts_supported_feature() {
    let witness = evaluate_static_supports_condition(
        "(color: light-dark(#000, #fff))",
        StaticSupportsAssumptionV0::TargetCapability(SupportsTargetCapabilityV0::all_supported()),
    );

    assert_eq!(witness.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
    assert!(witness.provenance_preserved);
}

#[test]
fn supports_target_capability_preserves_unmapped_condition() {
    let target = evaluate_static_supports_condition(
        "(display: grid)",
        StaticSupportsAssumptionV0::TargetCapability(SupportsTargetCapabilityV0::all_supported()),
    );
    let default = evaluate_static_supports_condition(
        "(display: grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );

    assert_eq!(target.verdict, StaticSupportsEvalVerdictV0::Unknown);
    assert_eq!(default.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
}

#[test]
fn supports_target_capability_folds_strict_subset_of_modern() {
    let conditions = [
        "(color: light-dark(#000, #fff))",
        "(color: color-mix(in srgb, red, blue))",
        "(color: oklch(60% 0.2 120))",
        "(display: grid)",
        "selector(:has(*))",
        "font-format(woff2)",
    ];
    let mut modern_only_count = 0usize;
    for condition in conditions {
        let target = evaluate_static_supports_condition(
            condition,
            StaticSupportsAssumptionV0::TargetCapability(
                SupportsTargetCapabilityV0::all_supported(),
            ),
        );
        let default = evaluate_static_supports_condition(
            condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        if target.verdict == StaticSupportsEvalVerdictV0::AlwaysTrue {
            assert_eq!(default.verdict, StaticSupportsEvalVerdictV0::AlwaysTrue);
        }
        if target.verdict != StaticSupportsEvalVerdictV0::AlwaysTrue
            && default.verdict == StaticSupportsEvalVerdictV0::AlwaysTrue
        {
            modern_only_count += 1;
        }
    }

    assert!(modern_only_count > 0);
}

#[test]
fn supports_target_capability_negation_of_lacking_feature_preserves() {
    let capability = SupportsTargetCapabilityV0 {
        supports_light_dark: false,
        ..SupportsTargetCapabilityV0::all_supported()
    };
    let witness = evaluate_static_supports_condition(
        "not (color: light-dark(#000, #fff))",
        StaticSupportsAssumptionV0::TargetCapability(capability),
    );

    assert_eq!(witness.verdict, StaticSupportsEvalVerdictV0::Unknown);
    assert!(!witness.provenance_preserved);
}

#[test]
fn proves_only_root_scope_flatten_candidates_without_competition() {
    let accepted = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
        root_selector: ":RoOt".to_string(),
        limit_selector: None,
        scoped_rule_count: 1,
        peer_scope_count: 0,
        competing_unscoped_rule_count: 0,
        inside_layer: false,
    });
    assert_eq!(accepted.product, "omena-cascade.scope-flatten-proof");
    assert!(accepted.accepted);
    assert!(accepted.provenance_preserved);

    let blocked = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
        root_selector: ".card".to_string(),
        limit_selector: None,
        scoped_rule_count: 1,
        peer_scope_count: 0,
        competing_unscoped_rule_count: 0,
        inside_layer: false,
    });
    assert!(!blocked.accepted);
    assert_eq!(
        blocked.blocked_reason,
        Some("non-root scope flattening requires selector/proximity equivalence proof")
    );
}

#[test]
fn proves_layer_flatten_only_for_closed_single_layer_candidates() {
    let accepted = prove_layer_flatten_candidate(LayerFlattenInputV0 {
        layer_name: Some("theme".to_string()),
        layer_rule_count: 1,
        peer_layer_count: 0,
        unlayered_rule_count: 0,
        important_declaration_count: 0,
        closed_bundle: true,
    });
    assert_eq!(accepted.product, "omena-cascade.layer-flatten-proof");
    assert!(accepted.accepted);
    assert!(accepted.provenance_preserved);

    let blocked = prove_layer_flatten_candidate(LayerFlattenInputV0 {
        layer_name: Some("theme".to_string()),
        layer_rule_count: 1,
        peer_layer_count: 0,
        unlayered_rule_count: 1,
        important_declaration_count: 0,
        closed_bundle: true,
    });
    assert!(!blocked.accepted);
    assert_eq!(
        blocked.blocked_reason,
        Some("unlayered rules compete differently from layered normal rules")
    );
}

#[test]
fn modal_check_witness_consolidates_existing_proof_witnesses_as_strict_superset() {
    let shorthand = prove_box_shorthand_combination(
        "margin",
        &[
            BoxLonghandInputV0 {
                property: "margin-top".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 1,
            },
            BoxLonghandInputV0 {
                property: "margin-right".to_string(),
                value: "2px".to_string(),
                important: false,
                source_order: 2,
            },
            BoxLonghandInputV0 {
                property: "margin-bottom".to_string(),
                value: "3px".to_string(),
                important: false,
                source_order: 3,
            },
            BoxLonghandInputV0 {
                property: "margin-left".to_string(),
                value: "4px".to_string(),
                important: false,
                source_order: 4,
            },
        ],
    );
    let supports = evaluate_static_supports_condition(
        "(display: grid)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    let scope = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
        root_selector: ":root".to_string(),
        limit_selector: None,
        scoped_rule_count: 1,
        peer_scope_count: 0,
        competing_unscoped_rule_count: 0,
        inside_layer: false,
    });
    let blocked_layer = prove_layer_flatten_candidate(LayerFlattenInputV0 {
        layer_name: Some("theme".to_string()),
        layer_rule_count: 1,
        peer_layer_count: 0,
        unlayered_rule_count: 1,
        important_declaration_count: 0,
        closed_bundle: true,
    });

    let summary = summarize_modal_check_witness_v0(vec![
        ModalCheckWitnessSourceV0::ShorthandCombination(shorthand.clone()),
        ModalCheckWitnessSourceV0::StaticSupportsEval(supports.clone()),
        ModalCheckWitnessSourceV0::ScopeFlatten(scope.clone()),
        ModalCheckWitnessSourceV0::LayerFlatten(blocked_layer.clone()),
    ]);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-cascade.modal-check-witness");
    assert_eq!(summary.modal_family, "cascadeProofObligationStrictSuperset");
    assert_eq!(summary.substrate, "omena-cascade.proof-witnesses");
    assert_eq!(summary.obligation_count, 4);
    assert_eq!(summary.accepted_count, 3);
    assert_eq!(summary.blocked_count, 1);
    assert!(!summary.all_provenance_preserved);
    assert_eq!(
        summary.source_products,
        vec![
            shorthand.product,
            supports.product,
            scope.product,
            blocked_layer.product
        ]
    );
    assert!(matches!(
        summary.witnesses[3],
        ModalCheckWitnessSourceV0::LayerFlatten(_)
    ));
}

#[test]
fn modal_check_witness_keeps_unknown_supports_as_blocked_fixture_evidence() {
    let unknown_supports = evaluate_static_supports_condition(
        "future-feature(foo)",
        StaticSupportsAssumptionV0::ModernBrowser,
    );
    let summary =
        summarize_modal_check_witness_v0(vec![ModalCheckWitnessSourceV0::StaticSupportsEval(
            unknown_supports.clone(),
        )]);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-cascade.modal-check-witness");
    assert_eq!(summary.obligation_count, 1);
    assert_eq!(summary.accepted_count, 0);
    assert_eq!(summary.blocked_count, 1);
    assert!(!summary.all_provenance_preserved);
    assert_eq!(summary.source_products, vec![unknown_supports.product]);
    assert!(matches!(
        summary.witnesses[0],
        ModalCheckWitnessSourceV0::StaticSupportsEval(_)
    ));
}

#[test]
fn reports_selector_context_witness_rank() {
    let root = selector_context_witness(&[":root".to_string()], &[".button".to_string()]);
    assert_eq!(root.kind, SelectorContextMatchKind::Root);
    assert_eq!(root.verdict, SelectorMatchVerdict::Yes);
    assert!(root.matched);
    assert_eq!(root.rank, 1);

    let exact = selector_context_witness(&[".button".to_string()], &[".button".to_string()]);
    assert_eq!(exact.kind, SelectorContextMatchKind::Exact);
    assert_eq!(exact.verdict, SelectorMatchVerdict::Yes);
    assert_eq!(exact.rank, 3);

    let descendant =
        selector_context_witness(&[".theme".to_string()], &[".theme .button".to_string()]);
    assert_eq!(descendant.kind, SelectorContextMatchKind::ContainsSelector);
    assert_eq!(descendant.verdict, SelectorMatchVerdict::Yes);
    assert_eq!(descendant.rank, 2);
    assert_eq!(
        descendant.reference_selector.as_deref(),
        Some(".theme .button")
    );

    let prefix_false_positive =
        selector_context_witness(&[".foo".to_string()], &[".foobar".to_string()]);
    assert_eq!(
        prefix_false_positive.kind,
        SelectorContextMatchKind::NoMatch
    );
    assert_eq!(prefix_false_positive.verdict, SelectorMatchVerdict::No);
    assert!(!prefix_false_positive.matched);

    let bem_suffix_false_positive =
        selector_context_witness(&[".btn".to_string()], &[".btn-primary".to_string()]);
    assert_eq!(
        bem_suffix_false_positive.kind,
        SelectorContextMatchKind::NoMatch
    );
    assert_eq!(bem_suffix_false_positive.verdict, SelectorMatchVerdict::No);
    assert!(!bem_suffix_false_positive.matched);

    let approximate =
        selector_context_witness(&[".card:unknown(.x)".to_string()], &[".button".to_string()]);
    assert_eq!(
        approximate.kind,
        SelectorContextMatchKind::ApproximateSelector
    );
    assert_eq!(approximate.verdict, SelectorMatchVerdict::Maybe);
    assert!(approximate.matched);

    let miss = selector_context_witness(&[".card".to_string()], &[".button".to_string()]);
    assert_eq!(miss.kind, SelectorContextMatchKind::NoMatch);
    assert_eq!(miss.verdict, SelectorMatchVerdict::No);
    assert!(!miss.matched);
}

#[test]
fn parses_simple_selector_specificity() {
    let signature = parse_simple_selector_signature("button#save.primary[data-state]:hover");
    assert!(signature.is_some());
    if let Some(signature) = signature {
        assert_eq!(signature.required_tag.as_deref(), Some("button"));
        assert_eq!(signature.required_id.as_deref(), Some("save"));
        assert!(signature.required_classes.contains("primary"));
        assert!(signature.required_attributes.contains("data-state"));
        assert!(signature.required_pseudo_states.contains("hover"));
        assert_eq!(signature.specificity, Specificity::new(1, 3, 1));
    }
}

#[test]
fn where_pseudo_contributes_zero_specificity() {
    // RFC-0007-B B3: `:where(.box)` must parse (not drop the rule) and contribute
    // zero specificity, so a bare `.box` still beats it.
    let Some(plain) = parse_simple_selector_signature(".box") else {
        unreachable!("plain class parses")
    };
    let Some(where_box) = parse_simple_selector_signature(":where(.box)") else {
        unreachable!(":where(.box) parses")
    };

    assert_eq!(where_box.specificity, Specificity::ZERO);
    assert!(plain.specificity > where_box.specificity);
    assert!(where_box.required_pseudo_states.contains("where"));
}

#[test]
fn is_pseudo_takes_most_specific_argument_specificity() {
    // RFC-0007-B B3: `:is(.a, #b)` takes `#b`'s specificity (the most specific
    // argument), not the first or the sum.
    let Some(signature) = parse_simple_selector_signature(":is(.a, #b)") else {
        unreachable!(":is(...) parses")
    };
    assert_eq!(signature.specificity, Specificity::new(1, 0, 0));
    assert!(signature.required_pseudo_states.contains("is"));
}

#[test]
fn is_pseudo_counts_complex_argument_specificity() {
    let Some(signature) = parse_simple_selector_signature(":is(#root .item)") else {
        unreachable!(":is(...) parses")
    };

    // Selectors L4: the argument is a complex selector, so both compounds
    // contribute to the functional pseudo-class specificity.
    assert_eq!(signature.specificity, Specificity::new(1, 1, 0));
    assert_eq!(
        signature.specificity_exactness,
        SpecificityExactnessV0::Exact
    );
}

#[test]
fn functional_pseudo_specificity_distinguishes_exact_and_lower_bound_estimates() {
    let cases = [
        (":not(.a.b)", Specificity::new(0, 2, 0)),
        (":has(> .x)", Specificity::new(0, 1, 0)),
        (":is(ul > li.active)", Specificity::new(0, 1, 2)),
    ];
    for (selector, expected) in cases {
        let Some(signature) = parse_simple_selector_signature(selector) else {
            unreachable!("standard functional pseudo parses")
        };
        assert_eq!(signature.specificity, expected, "{selector}");
        assert_eq!(
            signature.specificity_exactness,
            SpecificityExactnessV0::Exact,
            "{selector}"
        );
    }

    let Some(where_signature) = parse_simple_selector_signature(":where(#a .b)") else {
        unreachable!(":where(...) parses")
    };
    assert_eq!(where_signature.specificity, Specificity::ZERO);
    assert_eq!(
        where_signature.specificity_exactness,
        SpecificityExactnessV0::Exact
    );

    let Some(inexact) = parse_simple_selector_signature(":is(:unknown(.a), .b)") else {
        unreachable!("forgiving selector list keeps the modeled branch")
    };
    assert_eq!(inexact.specificity, Specificity::new(0, 1, 0));
    assert_eq!(
        inexact.specificity_exactness,
        SpecificityExactnessV0::Inexact
    );
}

#[test]
fn inexact_specificity_cannot_produce_a_definite_winner() {
    let Some(inexact_signature) = parse_simple_selector_signature(":is(:unknown(.a), .b)") else {
        unreachable!("forgiving selector list keeps the modeled branch")
    };
    assert_eq!(
        inexact_signature.specificity_exactness,
        SpecificityExactnessV0::Inexact
    );

    let outcome = cascade_property(
        [
            declaration_with_specificity_exactness(
                "inexact",
                "red",
                key(
                    CascadeLevel::AuthorNormal,
                    0,
                    0,
                    inexact_signature.specificity,
                    0,
                ),
                SpecificityExactnessV0::Inexact,
            ),
            declaration(
                "simple",
                "blue",
                key(
                    CascadeLevel::AuthorNormal,
                    0,
                    0,
                    Specificity::new(0, 1, 0),
                    1,
                ),
            ),
        ],
        "color",
    );

    assert!(matches!(outcome, CascadeOutcome::RankedSet(_)));
}

#[test]
fn inexact_specificity_reaches_computed_value_as_indeterminate() {
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![declaration_with_specificity_exactness(
            "inexact",
            "red",
            key(
                CascadeLevel::AuthorNormal,
                0,
                0,
                Specificity::new(0, 1, 0),
                0,
            ),
            SpecificityExactnessV0::Inexact,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
    });

    assert_eq!(result.status, ComputedCascadeValueStatusV0::Indeterminate);
    assert_eq!(result.value, CascadeValue::Indeterminate);
    assert_eq!(
        result.indeterminate_reason,
        Some(ComputedCascadeIndeterminateReasonV0::CascadeOutcomeIndeterminate)
    );
    assert_eq!(result.winner_declaration_id, None);
    assert!(
        result
            .derivation_steps
            .contains(&"cascadeOutcomeIndeterminate")
    );

    let status = match result.status {
        ComputedCascadeValueStatusV0::Indeterminate => "indeterminate",
        _ => "unexpected",
    };
    let value = match result.value {
        CascadeValue::Indeterminate => "indeterminate",
        _ => "unexpected",
    };
    let winner = result.winner_declaration_id.as_deref().unwrap_or("none");
    let observation = format!("status={status};value={value};winner={winner}");
    let census = serde_json::from_str::<serde_json::Value>(include_str!(
        "../data/specificity-exactness-divergences.json"
    ))
    .unwrap_or(serde_json::Value::Null);
    let row = census["rows"].as_array().and_then(|rows| {
        rows.iter()
            .find(|row| row["fixture"] == "inexact-specificity-ranked-set")
    });
    assert_eq!(
        row.and_then(|row| row["after"].as_str()),
        Some(observation.as_str())
    );
    assert_eq!(
        row.and_then(|row| row["downstreamDisposition"].as_str()),
        Some("typedIndeterminateContract")
    );
}

#[test]
fn open_world_inexact_specificity_cannot_be_promoted() {
    let inexact = declaration_with_specificity_exactness(
        "inexact",
        "red",
        key(
            CascadeLevel::AuthorImportant,
            0,
            0,
            Specificity::new(1, 0, 0),
            0,
        ),
        SpecificityExactnessV0::Inexact,
    );

    assert!(matches!(
        cascade_property_open_world([inexact.clone()], "color"),
        CascadeOutcome::RankedSet(_)
    ));
    assert!(matches!(
        cascade_property_open_world(
            [
                inexact,
                declaration(
                    "exact-weaker",
                    "blue",
                    key(CascadeLevel::AuthorNormal, 0, 0, Specificity::ZERO, 1,),
                ),
            ],
            "color",
        ),
        CascadeOutcome::RankedSet(_)
    ));
}

#[test]
fn cascade_ordering_sources_have_no_silent_zero_specificity_fallback() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| unreachable!("workspace crates directory"));
    let mut offenders = Vec::new();
    let scan_result = ["omena-cascade", "omena-query", "omena-transform-passes"]
        .into_iter()
        .try_for_each(|crate_name| {
            collect_rust_sources(
                crates_dir.join(crate_name).join("src").as_path(),
                &mut offenders,
            )
        });
    assert!(
        scan_result.is_ok(),
        "specificity source scan failed: {scan_result:?}"
    );
    assert!(
        offenders.is_empty(),
        "silent specificity fallbacks bypass exactness: {offenders:?}"
    );
}

fn collect_rust_sources(directory: &Path, offenders: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("failed to read directory entry: {error}"))?
            .path();
        if path.is_dir() {
            collect_rust_sources(path.as_path(), offenders)?;
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        let compact = source.split_whitespace().collect::<String>();
        let direct_fallback = [".unwrap", "_or(Specificity::ZERO)"].concat();
        let lazy_fallback = [".unwrap", "_or_else(||Specificity::ZERO)"].concat();
        if compact.contains(direct_fallback.as_str()) || compact.contains(lazy_fallback.as_str()) {
            offenders.push(path);
        }
    }
    Ok(())
}

#[test]
fn specificity_exactness_divergence_census_is_fully_adjudicated() {
    let census_result = serde_json::from_str::<serde_json::Value>(include_str!(
        "../data/specificity-exactness-divergences.json"
    ));
    assert!(
        census_result.is_ok(),
        "invalid specificity divergence census"
    );
    let census = census_result.unwrap_or(serde_json::Value::Null);
    let rows = census["rows"].as_array();
    assert_eq!(rows.map(Vec::len), Some(5));
    assert!(rows.is_some_and(|rows| rows.iter().all(|row| {
        matches!(
            row["adjudication"].as_str(),
            Some("fix" | "intendedCorrection")
        ) && row["surface"].as_str().is_some()
            && row["fixture"].as_str().is_some()
            && row["before"].as_str().is_some()
            && row["after"].as_str().is_some()
            && row["downstreamDisposition"].as_str().is_some()
    })));
    assert_eq!(
        rows.map(|rows| {
            rows.iter()
                .filter(|row| {
                    row["downstreamDisposition"].as_str() == Some("typedIndeterminateContract")
                })
                .count()
        }),
        Some(1)
    );
}

#[test]
fn not_pseudo_takes_most_specific_argument_specificity() {
    // RFC-0007-B B3: `:not()` mirrors `:is()` for specificity.
    let Some(signature) = parse_simple_selector_signature(":not(.a, #b)") else {
        unreachable!(":not(...) parses")
    };
    assert_eq!(signature.specificity, Specificity::new(1, 0, 0));
    assert!(signature.required_pseudo_states.contains("not"));
}

#[test]
fn has_pseudo_takes_most_specific_argument_specificity() {
    // Selectors L4: `:has(.a, #b)` takes the specificity of its most specific
    // argument (same rule as `:is`/`:not`), so the rule is no longer dropped.
    let Some(signature) = parse_simple_selector_signature(":has(.a, #b)") else {
        unreachable!(":has(...) parses")
    };
    assert_eq!(signature.specificity, Specificity::new(1, 0, 0));
    assert!(signature.required_pseudo_states.contains("has"));
}

#[test]
fn functional_pseudo_folds_into_compound_specificity() {
    // The functional-pseudo specificity adds to the rest of the compound, and a
    // bare pseudo-state (`:hover`) is unchanged by this lane (over-correction guard).
    let Some(compound) = parse_simple_selector_signature(".card:not(#x):hover") else {
        unreachable!("compound parses")
    };
    // `.card` (0,1,0) + `:not(#x)` (1,0,0) + `:hover` (0,1,0) = (1,2,0).
    assert_eq!(compound.specificity, Specificity::new(1, 2, 0));

    let Some(plain_hover) = parse_simple_selector_signature(".card:hover") else {
        unreachable!("plain pseudo parses")
    };
    assert_eq!(plain_hover.specificity, Specificity::new(0, 2, 0));
}

#[test]
fn unknown_functional_pseudo_is_still_unsupported() {
    // Over-correction guard: only the standardized functional pseudos gain
    // specificity modeling; unknown ones stay conservative (rule still dropped).
    assert!(parse_simple_selector_signature(":nth-child(2n)").is_none());
}

#[test]
fn selector_co_match_rejects_only_conflicting_single_valued_axes() {
    assert_eq!(
        selector_co_match_verdict("button.btn", "a.btn"),
        SelectorMatchVerdict::No
    );
    assert_eq!(
        selector_co_match_verdict("#save.primary", "#cancel.primary"),
        SelectorMatchVerdict::No
    );
}

#[test]
fn selector_co_match_keeps_additive_axes_compatible() {
    assert_eq!(
        selector_co_match_verdict(".btn", "button.btn"),
        SelectorMatchVerdict::Yes
    );
    assert_eq!(
        selector_co_match_verdict(".btn", ".btn.active[data-state]:hover"),
        SelectorMatchVerdict::Yes
    );
}

#[test]
fn selector_co_match_returns_maybe_for_unsupported_selector_syntax() {
    assert_eq!(
        selector_co_match_verdict(".btn:is(.active)", ".btn .icon"),
        SelectorMatchVerdict::Maybe
    );
    assert_eq!(
        selector_co_match_verdict("[type=text]", "[type=number]"),
        SelectorMatchVerdict::Maybe
    );
    assert_eq!(
        selector_co_match_verdict(".btn:is(.active)", ".btn"),
        SelectorMatchVerdict::Maybe
    );
    assert_eq!(
        selector_co_match_verdict(".btn::before", ".btn"),
        SelectorMatchVerdict::Maybe
    );
}

#[test]
fn matches_simple_compound_selectors_against_concrete_signature() {
    let mut element =
        ElementSignature::concrete(Some("button"), Some("save"), ["primary", "active"]);
    element.attributes.insert("data-state".to_string());
    element.pseudo_states.insert("hover".to_string());

    let witness = selector_match_witness("button#save.primary[data-state]:hover", &element);

    assert_eq!(witness.verdict, SelectorMatchVerdict::Yes);
    assert_eq!(witness.reason, SelectorMatchReason::SimpleCompound);
    assert_eq!(witness.specificity, Specificity::new(1, 3, 1));
}

#[test]
fn reports_missing_class_and_id_as_no_for_exact_signature() {
    let element = ElementSignature::concrete(Some("button"), Some("save"), ["primary"]);

    let class_miss = selector_match_witness(".missing", &element);
    assert_eq!(class_miss.verdict, SelectorMatchVerdict::No);
    assert_eq!(class_miss.reason, SelectorMatchReason::MissingClass);
    assert!(class_miss.missing_classes.contains("missing"));

    let id_miss = selector_match_witness("#cancel", &element);
    assert_eq!(id_miss.verdict, SelectorMatchVerdict::No);
    assert_eq!(id_miss.reason, SelectorMatchReason::MissingId);
    assert_eq!(id_miss.missing_id.as_deref(), Some("cancel"));
}

#[test]
fn returns_maybe_for_inexact_abstract_class_sets() {
    let element = ElementSignature::at_least_classes(["button"]);

    let witness = selector_match_witness(".button.primary", &element);

    assert_eq!(witness.verdict, SelectorMatchVerdict::Maybe);
    assert_eq!(witness.reason, SelectorMatchReason::MissingClass);
    assert!(witness.missing_classes.contains("primary"));
}

#[test]
fn selector_lists_choose_strongest_matching_branch() {
    let element = ElementSignature::concrete(Some("button"), Some("save"), ["primary"]);

    let witness = selector_match_witness(".missing, button#save.primary", &element);

    assert_eq!(witness.verdict, SelectorMatchVerdict::Yes);
    assert_eq!(witness.reason, SelectorMatchReason::SelectorList);
    assert_eq!(
        witness.matched_branch.as_deref(),
        Some("button#save.primary")
    );
    assert_eq!(witness.specificity, Specificity::new(1, 1, 1));
}

#[test]
fn unsupported_combinators_are_reported_as_maybe() {
    let element = ElementSignature::concrete(Some("span"), None::<String>, ["icon"]);

    let witness = selector_match_witness(".button > .icon", &element);

    assert_eq!(witness.verdict, SelectorMatchVerdict::Maybe);
    assert_eq!(witness.reason, SelectorMatchReason::UnsupportedSelector);
    assert_eq!(witness.unsupported_branches, vec![".button > .icon"]);
}

#[test]
fn substitutes_custom_property_fallbacks_and_references() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        "--brand".to_string(),
        CascadeValue::Literal("red".to_string()),
    );

    let resolved = substitute_custom_properties(
        &CascadeValue::Var {
            name: "--brand".to_string(),
            fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
        },
        &env,
    );
    assert_eq!(resolved, CascadeValue::Literal("red".to_string()));

    let fallback = substitute_custom_properties(
        &CascadeValue::Var {
            name: "--missing".to_string(),
            fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
        },
        &env,
    );
    assert_eq!(fallback, CascadeValue::Literal("blue".to_string()));
}

#[test]
fn substitutes_custom_properties_inside_composite_values() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        "--gap".to_string(),
        CascadeValue::Literal("2px".to_string()),
    );
    env.insert(
        "--shadow".to_string(),
        CascadeValue::Composite(vec![
            CascadeValue::Literal("0 0 ".to_string()),
            CascadeValue::Var {
                name: "--gap".to_string(),
                fallback: None,
            },
        ]),
    );
    env.insert(
        "--invalid-shadow".to_string(),
        CascadeValue::Composite(vec![
            CascadeValue::Literal("0 0 ".to_string()),
            CascadeValue::Var {
                name: "--missing".to_string(),
                fallback: None,
            },
        ]),
    );

    let resolved = substitute_custom_properties(
        &CascadeValue::Var {
            name: "--shadow".to_string(),
            fallback: None,
        },
        &env,
    );
    assert_eq!(
        resolved,
        CascadeValue::Composite(vec![
            CascadeValue::Literal("0 0 ".to_string()),
            CascadeValue::Literal("2px".to_string()),
        ])
    );

    let fallback = substitute_custom_properties(
        &CascadeValue::Var {
            name: "--invalid-shadow".to_string(),
            fallback: Some(Box::new(CascadeValue::Literal("none".to_string()))),
        },
        &env,
    );
    assert_eq!(fallback, CascadeValue::Literal("none".to_string()));
}

#[test]
fn substitutes_cycles_to_guaranteed_invalid() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        "--a".to_string(),
        CascadeValue::Var {
            name: "--b".to_string(),
            fallback: None,
        },
    );
    env.insert(
        "--b".to_string(),
        CascadeValue::Var {
            name: "--a".to_string(),
            fallback: None,
        },
    );

    let resolved = substitute_custom_properties(
        &CascadeValue::Var {
            name: "--a".to_string(),
            fallback: None,
        },
        &env,
    );

    assert_eq!(resolved, CascadeValue::GuaranteedInvalid);

    let fallback = substitute_custom_properties(
        &CascadeValue::Var {
            name: "--a".to_string(),
            fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
        },
        &env,
    );

    assert_eq!(fallback, CascadeValue::Literal("blue".to_string()));
}

#[test]
fn summarizes_custom_property_least_fixed_point() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        "--brand".to_string(),
        CascadeValue::Literal("red".to_string()),
    );
    env.insert(
        "--alias".to_string(),
        CascadeValue::Var {
            name: "--brand".to_string(),
            fallback: None,
        },
    );
    env.insert(
        "--shadow".to_string(),
        CascadeValue::Composite(vec![
            CascadeValue::Literal("0 0 ".to_string()),
            CascadeValue::Var {
                name: "--alias".to_string(),
                fallback: None,
            },
        ]),
    );
    env.insert(
        "--cycle-a".to_string(),
        CascadeValue::Var {
            name: "--cycle-b".to_string(),
            fallback: None,
        },
    );
    env.insert(
        "--cycle-b".to_string(),
        CascadeValue::Var {
            name: "--cycle-a".to_string(),
            fallback: None,
        },
    );

    let summary = summarize_custom_property_least_fixed_point(&env);

    assert_eq!(
        summary.product,
        "omena-cascade.custom-property-least-fixed-point"
    );
    assert_eq!(summary.input_count, 5);
    assert_eq!(summary.resolved_count, 3);
    assert_eq!(summary.guaranteed_invalid_count, 2);
    assert!(summary.iteration_count >= 2);
    assert_eq!(summary.iteration_bound, 6);
    assert!(summary.reached_fixed_point);
    assert!(summary.monotone_witness_valid);
    assert_eq!(summary.iteration_trace.len(), summary.iteration_count);
    assert!(
        summary
            .iteration_trace
            .windows(2)
            .all(|pair| pair[0].settled_count <= pair[1].settled_count)
    );
    assert_eq!(
        summary.proof.iteration_bound_formula,
        "max(1, env.len() + 1)"
    );
    assert!(
        summary
            .proof
            .proof_obligations
            .contains(&"explicit fixed-point equality check")
    );
    assert!(
        summary
            .proof
            .proof_obligations
            .contains(&"nondecreasing settled-value trace")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyLeastFixedPoint")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyLeastFixedPointProof")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyLeastFixedPointTrace")
    );
    assert!(summary.entries.iter().any(|entry| {
        entry.name == "--alias" && entry.resolved == CascadeValue::Literal("red".to_string())
    }));
    assert!(summary.entries.iter().any(|entry| {
        entry.name == "--shadow"
            && entry.resolved
                == CascadeValue::Composite(vec![
                    CascadeValue::Literal("0 0 ".to_string()),
                    CascadeValue::Literal("red".to_string()),
                ])
    }));
    assert!(summary.entries.iter().any(|entry| {
        entry.name == "--cycle-a" && entry.resolved == CascadeValue::GuaranteedInvalid
    }));
}

#[test]
fn fuzz_seed_corpus_preserves_cascade_and_var_invariants() {
    let report = run_cascade_fuzz_seed_corpus();

    assert_eq!(report.product, "omena-cascade.fuzz-seed-corpus");
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.passed_count, report.case_count);
    assert!(
        report
            .var_results
            .iter()
            .any(|result| result.cycle && matches!(result.result, CascadeValue::Literal(_)))
    );
}

#[test]
fn summarizes_current_boundary_status() {
    let summary = summarize_cascade_boundary();

    assert_eq!(summary.product, "omena-cascade.boundary");
    assert_eq!(summary.ordering_model, "lexicographicCascadeKey");
    assert_eq!(
        summary.least_fixed_point_proof_model,
        "finite-env monotone custom-property substitution with cycle-to-guaranteed-invalid bottoming and env-size iteration bound"
    );
    assert!(summary.ready_surfaces.contains(&"cascadeKeyOrdering"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyLeastFixedPoint")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyLeastFixedPointProof")
    );
    assert!(summary.ready_surfaces.contains(&"genericCascadeWinner"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"semanticDesignTokenRanking")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"queryReadCascadeAtPosition")
    );
    assert!(summary.ready_surfaces.contains(&"selectorContextWitness"));
    assert!(summary.ready_surfaces.contains(&"selectorMatchWitness"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"supportsStaticEvalWitness")
    );
    assert!(summary.ready_surfaces.contains(&"scopeFlattenProof"));
    assert!(summary.ready_surfaces.contains(&"layerFlattenProof"));
    assert!(summary.ready_surfaces.contains(&"wptCascadeSeedCorpus"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"cascadeConformanceSeedCorpus")
    );
    assert!(!summary.not_ready_surfaces.contains(&"selectorMatchWitness"));
    assert!(!summary.not_ready_surfaces.contains(&"wptCascadeCorpus"));
    assert!(summary.not_ready_surfaces.contains(&"fullWptCascadeCorpus"));
}

#[test]
fn seed_conformance_corpus_passes_current_cascade_model() {
    let report = run_cascade_conformance_seed_corpus();

    assert_eq!(report.product, "omena-cascade.conformance-seed-corpus");
    assert_eq!(report.case_count, 17);
    assert_eq!(report.passed_count, report.case_count);
    assert_eq!(report.failed_count, 0);
    assert!(report.results.iter().all(|result| result.passed));

    let inversion_pin = report
        .results
        .iter()
        .find(|result| result.name == "complex-functional-specificity-beats-source-order")
        .map(|result| (result.actual_outcome, result.actual_winner_id.as_deref()));
    assert_eq!(inversion_pin, Some(("definite", Some("complex"))));
}

#[test]
fn wpt_cascade_seed_corpus_passes_current_cascade_model() {
    let report = run_wpt_cascade_seed_corpus();

    assert_eq!(report.product, "omena-cascade.wpt-cascade-seed-corpus");
    assert!(report.case_count >= 200);
    assert_eq!(report.passed_count, report.case_count);
    assert_eq!(report.failed_count, 0);
    assert!(report.results.iter().all(|result| result.passed));
}
