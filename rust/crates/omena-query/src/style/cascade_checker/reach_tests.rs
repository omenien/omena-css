use super::*;

fn shipped_style_diagnostics(
    source: &str,
) -> Result<crate::OmenaQueryStyleDiagnosticsForFileV0, &'static str> {
    let style_uri = "file:///tmp/cascade-scope.css";
    let candidates = crate::summarize_omena_query_style_hover_candidates(style_uri, source)
        .ok_or("style hover candidates")?;
    Ok(crate::summarize_omena_query_style_diagnostics_for_file(
        style_uri,
        source,
        candidates.candidates.as_slice(),
    ))
}

#[test]
fn css_scope_guards_do_not_supply_cascade_proximity() -> Result<(), &'static str> {
    let source = "@scope (.outer) { div#target { color: red; } } @scope (.inner) { .target { color: blue; } }";
    let declarations = collect_query_checker_cascade_declarations(source);

    assert_eq!(declarations.len(), 2);
    assert_eq!(
        declarations
            .iter()
            .map(|declaration| declaration.input.condition_context.clone())
            .collect::<Vec<_>>(),
        vec![
            vec!["@scope (.outer)".to_string()],
            vec!["@scope (.inner)".to_string()],
        ],
        "both @scope guards must reach the cascade input collector"
    );

    let runtime_declarations = declarations
        .iter()
        .map(|declaration| query_runtime_cascade_declaration_from_input(&declaration.input))
        .collect::<Vec<_>>();
    assert_eq!(
        runtime_declarations
            .iter()
            .map(|declaration| declaration.key.specificity)
            .collect::<Vec<_>>(),
        vec![
            omena_cascade::Specificity::new(1, 0, 1),
            omena_cascade::Specificity::new(0, 1, 0),
        ],
        "the fixture must retain opposing selector specificity"
    );
    for declaration in &runtime_declarations {
        assert_eq!(
            declaration.key.scope_proximity, 0,
            "scopeProximity must remain zero in query_runtime_cascade_declaration_from_input for @scope-derived checker declarations"
        );
    }

    let diagnostics = shipped_style_diagnostics(source)?;
    assert_eq!(diagnostics.diagnostic_count, 0);
    assert!(diagnostics.diagnostics.is_empty());
    Ok(())
}

#[test]
fn distinct_scope_and_media_guards_have_equivalent_diagnostics_with_a_live_control()
-> Result<(), &'static str> {
    let scope_source =
        "@scope (.outer) { .target { color: red; } } @scope (.inner) { .target { color: blue; } }";
    let media_source = "@media (min-width: 1px) { .target { color: red; } } @media (min-width: 2px) { .target { color: blue; } }";
    let live_source = ".target { color: red; color: blue; }";

    for guarded_source in [scope_source, media_source] {
        let declarations = collect_query_checker_cascade_declarations(guarded_source);
        assert_eq!(
            declarations.len(),
            2,
            "both guarded declarations must reach the cascade input collector"
        );
        assert!(
            declarations
                .iter()
                .all(|declaration| !declaration.input.condition_context.is_empty()),
            "each guarded declaration must retain its condition context"
        );
        assert_ne!(
            declarations[0].input.condition_context, declarations[1].input.condition_context,
            "the two guards must remain distinct comparison sites"
        );
    }

    let scope_diagnostics = shipped_style_diagnostics(scope_source)?;
    let media_diagnostics = shipped_style_diagnostics(media_source)?;
    assert_eq!(scope_diagnostics, media_diagnostics);
    assert_eq!(scope_diagnostics.diagnostic_count, 0);

    let live_diagnostics = shipped_style_diagnostics(live_source)?;
    assert_eq!(live_diagnostics.diagnostic_count, 2);
    assert_eq!(
        live_diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec!["unreachableDeclaration", "unspecifiedCascadeTie"],
        "the unguarded duplicate keeps the diagnostic plane live"
    );
    Ok(())
}

#[test]
fn same_scope_duplicates_step_down_to_conditional_certainty() -> Result<(), &'static str> {
    let unscoped = shipped_style_diagnostics(".target { color: red; color: blue; }")?;
    let scoped =
        shipped_style_diagnostics("@scope (.outer) { .target { color: red; color: blue; } }")?;

    let cases = [
        (
            &unscoped,
            "staticDefinite",
            "staticDefiniteWithinModeledEnvironment",
            "staticDefinite",
            "staticDefiniteWithinModeledEnvironment",
            Vec::<&str>::new(),
            "pseudoState",
        ),
        (
            &scoped,
            "conditionalDefinite",
            "conditionalDefiniteWithinModeledEnvironment",
            "conditionalDefinite",
            "conditionalDefiniteWithinModeledEnvironment",
            vec!["@scope (.outer)"],
            "mediaEnvironment",
        ),
    ];

    for (
        diagnostics,
        expected_confidence,
        expected_confidence_within_modeled_environment,
        expected_certainty,
        expected_certainty_within_modeled_environment,
        expected_context,
        expected_scenario_kind,
    ) in cases
    {
        assert_eq!(diagnostics.diagnostic_count, 2);
        assert_eq!(
            diagnostics
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec!["unreachableDeclaration", "unspecifiedCascadeTie"]
        );

        for diagnostic in &diagnostics.diagnostics {
            let narrowing = diagnostic
                .cascade_narrowing
                .as_ref()
                .ok_or("cascade narrowing evidence")?;
            assert_eq!(
                narrowing
                    .condition_context
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                expected_context
            );
            let runtime_state = narrowing
                .runtime_state
                .as_ref()
                .ok_or("runtime state scenario evidence")?;
            assert_eq!(runtime_state.confidence_tier, expected_confidence);
            assert_eq!(
                runtime_state.confidence_tier_within_modeled_environment,
                expected_confidence_within_modeled_environment
            );
            assert_eq!(runtime_state.result_certainty(), expected_certainty);
            assert_eq!(
                runtime_state.result_certainty_within_modeled_environment(),
                expected_certainty_within_modeled_environment
            );
            assert_eq!(
                runtime_state
                    .scenarios
                    .first()
                    .ok_or("runtime state scenario")?
                    .condition_context
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                expected_context
            );
            let scenario = runtime_state
                .scenarios
                .first()
                .ok_or("runtime state scenario")?;
            assert_eq!(scenario.scenario_kind, expected_scenario_kind);
            assert_eq!(scenario.winner_value.as_deref(), Some("blue"));
        }
    }

    Ok(())
}
