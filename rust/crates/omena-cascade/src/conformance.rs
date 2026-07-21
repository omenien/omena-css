//! Conformance seed corpora for the cascade algebra.
//!
//! The cases here are intentionally small and explicit so H1 gates can prove
//! the cascade ordering and WPT-derived seed policies without claiming full WPT
//! coverage.

use crate::{
    CascadeConformanceSeedCase, CascadeConformanceSeedReport, CascadeConformanceSeedResult,
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeOutcome, CascadeValue, LayerRank,
    ModuleRank, Specificity, SpecificityExactnessV0, cascade_property,
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

pub fn run_wpt_cascade_seed_corpus() -> CascadeConformanceSeedReport {
    let results = wpt_cascade_seed_cases()
        .into_iter()
        .map(run_cascade_conformance_seed_case)
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    CascadeConformanceSeedReport {
        schema_version: "0",
        product: "omena-cascade.wpt-cascade-seed-corpus",
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
    vec![
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
            name: "layer-rank-beats-specificity-within-level".to_string(),
            property: "color",
            declarations: vec![
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
            name: "scope-proximity-beats-specificity-tie".to_string(),
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
    ]
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

fn wpt_cascade_seed_cases() -> Vec<CascadeConformanceSeedCase> {
    let levels = [
        CascadeLevel::UserAgentNormal,
        CascadeLevel::UserNormal,
        CascadeLevel::AuthorNormal,
        CascadeLevel::InlineNormal,
        CascadeLevel::Animation,
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
                name: format!("wpt-origin-importance-order-{left:?}-vs-{right:?}"),
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

    for layer_left in -3..=3 {
        for layer_right in -3..=3 {
            if layer_left == layer_right {
                continue;
            }

            let winner = if layer_left > layer_right {
                "left"
            } else {
                "right"
            };
            cases.push(CascadeConformanceSeedCase {
                name: format!("wpt-layer-order-{layer_left}-vs-{layer_right}"),
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
                name: format!("wpt-scope-proximity-{scope_left}-vs-{scope_right}"),
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
                name: format!("wpt-specificity-order-{left:?}-vs-{right:?}"),
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
                name: format!("wpt-source-order-{source_left}-vs-{source_right}"),
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
    CascadeKey::new(
        level,
        LayerRank(layer_rank),
        scope_proximity,
        specificity,
        ModuleRank::ZERO,
        source_order,
    )
}

fn conformance_decl(id: &str, property: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: property.to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key,
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
