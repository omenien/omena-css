use super::*;
use omena_syntax::ident::{AuthoredPropertyTextV0, CanonicalCustomPropertyNameV0, PropertyNameV0};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn custom_property_key(name: &str) -> CanonicalCustomPropertyNameV0 {
    PropertyNameV0::canonical_custom_key(name)
}

struct FixtureStandardValueValidator;

impl CascadeStandardValueValidatorV0 for FixtureStandardValueValidator {
    fn validate_standard_property_value(
        &self,
        property: &PropertyNameV0,
        value: &str,
    ) -> CascadeStandardValueVerdictV0 {
        match (property.canonical_name(), value) {
            ("color", "red") => CascadeStandardValueVerdictV0::Matched,
            ("color", _) => CascadeStandardValueVerdictV0::Unmatched,
            _ => CascadeStandardValueVerdictV0::Unknown,
        }
    }
}

fn declaration(id: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    declaration_with_specificity_exactness(id, value, key, SpecificityExactnessV0::Exact)
}

fn declaration_with_tie_evidence(
    id: &str,
    value: &str,
    key: CascadeKey,
    open_world_tie_evidence: OpenWorldTieEvidence,
) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: AuthoredPropertyTextV0::new("color"),
        property_key: PropertyNameV0::standard("color").canonical_key(),
        value: CascadeValue::Literal(value.to_string()),
        key,
        open_world_tie_evidence,
        specificity_exactness: SpecificityExactnessV0::Exact,
    }
}

fn declaration_with_specificity_exactness(
    id: &str,
    value: &str,
    key: CascadeKey,
    specificity_exactness: SpecificityExactnessV0,
) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: AuthoredPropertyTextV0::new("color"),
        property_key: PropertyNameV0::standard("color").canonical_key(),
        value: CascadeValue::Literal(value.to_string()),
        key,
        open_world_tie_evidence: OpenWorldTieEvidence::NONE,
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
        property: AuthoredPropertyTextV0::new(property),
        property_key: PropertyNameV0::from_authored(property).canonical_key(),
        value,
        key: key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            source_order,
        ),
        open_world_tie_evidence: OpenWorldTieEvidence::NONE,
        specificity_exactness: SpecificityExactnessV0::Exact,
    }
}

#[test]
fn cascade_property_ranking_uses_standard_and_custom_canonical_identity() -> Result<(), String> {
    let lower = property_declaration(
        "lower",
        "--foo",
        CascadeValue::Literal("red".to_string()),
        1,
    );
    let upper = property_declaration(
        "upper",
        "--FOO",
        CascadeValue::Literal("blue".to_string()),
        2,
    );

    let (winner, also_considered) = match cascade_property([lower, upper], r"--f\6f o") {
        CascadeOutcome::Definite {
            winner,
            also_considered,
            ..
        } => (winner, also_considered),
        outcome => {
            return Err(format!(
                "escape-equivalent custom property should have a definite winner: {outcome:?}"
            ));
        }
    };
    assert_eq!(winner.id, "lower");
    assert!(
        also_considered.is_empty(),
        "custom property case must not merge"
    );

    let standard = property_declaration(
        "standard",
        "COLOR",
        CascadeValue::Literal("green".to_string()),
        0,
    );
    let winner = match cascade_property([standard], "color") {
        CascadeOutcome::Definite { winner, .. } => winner,
        outcome => {
            return Err(format!(
                "standard property ASCII case must share one identity: {outcome:?}"
            ));
        }
    };
    assert_eq!(winner.id, "standard");
    Ok(())
}

fn key(
    level: CascadeLevel,
    layer_rank: i32,
    scope_proximity: u32,
    specificity: Specificity,
    source_order: u32,
) -> CascadeKey {
    let Some(layer_ordinal) = LayerOrdinal::new(layer_rank) else {
        unreachable!("test fixtures only use sentinel-safe layer ordinals");
    };
    CascadeKey::new(
        level,
        normalized_layer_rank(false, Some(layer_ordinal)),
        scope_proximity,
        specificity,
        source_order,
    )
}

fn generated_cascade_keys() -> Vec<CascadeKey> {
    let mut keys = Vec::new();
    for level in [CascadeLevel::AuthorNormal, CascadeLevel::AuthorImportant] {
        for layer_ordinal in [0, 1] {
            for scope_proximity in [1, 3] {
                for specificity in [Specificity::ZERO, Specificity::new(0, 1, 0)] {
                    for source_order in [1, 2] {
                        keys.push(CascadeKey::new(
                            level,
                            normalized_layer_rank(false, LayerOrdinal::new(layer_ordinal)),
                            scope_proximity,
                            specificity,
                            source_order,
                        ));
                    }
                }
            }
        }
    }
    keys
}

fn token_texts(word: &OrderedTokenWordV0) -> Vec<&str> {
    word.tokens()
        .iter()
        .map(omena_syntax::ident::CanonicalClassKeyV0::as_str)
        .collect()
}

fn known_dom_word(value: &str) -> OrderedTokenWordV0 {
    let DomClassTokenizationV0::Known { word, .. } = tokenize_dom_class_attribute_v0(Some(value))
    else {
        unreachable!("a supplied class attribute is a known tokenizer input");
    };
    word
}

#[test]
fn dom_class_tokenizer_preserves_first_order_and_uses_only_ascii_whitespace() {
    assert_eq!(token_texts(&known_dom_word("b a b")), vec!["b", "a"]);
    assert_eq!(
        token_texts(&known_dom_word("a\u{00a0}b")),
        vec!["a\u{00a0}b"]
    );
    assert_eq!(
        token_texts(&known_dom_word("a\u{000b}b")),
        vec!["a\u{000b}b"],
        "vertical tab is ASCII but not DOM class whitespace"
    );
    assert!(matches!(
        tokenize_dom_class_attribute_v0(None),
        DomClassTokenizationV0::Unknown {
            cause: DomClassTokenizationUnknownCauseV0::InputUnavailable
        }
    ));
}

#[test]
fn token_support_enforces_subset_and_matches_ordered_support() {
    let left = known_dom_word("button primary");
    let right = known_dom_word("primary large");
    let combined = left.combine_first_occurrence(&right);
    assert_eq!(token_texts(&combined), vec!["button", "primary", "large"]);
    let support = token_support_v0(&combined);
    let left_support = token_support_v0(&left);
    let right_support = token_support_v0(&right);
    let expected_union = left_support
        .may()
        .union(right_support.may())
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        support
            .must()
            .iter()
            .map(omena_syntax::ident::CanonicalClassKeyV0::as_str)
            .collect::<Vec<_>>(),
        vec!["button", "large", "primary"]
    );
    assert_eq!(support.must(), support.may());
    assert_eq!(support.may(), &expected_union);

    let button = omena_syntax::ident::ClassNameV0::new("button").canonical_key();
    let primary = omena_syntax::ident::ClassNameV0::new("primary").canonical_key();
    assert!(TokenSupportV0::new([button.clone()], [button.clone(), primary.clone()]).is_some());
    assert!(TokenSupportV0::new([button, primary.clone()], [primary]).is_none());
}

#[test]
fn attribute_and_selector_escape_planes_share_keys_without_escape_aware_tokenization() {
    let attribute = known_dom_word(r"a\2d b");
    assert_eq!(token_texts(&attribute), vec!["a-", "b"]);

    let escaped_selector = omena_syntax::ident::class_selector_names(r".a\2d b");
    let plain_selector = omena_syntax::ident::class_selector_names(".a-b");
    assert_eq!(escaped_selector.len(), 1);
    assert_eq!(plain_selector.len(), 1);
    assert_eq!(
        escaped_selector[0].name.clone().canonical_key(),
        plain_selector[0].name.clone().canonical_key()
    );
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
        CascadeLevel::InlineImportant,
        CascadeLevel::AuthorImportant,
        CascadeLevel::UserImportant,
        CascadeLevel::UserAgentImportant,
    ]);

    assert_eq!(driven_levels, expected);
    assert_eq!(cascade_level_catalog_v0().len(), 10);
    assert_eq!(driven_levels.len(), 8);
    assert_eq!(
        cascade_level_for_origin(CascadeOriginV0::Inline, true),
        CascadeLevel::InlineImportant
    );
}

#[test]
fn element_attached_style_outranks_author_rules_across_adverse_layers() {
    let strongest_specificity = Specificity::new(u32::MAX, u32::MAX, u32::MAX);
    let normal_inline = CascadeKey::new(
        cascade_level_for_origin(CascadeOriginV0::Inline, false),
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        u32::MAX,
        Specificity::ZERO,
        0,
    );
    let normal_author = CascadeKey::new(
        cascade_level_for_origin(CascadeOriginV0::Author, false),
        normalized_layer_rank(false, None),
        0,
        strongest_specificity,
        u32::MAX,
    );
    let important_inline = CascadeKey::new(
        cascade_level_for_origin(CascadeOriginV0::Inline, true),
        normalized_layer_rank(true, None),
        u32::MAX,
        Specificity::ZERO,
        0,
    );
    let important_author = CascadeKey::new(
        cascade_level_for_origin(CascadeOriginV0::Author, true),
        normalized_layer_rank(true, LayerOrdinal::new(0)),
        0,
        strongest_specificity,
        u32::MAX,
    );

    assert!(normal_inline > normal_author);
    assert!(important_inline > important_author);
    for (inline_key, author_key, expected_id) in [
        (normal_inline, normal_author, "inline-normal"),
        (important_inline, important_author, "inline-important"),
    ] {
        let outcome = cascade_property(
            [
                declaration(expected_id, "inline", inline_key),
                declaration("author-rule", "author", author_key),
            ],
            "color",
        );
        let winner_id = match outcome {
            CascadeOutcome::Definite { winner, .. } => Some(winner.id),
            _ => None,
        };
        assert_eq!(
            winner_id.as_deref(),
            Some(expected_id),
            "cross-level style-attribute comparison must be definite"
        );
    }
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
fn orders_cascade_keys_by_level_layer_specificity_scope_and_source() {
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
fn library_axis_order_prefers_specificity_before_scope_proximity() {
    let outer = declaration(
        "outer-high-specificity",
        "OUTER",
        key(
            CascadeLevel::AuthorNormal,
            0,
            2,
            Specificity::new(1, 0, 1),
            0,
        ),
    );
    let inner = declaration(
        "inner-low-specificity",
        "INNER",
        key(
            CascadeLevel::AuthorNormal,
            0,
            0,
            Specificity::new(0, 1, 0),
            1,
        ),
    );

    let observed = [
        [outer.clone(), inner.clone()],
        [inner.clone(), outer.clone()],
    ]
    .into_iter()
    .map(
        |declarations| match cascade_property(declarations, "color") {
            CascadeOutcome::Definite { winner, .. } => {
                ("Definite", Some(winner.id), Some(winner.value))
            }
            CascadeOutcome::RankedSet(_) => ("RankedSet", None, None),
            CascadeOutcome::Inherit => ("Inherit", None, None),
            CascadeOutcome::Top => ("Top", None, None),
        },
    )
    .collect::<Vec<_>>();
    let expected = vec![
        (
            "Definite",
            Some("outer-high-specificity".to_string()),
            Some(CascadeValue::Literal("OUTER".to_string())),
        ),
        (
            "Definite",
            Some("outer-high-specificity".to_string()),
            Some(CascadeValue::Literal("OUTER".to_string())),
        ),
    ];

    assert_eq!(
        observed, expected,
        "the published cascade order requires specificity to precede scoping proximity"
    );
}

#[test]
fn equal_scope_proximity_prefers_high_specificity_definite_winner() -> Result<(), String> {
    let low = declaration(
        "low-specificity",
        "LOW",
        key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(0, 1, 0),
            2,
        ),
    );
    let high = declaration(
        "high-specificity",
        "HIGH",
        key(
            CascadeLevel::AuthorNormal,
            0,
            1,
            Specificity::new(1, 0, 0),
            1,
        ),
    );

    for declarations in [[low.clone(), high.clone()], [high.clone(), low.clone()]] {
        let CascadeOutcome::Definite { winner, .. } = cascade_property(declarations, "color")
        else {
            return Err("equal-proximity exact declarations must produce a definite winner".into());
        };
        assert_eq!(winner.id, "high-specificity");
        assert_eq!(winner.value, CascadeValue::Literal("HIGH".to_string()));
    }
    Ok(())
}

#[test]
fn open_world_tie_evidence_is_not_a_cascade_key_axis() {
    let css_specificity_winner = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        1,
        Specificity::new(0, 2, 0),
        1,
    );
    let weaker_specificity = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        1,
        Specificity::new(0, 1, 0),
        2,
    );
    assert!(
        css_specificity_winner > weaker_specificity,
        "real CSS specificity must outrank import-graph provenance evidence"
    );

    let earlier_source = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        1,
        Specificity::ZERO,
        1,
    );
    let later_source = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        1,
        Specificity::ZERO,
        2,
    );
    assert!(
        later_source > earlier_source,
        "exact cascade ordering uses source order, not module provenance rank"
    );

    let evidence = OpenWorldTieEvidence::new(ModuleRank::new(1, 2, 3));
    assert_eq!(evidence.module_rank, ModuleRank::new(1, 2, 3));
    assert_eq!(OpenWorldTieEvidence::NONE, OpenWorldTieEvidence::ZERO);
}

#[test]
fn generated_cascade_key_equality_matches_total_order_equality() {
    let mut keys = generated_cascade_keys();
    keys.extend(keys.iter().copied().take(2).collect::<Vec<_>>());

    for (left_index, left) in keys.iter().enumerate() {
        for (right_index, right) in keys.iter().enumerate() {
            assert_eq!(
                left == right,
                left.cmp(right) == std::cmp::Ordering::Equal,
                "Eq and Ord diverged for generated pair ({left_index}, {right_index})"
            );
        }
    }
}

#[test]
fn generated_btree_set_lookup_returns_only_stored_equal_keys() {
    let stored = generated_cascade_keys()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut probes = generated_cascade_keys();
    probes.push(CascadeKey::new(
        CascadeLevel::Transition,
        normalized_layer_rank(false, LayerOrdinal::new(7)),
        9,
        Specificity::new(3, 4, 5),
        99,
    ));

    for (probe_index, probe) in probes.into_iter().enumerate() {
        let equal_key_is_stored = stored.iter().any(|stored_key| *stored_key == probe);
        assert_eq!(
            stored.contains(&probe),
            equal_key_is_stored,
            "BTreeSet::contains disagreed with Eq for probe {probe_index}"
        );
        assert_eq!(
            stored
                .get(&probe)
                .is_some_and(|stored_key| *stored_key == probe),
            equal_key_is_stored,
            "BTreeSet::get returned a non-equal key for probe {probe_index}"
        );
    }
}

#[test]
fn generated_binary_search_hits_if_and_only_if_a_key_is_equal() {
    let mut sorted = generated_cascade_keys();
    sorted.sort();
    sorted.dedup();
    let mut probes = generated_cascade_keys();
    probes.push(CascadeKey::new(
        CascadeLevel::UserAgentNormal,
        normalized_layer_rank(false, LayerOrdinal::new(7)),
        9,
        Specificity::new(3, 4, 5),
        99,
    ));

    for (probe_index, probe) in probes.into_iter().enumerate() {
        let equal_key_is_stored = sorted.contains(&probe);
        let search = sorted.binary_search(&probe);
        assert_eq!(
            search.is_ok(),
            equal_key_is_stored,
            "binary_search disagreed with Eq for probe {probe_index}"
        );
        if let Ok(found_index) = search {
            assert_eq!(sorted[found_index], probe);
        }
    }
}

#[test]
fn open_world_ambiguity_returns_ranked_set_with_module_rank_hint() {
    let tied_key = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        1,
        Specificity::ZERO,
        1,
    );
    let weaker_module_hint = declaration_with_tie_evidence(
        "weaker-module-hint",
        "red",
        tied_key,
        OpenWorldTieEvidence::NONE,
    );
    let stronger_module_hint = declaration_with_tie_evidence(
        "stronger-module-hint",
        "blue",
        tied_key,
        OpenWorldTieEvidence::new(ModuleRank::new(u32::MAX, u32::MAX, u32::MAX)),
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
fn generated_open_world_tie_evidence_is_independent_of_input_order() -> Result<(), String> {
    for (key_index, tied_key) in generated_cascade_keys().into_iter().enumerate() {
        for stronger_rank in [ModuleRank::new(1, 0, 0), ModuleRank::new(2, 3, 5)] {
            let weaker = (tied_key, OpenWorldTieEvidence::NONE);
            let stronger = (tied_key, OpenWorldTieEvidence::new(stronger_rank));

            assert_eq!(
                weaker.0.cmp(&stronger.0),
                std::cmp::Ordering::Equal,
                "open-world module provenance must stay outside the specification-key order for generated key {key_index}"
            );
            assert_ne!(
                weaker.1.module_rank, stronger.1.module_rank,
                "the independence arm requires distinct open-world evidence for generated key {key_index}"
            );

            for items in [
                [("weaker", weaker), ("stronger", stronger)],
                [("stronger", stronger), ("weaker", weaker)],
            ] {
                let (winner, _) = select_open_world_cascade_winner(items, |(_, ranked)| *ranked)
                    .ok_or_else(|| "the fixture always contains two candidates".to_string())?;
                assert_eq!(
                    winner.0, "stronger",
                    "tie evidence depended on input order for generated key {key_index}"
                );
            }

            for declarations in [
                [
                    declaration_with_tie_evidence(
                        "weaker",
                        "red",
                        tied_key,
                        OpenWorldTieEvidence::NONE,
                    ),
                    declaration_with_tie_evidence(
                        "stronger",
                        "blue",
                        tied_key,
                        OpenWorldTieEvidence::new(stronger_rank),
                    ),
                ],
                [
                    declaration_with_tie_evidence(
                        "stronger",
                        "blue",
                        tied_key,
                        OpenWorldTieEvidence::new(stronger_rank),
                    ),
                    declaration_with_tie_evidence(
                        "weaker",
                        "red",
                        tied_key,
                        OpenWorldTieEvidence::NONE,
                    ),
                ],
            ] {
                let CascadeOutcome::RankedSet(ranked) =
                    cascade_property_open_world(declarations, "color")
                else {
                    return Err(format!(
                        "tie evidence fabricated a definite winner for generated key {key_index}"
                    ));
                };
                assert_eq!(ranked[0].id, "stronger");
            }
        }
    }
    Ok(())
}

#[test]
fn open_world_module_provenance_remains_below_source_order() -> Result<(), String> {
    let earlier_with_stronger_provenance = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        1,
        Specificity::ZERO,
        1,
    );
    let later_with_weaker_provenance = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        normalized_layer_rank(false, LayerOrdinal::new(0)),
        1,
        Specificity::ZERO,
        2,
    );

    let (winner, _) = select_open_world_cascade_winner(
        [
            (
                "earlier",
                (
                    earlier_with_stronger_provenance,
                    OpenWorldTieEvidence::new(ModuleRank::new(u32::MAX, u32::MAX, u32::MAX)),
                ),
            ),
            (
                "later",
                (later_with_weaker_provenance, OpenWorldTieEvidence::NONE),
            ),
        ],
        |(_, ranked)| *ranked,
    )
    .ok_or_else(|| "the fixture always contains two candidates".to_string())?;
    assert_eq!(
        winner.0, "later",
        "reversion: promoting module provenance above CascadeKey::Ord makes the earlier candidate win"
    );
    Ok(())
}

#[test]
fn open_world_selector_matches_the_hand_written_axis_order() -> Result<(), String> {
    let mut candidates = Vec::new();
    for level in [CascadeLevel::AuthorNormal, CascadeLevel::AuthorImportant] {
        for layer_ordinal in [0, 1] {
            for scope_proximity in [1, 3] {
                for specificity in [Specificity::ZERO, Specificity::new(0, 1, 0)] {
                    for source_order in [1, 2] {
                        for module_rank in [ModuleRank::ZERO, ModuleRank::new(1, 0, 0)] {
                            candidates.push((
                                CascadeKey::new(
                                    level,
                                    normalized_layer_rank(false, LayerOrdinal::new(layer_ordinal)),
                                    scope_proximity,
                                    specificity,
                                    source_order,
                                ),
                                OpenWorldTieEvidence::new(module_rank),
                            ));
                        }
                    }
                }
            }
        }
    }

    for (left_index, left) in candidates.iter().copied().enumerate() {
        for (right_index, right) in candidates.iter().copied().enumerate() {
            if left_index == right_index {
                continue;
            }

            // This test-only tuple is projected from CSS Cascading and Inheritance
            // Level 6 section 2.1, Cascade Sorting Order (W3C Working Draft,
            // 6 September 2024): specificity precedes scoping proximity. Module
            // provenance is Omena's post-key tie-break, not a specification axis.
            let oracle_key = |(key, evidence): (CascadeKey, OpenWorldTieEvidence)| {
                (
                    key.level,
                    key.layer_rank.get(),
                    key.specificity.ids,
                    key.specificity.classes,
                    key.specificity.elements,
                    Reverse(key.scope_proximity),
                    key.source_order,
                    evidence.module_rank.distance_priority,
                    evidence.module_rank.import_order_priority,
                    evidence.module_rank.file_order_priority,
                )
            };
            let expected = if oracle_key(left) > oracle_key(right) {
                "left"
            } else {
                "right"
            };
            let (selected, _) = select_open_world_cascade_winner(
                [("left", left), ("right", right)],
                |(_, ranked)| *ranked,
            )
            .ok_or_else(|| "the enumerated fixture always contains two candidates".to_string())?;

            let outcome = cascade_property_open_world(
                [
                    declaration_with_tie_evidence("left", "red", left.0, left.1),
                    declaration_with_tie_evidence("right", "blue", right.0, right.1),
                ],
                "color",
            );
            let ranked = match outcome {
                CascadeOutcome::Definite {
                    winner,
                    also_considered,
                    ..
                } => std::iter::once(winner)
                    .chain(also_considered)
                    .collect::<Vec<_>>(),
                CascadeOutcome::RankedSet(ranked) => ranked,
                other => {
                    return Err(format!(
                        "two matching declarations must be ranked, got {other:?}"
                    ));
                }
            };

            assert_eq!(
                selected.0, expected,
                "open-world selector disagreed for pair ({left_index}, {right_index})"
            );
            assert_eq!(
                ranked[0].id, expected,
                "cascade_property_open_world disagreed for pair ({left_index}, {right_index})"
            );
        }
    }
    Ok(())
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
            "specificityIds",
            "specificityClasses",
            "specificityElements",
            "scopeProximity",
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
        custom_property_key("--brand"),
        CascadeValue::Literal("red".to_string()),
    );

    let result = compute_cascade_computed_value_with_standard_value_validator_v0(
        CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: vec![property_declaration(
                "color-decl",
                "color",
                CascadeValue::Var {
                    name: custom_property_key("--brand"),
                    fallback: None,
                },
                1,
            )],
            custom_property_env: env,
            parent_computed_value: Some(CascadeValue::Literal("blue".to_string())),
            registered_custom_property: None,
            standard_property_value_verdicts: BTreeMap::from([(
                "color-decl".to_string(),
                CascadeStandardValueVerdictV0::Unknown,
            )]),
        },
        &FixtureStandardValueValidator,
    );

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
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
    });
    assert_eq!(valid.status, ComputedCascadeValueStatusV0::Resolved);
    assert_eq!(valid.value, CascadeValue::Literal("12px".to_string()));
}

#[test]
fn standard_property_syntax_unmatched_uses_iacvt_fallback() {
    let declaration_id = "invalid-color";
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            declaration_id,
            "color",
            CascadeValue::Literal("definitely-not-a-color".to_string()),
            1,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::from([(
            declaration_id.to_string(),
            CascadeStandardValueVerdictV0::Unmatched,
        )]),
    });

    // The grammar-owning caller emits Unmatched for this definite invalid
    // literal. Removing or relabeling that verdict observes Resolved instead.
    assert_eq!(
        result.status,
        ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
    );
    assert!(result.invalid_at_computed_value_time);
    assert!(
        result
            .derivation_steps
            .contains(&"standardPropertySyntaxUnmatched")
    );
    assert!(
        result
            .derivation_steps
            .contains(&"invalidAtComputedValueTimeFallsBackAsUnset")
    );
}

#[test]
fn standard_property_syntax_unknown_is_typed_indeterminate() {
    let declaration_id = "unknown-color";
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            declaration_id,
            "color",
            CascadeValue::Literal("future-color-function(1)".to_string()),
            1,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::from([(
            declaration_id.to_string(),
            CascadeStandardValueVerdictV0::Unknown,
        )]),
    });

    // NotValidatable is a live caller output for unsupported grammar forms.
    // Mapping it to Matched makes both the status and typed reason differ.
    assert_eq!(result.status, ComputedCascadeValueStatusV0::Indeterminate);
    assert_eq!(
        result.indeterminate_reason,
        Some(ComputedCascadeIndeterminateReasonV0::StandardPropertySyntaxIndeterminate)
    );
}

#[test]
fn standard_property_syntax_is_revalidated_after_var_substitution() {
    for (custom_value, expected_status) in [
        ("red", ComputedCascadeValueStatusV0::Resolved),
        (
            "12px",
            ComputedCascadeValueStatusV0::InvalidAtComputedValueTime,
        ),
    ] {
        let declaration_id = format!("variable-color-{custom_value}");
        let mut custom_property_env = CustomPropertyEnv::new();
        custom_property_env.insert(
            custom_property_key("--tone"),
            CascadeValue::Literal(custom_value.to_string()),
        );
        let result = compute_cascade_computed_value_with_standard_value_validator_v0(
            CascadeComputedValueInputV0 {
                property: "color".to_string(),
                declarations: vec![property_declaration(
                    declaration_id.as_str(),
                    "color",
                    CascadeValue::Var {
                        name: custom_property_key("--tone"),
                        fallback: None,
                    },
                    1,
                )],
                custom_property_env,
                parent_computed_value: None,
                registered_custom_property: None,
                standard_property_value_verdicts: BTreeMap::from([(
                    declaration_id,
                    CascadeStandardValueVerdictV0::Unknown,
                )]),
            },
            &FixtureStandardValueValidator,
        );

        assert_eq!(result.status, expected_status, "{custom_value}");
        assert!(
            result
                .derivation_steps
                .contains(&"standardPropertySyntaxDeferredByVarReference")
        );
        if custom_value == "12px" {
            assert!(result.invalid_at_computed_value_time);
            assert!(
                result
                    .derivation_steps
                    .contains(&"postSubstitutionStandardPropertySyntaxUnmatched")
            );
        } else {
            assert_eq!(result.value, CascadeValue::Literal("red".to_string()));
            assert!(
                result
                    .derivation_steps
                    .contains(&"postSubstitutionStandardPropertySyntaxMatched")
            );
        }
    }
}

#[test]
fn missing_standard_property_verdict_is_explicitly_unavailable() {
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            "unchecked-color",
            "color",
            CascadeValue::Literal("!!! not-a-color 42px };drop".to_string()),
            1,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::new(),
    });

    assert_eq!(result.status, ComputedCascadeValueStatusV0::Indeterminate);
    assert_eq!(
        result.indeterminate_reason,
        Some(ComputedCascadeIndeterminateReasonV0::StandardPropertySyntaxIndeterminate)
    );
    assert!(
        result
            .derivation_steps
            .contains(&"standardPropertySyntaxVerdictUnavailable")
    );
    assert!(
        !result
            .derivation_steps
            .contains(&"standardPropertySyntaxMatched")
    );
}

#[test]
fn iacvt_fallback_preserves_its_indeterminate_reason_separately() {
    let declaration_id = "invalid-future-property";
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "future-property".to_string(),
        declarations: vec![property_declaration(
            declaration_id,
            "future-property",
            CascadeValue::Literal("invalid".to_string()),
            1,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::from([(
            declaration_id.to_string(),
            CascadeStandardValueVerdictV0::Unmatched,
        )]),
    });

    // Unknown inheritance metadata is a live fallback state. Clearing the
    // sibling reason at the IACVT choke point makes this assertion observe None.
    assert_eq!(
        result.status,
        ComputedCascadeValueStatusV0::InvalidAtComputedValueTime
    );
    assert_eq!(result.indeterminate_reason, None);
    assert_eq!(
        result.fallback_indeterminate_reason,
        Some(ComputedCascadeIndeterminateReasonV0::PropertyInheritanceMetadataUnavailable)
    );
}

#[test]
fn unregistered_custom_property_keeps_the_inherited_computed_value_contract() {
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "--gap".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: Some(CascadeValue::Literal("16px".to_string())),
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::new(),
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
    let first_key = PropertyNameV0::canonical_standard_key(first_name);
    let outside_key = PropertyNameV0::canonical_standard_key(outside_name);

    assert_eq!(
        css_property_metadata_for_property_in_records(&first_key, prefix)
            .map(|record| record.canonical_name),
        Some(first_name)
    );
    assert!(css_property_metadata_for_property_in_records(&outside_key, prefix).is_none());
    assert_eq!(
        css_property_metadata_for_property_in_records(
            &outside_key,
            CSS_PROPERTY_METADATA_RECORDS_V1
        )
        .map(|record| record.canonical_name),
        Some(outside_name)
    );
    assert_eq!(
        css_property_metadata_for_property("COLOR").map(|record| record.canonical_name),
        Some("color")
    );
    assert_eq!(
        css_property_metadata_for_property(r"c\6f lor").map(|record| record.canonical_name),
        Some("color")
    );
    assert!(css_property_metadata_for_property("--color").is_none());
}

#[test]
fn unknown_property_metadata_is_typed_as_indeterminate() {
    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "future-property".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
    });

    let unknown_initial_value = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "background".to_string(),
        declarations: Vec::new(),
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::new(),
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
        standard_property_value_verdicts: BTreeMap::new(),
    });

    let unknown_standard_syntax = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            "unknown-color",
            "color",
            CascadeValue::Literal("future-color-function(1)".to_string()),
            1,
        )],
        custom_property_env: CustomPropertyEnv::new(),
        parent_computed_value: None,
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::from([(
            "unknown-color".to_string(),
            CascadeStandardValueVerdictV0::Unknown,
        )]),
    });

    let inherited_from_indeterminate =
        compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: "color".to_string(),
            declarations: Vec::new(),
            custom_property_env: CustomPropertyEnv::new(),
            parent_computed_value: Some(CascadeValue::Indeterminate),
            registered_custom_property: None,
            standard_property_value_verdicts: BTreeMap::new(),
        });

    let fixtures = [
        cascade_outcome,
        unknown_inheritance,
        unknown_initial_value,
        unknown_registered_syntax,
        unknown_standard_syntax,
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
            custom_property_key("--cyclic"),
            CascadeValue::Var {
                name: custom_property_key("--cyclic"),
                fallback: None,
            },
        );
        let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
            property: property.to_string(),
            declarations: vec![property_declaration(
                "cyclic-value",
                property,
                CascadeValue::Var {
                    name: custom_property_key("--cyclic"),
                    fallback: None,
                },
                1,
            )],
            custom_property_env: env,
            parent_computed_value: None,
            registered_custom_property: None,
            standard_property_value_verdicts: BTreeMap::new(),
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
        custom_property_key("--a"),
        CascadeValue::Var {
            name: custom_property_key("--b"),
            fallback: None,
        },
    );
    env.insert(
        custom_property_key("--b"),
        CascadeValue::Var {
            name: custom_property_key("--a"),
            fallback: None,
        },
    );

    let result = compute_cascade_computed_value(CascadeComputedValueInputV0 {
        property: "color".to_string(),
        declarations: vec![property_declaration(
            "cycle-color",
            "color",
            CascadeValue::Var {
                name: custom_property_key("--a"),
                fallback: None,
            },
            1,
        )],
        custom_property_env: env,
        parent_computed_value: Some(CascadeValue::Literal("canvas".to_string())),
        registered_custom_property: None,
        standard_property_value_verdicts: BTreeMap::new(),
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
fn longhand_merge_canonicalizes_standard_property_identity_and_preserves_authored_names() {
    let proof = prove_longhand_merge(
        "PLACE-CONTENT",
        &["align-content", "justify-content"],
        &[
            LonghandMergeInputV0 {
                property: r"ALIGN-\63 ONTENT".to_string(),
                value: "center".to_string(),
                important: false,
                source_order: 10,
            },
            LonghandMergeInputV0 {
                property: "JUSTIFY-CONTENT".to_string(),
                value: "space-between".to_string(),
                important: false,
                source_order: 11,
            },
        ],
    );

    assert!(proof.accepted);
    assert_eq!(proof.shorthand_property, "PLACE-CONTENT");
    assert_eq!(
        proof.ordered_longhand_properties,
        vec![
            r"ALIGN-\63 ONTENT".to_string(),
            "JUSTIFY-CONTENT".to_string()
        ]
    );
}

#[test]
fn box_shorthand_uses_property_authority_without_rewriting_authored_names() {
    let proof = prove_box_shorthand_combination(
        "MARGIN",
        &[
            BoxLonghandInputV0 {
                property: r"MARGIN-\74 OP".to_string(),
                value: "1px".to_string(),
                important: false,
                source_order: 1,
            },
            BoxLonghandInputV0 {
                property: "MARGIN-RIGHT".to_string(),
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

    assert!(proof.accepted);
    assert_eq!(proof.shorthand_property, "MARGIN");
    assert_eq!(
        proof.ordered_longhand_properties,
        vec![
            r"MARGIN-\74 OP".to_string(),
            "MARGIN-RIGHT".to_string(),
            "margin-bottom".to_string(),
            "margin-left".to_string(),
        ]
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
fn supports_custom_property_name_does_not_acquire_a_standard_property_feature() {
    let witness = evaluate_static_supports_condition(
        "(--theme-inline: 1)",
        StaticSupportsAssumptionV0::TargetCapability(SupportsTargetCapabilityV0::all_supported()),
    );

    assert_eq!(witness.verdict, StaticSupportsEvalVerdictV0::Unknown);
    assert!(!witness.provenance_preserved);
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
fn simple_selector_signature_uses_escape_aware_class_names() {
    let signature = parse_simple_selector_signature(r".a\.b.카드");
    assert!(
        signature.is_some(),
        "escape-aware class selector should parse"
    );
    if let Some(signature) = signature {
        // Both class spellings are emitted directly by this selector; an ASCII or
        // non-escape-aware extractor makes one of these assertions false.
        assert!(signature.required_classes.contains(r"a\.b"));
        assert!(signature.required_classes.contains("카드"));
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
    assert_eq!(
        where_box
            .functional_pseudo_constraints
            .iter()
            .map(|constraint| constraint.name.as_str())
            .collect::<Vec<_>>(),
        vec!["where"]
    );
    assert!(!where_box.required_pseudo_states.contains("where"));
}

#[test]
fn is_pseudo_takes_most_specific_argument_specificity() {
    // RFC-0007-B B3: `:is(.a, #b)` takes `#b`'s specificity (the most specific
    // argument), not the first or the sum.
    let Some(signature) = parse_simple_selector_signature(":is(.a, #b)") else {
        unreachable!(":is(...) parses")
    };
    assert_eq!(signature.specificity, Specificity::new(1, 0, 0));
    assert_eq!(
        signature
            .functional_pseudo_constraints
            .iter()
            .map(|constraint| (constraint.name.as_str(), constraint.arguments.as_str()))
            .collect::<Vec<_>>(),
        vec![("is", ".a, #b")]
    );
    assert!(!signature.required_pseudo_states.contains("is"));
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
fn functional_pseudo_matching_does_not_invent_a_definite_non_match() {
    let class_element = ElementSignature::concrete(None::<String>, None::<String>, ["foo"]);
    let class_witness = selector_match_witness(":is(.foo)", &class_element);
    assert_ne!(class_witness.verdict, SelectorMatchVerdict::No);

    let complex_element = ElementSignature::concrete(None::<String>, Some("root"), ["item"]);
    let complex_witness = selector_match_witness(":is(#root .item)", &complex_element);
    assert_ne!(complex_witness.verdict, SelectorMatchVerdict::No);

    let plain_witness = selector_match_witness(".foo", &class_element);
    assert_eq!(plain_witness.verdict, SelectorMatchVerdict::Yes);
}

#[test]
fn lossy_selector_matching_does_not_invent_a_definite_match() {
    let mut attribute_element =
        ElementSignature::concrete(None::<String>, None::<String>, Vec::<String>::new());
    attribute_element.attributes.insert("type".to_string());
    let attribute_witness = selector_match_witness("[type=\"text\"]", &attribute_element);
    assert_eq!(attribute_witness.verdict, SelectorMatchVerdict::Maybe);

    let class_element = ElementSignature::concrete(None::<String>, None::<String>, ["button"]);
    let pseudo_element_witness = selector_match_witness(".button::before", &class_element);
    assert_eq!(pseudo_element_witness.verdict, SelectorMatchVerdict::Maybe);
}

#[test]
fn selector_match_witness_reports_specificity_exactness() {
    let element = ElementSignature::concrete(None::<String>, None::<String>, ["b"]);
    let witness = selector_match_witness(":is(:unknown(.a), .b)", &element);

    assert_eq!(witness.verdict, SelectorMatchVerdict::Maybe);
    assert_eq!(
        witness.specificity_exactness,
        SpecificityExactnessV0::Inexact
    );
    let serialized = serde_json::to_value(&witness).unwrap_or(serde_json::Value::Null);
    assert_eq!(
        serialized["specificityExactness"],
        serde_json::Value::String("inexact".to_string())
    );
}

#[test]
fn selector_matching_fuzz_seed_corpus_respects_the_co_match_ceiling() {
    let cases = [
        (":is(.foo)", vec!["foo"], Vec::<&str>::new()),
        (":where(.foo)", vec!["foo"], Vec::<&str>::new()),
        (":not(.bar)", vec!["foo"], Vec::<&str>::new()),
        ("[type=\"text\"]", Vec::<&str>::new(), vec!["type"]),
        ("[data-kind^=\"x\"]", Vec::<&str>::new(), vec!["data-kind"]),
        (".button::before", vec!["button"], Vec::<&str>::new()),
        (".button::after", vec!["button"], Vec::<&str>::new()),
        (".parent > .child", vec!["child"], Vec::<&str>::new()),
    ];

    for (selector, classes, attributes) in cases {
        let mut element = ElementSignature::concrete(None::<String>, None::<String>, classes);
        element.attributes = attributes.into_iter().map(str::to_string).collect();
        let ceiling = selector_co_match_verdict(selector, selector);
        let direct = selector_match_witness(selector, &element).verdict;

        if ceiling == SelectorMatchVerdict::Maybe {
            assert_eq!(direct, SelectorMatchVerdict::Maybe, "{selector}");
        }
    }
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
        standard_property_value_verdicts: BTreeMap::new(),
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
    assert_eq!(
        signature
            .functional_pseudo_constraints
            .iter()
            .map(|constraint| constraint.name.as_str())
            .collect::<Vec<_>>(),
        vec!["not"]
    );
    assert!(!signature.required_pseudo_states.contains("not"));
}

#[test]
fn has_pseudo_takes_most_specific_argument_specificity() {
    // Selectors L4: `:has(.a, #b)` takes the specificity of its most specific
    // argument (same rule as `:is`/`:not`), so the rule is no longer dropped.
    let Some(signature) = parse_simple_selector_signature(":has(.a, #b)") else {
        unreachable!(":has(...) parses")
    };
    assert_eq!(signature.specificity, Specificity::new(1, 0, 0));
    assert_eq!(
        signature
            .functional_pseudo_constraints
            .iter()
            .map(|constraint| constraint.name.as_str())
            .collect::<Vec<_>>(),
        vec!["has"]
    );
    assert!(!signature.required_pseudo_states.contains("has"));
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
        custom_property_key("--brand"),
        CascadeValue::Literal("red".to_string()),
    );

    let resolved = substitute_custom_properties(
        &CascadeValue::Var {
            name: custom_property_key("--brand"),
            fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
        },
        &env,
    );
    assert_eq!(resolved, CascadeValue::Literal("red".to_string()));

    let fallback = substitute_custom_properties(
        &CascadeValue::Var {
            name: custom_property_key("--missing"),
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
        custom_property_key("--gap"),
        CascadeValue::Literal("2px".to_string()),
    );
    env.insert(
        custom_property_key("--shadow"),
        CascadeValue::Composite(vec![
            CascadeValue::Literal("0 0 ".to_string()),
            CascadeValue::Var {
                name: custom_property_key("--gap"),
                fallback: None,
            },
        ]),
    );
    env.insert(
        custom_property_key("--invalid-shadow"),
        CascadeValue::Composite(vec![
            CascadeValue::Literal("0 0 ".to_string()),
            CascadeValue::Var {
                name: custom_property_key("--missing"),
                fallback: None,
            },
        ]),
    );

    let resolved = substitute_custom_properties(
        &CascadeValue::Var {
            name: custom_property_key("--shadow"),
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
            name: custom_property_key("--invalid-shadow"),
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
        custom_property_key("--a"),
        CascadeValue::Var {
            name: custom_property_key("--b"),
            fallback: None,
        },
    );
    env.insert(
        custom_property_key("--b"),
        CascadeValue::Var {
            name: custom_property_key("--a"),
            fallback: None,
        },
    );

    let resolved = substitute_custom_properties(
        &CascadeValue::Var {
            name: custom_property_key("--a"),
            fallback: None,
        },
        &env,
    );

    assert_eq!(resolved, CascadeValue::GuaranteedInvalid);

    let fallback = substitute_custom_properties(
        &CascadeValue::Var {
            name: custom_property_key("--a"),
            fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
        },
        &env,
    );

    assert_eq!(fallback, CascadeValue::Literal("blue".to_string()));
}

#[test]
fn marks_every_cycle_member_invalid_before_resolving_an_outer_fallback() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        custom_property_key("--cycle-a"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-b"),
            fallback: Some(Box::new(CascadeValue::Literal("red".to_string()))),
        },
    );
    env.insert(
        custom_property_key("--cycle-b"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-a"),
            fallback: Some(Box::new(CascadeValue::Literal("green".to_string()))),
        },
    );
    env.insert(
        custom_property_key("--outer"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-a"),
            fallback: Some(Box::new(CascadeValue::Literal("gold".to_string()))),
        },
    );

    let resolved = resolve_custom_property_env_least_fixed_point(&env);

    assert_eq!(
        resolved.get(&custom_property_key("--cycle-a")),
        Some(&CascadeValue::GuaranteedInvalid)
    );
    assert_eq!(
        resolved.get(&custom_property_key("--cycle-b")),
        Some(&CascadeValue::GuaranteedInvalid)
    );
    assert_eq!(
        resolved.get(&custom_property_key("--outer")),
        Some(&CascadeValue::Literal("gold".to_string()))
    );
}

#[test]
fn fallback_edges_make_a_three_node_cycle_invalid_when_entered_mid_chain() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        custom_property_key("--entry"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-b"),
            fallback: None,
        },
    );
    env.insert(
        custom_property_key("--cycle-a"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-b"),
            fallback: Some(Box::new(CascadeValue::Literal("red".to_string()))),
        },
    );
    env.insert(
        custom_property_key("--cycle-b"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-c"),
            fallback: Some(Box::new(CascadeValue::Literal("green".to_string()))),
        },
    );
    env.insert(
        custom_property_key("--cycle-c"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-a"),
            fallback: Some(Box::new(CascadeValue::Literal("blue".to_string()))),
        },
    );

    let resolved = resolve_custom_property_env_least_fixed_point(&env);

    for name in ["--entry", "--cycle-a", "--cycle-b", "--cycle-c"] {
        assert_eq!(
            resolved.get(&custom_property_key(name)),
            Some(&CascadeValue::GuaranteedInvalid),
            "{name}"
        );
    }
}

#[test]
fn summarizes_custom_property_least_fixed_point() {
    let mut env = CustomPropertyEnv::new();
    env.insert(
        custom_property_key("--brand"),
        CascadeValue::Literal("red".to_string()),
    );
    env.insert(
        custom_property_key("--alias"),
        CascadeValue::Var {
            name: custom_property_key("--brand"),
            fallback: None,
        },
    );
    env.insert(
        custom_property_key("--shadow"),
        CascadeValue::Composite(vec![
            CascadeValue::Literal("0 0 ".to_string()),
            CascadeValue::Var {
                name: custom_property_key("--alias"),
                fallback: None,
            },
        ]),
    );
    env.insert(
        custom_property_key("--cycle-a"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-b"),
            fallback: None,
        },
    );
    env.insert(
        custom_property_key("--cycle-b"),
        CascadeValue::Var {
            name: custom_property_key("--cycle-a"),
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
    assert_eq!(summary.iteration_count, 4);
    assert_eq!(summary.iteration_bound, 4);
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
        "max(1, strongly_connected_component_count)"
    );
    assert!(
        summary
            .proof
            .proof_obligations
            .contains(&"complete strongly connected component partition")
    );
    assert!(
        summary
            .proof
            .proof_obligations
            .contains(&"no non-converged approximation return")
    );
    assert_eq!(
        summary.proof.bounded_fixed_point_computation_witness,
        "every strongly connected component is processed exactly once; no non-converged approximation is returned"
    );
    assert_eq!(
        summary.proof.monotonic_progress_witness,
        "each scheduled component only adds finalized bindings to the resolved environment"
    );
    let preferred_witness = custom_property_bounded_fixed_point_computation_witness();
    let legacy_witness: CustomPropertyLeastFixedPointProofV0 = preferred_witness.clone();
    assert_eq!(preferred_witness, legacy_witness);
    let serialized_witness =
        serde_json::to_value(&preferred_witness).unwrap_or(serde_json::Value::Null);
    let serialized_keys = serialized_witness
        .as_object()
        .map(|object| object.keys().cloned().collect::<BTreeSet<_>>())
        .unwrap_or_default();
    assert_eq!(
        serialized_keys,
        BTreeSet::from([
            "cyclePolicy".to_string(),
            "finiteDomain".to_string(),
            "iterationBoundFormula".to_string(),
            "monotoneWitness".to_string(),
            "proofObligations".to_string(),
            "transferFunction".to_string(),
        ])
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
fn generated_invariant_self_check_corpus_preserves_cascade_and_var_invariants() {
    let report = run_generated_cascade_invariant_self_check_corpus();

    assert_eq!(
        report.product,
        "omena-cascade.generated-invariant-self-check-corpus"
    );
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
        "canonical custom-property dependency graph with cyclic-SCC invalidation and dependency-ordered acyclic substitution"
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
    assert!(
        summary
            .ready_surfaces
            .contains(&"cascadeOrderingAxisSelfCheckCorpus")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cascadeConformanceSeedCorpus")
    );
    assert!(!summary.not_ready_surfaces.contains(&"selectorMatchWitness"));
    assert!(summary.not_ready_surfaces.contains(&"fullWptCascadeCorpus"));
}

#[test]
fn seed_conformance_corpus_passes_current_cascade_model() {
    let report = run_cascade_conformance_seed_corpus();

    assert_eq!(report.product, "omena-cascade.conformance-seed-corpus");
    assert_eq!(report.case_count, 39);
    let important_origin_pin = report
        .results
        .iter()
        .find(|result| result.name == "inline-important-outranks-author-important")
        .map(|result| (result.actual_outcome, result.actual_winner_id.as_deref()));
    // Element-attached declarations outrank style-rule declarations at equal importance.
    assert_eq!(
        important_origin_pin,
        Some(("definite", Some("inline-important")))
    );
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
fn conformance_corpus_counts_direction_conflicts() {
    let conformance_report = run_cascade_conformance_seed_corpus();
    let direction_conflicts = conformance_report
        .results
        .iter()
        .filter(|result| {
            result
                .name
                .starts_with("specificity-precedes-opposed-scope-")
        })
        .collect::<Vec<_>>();
    let failures = direction_conflicts
        .iter()
        .filter(|result| !result.passed)
        .collect::<Vec<_>>();

    for result in &failures {
        eprintln!(
            "direction_conflict_failure case={} expected_deciding_axis=specificity expected_winner={:?} actual_winner={:?}",
            result.name, result.expected_winner_id, result.actual_winner_id
        );
    }
    eprintln!(
        "direction_conflict_count={} direction_conflict_failure_count={}",
        direction_conflicts.len(),
        failures.len()
    );
    assert_eq!(direction_conflicts.len(), 18);
    assert_eq!(failures.len(), 0);
}

#[test]
fn equal_specificity_proximity_sweep_remains_complement() {
    let self_check_report = run_cascade_ordering_axis_self_check_corpus();
    let equal_specificity_proximity_complement = self_check_report
        .results
        .iter()
        .filter(|result| result.name.starts_with("self-check-scope-proximity-"))
        .collect::<Vec<_>>();
    let failure_count = equal_specificity_proximity_complement
        .iter()
        .filter(|result| !result.passed)
        .count();

    eprintln!(
        "equal_specificity_proximity_complement_count={} equal_specificity_proximity_failure_count={failure_count}",
        equal_specificity_proximity_complement.len(),
    );
    assert_eq!(equal_specificity_proximity_complement.len(), 56);
    assert_eq!(failure_count, 0);
}

fn hand_written_cascade_winner(case_name: &str) -> Option<String> {
    run_cascade_conformance_seed_corpus()
        .results
        .into_iter()
        .find(|result| result.name == case_name)
        .and_then(|result| result.passed.then_some(result.actual_winner_id))
        .flatten()
}

#[test]
fn normal_layer_order_control_remains_spec_aligned() {
    assert_eq!(
        hand_written_cascade_winner("layer-rank-beats-specificity-within-level").as_deref(),
        Some("higher-layer")
    );
}

#[test]
fn important_layer_order_conformance_is_hand_written() {
    assert_eq!(
        hand_written_cascade_winner("important-layer-order-is-reversed").as_deref(),
        Some("earlier-layer")
    );
}

#[test]
fn unlayered_normal_conformance_is_hand_written() {
    assert_eq!(
        hand_written_cascade_winner("unlayered-normal-outranks-layered-normal").as_deref(),
        Some("unlayered")
    );
}

#[test]
fn unlayered_important_conformance_is_hand_written() {
    assert_eq!(
        hand_written_cascade_winner("layered-important-outranks-unlayered-important").as_deref(),
        Some("layered")
    );
}

#[test]
fn ordering_axis_self_check_corpus_passes_current_cascade_model() {
    let report = run_cascade_ordering_axis_self_check_corpus();

    assert_eq!(
        report.product,
        "omena-cascade.ordering-axis-self-check-corpus"
    );
    assert!(report.case_count >= 200);
    assert_eq!(report.passed_count, report.case_count);
    assert_eq!(report.failed_count, 0);
    assert!(report.results.iter().all(|result| result.passed));
}
