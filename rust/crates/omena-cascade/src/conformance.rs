//! Conformance seed corpora for the cascade algebra.
//!
//! The hand-written cases are intentionally small and explicit. The generated
//! ordering-axis sweep is only an implementation self-check and is not evidence
//! for browser conformance or for regressions covered by the hand-written cases.

use crate::{
    CascadeConformanceSeedCase, CascadeConformanceSeedReport, CascadeConformanceSeedResult,
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeOriginV0, CascadeOutcome, CascadeValue,
    LayerOrdinal, OpenWorldTieEvidence, Specificity, SpecificityExactnessV0,
    cascade_level_for_origin, cascade_property, normalized_layer_rank,
    parse_simple_selector_signature,
};

struct SelectorSpecificitySeedDeclaration {
    id: String,
    value: String,
    selector: String,
    expected_specificity: Specificity,
    expected_exactness: SpecificityExactnessV0,
    source_order: u32,
}

struct SelectorSpecificitySeedCase {
    name: String,
    property: &'static str,
    declarations: Vec<SelectorSpecificitySeedDeclaration>,
    expected_outcome: &'static str,
    expected_winner_id: Option<String>,
}

pub fn run_cascade_conformance_seed_corpus() -> CascadeConformanceSeedReport {
    let mut results = cascade_conformance_seed_cases()
        .into_iter()
        .map(run_cascade_conformance_seed_case)
        .collect::<Vec<_>>();
    results.extend(
        selector_specificity_conformance_seed_cases()
            .into_iter()
            .map(run_selector_specificity_seed_case),
    );
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    CascadeConformanceSeedReport {
        schema_version: "0",
        product: "omena-cascade.conformance-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count.saturating_sub(passed_count),
        results,
    }
}

fn run_selector_specificity_seed_case(
    case: SelectorSpecificitySeedCase,
) -> CascadeConformanceSeedResult {
    let expected_declaration_count = case.declarations.len();
    let mut signatures_match = true;
    let declarations = case
        .declarations
        .into_iter()
        .filter_map(|expected| {
            let signature = parse_simple_selector_signature(expected.selector.as_str());
            let Some(signature) = signature else {
                signatures_match = false;
                return None;
            };
            signatures_match &= signature.specificity == expected.expected_specificity
                && signature.specificity_exactness == expected.expected_exactness;
            Some(CascadeDeclaration {
                id: expected.id,
                property: case.property.to_string(),
                value: CascadeValue::Literal(expected.value),
                key: conformance_key(
                    CascadeLevel::AuthorNormal,
                    0,
                    0,
                    signature.specificity,
                    expected.source_order,
                ),
                open_world_tie_evidence: OpenWorldTieEvidence::NONE,
                specificity_exactness: signature.specificity_exactness,
            })
        })
        .collect::<Vec<_>>();

    let (actual_outcome, actual_winner_id) = if declarations.len() != expected_declaration_count {
        ("selectorUnavailable", None)
    } else {
        match cascade_property(declarations, case.property) {
            CascadeOutcome::Definite { winner, .. } => ("definite", Some(winner.id)),
            CascadeOutcome::RankedSet(_) => ("rankedSet", None),
            CascadeOutcome::Inherit => ("inherit", None),
            CascadeOutcome::Top => ("top", None),
        }
    };
    let passed = signatures_match
        && actual_outcome == case.expected_outcome
        && actual_winner_id == case.expected_winner_id;

    CascadeConformanceSeedResult {
        name: case.name,
        passed,
        expected_outcome: case.expected_outcome,
        actual_outcome,
        expected_winner_id: case.expected_winner_id,
        actual_winner_id,
    }
}

pub fn run_cascade_ordering_axis_self_check_corpus() -> CascadeConformanceSeedReport {
    let results = cascade_ordering_axis_self_check_cases()
        .into_iter()
        .map(run_cascade_conformance_seed_case)
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    CascadeConformanceSeedReport {
        schema_version: "0",
        product: "omena-cascade.ordering-axis-self-check-corpus",
        case_count,
        passed_count,
        failed_count: case_count.saturating_sub(passed_count),
        results,
    }
}

fn run_cascade_conformance_seed_case(
    case: CascadeConformanceSeedCase,
) -> CascadeConformanceSeedResult {
    let outcome = cascade_property(case.declarations, case.property);
    let (actual_outcome, actual_winner_id) = match outcome {
        CascadeOutcome::Definite { winner, .. } => ("definite", Some(winner.id)),
        CascadeOutcome::RankedSet(_) => ("rankedSet", None),
        CascadeOutcome::Inherit => ("inherit", None),
        CascadeOutcome::Top => ("top", None),
    };
    let passed =
        actual_outcome == case.expected_outcome && actual_winner_id == case.expected_winner_id;

    CascadeConformanceSeedResult {
        name: case.name,
        passed,
        expected_outcome: case.expected_outcome,
        actual_outcome,
        expected_winner_id: case.expected_winner_id,
        actual_winner_id,
    }
}

fn cascade_conformance_seed_cases() -> Vec<CascadeConformanceSeedCase> {
    let mut cases = vec![
        CascadeConformanceSeedCase {
            name: "source-order-breaks-identical-key".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "source-earlier",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
                conformance_decl(
                    "source-later",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("source-later".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "specificity-beats-source-order".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "specificity-low-later",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "specificity-high-earlier",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        0,
                        Specificity::new(1, 0, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("specificity-high-earlier".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "important-origin-beats-inline-normal".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "inline-normal",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::InlineNormal,
                        0,
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "author-important",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorImportant,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("author-important".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "inline-important-outranks-author-important".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "author-important",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorImportant,
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "inline-important",
                    "color",
                    "red",
                    conformance_key(
                        cascade_level_for_origin(CascadeOriginV0::Inline, true),
                        0,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("inline-important".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "layer-rank-beats-specificity-within-level".to_string(),
            property: "color",
            declarations: vec![
                // Normal declarations in later layers outrank earlier layers.
                // Reversion: collapse the higher layer ordinal to the lower ordinal.
                conformance_decl(
                    "lower-layer-specific",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        1,
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "higher-layer",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        2,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("higher-layer".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "important-layer-order-is-reversed".to_string(),
            property: "color",
            declarations: vec![
                // Important declarations in earlier layers outrank later layers.
                // Reversion: normalize the important layer ranks as normal ranks.
                conformance_decl(
                    "earlier-layer",
                    "color",
                    "red",
                    conformance_layer_key(
                        CascadeLevel::AuthorImportant,
                        true,
                        Some(0),
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
                conformance_decl(
                    "later-layer-specific",
                    "color",
                    "blue",
                    conformance_layer_key(
                        CascadeLevel::AuthorImportant,
                        true,
                        Some(1),
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("earlier-layer".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "unlayered-normal-outranks-layered-normal".to_string(),
            property: "color",
            declarations: vec![
                // Normal declarations outside a layer outrank all layered declarations.
                // Reversion: map the unlayered declaration to layer ordinal zero.
                conformance_decl(
                    "layered-specific",
                    "color",
                    "red",
                    conformance_layer_key(
                        CascadeLevel::AuthorNormal,
                        false,
                        Some(1),
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "unlayered",
                    "color",
                    "blue",
                    conformance_layer_key(
                        CascadeLevel::AuthorNormal,
                        false,
                        None,
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("unlayered".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "layered-important-outranks-unlayered-important".to_string(),
            property: "color",
            declarations: vec![
                // Important declarations reverse the layer order, including the
                // implicit outer layer occupied by unlayered declarations.
                // Reversion: map the unlayered declaration to rank zero.
                conformance_decl(
                    "layered",
                    "color",
                    "red",
                    conformance_layer_key(
                        CascadeLevel::AuthorImportant,
                        true,
                        Some(1),
                        0,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
                conformance_decl(
                    "unlayered-specific",
                    "color",
                    "blue",
                    conformance_layer_key(
                        CascadeLevel::AuthorImportant,
                        true,
                        None,
                        0,
                        Specificity::new(1, 0, 0),
                        2,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("layered".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "nearer-scope-breaks-equal-specificity-tie".to_string(),
            property: "color",
            declarations: vec![
                conformance_decl(
                    "far-scope",
                    "color",
                    "red",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        5,
                        Specificity::new(0, 1, 0),
                        2,
                    ),
                ),
                conformance_decl(
                    "near-scope",
                    "color",
                    "blue",
                    conformance_key(
                        CascadeLevel::AuthorNormal,
                        0,
                        1,
                        Specificity::new(0, 1, 0),
                        1,
                    ),
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("near-scope".to_string()),
        },
        CascadeConformanceSeedCase {
            name: "missing-property-inherits".to_string(),
            property: "background",
            declarations: vec![conformance_decl(
                "color-only",
                "color",
                "red",
                conformance_key(
                    CascadeLevel::AuthorNormal,
                    0,
                    0,
                    Specificity::new(0, 1, 0),
                    1,
                ),
            )],
            expected_outcome: "inherit",
            expected_winner_id: None,
        },
    ];

    cases.extend(direction_conflict_conformance_seed_cases());
    cases
}

fn direction_conflict_conformance_seed_cases() -> Vec<CascadeConformanceSeedCase> {
    let specificities = [
        ("zero", Specificity::ZERO),
        ("class", Specificity::new(0, 1, 0)),
        ("id", Specificity::new(1, 0, 0)),
    ];
    let proximities = [0, 1, 3];
    let mut cases = Vec::new();

    for (left_specificity_rank, (left_specificity_name, left_specificity)) in
        specificities.into_iter().enumerate()
    {
        for (right_specificity_rank, (right_specificity_name, right_specificity)) in
            specificities.into_iter().enumerate()
        {
            if left_specificity_rank == right_specificity_rank {
                continue;
            }

            for left_proximity in proximities {
                for right_proximity in proximities {
                    if left_proximity == right_proximity {
                        continue;
                    }

                    let specificity_prefers_left = left_specificity_rank > right_specificity_rank;
                    let proximity_prefers_left = left_proximity < right_proximity;
                    if specificity_prefers_left == proximity_prefers_left {
                        continue;
                    }

                    // CSS Cascading Level 6 applies specificity before scoping
                    // proximity. These inputs deliberately make the two axes
                    // disagree, so the spec-authored winner is determined only
                    // from the enumerated specificity rank above.
                    let expected_winner = if specificity_prefers_left {
                        "left"
                    } else {
                        "right"
                    };
                    cases.push(CascadeConformanceSeedCase {
                        name: format!(
                            "specificity-precedes-opposed-scope-{left_specificity_name}-{left_proximity}-vs-{right_specificity_name}-{right_proximity}"
                        ),
                        property: "color",
                        declarations: vec![
                            conformance_decl(
                                "left",
                                "color",
                                "red",
                                conformance_key(
                                    CascadeLevel::AuthorNormal,
                                    0,
                                    left_proximity,
                                    left_specificity,
                                    1,
                                ),
                            ),
                            conformance_decl(
                                "right",
                                "color",
                                "blue",
                                conformance_key(
                                    CascadeLevel::AuthorNormal,
                                    0,
                                    right_proximity,
                                    right_specificity,
                                    2,
                                ),
                            ),
                        ],
                        expected_outcome: "definite",
                        expected_winner_id: Some(expected_winner.to_string()),
                    });
                }
            }
        }
    }

    cases
}

fn selector_specificity_conformance_seed_cases() -> Vec<SelectorSpecificitySeedCase> {
    vec![
        SelectorSpecificitySeedCase {
            name: "complex-functional-specificity-beats-source-order".to_string(),
            property: "color",
            declarations: vec![
                // L4: `:is()` takes its complex argument, so #root + .item = (1,1,0).
                selector_conformance_decl(
                    "complex",
                    "red",
                    ":is(#root .item)",
                    Specificity::new(1, 1, 0),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
                selector_conformance_decl(
                    "simple",
                    "blue",
                    ".item",
                    Specificity::new(0, 1, 0),
                    SpecificityExactnessV0::Exact,
                    2,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("complex".to_string()),
        },
        SelectorSpecificitySeedCase {
            name: "nested-is-not-specificity".to_string(),
            property: "color",
            declarations: vec![
                // L4: max(:not(#a), .b) = max((1,0,0), (0,1,0)) = (1,0,0).
                selector_conformance_decl(
                    "nested",
                    "red",
                    ":is(:not(#a), .b)",
                    Specificity::new(1, 0, 0),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
                selector_conformance_decl(
                    "classes",
                    "blue",
                    ".b.c",
                    Specificity::new(0, 2, 0),
                    SpecificityExactnessV0::Exact,
                    2,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("nested".to_string()),
        },
        SelectorSpecificitySeedCase {
            name: "nested-not-is-complex-specificity".to_string(),
            property: "color",
            declarations: vec![
                // L4: :is(#a .b) sums its compounds to (1,1,0); :not() adopts that value.
                selector_conformance_decl(
                    "nested",
                    "red",
                    ":not(:is(#a .b))",
                    Specificity::new(1, 1, 0),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
                selector_conformance_decl(
                    "id-only",
                    "blue",
                    "#a",
                    Specificity::new(1, 0, 0),
                    SpecificityExactnessV0::Exact,
                    2,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("nested".to_string()),
        },
        SelectorSpecificitySeedCase {
            name: "compound-not-specificity".to_string(),
            property: "color",
            declarations: vec![
                // L4: all three classes in the compound argument contribute, yielding (0,3,0).
                selector_conformance_decl(
                    "compound",
                    "red",
                    ":not(.a.b.c)",
                    Specificity::new(0, 3, 0),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
                selector_conformance_decl(
                    "two-classes",
                    "blue",
                    ".a.b",
                    Specificity::new(0, 2, 0),
                    SpecificityExactnessV0::Exact,
                    2,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("compound".to_string()),
        },
        SelectorSpecificitySeedCase {
            name: "relative-has-specificity".to_string(),
            property: "color",
            declarations: vec![
                // L4: the leading combinator adds nothing; .x + .y = (0,2,0).
                selector_conformance_decl(
                    "relative",
                    "red",
                    ":has(> .x .y)",
                    Specificity::new(0, 2, 0),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
                selector_conformance_decl(
                    "single-class",
                    "blue",
                    ".x",
                    Specificity::new(0, 1, 0),
                    SpecificityExactnessV0::Exact,
                    2,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("relative".to_string()),
        },
        SelectorSpecificitySeedCase {
            name: "where-specificity-remains-zero".to_string(),
            property: "color",
            declarations: vec![
                // L4: :where() is always (0,0,0), even when its argument contains an ID.
                selector_conformance_decl(
                    "where",
                    "red",
                    ":where(#a)",
                    Specificity::ZERO,
                    SpecificityExactnessV0::Exact,
                    2,
                ),
                selector_conformance_decl(
                    "type",
                    "blue",
                    "div",
                    Specificity::new(0, 0, 1),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("type".to_string()),
        },
        SelectorSpecificitySeedCase {
            name: "complex-type-and-class-specificity".to_string(),
            property: "color",
            declarations: vec![
                // L4: ul + li + .active across the complex argument = (0,1,2).
                selector_conformance_decl(
                    "complex",
                    "red",
                    ":is(ul > li.active)",
                    Specificity::new(0, 1, 2),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
                selector_conformance_decl(
                    "class",
                    "blue",
                    ".active",
                    Specificity::new(0, 1, 0),
                    SpecificityExactnessV0::Exact,
                    2,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("complex".to_string()),
        },
        inexact_selector_specificity_case(
            "unknown-functional-class-argument-stays-unranked",
            ":is(:unknown(.a), .b)",
            Specificity::new(0, 1, 0),
        ),
        inexact_selector_specificity_case(
            "unknown-functional-token-argument-stays-unranked",
            ":is(:unknown(2), .b)",
            Specificity::new(0, 1, 0),
        ),
        inexact_selector_specificity_case(
            "forgiving-garbage-argument-stays-unranked",
            ":is(#it/typo, .ok)",
            Specificity::new(0, 1, 0),
        ),
        SelectorSpecificitySeedCase {
            name: "equal-exact-specificity-uses-source-order".to_string(),
            property: "color",
            declarations: vec![
                // L4: :not(.a) and .b are both exact (0,1,0); source order breaks the tie.
                selector_conformance_decl(
                    "earlier",
                    "red",
                    ":not(.a)",
                    Specificity::new(0, 1, 0),
                    SpecificityExactnessV0::Exact,
                    1,
                ),
                selector_conformance_decl(
                    "later",
                    "blue",
                    ".b",
                    Specificity::new(0, 1, 0),
                    SpecificityExactnessV0::Exact,
                    2,
                ),
            ],
            expected_outcome: "definite",
            expected_winner_id: Some("later".to_string()),
        },
    ]
}

fn inexact_selector_specificity_case(
    name: &str,
    selector: &str,
    expected_lower_bound: Specificity,
) -> SelectorSpecificitySeedCase {
    SelectorSpecificitySeedCase {
        name: name.to_string(),
        property: "color",
        declarations: vec![
            // An unmodeled forgiving-list branch makes the numeric result a lower bound.
            selector_conformance_decl(
                "inexact",
                "red",
                selector,
                expected_lower_bound,
                SpecificityExactnessV0::Inexact,
                1,
            ),
            selector_conformance_decl(
                "exact",
                "blue",
                ".b",
                Specificity::new(0, 1, 0),
                SpecificityExactnessV0::Exact,
                2,
            ),
        ],
        expected_outcome: "rankedSet",
        expected_winner_id: None,
    }
}

fn cascade_ordering_axis_self_check_cases() -> Vec<CascadeConformanceSeedCase> {
    let levels = [
        CascadeLevel::UserAgentNormal,
        CascadeLevel::UserNormal,
        CascadeLevel::AuthorNormal,
        CascadeLevel::InlineNormal,
        CascadeLevel::Animation,
        CascadeLevel::InlineImportant,
        CascadeLevel::AuthorImportant,
        CascadeLevel::UserImportant,
        CascadeLevel::UserAgentImportant,
        CascadeLevel::Transition,
    ];
    let specificities = [
        Specificity::new(0, 0, 1),
        Specificity::new(0, 1, 0),
        Specificity::new(1, 0, 0),
    ];

    let mut cases = Vec::new();

    for left in levels {
        for right in levels {
            if left == right {
                continue;
            }

            let winner = if left > right { "left" } else { "right" };
            cases.push(CascadeConformanceSeedCase {
                name: format!("self-check-origin-importance-order-{left:?}-vs-{right:?}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(left, 0, 0, Specificity::new(0, 1, 0), 1),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(right, 0, 0, Specificity::new(0, 1, 0), 2),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for layer_left in 0..=6 {
        for layer_right in 0..=6 {
            if layer_left == layer_right {
                continue;
            }

            let winner = if layer_left > layer_right {
                "left"
            } else {
                "right"
            };
            cases.push(CascadeConformanceSeedCase {
                name: format!("self-check-layer-order-{layer_left}-vs-{layer_right}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            layer_left,
                            0,
                            Specificity::new(0, 1, 0),
                            2,
                        ),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            layer_right,
                            0,
                            Specificity::new(1, 0, 0),
                            1,
                        ),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for scope_left in 0..=7 {
        for scope_right in 0..=7 {
            if scope_left == scope_right {
                continue;
            }

            let winner = if scope_left < scope_right {
                "left"
            } else {
                "right"
            };
            cases.push(CascadeConformanceSeedCase {
                name: format!("self-check-scope-proximity-{scope_left}-vs-{scope_right}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            scope_left,
                            Specificity::new(0, 1, 0),
                            2,
                        ),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            scope_right,
                            Specificity::new(0, 1, 0),
                            1,
                        ),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for left in specificities {
        for right in specificities {
            if left == right {
                continue;
            }

            let winner = if left > right { "left" } else { "right" };
            cases.push(CascadeConformanceSeedCase {
                name: format!("self-check-specificity-order-{left:?}-vs-{right:?}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(CascadeLevel::AuthorNormal, 0, 0, left, 1),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(CascadeLevel::AuthorNormal, 0, 0, right, 2),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    for source_left in 0..=15 {
        for source_right in 0..=15 {
            if source_left == source_right {
                continue;
            }

            let winner = if source_left > source_right {
                "left"
            } else {
                "right"
            };
            cases.push(CascadeConformanceSeedCase {
                name: format!("self-check-source-order-{source_left}-vs-{source_right}"),
                property: "color",
                declarations: vec![
                    conformance_decl(
                        "left",
                        "color",
                        "red",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            0,
                            Specificity::new(0, 1, 0),
                            source_left,
                        ),
                    ),
                    conformance_decl(
                        "right",
                        "color",
                        "blue",
                        conformance_key(
                            CascadeLevel::AuthorNormal,
                            0,
                            0,
                            Specificity::new(0, 1, 0),
                            source_right,
                        ),
                    ),
                ],
                expected_outcome: "definite",
                expected_winner_id: Some(winner.to_string()),
            });
        }
    }

    cases
}

fn conformance_key(
    level: CascadeLevel,
    layer_rank: i32,
    scope_proximity: u32,
    specificity: Specificity,
    source_order: u32,
) -> CascadeKey {
    let Some(layer_ordinal) = LayerOrdinal::new(layer_rank) else {
        unreachable!("the conformance corpus only emits sentinel-safe layer ordinals");
    };
    CascadeKey::new(
        level,
        normalized_layer_rank(false, Some(layer_ordinal)),
        scope_proximity,
        specificity,
        source_order,
    )
}

fn conformance_layer_key(
    level: CascadeLevel,
    important: bool,
    layer_ordinal: Option<i32>,
    scope_proximity: u32,
    specificity: Specificity,
    source_order: u32,
) -> CascadeKey {
    let layer_ordinal = layer_ordinal.map(|ordinal| {
        let Some(ordinal) = LayerOrdinal::new(ordinal) else {
            unreachable!("the conformance corpus only emits sentinel-safe layer ordinals");
        };
        ordinal
    });
    CascadeKey::new(
        level,
        normalized_layer_rank(important, layer_ordinal),
        scope_proximity,
        specificity,
        source_order,
    )
}

fn conformance_decl(id: &str, property: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: property.to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key,
        open_world_tie_evidence: OpenWorldTieEvidence::NONE,
        specificity_exactness: SpecificityExactnessV0::Exact,
    }
}

fn selector_conformance_decl(
    id: &str,
    value: &str,
    selector: &str,
    expected_specificity: Specificity,
    expected_exactness: SpecificityExactnessV0,
    source_order: u32,
) -> SelectorSpecificitySeedDeclaration {
    SelectorSpecificitySeedDeclaration {
        id: id.to_string(),
        value: value.to_string(),
        selector: selector.to_string(),
        expected_specificity,
        expected_exactness,
        source_order,
    }
}
