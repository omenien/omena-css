use omena_cascade::{
    CascadeKey, CascadeLevel, FirstWitnessManagerConfigV0, FirstWitnessManagerV0,
    GuardedCascadeCandidateV0, GuardedCascadeConditionAtomV0, GuardedCascadeFragmentRefusalV0,
    GuardedCascadeFragmentV0, GuardedCascadeSpecificityExactnessV0, LayerOrdinal, Specificity,
    VariableOrderRegistrationV0, build_guarded_cascade_winner_v0,
    evaluate_guarded_cascade_winner_v0, normalized_layer_rank, same_canonical_winner_function_v0,
};
use omena_syntax::ident::AuthoredPropertyTextV0;
use serde::Serialize;

fn candidate(
    declaration_id: u32,
    cascade_key: u32,
    conditions: Vec<GuardedCascadeConditionAtomV0>,
) -> GuardedCascadeCandidateV0<u32> {
    GuardedCascadeCandidateV0::new(
        declaration_id,
        "button.primary",
        AuthoredPropertyTextV0::new("color"),
        cascade_key,
        GuardedCascadeSpecificityExactnessV0::Exact,
        0,
        conditions,
    )
}

#[derive(Clone, Copy)]
enum SyntheticGuardFamily {
    IndependentRandom,
    LaminarNested,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntheticGuardCorpus {
    declared_synthetic: bool,
    family: &'static str,
    seed: u64,
    fragment: GuardedCascadeFragmentV0<u32>,
}

fn next_seed(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn synthetic_guard_corpus(
    seed: u64,
    family: SyntheticGuardFamily,
) -> Result<SyntheticGuardCorpus, GuardedCascadeFragmentRefusalV0> {
    const VARIABLE_COUNT: usize = 6;
    const DECLARATION_COUNT: usize = 18;
    let alphabet = (0..VARIABLE_COUNT)
        .map(|index| format!("condition-{index}"))
        .collect::<Vec<_>>();
    let mut state = seed;
    let candidates = (0..DECLARATION_COUNT)
        .map(|index| {
            let guard_indices = match family {
                SyntheticGuardFamily::IndependentRandom => {
                    let bits = next_seed(&mut state);
                    (0..VARIABLE_COUNT)
                        .filter(|variable| bits & (1_u64 << variable) != 0)
                        .collect::<Vec<_>>()
                }
                SyntheticGuardFamily::LaminarNested => {
                    let depth = 1 + next_seed(&mut state) as usize % VARIABLE_COUNT;
                    (0..depth).collect::<Vec<_>>()
                }
            };
            let conditions = guard_indices
                .into_iter()
                .map(|variable| {
                    GuardedCascadeConditionAtomV0::media(
                        alphabet[variable].clone(),
                        [variable as u32],
                        false,
                    )
                })
                .collect();
            candidate(
                (index + 1) as u32,
                (DECLARATION_COUNT - index) as u32,
                conditions,
            )
        })
        .collect::<Vec<_>>();
    Ok(SyntheticGuardCorpus {
        declared_synthetic: true,
        family: match family {
            SyntheticGuardFamily::IndependentRandom => "independentRandom",
            SyntheticGuardFamily::LaminarNested => "laminarNested",
        },
        seed,
        fragment: GuardedCascadeFragmentV0::admit(alphabet, candidates)?,
    })
}

#[test]
fn guarded_winner_matches_the_first_applicable_declaration_at_every_assignment()
-> Result<(), Box<dyn std::error::Error>> {
    let candidates = vec![
        candidate(
            30,
            30,
            vec![GuardedCascadeConditionAtomV0::media(
                "min-width:40rem",
                [0],
                true,
            )],
        ),
        candidate(
            20,
            20,
            vec![GuardedCascadeConditionAtomV0::supports(
                "display:grid",
                [1],
                false,
            )],
        ),
        candidate(10, 10, Vec::new()),
    ];
    let fragment =
        GuardedCascadeFragmentV0::admit(["min-width:40rem", "display:grid"], candidates)?;
    let mut manager = FirstWitnessManagerV0::new(
        VariableOrderRegistrationV0::site_first_appearance(["min-width:40rem", "display:grid"])?,
        FirstWitnessManagerConfigV0 {
            shortcuts: false,
            ..FirstWitnessManagerConfigV0::default()
        },
    );
    let root = build_guarded_cascade_winner_v0(&mut manager, &fragment)?;

    let cases = [
        ([false, false], Some(10)),
        ([true, false], Some(30)),
        ([false, true], Some(20)),
        ([true, true], Some(30)),
    ];
    for (assignment, expected) in cases {
        assert_eq!(
            evaluate_guarded_cascade_winner_v0(&manager, root, &assignment)?,
            expected,
            "the diagram must agree with a linear first-applicable scan"
        );
    }

    let rebuilt = build_guarded_cascade_winner_v0(&mut manager, &fragment)?;
    assert!(same_canonical_winner_function_v0(root, rebuilt));
    assert_eq!(root.node_id(), rebuilt.node_id());
    Ok(())
}

#[test]
fn cascade_key_input_orders_build_one_canonical_winner_root()
-> Result<(), Box<dyn std::error::Error>> {
    let candidate = |declaration_id, source_order, condition: &str| {
        GuardedCascadeCandidateV0::new(
            declaration_id,
            "button.primary",
            AuthoredPropertyTextV0::new("color"),
            CascadeKey::new(
                CascadeLevel::AuthorNormal,
                normalized_layer_rank(false, LayerOrdinal::new(0)),
                0,
                Specificity::new(0, 1, 0),
                source_order,
            ),
            GuardedCascadeSpecificityExactnessV0::Exact,
            0,
            vec![GuardedCascadeConditionAtomV0::media(
                condition,
                [source_order],
                false,
            )],
        )
    };
    let first = candidate(10, 1, "condition-a");
    let second = candidate(20, 2, "condition-b");
    assert_ne!(first.cascade_key(), second.cascade_key());
    assert_ne!(
        first.cascade_key().cmp(second.cascade_key()),
        std::cmp::Ordering::Equal
    );

    let forward = GuardedCascadeFragmentV0::admit(
        ["condition-a", "condition-b"],
        [first.clone(), second.clone()],
    )?;
    let reverse = GuardedCascadeFragmentV0::admit(["condition-a", "condition-b"], [second, first])?;
    assert_eq!(forward.candidates(), reverse.candidates());

    let build = |fragment: &GuardedCascadeFragmentV0<CascadeKey>| {
        let mut manager = FirstWitnessManagerV0::new(
            VariableOrderRegistrationV0::site_first_appearance(["condition-a", "condition-b"])?,
            FirstWitnessManagerConfigV0 {
                shortcuts: false,
                ..FirstWitnessManagerConfigV0::default()
            },
        );
        let root = build_guarded_cascade_winner_v0(&mut manager, fragment)?;
        Ok::<_, omena_cascade::FirstWitnessErrorV0>((root, manager))
    };
    let (forward_root, forward_manager) = build(&forward)?;
    let (reverse_root, reverse_manager) = build(&reverse)?;
    assert_eq!(forward_root.node_id(), reverse_root.node_id());
    for assignment in [[false, false], [true, false], [false, true], [true, true]] {
        assert_eq!(
            evaluate_guarded_cascade_winner_v0(&forward_manager, forward_root, &assignment,)?,
            evaluate_guarded_cascade_winner_v0(&reverse_manager, reverse_root, &assignment,)?,
        );
    }
    eprintln!(
        "CASCADE_KEY_INPUT_ORDER={{\"forwardRoot\":{},\"reverseRoot\":{},\"assignmentCount\":4}}",
        forward_root.node_id(),
        reverse_root.node_id(),
    );
    Ok(())
}

#[test]
fn fragment_admission_returns_closed_reasons_for_unsupported_candidates() {
    let inexact = GuardedCascadeCandidateV0::new(
        1,
        "button.primary",
        AuthoredPropertyTextV0::new("color"),
        1_u32,
        GuardedCascadeSpecificityExactnessV0::Inexact,
        0,
        Vec::new(),
    );
    assert!(matches!(
        GuardedCascadeFragmentV0::admit(Vec::<String>::new(), [inexact]),
        Err(GuardedCascadeFragmentRefusalV0::InexactSpecificity { declaration_id: 1 })
    ));

    let container = candidate(
        2,
        2,
        vec![GuardedCascadeConditionAtomV0::container(
            "inline-size>30rem",
            [0],
        )],
    );
    assert!(matches!(
        GuardedCascadeFragmentV0::admit(["inline-size>30rem"], [container]),
        Err(GuardedCascadeFragmentRefusalV0::ContainerCondition {
            declaration_id: 2,
            ..
        })
    ));
}

#[test]
fn fixed_seed_guard_generators_are_byte_reproducible() -> Result<(), Box<dyn std::error::Error>> {
    let first = [
        synthetic_guard_corpus(0x5eed_cafe, SyntheticGuardFamily::IndependentRandom)?,
        synthetic_guard_corpus(0x5eed_cafe, SyntheticGuardFamily::LaminarNested)?,
    ];
    let second = [
        synthetic_guard_corpus(0x5eed_cafe, SyntheticGuardFamily::IndependentRandom)?,
        synthetic_guard_corpus(0x5eed_cafe, SyntheticGuardFamily::LaminarNested)?,
    ];
    let first = serde_json::to_string(&first)?;
    let second = serde_json::to_string(&second)?;
    assert_eq!(first, second);
    eprintln!("SYNTHETIC_BYTES={first}");
    Ok(())
}

#[test]
fn synthetic_refusal_corpus_reports_each_attempted_boundary()
-> Result<(), Box<dyn std::error::Error>> {
    let exact = |declaration_id, element: &str, property: &str, key, conditions| {
        GuardedCascadeCandidateV0::new(
            declaration_id,
            element,
            AuthoredPropertyTextV0::new(property),
            key,
            GuardedCascadeSpecificityExactnessV0::Exact,
            0,
            conditions,
        )
    };
    fn refusal<K>(
        result: Result<GuardedCascadeFragmentV0<K>, GuardedCascadeFragmentRefusalV0>,
        message: &str,
    ) -> Result<GuardedCascadeFragmentRefusalV0, std::io::Error> {
        result.err().ok_or_else(|| std::io::Error::other(message))
    }
    let refusals = vec![
        refusal(
            GuardedCascadeFragmentV0::<u32>::admit(Vec::<String>::new(), Vec::new()),
            "an empty candidate set must be outside the fragment",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [GuardedCascadeCandidateV0::new(
                    1,
                    "button",
                    AuthoredPropertyTextV0::new("color"),
                    10,
                    GuardedCascadeSpecificityExactnessV0::Inexact,
                    0,
                    Vec::new(),
                )],
            ),
            "inexact specificity must be refused",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [GuardedCascadeCandidateV0::new(
                    2,
                    "button",
                    AuthoredPropertyTextV0::new("color"),
                    10,
                    GuardedCascadeSpecificityExactnessV0::Exact,
                    1,
                    Vec::new(),
                )],
            ),
            "scope proximity must be outside the fragment",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                ["container"],
                [exact(
                    3,
                    "button",
                    "color",
                    10,
                    vec![GuardedCascadeConditionAtomV0::container("container", [0])],
                )],
            ),
            "container queries must be outside the fragment",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                [":has(.active)"],
                [exact(
                    4,
                    "button",
                    "color",
                    10,
                    vec![GuardedCascadeConditionAtomV0::structural_pseudo(
                        ":has(.active)",
                    )],
                )],
            ),
            "structural pseudo conditions must be outside the fragment",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [exact(
                    5,
                    "button",
                    "color",
                    10,
                    vec![GuardedCascadeConditionAtomV0::media(
                        "min-width:40rem",
                        [0],
                        true,
                    )],
                )],
            ),
            "numeric atoms must require declaration in the alphabet",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [exact(
                    6,
                    "button",
                    "color",
                    10,
                    vec![GuardedCascadeConditionAtomV0::supports(
                        "display:grid",
                        [0],
                        false,
                    )],
                )],
            ),
            "non-numeric atoms must require declaration in the alphabet",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [
                    exact(7, "button", "color", 20, Vec::new()),
                    exact(8, "button", "display", 10, Vec::new()),
                ],
            ),
            "one fragment must have one property",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [
                    exact(9, "button", "color", 20, Vec::new()),
                    exact(10, "a", "color", 10, Vec::new()),
                ],
            ),
            "one fragment must have one element signature",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [
                    exact(11, "button", "color", 20, Vec::new()),
                    exact(11, "button", "color", 10, Vec::new()),
                ],
            ),
            "declaration ids must be unique",
        )?,
        refusal(
            GuardedCascadeFragmentV0::admit(
                Vec::<String>::new(),
                [
                    exact(12, "button", "color", 20, Vec::new()),
                    exact(13, "button", "color", 20, Vec::new()),
                ],
            ),
            "equal cascade keys must be a typed ambiguity",
        )?,
        refusal(
            GuardedCascadeFragmentV0::<u32>::admit(
                (0..=usize::from(u16::MAX) + 1).map(|index| format!("atom-{index}")),
                [exact(14, "button", "color", 10, Vec::new())],
            ),
            "an oversized condition alphabet must be refused before diagram construction",
        )?,
    ];

    let mut counts = std::collections::BTreeMap::new();
    for refusal in refusals {
        *counts.entry(refusal.name()).or_insert(0_usize) += 1;
    }
    assert_eq!(counts.len(), 12);
    assert!(counts.values().all(|count| *count == 1));
    let serialized = serde_json::to_string(&counts)?;
    eprintln!("REFUSAL_CENSUS={serialized}");
    Ok(())
}
