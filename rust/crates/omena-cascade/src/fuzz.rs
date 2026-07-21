//! Deterministic fuzz harness entry points for cascade invariants.
//!
//! The public functions convert compact seed cases into repeatable cascade
//! ranking and custom-property substitution checks used by cargo-fuzz smoke
//! gates and the H1 readiness bundle.

use crate::{
    CascadeDeclaration, CascadeEvaluationFuzzCaseV0, CascadeEvaluationFuzzResultV0,
    CascadeFuzzSeedReportV0, CascadeKey, CascadeLevel, CascadeOutcome, CascadeValue,
    CustomPropertyEnv, LayerRank, ModuleRank, Specificity, VarSubstitutionFuzzCaseV0,
    VarSubstitutionFuzzResultV0, cascade_property, rank_cascade_items,
    substitute_custom_properties,
};

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
                    ModuleRank::new(
                        (fuzz_next(&mut state) % 5) as u32,
                        (fuzz_next(&mut state) % 7) as u32,
                        (fuzz_next(&mut state) % 11) as u32,
                    ),
                    index as u32,
                ),
                specificity_exactness: crate::SpecificityExactnessV0::Exact,
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
