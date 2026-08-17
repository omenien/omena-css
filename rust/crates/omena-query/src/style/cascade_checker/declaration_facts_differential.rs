use omena_query_checker_orchestrator::{
    OmenaCheckerCascadeInputV0,
    run_omena_query_checker_cascade_gate_with_standard_property_value_verdicts_v0,
    standard_property_value_verdicts_v0,
};
use omena_syntax::StyleDialect;

use crate::{
    OmenaQueryTransformExecutionContextV0,
    execute_omena_query_consumer_build_style_source_with_context,
    semantic_omena_query_minify_build_profile,
};

use super::input::{
    QueryCheckerCascadeDeclaration, collect_query_checker_cascade_declarations_from_facts,
    collect_query_checker_cascade_declarations_from_scanner,
};
use super::summarize_query_cascade_checker_diagnostics_with_deep_analysis;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateSignature {
    declaration_id: String,
    selector: String,
    property: String,
    value: String,
    source_order: u32,
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
    important: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EvaluationSignature {
    rule_code_name: &'static str,
    declaration_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EmissionCandidateSignature {
    selector: String,
    property: String,
    value: String,
    important: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DifferentialDisposition {
    Agreement,
    ScannerWrong,
    FactsWrong,
}

#[derive(Debug, Clone, Copy)]
struct DifferentialCase {
    name: &'static str,
    path: &'static str,
    source: &'static str,
    dialect: StyleDialect,
    expected: DifferentialDisposition,
}

fn candidate_signatures(
    declarations: &[QueryCheckerCascadeDeclaration],
) -> Vec<CandidateSignature> {
    declarations
        .iter()
        .map(|declaration| CandidateSignature {
            declaration_id: declaration.input.declaration_id.clone(),
            selector: declaration.input.selector.as_str().to_string(),
            property: declaration.input.property.clone(),
            value: declaration.input.value.clone(),
            source_order: declaration.input.source_order,
            condition_context: declaration.input.condition_context.clone(),
            layer_name: declaration.input.layer_name.clone(),
            layer_order: declaration.input.layer_order,
            important: declaration.input.important,
        })
        .collect()
}

fn evaluation_signatures(
    declarations: &[QueryCheckerCascadeDeclaration],
) -> Vec<EvaluationSignature> {
    let declarations = declarations
        .iter()
        .map(|declaration| declaration.input.clone())
        .collect::<Vec<_>>();
    let verdicts = standard_property_value_verdicts_v0(declarations.as_slice());
    let mut evaluations =
        run_omena_query_checker_cascade_gate_with_standard_property_value_verdicts_v0(
            OmenaCheckerCascadeInputV0 {
                declarations,
                custom_properties: Vec::new(),
                custom_property_registrations: Vec::new(),
            },
            &verdicts,
        )
        .evaluations
        .into_iter()
        .map(|evaluation| EvaluationSignature {
            rule_code_name: evaluation.rule_code_name,
            declaration_ids: evaluation.declaration_ids,
        })
        .collect::<Vec<_>>();
    evaluations.sort();
    evaluations
}

fn emission_candidate_signature(
    declaration: &QueryCheckerCascadeDeclaration,
) -> EmissionCandidateSignature {
    EmissionCandidateSignature {
        selector: declaration.input.selector.as_str().to_string(),
        property: declaration.input.property.clone(),
        value: declaration.input.value.clone(),
        important: declaration.input.important,
    }
}

fn facts_match_semantic_emission(
    case: DifferentialCase,
    facts: &[QueryCheckerCascadeDeclaration],
) -> Result<bool, String> {
    let evaluations = evaluation_signatures(facts);
    let Some(losing_declaration_id) = evaluations
        .iter()
        .find(|evaluation| evaluation.rule_code_name == "unreachable-declaration")
        .and_then(|evaluation| evaluation.declaration_ids.first())
    else {
        return Ok(false);
    };
    let expected_winners = facts
        .iter()
        .filter(|declaration| declaration.input.declaration_id != *losing_declaration_id)
        .map(emission_candidate_signature)
        .collect::<Vec<_>>();
    if expected_winners.is_empty() {
        return Ok(false);
    }

    let profile = semantic_omena_query_minify_build_profile();
    let pass_ids = profile
        .pass_ids
        .iter()
        .map(|pass_id| (*pass_id).to_string())
        .collect::<Vec<_>>();
    let emission = execute_omena_query_consumer_build_style_source_with_context(
        case.path,
        case.source,
        pass_ids.as_slice(),
        &OmenaQueryTransformExecutionContextV0::default(),
    )
    .execution
    .output_css;
    let emitted_facts = collect_query_checker_cascade_declarations_from_facts(
        case.path,
        emission.as_str(),
        case.dialect,
    );
    if evaluation_signatures(facts) == evaluation_signatures(&emitted_facts) {
        return Ok(true);
    }
    let emitted = emitted_facts
        .iter()
        .map(emission_candidate_signature)
        .collect::<Vec<_>>();
    Ok(expected_winners
        .iter()
        .all(|winner| emitted.contains(winner)))
}

fn classify_against_scanner_and_emission(
    case: DifferentialCase,
    scanner: &[QueryCheckerCascadeDeclaration],
    facts: &[QueryCheckerCascadeDeclaration],
) -> Result<DifferentialDisposition, String> {
    if candidate_signatures(scanner) == candidate_signatures(facts)
        && evaluation_signatures(scanner) == evaluation_signatures(facts)
    {
        Ok(DifferentialDisposition::Agreement)
    } else if facts_match_semantic_emission(case, facts)? {
        Ok(DifferentialDisposition::ScannerWrong)
    } else {
        Ok(DifferentialDisposition::FactsWrong)
    }
}

fn facts_for(case: DifferentialCase) -> Vec<QueryCheckerCascadeDeclaration> {
    let mut facts =
        collect_query_checker_cascade_declarations_from_facts(case.path, case.source, case.dialect);
    if std::env::var_os("OMENA_QUERY_CASCADE_FACTS_INJECT_MISJOIN").is_some()
        && case.name == "real-button-variants"
        && let Some(first) = facts.first_mut()
    {
        first.input.selector = omena_query_checker_orchestrator::CanonicalSelector::from_canonical(
            ".injected-wrong-owner",
        );
    }
    facts
}

fn scanner_for(case: DifferentialCase) -> Vec<QueryCheckerCascadeDeclaration> {
    collect_query_checker_cascade_declarations_from_scanner(case.source, case.dialect)
}

fn cascade_codes(source: &str) -> Vec<&'static str> {
    summarize_query_cascade_checker_diagnostics_with_deep_analysis("fixture.scss", source, false)
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .filter(|code| matches!(*code, "unreachableDeclaration" | "unspecifiedCascadeTie"))
        .collect()
}

#[test]
fn production_cst_input_keeps_url_winners_and_wire_ids() -> Result<(), String> {
    for (path, source) in [
        (
            "url-semicolon.css",
            include_str!("../../tests/fixtures/cascade-input-authority/url-semicolon.css"),
        ),
        (
            "url-brace.css",
            include_str!("../../tests/fixtures/cascade-input-authority/url-brace.css"),
        ),
    ] {
        let diagnostics =
            summarize_query_cascade_checker_diagnostics_with_deep_analysis(path, source, false);
        let unreachable = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "unreachableDeclaration")
            .ok_or_else(|| format!("{path} did not produce the expected cascade diagnostic"))?;
        assert_eq!(unreachable.range.start.line, 2, "{path}: {unreachable:#?}");
        let narrowing = unreachable
            .cascade_narrowing
            .as_ref()
            .ok_or_else(|| format!("{path} did not produce cascade narrowing"))?;
        assert_eq!(
            serde_json::to_string(&narrowing.declaration_ids).map_err(|error| error.to_string())?,
            r#"["decl-0","decl-1"]"#
        );
        let runtime = narrowing
            .runtime_state
            .as_ref()
            .ok_or_else(|| format!("{path} did not produce runtime evidence"))?;
        assert_eq!(
            runtime.scenarios[0].winner_declaration_id.as_deref(),
            Some("decl-0")
        );
        assert!(
            runtime.scenarios[0]
                .winner_value
                .as_deref()
                .is_some_and(|value| value.starts_with("url(a") && value.ends_with("b.png)"))
        );
    }
    Ok(())
}

#[test]
fn production_cst_input_keeps_comment_adjacent_control_equivalent() {
    let commented =
        include_str!("../../tests/fixtures/cascade-input-authority/comment-adjacent.scss");
    let control = ".target { color: red; color: blue; }";
    assert_eq!(cascade_codes(commented), cascade_codes(control));
    assert_eq!(
        cascade_codes(commented),
        ["unreachableDeclaration", "unspecifiedCascadeTie"]
    );
}

#[test]
fn production_cst_input_keeps_leading_comment_cross_rule_control_equivalent() {
    let commented = include_str!(
        "../../tests/fixtures/cascade-input-authority/comment-leading-cross-rule.scss"
    );
    let control = ".target { color: red; } .target { color: blue; }";
    assert_eq!(cascade_codes(commented), cascade_codes(control));
    assert_eq!(
        cascade_codes(commented),
        ["unreachableDeclaration", "unspecifiedCascadeTie"]
    );
}

#[test]
fn scanner_and_cst_facts_have_an_adjudicated_differential_corpus() -> Result<(), String> {
    const CASES: &[DifferentialCase] = &[
        DifferentialCase {
            name: "url-semicolon",
            path: "url-semicolon.css",
            source: include_str!("../../tests/fixtures/cascade-input-authority/url-semicolon.css"),
            dialect: StyleDialect::Css,
            expected: DifferentialDisposition::ScannerWrong,
        },
        DifferentialCase {
            name: "url-brace",
            path: "url-brace.css",
            source: include_str!("../../tests/fixtures/cascade-input-authority/url-brace.css"),
            dialect: StyleDialect::Css,
            expected: DifferentialDisposition::ScannerWrong,
        },
        DifferentialCase {
            name: "comment-adjacent",
            path: "comment-adjacent.scss",
            source: include_str!(
                "../../tests/fixtures/cascade-input-authority/comment-adjacent.scss"
            ),
            dialect: StyleDialect::Scss,
            expected: DifferentialDisposition::Agreement,
        },
        DifferentialCase {
            name: "comment-leading-cross-rule",
            path: "comment-leading-cross-rule.scss",
            source: include_str!(
                "../../tests/fixtures/cascade-input-authority/comment-leading-cross-rule.scss"
            ),
            dialect: StyleDialect::Scss,
            expected: DifferentialDisposition::ScannerWrong,
        },
        DifferentialCase {
            name: "real-button-variants",
            path: "ButtonVariants.module.scss",
            source: include_str!(
                "../../../../../../test/_fixtures/real-project-corpus/ButtonVariants.module.scss"
            ),
            dialect: StyleDialect::Scss,
            expected: DifferentialDisposition::Agreement,
        },
        DifferentialCase {
            name: "real-analytics-grid",
            path: "AnalyticsGrid.module.less",
            source: include_str!(
                "../../../../../../test/_fixtures/real-project-corpus/AnalyticsGrid.module.less"
            ),
            dialect: StyleDialect::Less,
            expected: DifferentialDisposition::Agreement,
        },
        DifferentialCase {
            name: "sdk-basic",
            path: "basic.module.css",
            source: include_str!(
                "../../../../../../test/_fixtures/sdk-cross-surface-parity/basic.module.css"
            ),
            dialect: StyleDialect::Css,
            expected: DifferentialDisposition::Agreement,
        },
    ];

    let mut agreement = 0usize;
    let mut scanner_wrong = 0usize;
    let mut facts_wrong = 0usize;
    for case in CASES {
        let scanner = scanner_for(*case);
        let facts = facts_for(*case);
        let actual = classify_against_scanner_and_emission(*case, &scanner, &facts)?;
        assert_eq!(actual, case.expected, "{} diverged", case.name);
        match actual {
            DifferentialDisposition::Agreement => agreement += 1,
            DifferentialDisposition::ScannerWrong => scanner_wrong += 1,
            DifferentialDisposition::FactsWrong => facts_wrong += 1,
        }
    }

    assert_eq!(agreement, 4);
    assert_eq!(scanner_wrong, 3);
    assert_eq!(facts_wrong, 0);
    Ok(())
}

#[test]
fn facts_authority_keeps_scanner_wrong_winners_aligned_with_emission() -> Result<(), String> {
    for case in [
        DifferentialCase {
            name: "url-semicolon",
            path: "url-semicolon.css",
            source: include_str!("../../tests/fixtures/cascade-input-authority/url-semicolon.css"),
            dialect: StyleDialect::Css,
            expected: DifferentialDisposition::ScannerWrong,
        },
        DifferentialCase {
            name: "url-brace",
            path: "url-brace.css",
            source: include_str!("../../tests/fixtures/cascade-input-authority/url-brace.css"),
            dialect: StyleDialect::Css,
            expected: DifferentialDisposition::ScannerWrong,
        },
        DifferentialCase {
            name: "comment-leading-cross-rule",
            path: "comment-leading-cross-rule.scss",
            source: include_str!(
                "../../tests/fixtures/cascade-input-authority/comment-leading-cross-rule.scss"
            ),
            dialect: StyleDialect::Scss,
            expected: DifferentialDisposition::ScannerWrong,
        },
    ] {
        let facts = facts_for(case);
        let signatures = candidate_signatures(&facts);
        assert_eq!(signatures.len(), 2, "{}: {signatures:#?}", case.name);
        assert_eq!(signatures[0].declaration_id, "decl-0");
        assert_eq!(signatures[1].declaration_id, "decl-1");
        if case.name.starts_with("url-") {
            assert!(signatures[0].important, "{}: {signatures:#?}", case.name);
            assert!(signatures[0].value.starts_with("url(a"));
            assert!(signatures[0].value.ends_with("b.png)"));
            assert!(!signatures[1].important);
        }

        let evaluations = evaluation_signatures(&facts);
        let unreachable = evaluations
            .iter()
            .find(|evaluation| evaluation.rule_code_name == "unreachable-declaration")
            .ok_or_else(|| format!("{}: {evaluations:#?}", case.name))?;
        assert!(
            facts_match_semantic_emission(case, &facts)?,
            "{} facts-side winner must agree with the semantic emission plane: {unreachable:#?}",
            case.name,
        );
    }
    Ok(())
}

#[test]
fn seeded_facts_side_misjoin_is_classified_as_facts_wrong() {
    let case = DifferentialCase {
        name: "seeded-misjoin",
        path: "seeded-misjoin.css",
        source: ".target { color: red; color: blue; }",
        dialect: StyleDialect::Css,
        expected: DifferentialDisposition::Agreement,
    };
    let scanner = scanner_for(case);
    let mut facts = facts_for(case);
    facts[0].input.selector =
        omena_query_checker_orchestrator::CanonicalSelector::from_canonical(".wrong-owner");
    assert_eq!(
        classify_against_scanner_and_emission(case, &scanner, &facts),
        Ok(DifferentialDisposition::FactsWrong)
    );
}
