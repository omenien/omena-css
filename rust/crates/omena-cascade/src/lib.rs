//! Cascade-formal substrate for the Omena CSS track.
//!
//! The crate starts with the load-bearing algebra from the research plan:
//! lexicographic cascade keys, specificity, provenance proofs, and a finite
//! custom-property substitution function with explicit cycle handling.

mod custom_property;
mod model;
mod proofs;
mod selector;

pub use custom_property::*;
pub use model::*;
pub use proofs::*;
pub use selector::*;

use std::cmp::Reverse;

pub fn summarize_cascade_boundary() -> CascadeBoundarySummary {
    CascadeBoundarySummary {
        product: "omena-cascade.boundary",
        ordering_model: "lexicographicCascadeKey",
        substitution_model: "finiteCustomPropertyLeastFixedPoint",
        least_fixed_point_proof_model: "finite-env monotone custom-property substitution with cycle-to-guaranteed-invalid bottoming and env-size iteration bound",
        ready_surfaces: vec![
            "cascadeKeyOrdering",
            "specificityOrdering",
            "cascadeOutcomeProof",
            "genericCascadeWinner",
            "semanticDesignTokenRanking",
            "queryReadCascadeAtPosition",
            "selectorContextWitness",
            "selectorMatchWitness",
            "cascadeConformanceSeedCorpus",
            "customPropertySubstitution",
            "customPropertyLeastFixedPoint",
            "customPropertyLeastFixedPointProof",
            "customPropertyLeastFixedPointTrace",
            "cycleToGuaranteedInvalid",
            "computedValueResolutionSeed",
            "inheritanceInitialValueSeed",
            "shorthandCombinationProof",
            "supportsStaticEvalWitness",
            "scopeFlattenProof",
            "layerFlattenProof",
            "wptCascadeSeedCorpus",
        ],
        not_ready_surfaces: vec!["fullInitialValueTable", "fullWptCascadeCorpus"],
    }
}

pub fn run_cascade_conformance_seed_corpus() -> CascadeConformanceSeedReport {
    let results = cascade_conformance_seed_cases()
        .into_iter()
        .map(run_cascade_conformance_seed_case)
        .collect::<Vec<_>>();
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
        source_order,
    )
}

fn conformance_decl(id: &str, property: &str, value: &str, key: CascadeKey) -> CascadeDeclaration {
    CascadeDeclaration {
        id: id.to_string(),
        property: property.to_string(),
        value: CascadeValue::Literal(value.to_string()),
        key,
    }
}

pub fn cascade_property(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let mut matching: Vec<CascadeDeclaration> = declarations
        .into_iter()
        .filter(|declaration| declaration.property == property)
        .collect();

    if matching.is_empty() {
        return CascadeOutcome::Inherit;
    }

    matching.sort_by_key(|declaration| std::cmp::Reverse(declaration.key));
    let winner = matching.remove(0);
    let proof = CascadeProof::from_declaration(&winner);
    CascadeOutcome::Definite {
        winner,
        proof,
        also_considered: matching,
    }
}

pub fn compute_cascade_computed_value(
    input: CascadeComputedValueInputV0,
) -> CascadeComputedValueResultV0 {
    let property = input.property.clone();
    let outcome = cascade_property(input.declarations, &property);
    let (winner_declaration_id, cascaded_value, mut derivation_steps) = match outcome {
        CascadeOutcome::Definite { winner, .. } => (
            Some(winner.id),
            winner.value,
            vec!["cascadeWinnerSelected", "computedValueResolutionStarted"],
        ),
        CascadeOutcome::Inherit => (
            None,
            if property_is_inherited(&property) {
                CascadeValue::Inherit
            } else {
                CascadeValue::Initial
            },
            vec!["noCascadeWinner", "inheritanceOrInitialSelected"],
        ),
        CascadeOutcome::RankedSet(_) | CascadeOutcome::Top => {
            return CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::InvalidAtComputedValueTime,
                value: CascadeValue::GuaranteedInvalid,
                winner_declaration_id: None,
                inherited: false,
                used_initial_value: false,
                invalid_at_computed_value_time: true,
                derivation_steps: vec!["cascadeOutcomeIndeterminate"],
            };
        }
    };

    let substituted_value =
        substitute_custom_properties(&cascaded_value, &input.custom_property_env);
    if substituted_value == CascadeValue::GuaranteedInvalid {
        derivation_steps.push("substitutionProducedGuaranteedInvalid");
        derivation_steps.push("invalidAtComputedValueTimeFallsBackAsUnset");
        return computed_value_from_unset(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            true,
            derivation_steps,
        );
    }

    match substituted_value {
        CascadeValue::Unset => computed_value_from_unset(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            false,
            {
                derivation_steps.push("unsetKeywordResolved");
                derivation_steps
            },
        ),
        CascadeValue::Inherit => computed_value_from_inherit(
            property,
            winner_declaration_id,
            input.parent_computed_value,
            {
                derivation_steps.push("inheritKeywordResolved");
                derivation_steps
            },
        ),
        CascadeValue::Initial => computed_value_from_initial(property, winner_declaration_id, {
            derivation_steps.push("initialKeywordResolved");
            derivation_steps
        }),
        value => {
            derivation_steps.push("computedValueResolved");
            CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::Resolved,
                value,
                winner_declaration_id,
                inherited: false,
                used_initial_value: false,
                invalid_at_computed_value_time: false,
                derivation_steps,
            }
        }
    }
}

fn computed_value_from_unset(
    property: String,
    winner_declaration_id: Option<String>,
    parent_computed_value: Option<CascadeValue>,
    invalid_at_computed_value_time: bool,
    mut derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    if property_is_inherited(&property) {
        derivation_steps.push("unsetForInheritedPropertyUsesInheritance");
        return computed_value_from_inherit(
            property,
            winner_declaration_id,
            parent_computed_value,
            derivation_steps,
        )
        .with_invalid_at_computed_value_time(invalid_at_computed_value_time);
    }

    derivation_steps.push("unsetForNonInheritedPropertyUsesInitial");
    computed_value_from_initial(property, winner_declaration_id, derivation_steps)
        .with_invalid_at_computed_value_time(invalid_at_computed_value_time)
}

fn computed_value_from_inherit(
    property: String,
    winner_declaration_id: Option<String>,
    parent_computed_value: Option<CascadeValue>,
    mut derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    match parent_computed_value {
        Some(value) => {
            derivation_steps.push("parentComputedValueUsed");
            CascadeComputedValueResultV0 {
                schema_version: "0",
                product: "omena-cascade.computed-value",
                property,
                status: ComputedCascadeValueStatusV0::Inherited,
                value,
                winner_declaration_id,
                inherited: true,
                used_initial_value: false,
                invalid_at_computed_value_time: false,
                derivation_steps,
            }
        }
        None => {
            derivation_steps.push("missingParentFallsBackToInitial");
            computed_value_from_initial(property, winner_declaration_id, derivation_steps)
        }
    }
}

fn computed_value_from_initial(
    property: String,
    winner_declaration_id: Option<String>,
    mut derivation_steps: Vec<&'static str>,
) -> CascadeComputedValueResultV0 {
    derivation_steps.push("initialValueTableConsulted");
    CascadeComputedValueResultV0 {
        schema_version: "0",
        product: "omena-cascade.computed-value",
        value: initial_cascade_value_for_property(&property),
        property,
        status: ComputedCascadeValueStatusV0::Initial,
        winner_declaration_id,
        inherited: false,
        used_initial_value: true,
        invalid_at_computed_value_time: false,
        derivation_steps,
    }
}

impl CascadeComputedValueResultV0 {
    fn with_invalid_at_computed_value_time(mut self, invalid_at_computed_value_time: bool) -> Self {
        if invalid_at_computed_value_time {
            self.status = ComputedCascadeValueStatusV0::InvalidAtComputedValueTime;
            self.invalid_at_computed_value_time = true;
        }
        self
    }
}

fn property_is_inherited(property: &str) -> bool {
    property.starts_with("--")
        || matches!(
            property,
            "color"
                | "cursor"
                | "direction"
                | "font"
                | "font-family"
                | "font-size"
                | "font-style"
                | "font-variant"
                | "font-weight"
                | "letter-spacing"
                | "line-height"
                | "text-align"
                | "text-indent"
                | "text-transform"
                | "visibility"
                | "white-space"
                | "word-spacing"
        )
}

fn initial_cascade_value_for_property(property: &str) -> CascadeValue {
    if property.starts_with("--") {
        return CascadeValue::GuaranteedInvalid;
    }

    let value = match property {
        "background-color" | "border-color" | "caret-color" | "outline-color" => "transparent",
        "border-style" | "display" => "none",
        "border-width" | "margin" | "padding" => "0",
        "box-shadow" | "text-shadow" => "none",
        "color" => "canvastext",
        "cursor" => "auto",
        "font-family" => "serif",
        "font-size" => "medium",
        "font-style" | "font-variant" | "font-weight" => "normal",
        "letter-spacing" | "line-height" | "word-spacing" => "normal",
        "opacity" => "1",
        "text-align" => "start",
        "text-indent" => "0",
        "text-transform" => "none",
        "visibility" => "visible",
        "white-space" => "normal",
        _ => "initial",
    };
    CascadeValue::Literal(value.to_string())
}

pub fn run_cascade_evaluation_fuzz_case(
    case: CascadeEvaluationFuzzCaseV0,
) -> CascadeEvaluationFuzzResultV0 {
    let declaration_count = case.declaration_count.clamp(1, 64);
    let declarations = generated_cascade_fuzz_declarations(case.seed, declaration_count);
    let matching = declarations
        .iter()
        .filter(|declaration| declaration.property == "color")
        .cloned()
        .collect::<Vec<_>>();
    let expected_winner_id = rank_cascade_items(matching.clone(), |declaration| declaration.key)
        .first()
        .map(|declaration| declaration.id.clone());
    let actual = cascade_property(declarations, "color");
    let actual_winner_id = match actual {
        CascadeOutcome::Definite { winner, .. } => Some(winner.id),
        _ => None,
    };
    let ranked_count = matching.len();
    let passed = actual_winner_id == expected_winner_id && ranked_count > 0;

    CascadeEvaluationFuzzResultV0 {
        seed: case.seed,
        declaration_count,
        actual_winner_id,
        expected_winner_id,
        ranked_count,
        passed,
    }
}

pub fn run_var_substitution_fuzz_case(
    case: VarSubstitutionFuzzCaseV0,
) -> VarSubstitutionFuzzResultV0 {
    let chain_len = case.chain_len.clamp(1, 32);
    let mut env = CustomPropertyEnv::new();
    let terminal = CascadeValue::Literal(format!("seed-{}", case.seed));

    for index in 0..chain_len {
        let name = fuzz_var_name(index);
        let next_value = if index == 0 && !case.cycle {
            terminal.clone()
        } else if index == 0 {
            CascadeValue::Var {
                name: fuzz_var_name(chain_len - 1),
                fallback: Some(Box::new(CascadeValue::Literal(
                    "cycle-fallback".to_string(),
                ))),
            }
        } else {
            CascadeValue::Var {
                name: fuzz_var_name(index - 1),
                fallback: None,
            }
        };
        env.insert(name, next_value);
    }

    let input = CascadeValue::Var {
        name: fuzz_var_name(chain_len - 1),
        fallback: Some(Box::new(CascadeValue::Literal(
            "outer-fallback".to_string(),
        ))),
    };
    let result = substitute_custom_properties(&input, &env);
    let expected = if case.cycle {
        CascadeValue::Literal("outer-fallback".to_string())
    } else {
        terminal
    };
    let passed = result == expected;

    VarSubstitutionFuzzResultV0 {
        seed: case.seed,
        chain_len,
        cycle: case.cycle,
        result,
        expected,
        passed,
    }
}

pub fn run_cascade_fuzz_seed_corpus() -> CascadeFuzzSeedReportV0 {
    let seeds = [0, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144];
    let cascade_results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_cascade_evaluation_fuzz_case(CascadeEvaluationFuzzCaseV0 {
                seed,
                declaration_count: index + 1,
            })
        })
        .collect::<Vec<_>>();
    let var_results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_var_substitution_fuzz_case(VarSubstitutionFuzzCaseV0 {
                seed,
                chain_len: index + 1,
                cycle: index % 3 == 0,
            })
        })
        .collect::<Vec<_>>();
    let passed_count = cascade_results
        .iter()
        .filter(|result| result.passed)
        .count()
        + var_results.iter().filter(|result| result.passed).count();
    let case_count = cascade_results.len() + var_results.len();

    CascadeFuzzSeedReportV0 {
        schema_version: "0",
        product: "omena-cascade.fuzz-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count - passed_count,
        cascade_results,
        var_results,
    }
}

pub fn rank_cascade_items<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Vec<T> {
    let mut ranked = items.into_iter().collect::<Vec<_>>();
    ranked.sort_by_key(|item| Reverse(key_for(item)));
    ranked
}

pub fn select_cascade_winner<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Option<(T, Vec<T>)> {
    let mut ranked = rank_cascade_items(items, key_for);
    if ranked.is_empty() {
        return None;
    }

    let winner = ranked.remove(0);
    Some((winner, ranked))
}

fn generated_cascade_fuzz_declarations(
    seed: u64,
    declaration_count: usize,
) -> Vec<CascadeDeclaration> {
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    (0..declaration_count)
        .map(|index| {
            let property = if index == 0 || fuzz_next(&mut state).is_multiple_of(3) {
                "color"
            } else {
                "margin"
            };
            CascadeDeclaration {
                id: format!("decl-{seed}-{index}"),
                property: property.to_string(),
                value: CascadeValue::Literal(format!("v{}", fuzz_next(&mut state) % 17)),
                key: CascadeKey::new(
                    fuzz_cascade_level(fuzz_next(&mut state)),
                    LayerRank((fuzz_next(&mut state) % 9) as i32 - 4),
                    (fuzz_next(&mut state) % 12) as u32,
                    Specificity::new(
                        (fuzz_next(&mut state) % 4) as u32,
                        (fuzz_next(&mut state) % 8) as u32,
                        (fuzz_next(&mut state) % 12) as u32,
                    ),
                    index as u32,
                ),
            }
        })
        .collect()
}

fn fuzz_cascade_level(value: u64) -> CascadeLevel {
    match value % 9 {
        0 => CascadeLevel::UserAgentNormal,
        1 => CascadeLevel::UserNormal,
        2 => CascadeLevel::AuthorNormal,
        3 => CascadeLevel::InlineNormal,
        4 => CascadeLevel::Animation,
        5 => CascadeLevel::AuthorImportant,
        6 => CascadeLevel::UserImportant,
        7 => CascadeLevel::UserAgentImportant,
        _ => CascadeLevel::Transition,
    }
}

fn fuzz_var_name(index: usize) -> String {
    format!("--fuzz-{index}")
}

fn fuzz_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

#[cfg(test)]
mod tests;
