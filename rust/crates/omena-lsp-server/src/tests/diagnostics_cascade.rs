use super::*;

#[test]
fn resolves_unnecessary_tags_for_cascade_style_diagnostics() -> TestResult {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ":root { --brand: red; }\n.btn { color: red; color: blue; }",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let unreachable = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("unreachableDeclaration")))
        .ok_or_else(|| std::io::Error::other("unreachable declaration diagnostic"))?;
    assert_eq!(unreachable.pointer("/severity"), Some(&json!(4)));
    assert_eq!(unreachable.pointer("/tags"), Some(&json!([1])));
    // Since the checker orchestrator boundary split (8e632aa2), the product
    // cascade diagnostics carry the upstream orchestrator-gate product as the
    // most-upstream provenance label, followed by the owning checker rules.
    assert_eq!(
        unreachable.pointer("/data/provenance/0"),
        Some(&json!("omena-query-checker-orchestrator.cascade-gate")),
    );
    assert_eq!(
        unreachable.pointer("/data/provenance/1"),
        Some(&json!("omena-checker.cascade-rules")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/product"),
        Some(&json!("omena-query.cascade-narrowing-evidence")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/selector"),
        Some(&json!(".btn")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/selectorClassNames"),
        Some(&json!(["btn"])),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyName"),
        Some(&json!("color")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyValueNarrowing/product"),
        Some(&json!("omena-abstract-value.property-value-narrowing")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyValueNarrowing/matchedCandidateCount"),
        Some(&json!(2)),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/elementClassIteration/product"),
        Some(&json!("omena-abstract-value.reduced-product-iteration")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeConfidence/product"),
        Some(&json!("omena-query.cascade-confidence")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeConfidence/featureGate"),
        Some(&json!("cascade-confidence-v0")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeConfidence/claimLevel"),
        Some(&json!("fixtureWitnessResearchHint")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeConfidence/theoremClaimed"),
        Some(&json!(false)),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeConfidence/publicSafetyClaimReady"),
        Some(&json!(false)),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeConfidence/calibrationStage"),
        Some(&json!("fixtureWitnessTierWeightSigmoidV0")),
    );
    assert_eq!(
        unreachable.pointer("/data/polynomialProvenance/product"),
        Some(&json!("omena-abstract-value.polynomial-provenance")),
    );
    assert_eq!(
        unreachable.pointer("/data/polynomialProvenance/claimLevel"),
        Some(&json!("fixtureWitnessPolynomialProjection")),
    );
    assert_eq!(
        unreachable.pointer("/data/polynomialProvenance/theoremClaimed"),
        Some(&json!(false)),
    );
    assert_eq!(
        unreachable.pointer("/data/polynomialProvenance/selectedLadder"),
        Some(&json!("diagnosticDefaultThreeTier")),
    );
    assert_eq!(
        unreachable.pointer("/data/runtimeState/product"),
        Some(&json!("omena-query.runtime-state-scenario-evidence")),
    );
    assert_eq!(
        unreachable.pointer("/data/runtimeState/staticBoundary/boundaryKind"),
        Some(&json!("staticValueAssumingNoRuntimeOverride")),
    );
    assert_eq!(
        unreachable.pointer("/data/runtimeState/confidenceTier"),
        Some(&json!("staticDefinite")),
    );
    assert_eq!(
        unreachable
            .pointer("/data/cascadeNarrowing/runtimeState/staticBoundary/tracksClassListMutation"),
        Some(&json!(false)),
    );
    Ok(())
}

#[test]
fn runtime_state_payload_preserves_unknown_selector_activation() -> TestResult {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".b { color: blue; color: navy; }\n.a:hover .b { color: red; }",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let unreachable = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("unreachableDeclaration")))
        .ok_or_else(|| std::io::Error::other("unreachable declaration diagnostic"))?;
    let runtime_state = unreachable
        .pointer("/data/runtimeState")
        .ok_or_else(|| std::io::Error::other("runtime state payload"))?;
    let scenarios = runtime_state
        .pointer("/scenarios")
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("runtime state scenarios"))?;

    assert_eq!(
        runtime_state.pointer("/confidenceTier"),
        Some(&json!("staticDefinite"))
    );
    assert_eq!(
        runtime_state.pointer("/resultCertainty"),
        Some(&json!("staticUnknown"))
    );
    assert_eq!(
        runtime_state.pointer("/resultCertaintyWithinModeledEnvironment"),
        Some(&json!("staticUnknownWithinModeledEnvironment"))
    );
    assert!(scenarios.iter().any(|scenario| {
        let Some(unknown_ids) = scenario
            .pointer("/unknownActivationDeclarationIds")
            .and_then(Value::as_array)
        else {
            return false;
        };
        let Some(declaration_ids) = scenario
            .pointer("/declarationIds")
            .and_then(Value::as_array)
        else {
            return false;
        };
        !unknown_ids.is_empty()
            && unknown_ids
                .iter()
                .all(|unknown_id| declaration_ids.contains(unknown_id))
    }));
    assert!(
        scenarios
            .iter()
            .all(|scenario| scenario.pointer("/winnerValue") != Some(&json!("navy")))
    );
    Ok(())
}

#[test]
fn cascade_narrowing_prunes_to_requested_condition_and_layer_branch() -> TestResult {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": "@media (min-width: 40rem) {\n  @layer base { .btn { color: red; } }\n  @layer theme { .btn { color: blue; } }\n}",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let Some(layered) = diagnostics.iter().find(|diagnostic| {
        diagnostic.pointer("/data/cascadeNarrowing/propertyValueNarrowing/requestedLayerName")
            == Some(&json!("base"))
    }) else {
        return Err(std::io::Error::other("layered cascade narrowing diagnostic").into());
    };

    assert_eq!(
        layered.pointer("/data/cascadeNarrowing/propertyValueNarrowing/requestedConditionContext"),
        Some(&json!(["@media (min-width: 40rem)"])),
    );
    assert_eq!(
        layered.pointer("/data/cascadeNarrowing/propertyValueNarrowing/requestedLayerOrder"),
        Some(&json!(0)),
    );
    assert_eq!(
        layered.pointer("/data/cascadeNarrowing/propertyValueNarrowing/requestedLayerScope"),
        Some(&json!("exactLayer")),
    );
    assert_eq!(
        layered.pointer("/data/cascadeNarrowing/propertyValueNarrowing/matchedCandidateCount"),
        Some(&json!(1)),
    );
    assert_eq!(
        layered.pointer("/data/cascadeNarrowing/propertyValueNarrowing/value/kind"),
        Some(&json!("exact")),
    );
    assert_eq!(
        layered.pointer("/data/cascadeNarrowing/propertyValueNarrowing/value/value"),
        Some(&json!("red")),
    );
    assert_eq!(
        layered.pointer("/data/runtimeState/confidenceTier"),
        Some(&json!("conditionalDefinite")),
    );
    Ok(())
}

#[test]
fn cascade_narrowing_prunes_statically_false_supports_runtime_branch() -> TestResult {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": ".button { color: red; color: maroon; }\n@supports (display: grid) { .button { color: green; } }\n@supports not (display: grid) { .button { color: blue; } }\n@supports font-tech(unknown-thing) { .button { color: teal; } }",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let mut codes = diagnostics
        .iter()
        .filter_map(|diagnostic| diagnostic.pointer("/code").and_then(Value::as_str))
        .collect::<Vec<_>>();
    codes.sort_unstable();
    assert_eq!(
        codes,
        vec!["unreachableDeclaration", "unspecifiedCascadeTie"]
    );
    let unreachable = &diagnostics[0];
    let scenarios = unreachable
        .pointer("/data/runtimeState/scenarios")
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("runtime scenarios"))?;
    assert!(
        !scenarios.iter().any(|scenario| scenario
            .pointer("/conditionContext")
            .is_some_and(|context| context == &json!(["@supports not (display: grid)"]))),
        "statically false @supports branch must be pruned: {scenarios:?}"
    );
    assert!(
        scenarios.iter().any(|scenario| scenario
            .pointer("/conditionContext")
            .is_some_and(|context| context == &json!(["@supports (display: grid)"]))),
        "statically true @supports branch must be kept: {scenarios:?}"
    );
    assert!(
        scenarios.iter().any(|scenario| scenario
            .pointer("/conditionContext")
            .is_some_and(|context| context == &json!(["@supports font-tech(unknown-thing)"]))),
        "unknown @supports branch must be kept: {scenarios:?}"
    );
    let pruning = unreachable
        .pointer("/data/runtimeState/staticConditionPruning/0")
        .ok_or_else(|| std::io::Error::other("static condition pruning evidence"))?;
    assert_eq!(
        pruning.pointer("/conditionContext"),
        Some(&json!(["@supports not (display: grid)"])),
    );
    assert_eq!(
        pruning.pointer("/assumption"),
        Some(&json!("modernBrowser"))
    );
    assert_eq!(pruning.pointer("/verdict"), Some(&json!("AlwaysFalse")));
    assert_eq!(pruning.pointer("/pruned"), Some(&json!(true)));
    assert_eq!(pruning.pointer("/anchorContext"), Some(&json!(false)));
    let media_driver = unreachable
        .pointer("/data/runtimeState/driverSummaries")
        .and_then(Value::as_array)
        .and_then(|drivers| {
            drivers.iter().find(|driver| {
                driver.pointer("/driver") == Some(&json!("mediaEnvironmentScenarioSweep"))
            })
        })
        .ok_or_else(|| std::io::Error::other("media driver summary"))?;
    assert_eq!(media_driver.pointer("/scenarioCount"), Some(&json!(2)));
    Ok(())
}

#[test]
fn cascade_narrowing_exempts_anchor_inside_statically_false_supports_branch() -> TestResult {
    let mut state = LspShellState::default();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                    "languageId": "scss",
                    "version": 1,
                    "text": "@supports not (display: grid) { .button { color: red; color: maroon; } }\n@supports (display: grid) { .button { color: green; } }\n@supports font-tech(unknown-thing) { .button { color: teal; } }",
                },
            },
        }),
    );

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let unreachable = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("unreachableDeclaration")))
        .ok_or_else(|| std::io::Error::other("unreachable declaration diagnostic"))?;
    let scenarios = unreachable
        .pointer("/data/runtimeState/scenarios")
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("runtime scenarios"))?;
    assert!(
        scenarios.iter().any(|scenario| scenario
            .pointer("/conditionContext")
            .is_some_and(|context| context == &json!(["@supports not (display: grid)"]))),
        "anchor context must be preserved even when statically false: {scenarios:?}"
    );
    let pruning = unreachable
        .pointer("/data/runtimeState/staticConditionPruning/0")
        .ok_or_else(|| std::io::Error::other("static condition pruning evidence"))?;
    assert_eq!(
        pruning.pointer("/conditionContext"),
        Some(&json!(["@supports not (display: grid)"])),
    );
    assert_eq!(pruning.pointer("/verdict"), Some(&json!("AlwaysFalse")));
    assert_eq!(pruning.pointer("/pruned"), Some(&json!(false)));
    assert_eq!(pruning.pointer("/anchorContext"), Some(&json!(true)));
    let media_driver = unreachable
        .pointer("/data/runtimeState/driverSummaries")
        .and_then(Value::as_array)
        .and_then(|drivers| {
            drivers.iter().find(|driver| {
                driver.pointer("/driver") == Some(&json!("mediaEnvironmentScenarioSweep"))
            })
        })
        .ok_or_else(|| std::io::Error::other("media driver summary"))?;
    assert_eq!(media_driver.pointer("/scenarioCount"), Some(&json!(3)));
    Ok(())
}

#[test]
fn cascade_narrowing_uses_reachable_module_graph_candidates() -> TestResult {
    let mut state = LspShellState::default();
    for (uri, text) in [
        (
            "file:///workspace-a/src/App.module.scss",
            "@use \"./theme\";\n.btn { color: red; }\n.btn { color: green; }",
        ),
        (
            "file:///workspace-a/src/_theme.scss",
            ".btn { color: blue; }\n.unrelated { color: black; }",
        ),
        (
            "file:///workspace-a/src/_other.scss",
            ".btn { color: orange; }",
        ),
    ] {
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": text,
                    },
                },
            }),
        );
    }

    let diagnostics_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": STYLE_DIAGNOSTICS_REQUEST,
            "params": {
                "textDocument": {
                    "uri": "file:///workspace-a/src/App.module.scss",
                },
            },
        }),
    );
    let diagnostics = diagnostics_response
        .as_ref()
        .and_then(|value| value.pointer("/result"))
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("style diagnostics result"))?;
    let unreachable = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!("unreachableDeclaration")))
        .ok_or_else(|| std::io::Error::other("unreachable declaration diagnostic"))?;
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyValueNarrowing/stylesheetScope"),
        Some(&json!("moduleGraph")),
    );
    assert_eq!(
        unreachable.pointer("/data/cascadeNarrowing/propertyValueNarrowing/candidateCount"),
        Some(&json!(3)),
    );
    let display_values = unreachable
        .pointer("/data/cascadeNarrowing/propertyValueNarrowing/displayValues")
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("finite property display values"))?;
    assert!(
        display_values.iter().any(|value| value == &json!("blue")),
        "reachable imported module value should participate: {display_values:?}"
    );
    assert!(
        display_values.iter().all(|value| value != &json!("orange")),
        "unreachable module value should not participate: {display_values:?}"
    );
    Ok(())
}
