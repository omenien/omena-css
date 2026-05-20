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
mod tests {
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
        } = outcome
        {
            assert_eq!(winner.id, "later");
            assert_eq!(proof.declaration_id, "later");
            assert_eq!(also_considered.len(), 1);
        }
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
        assert!(proof.cascade_safe_witness.contains("canonical order"));

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
    fn reports_selector_context_witness_rank() {
        let root = selector_context_witness(&[":root".to_string()], &[".button".to_string()]);
        assert_eq!(root.kind, SelectorContextMatchKind::Root);
        assert!(root.matched);
        assert_eq!(root.rank, 1);

        let exact = selector_context_witness(&[".button".to_string()], &[".button".to_string()]);
        assert_eq!(exact.kind, SelectorContextMatchKind::Exact);
        assert_eq!(exact.rank, 2);

        let descendant =
            selector_context_witness(&[".theme".to_string()], &[".theme .button".to_string()]);
        assert_eq!(descendant.kind, SelectorContextMatchKind::ContainsSelector);
        assert_eq!(
            descendant.reference_selector.as_deref(),
            Some(".theme .button")
        );

        let miss = selector_context_witness(&[".card".to_string()], &[".button".to_string()]);
        assert_eq!(miss.kind, SelectorContextMatchKind::NoMatch);
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
}
