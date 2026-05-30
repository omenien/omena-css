use crate::{
    OmenaQueryDynamicClassValueInputV0, OmenaQueryDynamicClassnameCallSiteV0,
    OmenaQueryDynamicClassnameMTierInputV0, ParserPositionV0, ParserRangeV0,
    summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth,
};

fn classname_reference_range(line: usize) -> ParserRangeV0 {
    ParserRangeV0 {
        start: ParserPositionV0 { line, character: 0 },
        end: ParserPositionV0 {
            line,
            character: 12,
        },
    }
}

fn dynamic_classname_input(max_context_depth: usize) -> OmenaQueryDynamicClassnameMTierInputV0 {
    OmenaQueryDynamicClassnameMTierInputV0 {
        source_uri: "file:///Routes.tsx".to_string(),
        selector_universe: vec!["btn-primary".to_string()],
        max_context_depth,
        call_sites: vec![
            OmenaQueryDynamicClassnameCallSiteV0 {
                callee_key: "classForVariant".to_string(),
                call_site_stack: vec![
                    "RouteA.tsx:render".to_string(),
                    "PrimaryButton.tsx:className".to_string(),
                ],
                exit_value: OmenaQueryDynamicClassValueInputV0::Exact {
                    value: "btn-primary".to_string(),
                },
                reference_range: classname_reference_range(10),
            },
            OmenaQueryDynamicClassnameCallSiteV0 {
                callee_key: "classForVariant".to_string(),
                call_site_stack: vec![
                    "RouteB.tsx:render".to_string(),
                    "SecondaryButton.tsx:className".to_string(),
                ],
                exit_value: OmenaQueryDynamicClassValueInputV0::Exact {
                    value: "btn-secondary".to_string(),
                },
                reference_range: classname_reference_range(20),
            },
        ],
    }
}

/// Non-tautological mechanism-depth test for the query/LSP product surface.
///
/// The two call sites share the `classForVariant` callee. At `k = 0` the
/// k-limited call-string analysis collapses them into the single `<root>`
/// context and joins their exit values into the finite set
/// `{btn-primary, btn-secondary}`; because `btn-secondary` is outside the
/// `btn-primary`-only selector universe, the checker M-tier rules raise a
/// `noImpossibleSelector` source diagnostic. At `k = 2` the two call sites keep
/// distinct contexts, so the primary call site narrows to a clean exact
/// `btn-primary` value (no diagnostics) while only the secondary keeps its
/// finding.
///
/// The load-bearing assertion is that the emitted query diagnostics differ with
/// `k`. If `analyze_k_limited_call_site_flows` were replaced with a constant or
/// an identity (no context-key join / no separation), both depths would produce
/// the same diagnostic set and the differential assertion would fail — so this
/// is not a tautology.
#[test]
fn dynamic_classname_m_tier_query_diagnostics_change_with_context_depth() {
    let zero_cfa = summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(
        &dynamic_classname_input(0),
    );
    let two_cfa = summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(
        &dynamic_classname_input(2),
    );

    assert_eq!(zero_cfa.file_kind, "source");
    assert_eq!(two_cfa.file_kind, "source");

    // 0-CFA: the joined root value trips noImpossibleSelector for the universe.
    assert!(
        zero_cfa
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "noImpossibleSelector"),
        "0-CFA join must surface a noImpossibleSelector query diagnostic"
    );
    assert!(
        zero_cfa.diagnostics.iter().all(|diagnostic| {
            diagnostic
                .provenance
                .contains(&"omena-abstract-value.k-limited-call-site-flow")
        }),
        "diagnostics must be provenance-tagged to the k-limited flow mechanism"
    );

    // 2-CFA: only the secondary call site keeps a diagnostic, anchored at its
    // own reference range (line 20), and the primary clean call site (line 10)
    // produces nothing.
    assert!(
        two_cfa
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "noImpossibleSelector"),
        "2-CFA must keep the impossible secondary selector diagnostic"
    );
    assert!(
        two_cfa
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.range.start.line == 20),
        "2-CFA diagnostics must anchor to the secondary call site, not the clean primary one"
    );

    // The differential: increasing the context-depth bound must change the
    // emitted query diagnostics, not just metadata.
    assert!(
        zero_cfa.diagnostic_count > two_cfa.diagnostic_count,
        "increasing k must reduce the query M-tier diagnostic count \
         (zero={}, two={})",
        zero_cfa.diagnostic_count,
        two_cfa.diagnostic_count,
    );
}

/// Clear half of the emit+clear pair: when every call site already agrees with
/// the selector universe, the real M-tier evaluation produces nothing at any k.
/// This guards against a degenerate implementation that always emits.
#[test]
fn dynamic_classname_m_tier_query_diagnostics_clear_for_in_universe_values() {
    let mut input = dynamic_classname_input(0);
    for call_site in &mut input.call_sites {
        call_site.exit_value = OmenaQueryDynamicClassValueInputV0::Exact {
            value: "btn-primary".to_string(),
        };
    }

    let summary =
        summarize_omena_query_dynamic_classname_m_tier_diagnostics_with_context_depth(&input);

    assert_eq!(
        summary.diagnostic_count, 0,
        "all-in-universe call sites must clear every M-tier diagnostic"
    );
}
