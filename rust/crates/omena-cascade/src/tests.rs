use super::*;

fn declaration(id: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: "color".to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key,
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
fn separates_module_rank_from_css_specificity() {
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
        "real CSS specificity must outrank import-graph provenance rank"
    );

    let nearer_module = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::ZERO,
        ModuleRank::new(10, 1, 1),
        1,
    );
    let farther_module = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::ZERO,
        ModuleRank::new(9, 99, 99),
        2,
    );
    assert!(nearer_module > farther_module);
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
    assert_eq!(schema.axis_order[0], "level");
    assert_eq!(schema.axis_order.last(), Some(&"sourceOrder"));
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
    });
    assert_eq!(inherited.status, ComputedCascadeValueStatusV0::Inherited);
    assert_eq!(inherited.value, CascadeValue::Literal("purple".to_string()));
    assert!(inherited.inherited);

    let initial = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "opacity".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("0.5".to_string())),
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
    });
    assert_eq!(unset_initial.status, ComputedCascadeValueStatusV0::Initial);
    assert_eq!(unset_initial.value, CascadeValue::Literal("1".to_string()));
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
fn proves_only_root_scope_flatten_candidates_without_competition() {
    let accepted = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
        root_selector: ":root".to_string(),
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
    assert_eq!(report.case_count, 6);
    assert_eq!(report.passed_count, report.case_count);
    assert_eq!(report.failed_count, 0);
    assert!(report.results.iter().all(|result| result.passed));
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
