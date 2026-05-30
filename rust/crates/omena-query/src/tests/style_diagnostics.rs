use crate::{
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDocumentInputV0,
    OmenaQueryStyleDiagnosticsForFileV0, OmenaQueryStyleSourceInputV0, ParserPositionV0,
    ParserRangeV0,
};

#[test]
fn missing_custom_property_diagnostics_are_query_owned() -> Result<(), serde_json::Error> {
    let source = ":root { --brand: red; }\n.alert { color: var(--missing); }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source);
    assert!(candidates.is_some());
    let Some(candidates) = candidates else {
        return Ok(());
    };

    let diagnostics = crate::summarize_omena_query_missing_custom_property_diagnostics(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code, "missingCustomProperty");
    assert_eq!(
        diagnostic.message,
        "CSS custom property '--missing' not found in indexed style tokens."
    );
    assert_eq!(
        diagnostic.range,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 20,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 29,
            },
        }
    );
    assert_eq!(
        diagnostic
            .create_custom_property
            .as_ref()
            .map(|action| action.new_text.as_str()),
        Some("\n\n:root {\n  --missing: ;\n}\n")
    );
    assert_eq!(
        diagnostic
            .create_custom_property
            .as_ref()
            .map(|action| action.range),
        Some(ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 33,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 33,
            },
        })
    );

    let summary = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/App.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    assert_eq!(summary.product, "omena-query.diagnostics-for-file");
    assert_eq!(summary.file_kind, "style");
    assert_eq!(summary.diagnostic_count, 1);
    assert_eq!(summary.diagnostics[0].code, "missingCustomProperty");
    assert_eq!(summary.diagnostics[0].severity, "warning");
    assert_eq!(
        summary.diagnostics[0].provenance.as_slice(),
        [
            "omena-parser.custom-property-facts",
            "omena-query.style-diagnostics",
            "omena-query-checker-orchestrator.product-diagnostic-gate",
            "omena-checker.rule-registry",
        ]
    );
    let linear_provenance = summary.diagnostics[0].linear_provenance();
    assert_eq!(
        linear_provenance.product,
        "omena-abstract-value.linear-provenance"
    );
    assert_eq!(
        linear_provenance.labels(),
        summary.diagnostics[0].provenance
    );
    assert_eq!(linear_provenance.term_count, 4);

    let serialized = serde_json::to_value(&summary.diagnostics[0])?;
    assert_eq!(
        serialized
            .pointer("/provenance/0")
            .and_then(|value| value.as_str()),
        Some("omena-parser.custom-property-facts")
    );
    assert!(
        serialized.get("linearProvenance").is_none(),
        "typed provenance is a strict-superset projection and must not change the current wire shape"
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"missingCustomPropertyDiagnostics")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"checkerProductDiagnosticGate")
    );
    Ok(())
}

#[test]
fn style_diagnostics_for_file_include_cascade_aware_lints() -> Result<(), &'static str> {
    let source = r#"
@layer base {
  .btn { color: red; }
  .dead { border-color: red; }
}
@layer overrides {
  .btn { color: blue; }
  .dead { border-color: blue; }
}
:root {
  --cycle-a: var(--cycle-b);
  --cycle-b: var(--cycle-a);
  --bad: var(--missing);
}
.card { color: var(--bad); }
.tie { color: red; color: green; }
"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert_eq!(diagnostics.product, "omena-query.diagnostics-for-file");
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cascadeAwareDiagnostics")
    );
    assert_eq!(
        diagnostics
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "guaranteedInvalidCustomProperty")
            .count(),
        3
    );
    let diagnostic_codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(diagnostic_codes.contains("unreachableDeclaration"));
    assert!(diagnostic_codes.contains("deadCascadeLayer"));
    assert!(diagnostic_codes.contains("iacvtProne"));
    assert!(diagnostic_codes.contains("circularVar"));
    assert!(diagnostic_codes.contains("unspecifiedCascadeTie"));
    assert_eq!(
        diagnostics
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "unreachableDeclaration")
            .ok_or("unreachable declaration diagnostic")?
            .tags
            .as_slice(),
        &[1]
    );
    assert_eq!(
        diagnostics
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "unreachableDeclaration")
            .ok_or("unreachable declaration diagnostic")?
            .severity,
        "hint"
    );
    assert_eq!(
        diagnostics
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "deadCascadeLayer")
            .ok_or("dead cascade layer diagnostic")?
            .tags
            .as_slice(),
        &[1]
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "iacvtProne")
            .ok_or("iacvt diagnostic")?
            .tags
            .is_empty()
    );
    Ok(())
}

#[test]
fn cascade_aware_lints_do_not_compare_across_conditional_contexts() -> Result<(), &'static str> {
    let source = r#"
.btn { color: red; }
@media (min-width: 40rem) {
  .btn { color: blue; }
}
@supports (display: grid) {
  .btn { color: green; }
}
"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    let diagnostic_codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cascadeAwareDiagnostics")
    );
    assert!(!diagnostic_codes.contains("unreachableDeclaration"));
    assert!(!diagnostic_codes.contains("unspecifiedCascadeTie"));
    Ok(())
}

#[test]
fn cascade_aware_lints_do_not_compare_nested_ampersand_across_parent_contexts()
-> Result<(), &'static str> {
    let source = r#"
.article {
  &.box {
    &.fill { padding: 1px 5px; }
  }
  &.capsule {
    &.fill { padding: 1px 6px; }
  }
}
"#;
    let diagnostic_codes = cascade_diagnostic_code_set(source)?;

    assert!(!diagnostic_codes.contains("unreachableDeclaration"));
    assert!(!diagnostic_codes.contains("unspecifiedCascadeTie"));
    Ok(())
}

#[test]
fn cascade_aware_lints_still_compare_duplicate_declarations_inside_same_nested_context()
-> Result<(), &'static str> {
    let source = r#"
.article {
  &.box {
    &.fill {
      padding: 1px 5px;
      padding: 1px 6px;
    }
  }
}
"#;
    let diagnostic_codes = cascade_diagnostic_code_set(source)?;

    assert!(diagnostic_codes.contains("unreachableDeclaration"));
    assert!(diagnostic_codes.contains("unspecifiedCascadeTie"));
    Ok(())
}

#[test]
fn cascade_aware_lints_do_not_flag_resassigned_sass_variable_as_cascade_tie()
-> Result<(), &'static str> {
    // RFC-0007-K (#51): re-binding a Sass `$`-variable inside a rule is a
    // compile-time binding (dart-sass rc=0, `.a { margin: 16px; }`). It must not
    // be misreported as a duplicate CSS declaration / cascade tie.
    let source = r#"
.a {
  $gap: 8px;
  $gap: 16px;
  margin: $gap;
}
"#;
    let diagnostic_codes = cascade_diagnostic_code_set(source)?;

    assert!(!diagnostic_codes.contains("unreachableDeclaration"));
    assert!(!diagnostic_codes.contains("unspecifiedCascadeTie"));
    Ok(())
}

#[test]
fn cascade_aware_lints_still_flag_real_tie_when_rule_also_rebinds_sass_variable()
-> Result<(), &'static str> {
    // Over-correction guard for #51: dropping the `$`-var assignment must not
    // suppress a genuine same-selector/same-property CSS duplicate that sits in
    // the same rule.
    let source = r#"
.a {
  $gap: 8px;
  $gap: 16px;
  color: red;
  color: green;
}
"#;
    let diagnostic_codes = cascade_diagnostic_code_set(source)?;

    assert!(diagnostic_codes.contains("unreachableDeclaration"));
    assert!(diagnostic_codes.contains("unspecifiedCascadeTie"));
    Ok(())
}

#[test]
fn cascade_aware_lints_carry_variational_designer_intent_evidence() -> Result<(), &'static str> {
    let source = r#"
.button--primary {
  color: red;
  color: blue;
}
.u-color-red {
  color: red;
  color: blue;
}
"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.css", source)
            .ok_or("style candidates")?;
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.css",
        source,
        candidates.candidates.as_slice(),
    );
    let designer_diagnostics = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "designerIntentInconsistency")
        .collect::<Vec<_>>();

    assert_eq!(designer_diagnostics.len(), 1);
    assert_eq!(designer_diagnostics[0].severity, "hint");
    assert!(
        designer_diagnostics[0]
            .provenance
            .contains(&"omena-variational.designer-intent-posterior")
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unspecifiedCascadeTie")
    );
    Ok(())
}

#[test]
fn cascade_aware_lints_carry_rg_flow_coupling_spectrum_evidence() -> Result<(), &'static str> {
    // Divergent stylesheet: the custom-property reference graph has a cycle
    // (--a -> --b -> --a), so the extracted coupling space grows its k_cycle
    // coordinate between the before/after RG step. The real
    // estimate_coupling_jacobian_spectrum_v0 linearization drives the spectral
    // radius above one and the rg-flow theory gate fires.
    //
    // WP7-b (#38): the rg-flow hint is an opt-in deep-analysis diagnostic and is
    // deduplicated against the product `circularVar` warning. On the DEFAULT
    // surface the hint is off entirely; with deep-analysis ON it is folded into
    // the `circularVar` diagnostic's provenance instead of triple-firing.
    let divergent = r#"
:root {
  --a: var(--b);
  --b: var(--a);
}
"#;
    let divergent_candidates =
        crate::summarize_omena_query_style_hover_candidates("Tokens.module.css", divergent)
            .ok_or("divergent candidates")?;

    // Default surface: the rg-flow theory hint must NOT appear.
    let default_diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Tokens.module.css",
        divergent,
        divergent_candidates.candidates.as_slice(),
    );
    assert!(
        default_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "rgFlowRelevantOperator"),
        "rg-flow hint must be off on the default surface"
    );

    // Deep-analysis surface: the hint is deduplicated into `circularVar`, so no
    // standalone `rgFlowRelevantOperator` diagnostic surfaces, but its provenance
    // is folded into the surviving `circularVar` warning.
    let deep_diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
            "file:///workspace/src/Tokens.module.css",
            divergent,
            divergent_candidates.candidates.as_slice(),
            true,
        );
    assert!(
        deep_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "rgFlowRelevantOperator"),
        "rg-flow hint must be deduplicated against circularVar under deep analysis"
    );
    let circular_var = deep_diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "circularVar")
        .ok_or("circularVar must still fire on a real var cycle")?;
    assert!(
        circular_var
            .provenance
            .contains(&"omena-rg-flow.coupling-jacobian-spectrum"),
        "rg-flow coupling-spectrum provenance should be folded into circularVar: {:?}",
        circular_var.provenance
    );
    assert!(
        circular_var
            .provenance
            .contains(&"omena-query-checker-orchestrator.rg-flow-gate"),
        "rg-flow gate provenance should be folded into circularVar: {:?}",
        circular_var.provenance
    );

    // Settled stylesheet: same number of custom properties and the same
    // var-reference fan-out, but acyclic (--a -> --b, --b literal). The coupling
    // space is identical before/after, the spectral radius is zero, and the gate
    // surfaces nothing even under deep analysis. If the spectrum were replaced by
    // a constant the divergent case would still emit but so would this one.
    let settled = r#"
:root {
  --a: var(--b);
  --b: 4px;
}
"#;
    let settled_candidates =
        crate::summarize_omena_query_style_hover_candidates("Tokens.module.css", settled)
            .ok_or("settled candidates")?;
    let settled_diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
            "file:///workspace/src/Tokens.module.css",
            settled,
            settled_candidates.candidates.as_slice(),
            true,
        );
    assert!(
        settled_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "rgFlowRelevantOperator")
    );
    Ok(())
}

#[test]
fn cascade_aware_lints_carry_categorical_functor_evidence() -> Result<(), &'static str> {
    // Cyclic custom-property ranking: --a -> --b -> --a. The least-fixed-point
    // ranking colimit cannot converge, so the cascade-ranking primitive is
    // forced to play a second, conflicting categorical role. The functor object
    // mapping becomes many-valued (one primitive -> two role objects), the real
    // apply_cascade_role_mapping_functor_v0 verdict rejects the mapping, and the
    // categorical theory gate fires.
    //
    // WP7-b (#38): the categorical hint is an opt-in deep-analysis diagnostic and
    // is deduplicated against the product `circularVar` warning — off entirely on
    // the default surface, folded into `circularVar`'s provenance with deep
    // analysis ON.
    let cyclic = r#"
:root {
  --a: var(--b);
  --b: var(--a);
}
.alert {
  color: var(--a);
}
"#;
    let cyclic_candidates =
        crate::summarize_omena_query_style_hover_candidates("Alert.module.css", cyclic)
            .ok_or("cyclic candidates")?;

    // Default surface: the categorical theory hint must NOT appear.
    let default_diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Alert.module.css",
        cyclic,
        cyclic_candidates.candidates.as_slice(),
    );
    assert!(
        default_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "categoricalCascadeEvidenceInconsistency"),
        "categorical hint must be off on the default surface"
    );

    // Deep-analysis surface: deduplicated into `circularVar`'s provenance.
    let deep_diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
            "file:///workspace/src/Alert.module.css",
            cyclic,
            cyclic_candidates.candidates.as_slice(),
            true,
        );
    assert!(
        deep_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "categoricalCascadeEvidenceInconsistency"),
        "categorical hint must be deduplicated against circularVar under deep analysis"
    );
    let circular_var = deep_diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "circularVar")
        .ok_or("circularVar must still fire on a real var cycle")?;
    assert!(
        circular_var
            .provenance
            .contains(&"omena-categorical.cascade-primitive-role-functor"),
        "categorical functor provenance should be folded into circularVar: {:?}",
        circular_var.provenance
    );
    assert!(
        circular_var
            .provenance
            .contains(&"omena-query-checker-orchestrator.categorical-gate"),
        "categorical gate provenance should be folded into circularVar: {:?}",
        circular_var.provenance
    );

    // Acyclic custom-property ranking: --a -> --b, --b literal. The ranking
    // colimit converges, the cascade-ranking primitive maps to exactly one
    // categorical role, the functor accepts the mapping, and nothing is surfaced
    // even under deep analysis. If the functor verdict were a constant the cyclic
    // case would still emit but so would this one.
    let acyclic = r#"
:root {
  --a: var(--b);
  --b: 4px;
}
.alert {
  margin: var(--a);
}
"#;
    let acyclic_candidates =
        crate::summarize_omena_query_style_hover_candidates("Alert.module.css", acyclic)
            .ok_or("acyclic candidates")?;
    let acyclic_diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
            "file:///workspace/src/Alert.module.css",
            acyclic,
            acyclic_candidates.candidates.as_slice(),
            true,
        );
    assert!(
        acyclic_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "categoricalCascadeEvidenceInconsistency")
    );
    Ok(())
}

#[test]
fn cascade_aware_lints_preserve_flatten_invariance_for_nested_ampersand() -> Result<(), &'static str>
{
    let nested = r#"
.article {
  &.box {
    &.fill {
      padding: 1px 5px;
      padding: 1px 6px;
    }
  }
}
"#;
    let flat = r#"
.article.box.fill {
  padding: 1px 5px;
  padding: 1px 6px;
}
"#;

    assert_eq!(
        cascade_diagnostic_code_set(nested)?,
        cascade_diagnostic_code_set(flat)?
    );
    Ok(())
}

#[test]
fn cascade_aware_lints_run_without_custom_property_declarations() -> Result<(), &'static str> {
    let source = ".btn { color: red; color: blue; }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.css", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.css",
        source,
        candidates.candidates.as_slice(),
    );
    let diagnostic_codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cascadeAwareDiagnostics")
    );
    assert!(diagnostic_codes.contains("unreachableDeclaration"));
    assert!(diagnostic_codes.contains("unspecifiedCascadeTie"));
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unreachableDeclaration"
                && diagnostic.tags.as_slice() == [1])
    );
    Ok(())
}

fn cascade_diagnostic_code_set(
    source: &str,
) -> Result<std::collections::BTreeSet<&'static str>, &'static str> {
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    Ok(diagnostics
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic.code {
            "unreachableDeclaration"
            | "deadCascadeLayer"
            | "iacvtProne"
            | "circularVar"
            | "unspecifiedCascadeTie" => Some(diagnostic.code),
            _ => None,
        })
        .collect())
}

#[test]
fn style_diagnostics_collect_uppercase_and_fallback_var_references() -> Result<(), &'static str> {
    let source = r#"
:root {
  --cycle-a: VAR(--missing, var(--cycle-b));
  --cycle-b: var(--cycle-a);
}
.card { color: var(--cycle-a); }
"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    let diagnostic_codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(diagnostic_codes.contains("circularVar"));
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "circularVar"
                && diagnostic.message == "Custom property dependency graph contains a cycle.")
    );
    Ok(())
}

#[test]
fn style_diagnostics_for_file_include_keyframes_resolution_lints() -> Result<(), &'static str> {
    let source = ".button { animation: fade 1s ease; }\n@keyframes spin { to { opacity: 1; } }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.css", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.css",
        source,
        candidates.candidates.as_slice(),
    );

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"missingKeyframesDiagnostics")
    );
    let keyframes_diagnostics = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingKeyframes")
        .collect::<Vec<_>>();
    assert_eq!(keyframes_diagnostics.len(), 1);
    assert_eq!(
        keyframes_diagnostics[0].message,
        "@keyframes 'fade' not found in this file."
    );
    Ok(())
}

#[test]
fn style_diagnostics_for_file_include_same_file_sass_symbol_lints() -> Result<(), &'static str> {
    let source = "$known: 1rem;\n@mixin raised() { box-shadow: 0 0 $known; }\n.button { color: $missing; @include absent; }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"missingSassSymbolDiagnostics")
    );
    let messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingSassSymbol")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![
            "Sass variable '$missing' not found in this file.",
            "Sass mixin '@mixin absent' not found in this file.",
        ]
    );
    Ok(())
}

#[test]
fn missing_sass_symbol_folds_hyphen_underscore_in_same_file() {
    // #48 FP no-fire: Sass treats `$a-b` and `$a_b` as the *same* identifier, so a
    // reference spelled with the opposite separator must NOT be flagged missing. This
    // hits the single-file inline key path that previously bypassed the fold chokepoint.
    let source = "$a-b: 1;\n.x { width: $a_b; }";
    let diagnostics = crate::summarize_omena_query_missing_sass_symbol_diagnostics(
        "file:///workspace/src/Component.module.scss",
        source,
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol"),
        "$a_b must resolve to the $a-b declaration (Sass identifier fold), got {:?}",
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
    );

    // Reverse direction must also hold: declare `$a_b`, reference `$a-b`.
    let reverse = "$a_b: 1;\n.x { width: $a-b; }";
    let reverse_diagnostics = crate::summarize_omena_query_missing_sass_symbol_diagnostics(
        "file:///workspace/src/Component.module.scss",
        reverse,
    );
    assert!(
        reverse_diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol"),
        "$a-b must resolve to the $a_b declaration (reverse fold), got {:?}",
        reverse_diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn missing_sass_symbol_fold_preserves_distinct_identifiers() {
    // #48 over-correction guard: the fold collapses only `_` vs `-`. Two genuinely
    // different identifiers ($foo vs $bar) must STILL be flagged as missing — the fix
    // must not silence true positives.
    let source = "$foo: 1;\n.x { width: $bar; }";
    let messages = crate::summarize_omena_query_missing_sass_symbol_diagnostics(
        "file:///workspace/src/Component.module.scss",
        source,
    )
    .into_iter()
    .filter(|diagnostic| diagnostic.code == "missingSassSymbol")
    .map(|diagnostic| diagnostic.message)
    .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec!["Sass variable '$bar' not found in this file.".to_string()],
        "$bar is a distinct identifier from $foo and must remain flagged"
    );
}

#[test]
fn missing_sass_symbol_folds_hyphen_underscore_across_files() {
    // #48 FP no-fire on the cross-file/workspace key path: an imported `$ns_token`
    // definition must satisfy a `$ns-token` reference (the real-corpus repro of a
    // hyphenated reference against an underscored forwarded definition).
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_tokens.scss".to_string(),
            style_source: "$ns_token: 1rem;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: "@import \"./tokens\";\n.root { width: $ns-token; }".to_string(),
        },
    ];
    let folded = crate::summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace(
        "/tmp/Button.module.scss",
        style_sources.as_slice(),
        &[],
    );
    assert!(
        folded
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol"),
        "$ns-token must resolve to the imported $ns_token (cross-file fold), got {:?}",
        folded
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
    );

    // Over-correction guard on the cross-file path: a genuinely-absent symbol still fires.
    let distinct_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_tokens.scss".to_string(),
            style_source: "$ns_token: 1rem;".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: "@import \"./tokens\";\n.root { width: $other-token; }".to_string(),
        },
    ];
    let distinct = crate::summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace(
        "/tmp/Button.module.scss",
        distinct_sources.as_slice(),
        &[],
    );
    assert!(
        distinct
            .iter()
            .any(|diagnostic| diagnostic.code == "missingSassSymbol"),
        "$other-token is genuinely absent and must remain flagged across files"
    );
}

#[test]
fn style_diagnostics_omena_ignore_next_line_suppresses_targeted_code_only()
-> Result<(), &'static str> {
    let source = r#"/* omena-ignore-next-line missingSassSymbol */
.button { color: $missing; animation: fade 1s ease; }"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"diagnosticSuppressionSyntax")
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol")
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missingKeyframes")
    );
    Ok(())
}

#[test]
fn style_diagnostics_omena_ignore_file_respects_rule_code_filters() -> Result<(), &'static str> {
    let source = r#"/* omena-ignore-file missingSassSymbol */
.button { color: $missing; animation: fade 1s ease; }"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    let codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(!codes.contains(&"missingSassSymbol"));
    assert!(codes.contains(&"missingKeyframes"));
    Ok(())
}

#[test]
fn style_diagnostics_omena_expect_error_suppresses_expected_diagnostic() -> Result<(), &'static str>
{
    let source = r#"/* omena-expect-error missingSassSymbol */
.button { color: $missing; }"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol"
                && diagnostic.code != "unusedOmenaExpectError")
    );
    Ok(())
}

#[test]
fn style_diagnostics_omena_expect_error_reports_unused_directive() -> Result<(), &'static str> {
    let source = r#"/* omena-expect-error missingSassSymbol */
.button { color: red; }"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    let unused = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "unusedOmenaExpectError")
        .collect::<Vec<_>>();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].message, "Unused omena-expect-error directive.");
    Ok(())
}

#[test]
fn style_diagnostics_for_file_suppresses_sass_builtins_and_hints_imports()
-> Result<(), &'static str> {
    let source = r#"@use "sass:color";
@use "sass:math" as m;
@use "sass:list";
@use "sass:map" as *;
@use "sass:meta";
@use "sass:string";
@use "sass:selector";
@import "./legacy";
.button {
  color: color.adjust(red);
  width: m.div(10px, 2);
  border-width: list.length(1px 2px 3px);
  z-index: get(("a": 1), "a");
  content: meta.inspect(red);
  font-family: string.quote(Demo);
  outline-color: selector.unify(".a", ".b");
  padding: $missing;
}"#;
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    let import_hints = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "deprecatedSassImport")
        .collect::<Vec<_>>();
    assert_eq!(import_hints.len(), 1);
    assert_eq!(import_hints[0].severity, "information");

    let missing_messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingSassSymbol")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        missing_messages,
        vec!["Sass variable '$missing' not found in this file."]
    );
    Ok(())
}

#[test]
fn deprecated_sass_import_skips_css_form_imports_but_still_flags_partials() {
    // #44 D1. Sass deprecated `@import` only for Sass partials. CSS-form imports
    // (`url(...)`, `.css` targets, protocol/`//` URLs, media-qualified targets) are
    // explicitly KEPT and must NOT be flagged. A genuine partial in the same file
    // MUST still warn.
    let source = r#"@import url(theme.css);
@import "vendor.css";
@import "//cdn.example/reset.css";
@import "https://x.com/y.css";
@import "print" print;
@import "responsive" (min-width: 100px);
@import 'partial';
.x { color: red; }"#;
    let diagnostics = crate::summarize_omena_query_sass_import_deprecation_hints(
        "file:///workspace/src/Component.module.scss",
        source,
    );
    let flagged: Vec<&str> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "deprecatedSassImport")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    // Exactly one deprecation hint: the Sass-form `'partial'`. The CSS-form imports
    // (incl. the two media-qualified ones) are suppressed.
    assert_eq!(
        flagged.len(),
        1,
        "only the Sass-form partial should warn; CSS-form imports must be skipped (got {flagged:?})"
    );
}

#[test]
fn deprecated_sass_import_classifies_per_target_in_multi_target_statement() {
    // #44 D1 over-correction guard: per-target classification. In a comma-peer
    // statement mixing a CSS target and a Sass partial, the partial MUST still warn
    // and the CSS target MUST stay suppressed (each peer is its own Import edge).
    let source = r#"@import "vendor.css", "partial";"#;
    let diagnostics = crate::summarize_omena_query_sass_import_deprecation_hints(
        "file:///workspace/src/Component.module.scss",
        source,
    );
    let flagged: Vec<&str> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "deprecatedSassImport")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(
        flagged.len(),
        1,
        "the Sass partial peer must still warn even when a CSS peer is suppressed (got {flagged:?})"
    );
}

#[test]
fn missing_sass_symbol_allows_meta_apply_and_get_mixin_but_flags_unknown() {
    // #44 D2. `meta.apply` (mixin) and `meta.get-mixin` (function) are real
    // `sass:meta` members in Sass 1.77 and must NOT be flagged. An actually-unknown
    // member MUST still flag (over-correction guard).
    let source = r#"@use "sass:meta";
@mixin theme($c) { color: $c; }
.x {
  @include meta.apply(meta.get-mixin("theme"), red);
  content: meta.not-a-real-member();
}"#;
    let diagnostics = crate::summarize_omena_query_missing_sass_symbol_diagnostics(
        "file:///workspace/src/Component.module.scss",
        source,
    );
    let flagged: Vec<&str> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingSassSymbol")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    // `meta.apply` and `meta.get-mixin` must be absent from the flagged set.
    assert!(
        !flagged.iter().any(|message| message.contains("apply")),
        "meta.apply must not be flagged (got {flagged:?})"
    );
    assert!(
        !flagged.iter().any(|message| message.contains("get-mixin")),
        "meta.get-mixin must not be flagged (got {flagged:?})"
    );
    // The genuinely-unknown member must still flag.
    assert!(
        flagged
            .iter()
            .any(|message| message.contains("not-a-real-member")),
        "an unknown sass:meta member must still be flagged (got {flagged:?})"
    );
}

#[test]
fn style_diagnostics_for_workspace_file_resolve_sass_module_graph_symbols()
-> Result<(), &'static str> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./tokens" as tokens;
@import "./legacy";
.button {
  color: tokens.$token-brand;
  @include tokens.token-tone;
  margin: $legacy-gap;
  border-color: tokens.$token-secret;
  padding: $missing;
}"#
            .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_tokens.scss".to_string(),
            style_source: r#"@forward "./palette" as token-* show $brand, tone;"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_palette.scss".to_string(),
            style_source: r#"$brand: red; $secret: blue; @mixin tone { color: $brand; }"#
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_legacy.scss".to_string(),
            style_source: r#"$legacy-gap: 1rem;"#.to_string(),
        },
    ];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"graphAwareSassSymbolDiagnostics")
    );
    let missing_messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingSassSymbol")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        missing_messages,
        vec![
            "Sass variable '$token-secret' not found in the visible Sass module graph.",
            "Sass variable '$missing' not found in the visible Sass module graph.",
        ]
    );
    let import_hints = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "deprecatedSassImport")
        .collect::<Vec<_>>();
    assert_eq!(import_hints.len(), 1);
    Ok(())
}

#[test]
fn style_diagnostics_resolve_load_path_rooted_use_symbols() -> Result<(), &'static str> {
    // RFC-0007-I (#49): a load-path-rooted `@use 'src/scss/design-system.scss' as *` (dart-sass
    // `--load-path=<pkg-root>`) must join the in-graph producer, so its symbols are visible and
    // do NOT become `missingSassSymbol` false positives. The consumer lives in a subdirectory so
    // the file-relative join misses; only the load-path root reaches the target.
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/pkg-root/components/consumer.scss".to_string(),
            style_source: r#"@use "src/scss/design-system.scss" as *;
.x { color: $brand; }"#
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/pkg-root/src/scss/design-system.scss".to_string(),
            style_source: r#"$brand: #c00;"#.to_string(),
        },
    ];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/pkg-root/components/consumer.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;

    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol"),
        "load-path-rooted @use should make $brand visible: {:?}",
        diagnostics.diagnostics
    );
    Ok(())
}

#[test]
fn style_diagnostics_still_flag_missing_load_path_rooted_target() -> Result<(), &'static str> {
    // Over-correction guard: when the load-path-rooted target is genuinely absent from the graph
    // (no in-graph file at any root), the symbol it would have provided MUST still flag as
    // `missingSassSymbol`. The fix must not blanket-suppress path-shaped @use specifiers.
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/pkg-root/components/consumer.scss".to_string(),
        style_source: r#"@use "src/scss/design-system.scss" as *;
.x { color: $brand; }"#
            .to_string(),
    }];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/pkg-root/components/consumer.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;

    let missing_messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingSassSymbol")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        missing_messages,
        vec!["Sass variable '$brand' not found in the visible Sass module graph."],
        "{:?}",
        diagnostics.diagnostics
    );
    Ok(())
}

#[test]
fn style_diagnostics_external_sif_mode_reports_missing_sif_boundary() -> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];

    let ignored_diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("ignored workspace diagnostics")?;
    assert!(
        ignored_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingExternalSif")
    );

    let sif_diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("sif workspace diagnostics")?;

    assert!(
        sif_diagnostics
            .ready_surfaces
            .contains(&"externalSifBoundaryDiagnostics")
    );
    assert!(
        sif_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol")
    );
    let boundary_messages = sif_diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingExternalSif")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        boundary_messages,
        vec![
            "External Sass module 'https://cdn.example/tokens.scss' is missing (topAny); generate or provide a SIF artifact, or use --external ignored.",
        ]
    );
    Ok(())
}

#[test]
fn style_diagnostics_external_sif_mode_resolves_symbols_from_sif_artifact()
-> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];
    let sif = omena_sif::OmenaSifV1::from_static_exports(
        "https://cdn.example/tokens.scss",
        omena_sif::OmenaSifGeneratorV1 {
            name: "fixture-sifgen".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "fixture-sifgen@0.1.0".to_string(),
        },
        omena_sif::OmenaSifSourceV1 {
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        },
        omena_sif::OmenaSifExportsV1 {
            variables: vec![omena_sif::OmenaSifVariableExportV1 {
                name: "$brand".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: Vec::new(),
            functions: Vec::new(),
            placeholders: Vec::new(),
            forwards: Vec::new(),
        },
        Vec::new(),
        b"$brand: red !default;",
    )
    .map_err(|_| "sif fixture")?;
    let external_sifs = vec![OmenaQueryExternalSifInputV0 {
        canonical_url: "https://cdn.example/tokens.scss".to_string(),
        sif,
    }];

    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
            external_sifs.as_slice(),
        )
        .ok_or("sif workspace diagnostics")?;

    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingExternalSif"
                && diagnostic.code != "missingSassSymbol")
    );
    Ok(())
}

#[test]
fn style_diagnostics_external_sif_mode_classifies_partial_boundary() -> Result<(), &'static str> {
    // The SIF in scope only exports `$brand`, but the file references both `$brand` and
    // `$accent` through the same namespace — a Partial boundary (#34).
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; border-color: remote.$accent; }"#
            .to_string(),
    }];
    let sif = omena_sif::OmenaSifV1::from_static_exports(
        "https://cdn.example/tokens.scss",
        omena_sif::OmenaSifGeneratorV1 {
            name: "fixture-sifgen".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "fixture-sifgen@0.1.0".to_string(),
        },
        omena_sif::OmenaSifSourceV1 {
            syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
        },
        omena_sif::OmenaSifExportsV1 {
            variables: vec![omena_sif::OmenaSifVariableExportV1 {
                name: "$brand".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: Vec::new(),
            functions: Vec::new(),
            placeholders: Vec::new(),
            forwards: Vec::new(),
        },
        Vec::new(),
        b"$brand: red !default;",
    )
    .map_err(|_| "sif fixture")?;
    let external_sifs = vec![OmenaQueryExternalSifInputV0 {
        canonical_url: "https://cdn.example/tokens.scss".to_string(),
        sif,
    }];

    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
            external_sifs.as_slice(),
        )
        .ok_or("sif workspace diagnostics")?;

    let partial = diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "partialExternalSif")
        .ok_or("expected partialExternalSif boundary diagnostic")?;
    assert_eq!(partial.severity, "information");
    assert!(
        partial.message.contains("partial (topAny)"),
        "{}",
        partial.message
    );
    // A Partial boundary stays TopAny, so the still-unknown `$accent` reference is NOT
    // double-flagged as a plain missing symbol.
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingExternalSif"
                && diagnostic.code != "missingSassSymbol")
    );
    Ok(())
}

#[test]
fn style_diagnostics_external_sif_mode_classifies_unresolved_boundary() -> Result<(), &'static str>
{
    // The fifth boundary state (#34): a bare specifier the resolver cannot canonicalize
    // (no SIF, not a relative path, not a `sass:`/`http(s)://` external) folds through the
    // resolver-error channel onto `Unresolved`. A second `https://` edge with no SIF in
    // scope is the over-correction guard: the existing Missing branch must still fire.
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"@use "bootstrap" as bs;
@use "https://cdn.example/tokens.scss" as remote;
.button { color: bs.$brand; border-color: remote.$accent; }"#
            .to_string(),
    }];

    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("sif workspace diagnostics")?;

    // The bare `bootstrap` edge surfaces the Unresolved boundary state.
    let unresolved = diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "unresolvedExternalReference")
        .ok_or("expected unresolvedExternalReference boundary diagnostic")?;
    assert_eq!(unresolved.severity, "warning");
    assert!(
        unresolved
            .message
            .contains("'bootstrap' is unresolved (topAny)"),
        "{}",
        unresolved.message
    );
    // Over-correction guard: the `https://` edge with no SIF still fires Missing.
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missingExternalSif"
                && diagnostic
                    .message
                    .contains("'https://cdn.example/tokens.scss' is missing")),
        "missing external boundary must still fire: {:?}",
        diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    );
    // A bare unresolved reference is NOT a workspace-local file, so it is never the hard
    // `missingModule` error — only the boundary state surfaces.
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingModule")
    );
    Ok(())
}

#[test]
fn style_diagnostics_external_sif_mode_does_not_double_flag_local_unresolved_boundary()
-> Result<(), &'static str> {
    // Over-correction guard for the Unresolved widening (#34): a workspace-local unresolved
    // specifier (`./missing`) is already a hard `missingModule` error, so it must NOT also
    // surface as an `unresolvedExternalReference` boundary state.
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: "@use \"./missing\";\n.a { color: red; }".to_string(),
    }];

    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("sif workspace diagnostics")?;

    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missingModule"),
        "workspace-local unresolved must still be a hard missingModule error: {:?}",
        diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "unresolvedExternalReference"),
        "workspace-local unresolved must not double-flag as a boundary state"
    );
    Ok(())
}

#[test]
fn style_diagnostics_external_sif_mode_classifies_stale_boundary() -> Result<(), &'static str> {
    // The root SIF records a dependency interface hash, but the dependency SIF in scope
    // has a different interface — a Stale boundary (#34).
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];
    let generator = || omena_sif::OmenaSifGeneratorV1 {
        name: "fixture-sifgen".to_string(),
        version: "0.1.0".to_string(),
        toolchain_id: "fixture-sifgen@0.1.0".to_string(),
    };
    let source = || omena_sif::OmenaSifSourceV1 {
        syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
    };
    let brand_exports = || omena_sif::OmenaSifExportsV1 {
        variables: vec![omena_sif::OmenaSifVariableExportV1 {
            name: "$brand".to_string(),
            defaulted: true,
            value_repr: Some("red".to_string()),
        }],
        mixins: Vec::new(),
        functions: Vec::new(),
        placeholders: Vec::new(),
        forwards: Vec::new(),
    };
    // Root SIF declares a dependency on `_base.scss` with a STALE interface hash.
    let root_sif = omena_sif::OmenaSifV1::from_static_exports(
        "https://cdn.example/tokens.scss",
        generator(),
        source(),
        brand_exports(),
        vec![omena_sif::OmenaSifDependencyInterfaceHashV1 {
            canonical_url: "https://cdn.example/_base.scss".to_string(),
            interface_hash: omena_sif::compute_omena_sif_interface_hash_v1(
                "fixture-sifgen@0.1.0",
                &omena_sif::OmenaSifExportsV1::default(),
            )
            .map_err(|_| "stale dep hash")?,
        }],
        b"$brand: red !default;",
    )
    .map_err(|_| "root sif fixture")?;
    // The dependency SIF actually in scope exports `$base`, so its interface hash differs
    // from the empty-exports hash recorded by the root SIF.
    let dependency_sif = omena_sif::OmenaSifV1::from_static_exports(
        "https://cdn.example/_base.scss",
        generator(),
        source(),
        omena_sif::OmenaSifExportsV1 {
            variables: vec![omena_sif::OmenaSifVariableExportV1 {
                name: "$base".to_string(),
                defaulted: false,
                value_repr: Some("black".to_string()),
            }],
            mixins: Vec::new(),
            functions: Vec::new(),
            placeholders: Vec::new(),
            forwards: Vec::new(),
        },
        Vec::new(),
        b"$base: black;",
    )
    .map_err(|_| "dependency sif fixture")?;
    let external_sifs = vec![
        OmenaQueryExternalSifInputV0 {
            canonical_url: "https://cdn.example/tokens.scss".to_string(),
            sif: root_sif,
        },
        OmenaQueryExternalSifInputV0 {
            canonical_url: "https://cdn.example/_base.scss".to_string(),
            sif: dependency_sif,
        },
    ];

    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
            external_sifs.as_slice(),
        )
        .ok_or("sif workspace diagnostics")?;

    let stale = diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "staleExternalSif")
        .ok_or("expected staleExternalSif boundary diagnostic")?;
    assert_eq!(stale.severity, "warning");
    assert!(
        stale.message.contains("stale (topAny)"),
        "{}",
        stale.message
    );
    Ok(())
}

#[test]
fn style_diagnostics_strictness_sigil_strict_escalates_missing_boundary_to_error()
-> Result<(), &'static str> {
    // RFC 0004 #28 / #35: `// @omena-strict: strict` escalates a Missing boundary's severity
    // from the default `warning` to `error`, while leaving the code/message intact.
    let strict_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"// @omena-strict: strict
@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];
    let strict =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            strict_sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("strict sif workspace diagnostics")?;
    assert!(
        strict.ready_surfaces.contains(&"strictnessSigilGating"),
        "strictnessSigilGating ready surface missing"
    );
    let strict_boundary = strict
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "missingExternalSif")
        .ok_or("expected missingExternalSif under strict")?;
    assert_eq!(strict_boundary.severity, "error");

    // Over-correction guard: the SAME file without the sigil keeps the default `warning`.
    let default_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];
    let default =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            default_sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("default sif workspace diagnostics")?;
    let default_boundary = default
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "missingExternalSif")
        .ok_or("expected missingExternalSif under default")?;
    assert_eq!(default_boundary.severity, "warning");
    // Code and message are identical — only severity differs between the two levels.
    assert_eq!(strict_boundary.code, default_boundary.code);
    assert_eq!(strict_boundary.message, default_boundary.message);
    Ok(())
}

#[test]
fn style_diagnostics_strictness_sigil_relaxed_suppresses_boundary() -> Result<(), &'static str> {
    // `// @omena-strict: relaxed` drops every external-boundary diagnostic.
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"// @omena-strict: relaxed
@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];
    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("relaxed sif workspace diagnostics")?;
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingExternalSif"),
        "relaxed must suppress missingExternalSif: {:?}",
        diagnostics.diagnostics
    );
    // The TopAny suppression still ran, so the external symbol is not re-flagged either.
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol")
    );
    Ok(())
}

#[test]
fn style_diagnostics_strictness_sigil_closed_escalates_unknown_external_symbol()
-> Result<(), &'static str> {
    // `// @omena-strict: closed` flips the boundary to TopOpaque: the genuinely-unknown
    // external `$brand` reference (no SIF in scope) is no longer suppressed and is escalated
    // to `error`.
    let closed_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"// @omena-strict: closed
@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];
    let closed =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            closed_sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("closed sif workspace diagnostics")?;
    let exposed = closed
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "missingSassSymbol")
        .ok_or("closed must expose the unknown external symbol")?;
    assert_eq!(exposed.severity, "error");

    // Default-level over-correction guard: the same file without the sigil keeps the symbol
    // suppressed (TopAny default behaviour, unchanged).
    let default_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];
    let default =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            default_sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("default sif workspace diagnostics")?;
    assert!(
        default
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingSassSymbol"),
        "default level must keep the unknown external symbol suppressed"
    );
    Ok(())
}

#[test]
fn style_diagnostics_suppression_applies_to_external_sif_boundary() -> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/App.module.scss".to_string(),
        style_source: r#"/* omena-ignore-next-line missingExternalSif */
@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
            .to_string(),
    }];

    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
            "/tmp/App.module.scss",
            sources.as_slice(),
            &[],
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Sif,
        )
        .ok_or("sif workspace diagnostics")?;

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"diagnosticSuppressionSyntax")
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingExternalSif")
    );
    Ok(())
}

#[test]
fn style_diagnostics_for_workspace_file_include_css_modules_resolution_lints()
-> Result<(), &'static str> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Component.module.css".to_string(),
            style_source: r#".button { composes: missingLocal; }
.missingModule { composes: root from "./Missing.module.css"; }
.external { composes: ghost from "./Base.module.css"; }
@value primary from "./MissingTokens.module.css";
@value absent from "./Tokens.module.css";"#
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Base.module.css".to_string(),
            style_source: ".base { color: blue; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Tokens.module.css".to_string(),
            style_source: "@value accent: blue;".to_string(),
        },
    ];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/Component.module.css",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace style diagnostics")?;

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cssModulesComposesResolutionDiagnostics")
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cssModulesValueResolutionDiagnostics")
    );
    let messages = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.message.as_str()))
        .collect::<Vec<_>>();
    assert!(messages.contains(&(
        "missingComposedSelector",
        "Selector '.missingLocal' not found in this file for composes.",
    )));
    assert!(messages.contains(&(
        "missingComposedModule",
        "Cannot resolve composed CSS Module './Missing.module.css'.",
    )));
    assert!(messages.contains(&(
        "missingComposedSelector",
        "Selector '.ghost' not found in composed module './Base.module.css'.",
    )));
    assert!(messages.contains(&(
        "missingValueModule",
        "Cannot resolve imported @value module './MissingTokens.module.css'.",
    )));
    assert!(messages.contains(&(
        "missingImportedValue",
        "@value 'absent' not found in './Tokens.module.css'.",
    )));
    Ok(())
}

#[test]
fn style_diagnostics_for_workspace_file_include_unused_selector_lints() -> Result<(), &'static str>
{
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/App.module.css".to_string(),
        style_source:
            ".used { color: red; }\n.ghost { color: blue; }\n.composed { composes: used; }"
                .to_string(),
    }];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/src/App.tsx".to_string(),
        source_source: r#"import styles from "./App.module.css";
export function App() {
  return <div className={styles.composed}>hi</div>;
}"#
        .to_string(),
    }];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/App.module.css",
        sources.as_slice(),
        source_documents.as_slice(),
        &[],
        None,
    )
    .ok_or("workspace style diagnostics")?;

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"unusedSelectorDiagnostics")
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"checkerProductDiagnosticGate")
    );
    let unused = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "unusedSelector")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        unused,
        vec!["Selector '.ghost' is declared but never used."]
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "unusedSelector"
                && diagnostic.severity == "hint"
                && diagnostic.provenance.as_slice()
                    == [
                        "omena-parser.selector-facts",
                        "omena-query.source-selector-usage",
                        "omena-query-checker-orchestrator.product-diagnostic-gate",
                        "omena-checker.rule-registry"
                    ])
    );
    Ok(())
}

#[test]
fn style_diagnostics_walk_anonymous_arrow_default_export_for_unused_selector_lints()
-> Result<(), &'static str> {
    // Regression for RFC-0007 #53: `export default () => <JSX/>` was never walked, so every
    // selector it referenced was wrongly flagged `unusedSelector`. Cover the three failing forms
    // (concise JSX, block body, parenthesized) plus a genuinely-unused selector that MUST stay
    // flagged so the fix does not silence true positives.
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/Arrow.module.css".to_string(),
        style_source: ".arrowUsed { color: red; }\n.arrowGhost { color: blue; }\n".to_string(),
    }];
    let forms = [
        // concise-expression body
        r#"import s from "./Arrow.module.css";
export default () => <i className={s.arrowUsed} />;"#,
        // block body with explicit return
        r#"import s from "./Arrow.module.css";
export default () => { return <i className={s.arrowUsed} />; };"#,
        // parenthesized body
        r#"import s from "./Arrow.module.css";
export default () => (<i className={s.arrowUsed} />);"#,
    ];

    for source_source in forms {
        let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
            source_path: "/workspace/src/Arrow.tsx".to_string(),
            source_source: source_source.to_string(),
        }];

        let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
            "/workspace/src/Arrow.module.css",
            sources.as_slice(),
            source_documents.as_slice(),
            &[],
            None,
        )
        .ok_or("anonymous arrow default export workspace style diagnostics")?;

        let unused = unused_selector_messages(&diagnostics);
        // FP gone: the referenced `.arrowUsed` is no longer flagged.
        assert!(
            !unused.contains(&"Selector '.arrowUsed' is declared but never used."),
            "anon-arrow form should mark .arrowUsed as used: {source_source}",
        );
        // True positive preserved: the genuinely-unused `.arrowGhost` is still flagged.
        assert_eq!(
            unused,
            vec!["Selector '.arrowGhost' is declared but never used."],
            "anon-arrow form must still flag .arrowGhost: {source_source}",
        );
    }

    Ok(())
}

#[test]
fn style_diagnostics_unused_selector_respects_classname_transform_aliases()
-> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/Button.module.scss".to_string(),
        style_source: ".btn-primary { color: red; }\n.orphan { color: blue; }\n".to_string(),
    }];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/src/App.tsx".to_string(),
        source_source: r#"import styles from "./Button.module.scss";
export function App() {
  return <div className={styles.btnPrimary}>hi</div>;
}"#
        .to_string(),
    }];

    let as_is = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/Button.module.scss",
        sources.as_slice(),
        source_documents.as_slice(),
        &[],
        Some("asIs"),
    )
    .ok_or("as-is workspace style diagnostics")?;
    assert!(
        unused_selector_messages(&as_is)
            .contains(&"Selector '.btn-primary' is declared but never used.")
    );

    let camel_case = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/Button.module.scss",
        sources.as_slice(),
        source_documents.as_slice(),
        &[],
        Some("camelCase"),
    )
    .ok_or("camel-case workspace style diagnostics")?;
    assert_eq!(
        unused_selector_messages(&camel_case),
        vec!["Selector '.orphan' is declared but never used."]
    );

    Ok(())
}

// RFC-0007-J (#50): a component that imports its style module through an unresolved workspace
// alias (`@/styles/...` with no tsconfig/bundler path mapping wired in) must NOT have every
// selector dimmed `unusedSelector`. References/goto stay lenient with the unresolved target, so
// the negative assertion has to be conservative too: an unresolvable style import means the file
// is "possibly using" its selectors, and the lint is suppressed for that target.
#[test]
fn style_diagnostics_alias_import_suppresses_unused_selector_fp() -> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/a.module.scss".to_string(),
        style_source: ".foo { color: red; }\n.bar { color: blue; }\n".to_string(),
    }];
    // `@/styles/a.module.scss` is a workspace alias; without path mappings it does not resolve to
    // any in-graph style path, so the `cx('foo')` usage cannot be attributed to a module.
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/src/component.tsx".to_string(),
        source_source: r#"import classNames from 'classnames/bind';
import styles from '@/styles/a.module.scss';
const cx = classNames.bind(styles);
export default () => <span className={cx('foo')} />;"#
            .to_string(),
    }];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/a.module.scss",
        sources.as_slice(),
        source_documents.as_slice(),
        &[],
        None,
    )
    .ok_or("alias-import workspace style diagnostics")?;

    // FP gone: neither selector is dimmed, even though only `.foo` is referenced — because we
    // cannot resolve the alias we refuse to assert ANY selector is unused.
    assert!(
        unused_selector_messages(&diagnostics).is_empty(),
        "alias-import doc must not dim any selector: {:?}",
        unused_selector_messages(&diagnostics)
    );
    Ok(())
}

// RFC-0007-J (#50) over-correction guard: when the same style module is imported through a
// RESOLVABLE relative specifier, a genuinely-unused selector MUST still flag `unusedSelector`.
// The alias safety net must not silence true positives in correctly-resolved modules, and a
// non-style unresolved import (e.g. `react`) must not trip the safety net either.
#[test]
fn style_diagnostics_resolved_import_still_flags_unused_selector() -> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/a.module.scss".to_string(),
        style_source: ".foo { color: red; }\n.bar { color: blue; }\n".to_string(),
    }];
    // A resolvable relative import alongside a non-style unresolved import (`react`). The relative
    // style import resolves, so usage is attributable and the safety net does NOT engage.
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/src/component.tsx".to_string(),
        source_source: r#"import React from 'react';
import styles from './a.module.scss';
export default () => <span className={styles.foo} />;"#
            .to_string(),
    }];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/a.module.scss",
        sources.as_slice(),
        source_documents.as_slice(),
        &[],
        None,
    )
    .ok_or("resolved-import workspace style diagnostics")?;

    // True positive preserved: `.foo` is used, `.bar` is genuinely unused and still flagged.
    assert_eq!(
        unused_selector_messages(&diagnostics),
        vec!["Selector '.bar' is declared but never used."],
        "resolved import must still flag the genuinely-unused .bar",
    );
    Ok(())
}

// RFC-0007-J (#50) root fix: when the workspace's tsconfig path mappings ARE wired in, the alias
// import `@/styles/a.module.scss` resolves to its real module, so the unused-selector usage
// collector attributes `cx('foo')` precisely — exactly like the reference/goto path. The lint then
// stays accurate rather than being globally suppressed: the used `.foo` is NOT dimmed, but the
// genuinely-unused `.bar` IS still flagged.
#[test]
fn style_diagnostics_alias_import_resolves_with_path_mappings() -> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/styles/a.module.scss".to_string(),
        style_source: ".foo { color: red; }\n.bar { color: blue; }\n".to_string(),
    }];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/src/component.tsx".to_string(),
        source_source: r#"import classNames from 'classnames/bind';
import styles from '@/styles/a.module.scss';
const cx = classNames.bind(styles);
export default () => <span className={cx('foo')} />;"#
            .to_string(),
    }];
    // tsconfig `paths`: `@/*` -> `src/*` rooted at the workspace, so `@/styles/a.module.scss`
    // resolves to `/workspace/src/styles/a.module.scss`.
    let resolution_inputs = crate::OmenaQueryStyleResolutionInputsV0 {
        tsconfig_path_mappings: vec![crate::OmenaQueryTsconfigPathMappingV0 {
            base_path: "/workspace".to_string(),
            pattern: "@/*".to_string(),
            target_patterns: vec!["src/*".to_string()],
        }],
        ..Default::default()
    };

    let diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
            "/workspace/src/styles/a.module.scss",
            sources.as_slice(),
            source_documents.as_slice(),
            &[],
            None,
            crate::OmenaQueryExternalModuleModeV0::Ignored,
            &[],
            &resolution_inputs,
        )
        .ok_or("alias-import workspace style diagnostics with path mappings")?;

    // Precise (not merely suppressed): `.foo` used -> not dimmed; `.bar` unused -> flagged.
    assert_eq!(
        unused_selector_messages(&diagnostics),
        vec!["Selector '.bar' is declared but never used."],
        "resolved alias import must dim only the genuinely-unused .bar: {:?}",
        unused_selector_messages(&diagnostics)
    );
    Ok(())
}

fn missing_keyframes_messages(diagnostics: &[crate::OmenaQueryStyleDiagnosticV0]) -> Vec<&str> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingKeyframes")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn missing_custom_property_messages(
    diagnostics: &[crate::OmenaQueryStyleDiagnosticV0],
) -> Vec<&str> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingCustomProperty")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

// RFC-0007-C / #43 C1: a vendor-prefixed `@-webkit-keyframes spin` resolves the
// `animation: spin` reference — no false `missingKeyframes`.
#[test]
fn vendor_prefixed_keyframes_suppress_missing_keyframes_fp() -> Result<(), &'static str> {
    let source = "@-webkit-keyframes spin { from { opacity: 0; } to { opacity: 1; } }\n.x { animation: spin 1s linear; }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    assert!(missing_keyframes_messages(&diagnostics.diagnostics).is_empty());
    Ok(())
}

// RFC-0007-C / #43 C1 over-correction guard: an animation referencing a keyframes name
// declared by NO at-rule (prefixed or not) must still fire `missingKeyframes`.
#[test]
fn truly_missing_keyframes_still_fires() -> Result<(), &'static str> {
    let source = ".x { animation: spin 1s linear; }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    assert_eq!(
        missing_keyframes_messages(&diagnostics.diagnostics),
        vec!["@keyframes 'spin' not found in this file."]
    );
    Ok(())
}

// RFC-0007-C / #43 C2: an interpolated animation name emits no keyframes reference on the
// literal fragment, so no `missingKeyframes` fires.
#[test]
fn interpolated_animation_name_suppresses_missing_keyframes_fp() -> Result<(), &'static str> {
    for source in [
        "$p: brand; .x { animation: #{$p}-spin 1s; }",
        "$p: brand; .x { animation: spin-#{$p} 1s; }",
        "$p: brand; .x { animation-name: #{$p}-spin; }",
    ] {
        let candidates =
            crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
                .ok_or("style candidates")?;
        let diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
            "file:///workspace/src/Component.module.scss",
            source,
            candidates.candidates.as_slice(),
        );
        assert!(
            missing_keyframes_messages(&diagnostics.diagnostics).is_empty(),
            "unexpected missingKeyframes for {source:?}"
        );
    }
    Ok(())
}

// RFC-0007-C / #43 C3: `var(--undeclared, blue)` with a fallback does not fire
// `missingCustomProperty` (the fallback guarantees a value).
#[test]
fn var_fallback_suppresses_missing_custom_property_fp() -> Result<(), &'static str> {
    let source = ".x { --declared: red; color: var(--undeclared, blue); }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;
    let diagnostics = crate::summarize_omena_query_missing_custom_property_diagnostics(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    assert!(missing_custom_property_messages(&diagnostics).is_empty());
    Ok(())
}

// RFC-0007-C / #43 C3 over-correction guard: a fallback-less `var(--undeclared)` still
// fires, and per-`var()` scoping keeps a nested fallback-less `var(--b)` live in
// `var(--a, var(--b))` (the outer `--a` is the only one suppressed).
#[test]
fn fallback_less_var_still_fires_missing_custom_property() -> Result<(), &'static str> {
    let source = ".x { --declared: red; color: var(--undeclared); }";
    let candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;
    let diagnostics = crate::summarize_omena_query_missing_custom_property_diagnostics(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    assert_eq!(
        missing_custom_property_messages(&diagnostics),
        vec!["CSS custom property '--undeclared' not found in indexed style tokens."]
    );

    let nested = ".x { --declared: red; color: var(--a, var(--b)); }";
    let nested_candidates =
        crate::summarize_omena_query_style_hover_candidates("Component.module.scss", nested)
            .ok_or("style candidates")?;
    let nested_diagnostics = crate::summarize_omena_query_missing_custom_property_diagnostics(
        "file:///workspace/src/Component.module.scss",
        nested,
        nested_candidates.candidates.as_slice(),
    );
    assert_eq!(
        missing_custom_property_messages(&nested_diagnostics),
        vec!["CSS custom property '--b' not found in indexed style tokens."]
    );
    Ok(())
}

fn unused_selector_messages(summary: &OmenaQueryStyleDiagnosticsForFileV0) -> Vec<&str> {
    summary
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "unusedSelector")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn sass_use_cycle_messages(summary: &OmenaQueryStyleDiagnosticsForFileV0) -> Vec<&str> {
    summary
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "sassUseCycle")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

// RFC-0007-E2 (#45): the `@use`/`@forward` cycle signal is already computed in
// `summarize_sass_module_cross_file_resolution` but no diagnostic read it. The cross-file
// resolution is `error`-severity in dart-sass; this proves the now-wired consumer fires on a real
// `a <-> b` module loop and anchors to the `@use` statement that closes it.
#[test]
fn style_diagnostics_for_workspace_file_flag_sass_use_cycle() -> Result<(), &'static str> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_a.scss".to_string(),
            style_source: r#"@use "./b";"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_b.scss".to_string(),
            style_source: r#"@use "./a";"#.to_string(),
        },
    ];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/_a.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"sassUseCycleDiagnostics")
    );
    let cycle_diagnostics = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "sassUseCycle")
        .collect::<Vec<_>>();
    assert_eq!(cycle_diagnostics.len(), 1, "exactly one cycle on _a.scss");
    let cycle = cycle_diagnostics[0];
    assert_eq!(cycle.severity, "error");
    assert!(
        cycle.message.contains("/tmp/_a.scss")
            && cycle.message.contains("/tmp/_b.scss")
            && cycle.message.contains("Sass module loop"),
        "message names the loop: {}",
        cycle.message
    );
    // The squiggle must land on the `@use "./b";` statement, not the whole file.
    assert_eq!(cycle.range.start.line, 0);
    assert!(cycle.range.end.character > cycle.range.start.character);
    Ok(())
}

// Self-loop (`@use './self'`) is likewise a hard error in dart-sass.
#[test]
fn style_diagnostics_for_workspace_file_flag_sass_use_self_loop() -> Result<(), &'static str> {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/_self.scss".to_string(),
        style_source: r#"@use "./self";"#.to_string(),
    }];

    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/_self.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;

    let messages = sass_use_cycle_messages(&diagnostics);
    assert_eq!(messages.len(), 1, "self-loop emits one cycle: {messages:?}");
    assert!(messages[0].contains("/tmp/_self.scss"));
    Ok(())
}

// Over-correction guard (#45): an acyclic `a -> b -> c` `@use` chain MUST emit no cycle
// diagnostic. A fix that silenced the FP by suppressing genuine cycles would also break the test
// above; this proves the consumer is cycle-discriminating, not blanket-silent.
#[test]
fn style_diagnostics_for_workspace_file_acyclic_use_chain_has_no_cycle() -> Result<(), &'static str>
{
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_a.scss".to_string(),
            style_source: r#"@use "./b";"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_b.scss".to_string(),
            style_source: r#"@use "./c";"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_c.scss".to_string(),
            style_source: r#"$gap: 1rem;"#.to_string(),
        },
    ];

    for entry in ["/tmp/_a.scss", "/tmp/_b.scss", "/tmp/_c.scss"] {
        let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
            entry,
            sources.as_slice(),
            &[],
            &[],
            None,
        )
        .ok_or("workspace diagnostics")?;
        assert!(
            sass_use_cycle_messages(&diagnostics).is_empty(),
            "acyclic chain must not flag {entry}: {:?}",
            sass_use_cycle_messages(&diagnostics)
        );
    }
    Ok(())
}

// RFC-0007-E1 (#45): `@extend` target validation.
#[test]
fn missing_extend_target_fires_on_unresolved_non_optional_targets() {
    let messages = crate::summarize_omena_query_missing_extend_target_diagnostics(
        "App.module.scss",
        ".a { @extend %nonexistent; } .b { @extend .missing; }",
    )
    .into_iter()
    .map(|diagnostic| {
        assert_eq!(diagnostic.code, "missingExtendTarget");
        assert_eq!(diagnostic.severity, "error");
        diagnostic.message
    })
    .collect::<Vec<_>>();
    assert_eq!(messages.len(), 2);
    assert!(
        messages
            .iter()
            .any(|message| message.contains("'%nonexistent'"))
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("'.missing'"))
    );
}

#[test]
fn missing_extend_target_skips_valid_targets_and_optional_flag() {
    // A resolvable `%real` / `.base`, and a missing-but-`!optional` target, all stay silent.
    let diagnostics = crate::summarize_omena_query_missing_extend_target_diagnostics(
        "App.module.scss",
        "%real { color: red; } .base { color: blue; } \
         .a { @extend %real; } .b { @extend .base; } .c { @extend %gone !optional; }",
    );
    assert!(
        diagnostics.is_empty(),
        "valid + optional extends must not fire: {:?}",
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn missing_extend_target_keeps_placeholder_and_class_namespaces_distinct() {
    // `%foo` declared but `.foo` extended: dart-sass errors (the class selector does not exist).
    let diagnostics = crate::summarize_omena_query_missing_extend_target_diagnostics(
        "App.module.scss",
        "%foo { color: red; } .b { @extend .foo; }",
    );
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message.contains("'.foo'"));
}

#[test]
fn missing_extend_target_workspace_resolves_cross_file_placeholder() -> Result<(), &'static str> {
    // A `%base` declared in an imported partial must NOT false-positive; a target declared nowhere
    // in the corpus still fires.
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: "@use \"base\";\n.a { @extend %base; }\n.b { @extend %gone; }"
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_base.scss".to_string(),
            style_source: "%base { color: red; }".to_string(),
        },
    ];
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;
    let extend_messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingExtendTarget")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(extend_messages.len(), 1, "got {extend_messages:?}");
    assert!(extend_messages[0].contains("'%gone'"));
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"missingExtendTargetDiagnostics")
    );
    Ok(())
}

#[test]
fn missing_extend_target_workspace_fires_on_unrelated_non_imported_placeholder()
-> Result<(), &'static str> {
    // RFC-0007-E1 (#45): an `@extend %lonely` where `%lonely` is declared ONLY in an unrelated file
    // that the target does NOT `@use`/`@forward`/`@import` must fire — the corpus-wide union used to
    // (wrongly) suppress it. The over-correction guard: a `%shared` placeholder that IS reachable via
    // `@use "shared"` stays silent in the same file.
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: "@use \"shared\";\n.a { @extend %shared; }\n.b { @extend %lonely; }"
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_shared.scss".to_string(),
            style_source: "%shared { color: red; }".to_string(),
        },
        // Unrelated: declares `%lonely` but is never imported by App.module.scss.
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/unrelated.module.scss".to_string(),
            style_source: "%lonely { color: blue; }".to_string(),
        },
    ];
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;
    let extend_messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingExtendTarget")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    // Exactly the unrelated-file target fires; the reachable `%shared` stays silent.
    assert_eq!(extend_messages.len(), 1, "got {extend_messages:?}");
    assert!(
        extend_messages[0].contains("'%lonely'"),
        "got {extend_messages:?}"
    );
    Ok(())
}

#[test]
fn missing_extend_target_workspace_resolves_class_through_transitive_forward()
-> Result<(), &'static str> {
    // RFC-0007-E1 (#45): a class reachable through a transitive `@forward` chain (App -> mid ->
    // leaf) must NOT fire — the closure walk follows resolved `@use`/`@forward` edges transitively.
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: "@use \"mid\";\n.a { @extend .leaf; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_mid.scss".to_string(),
            style_source: "@forward \"leaf\";".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_leaf.scss".to_string(),
            style_source: ".leaf { color: red; }".to_string(),
        },
    ];
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;
    let extend_messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingExtendTarget")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        extend_messages.is_empty(),
        "transitively-reachable .leaf must not fire: {extend_messages:?}"
    );
    Ok(())
}

// RFC-0007-E3 (#45): unresolved workspace-local Sass `@import`/`@use`.
#[test]
fn unresolved_sass_import_fires_on_local_path_but_not_external_or_bare() -> Result<(), &'static str>
{
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: "@import \"./missing\";\n@use \"../gone\";\n@use \"sass:math\";\n\
                           @import \"https://cdn.example/x.css\";\n@import \"bare-partial\";\n\
                           @use \"./present\";\n.a { color: red; }"
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/_present.scss".to_string(),
            style_source: "$x: 1;".to_string(),
        },
    ];
    let diagnostics = crate::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/App.module.scss",
        sources.as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;
    let module_messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingModule")
        .map(|diagnostic| {
            assert_eq!(diagnostic.severity, "error");
            diagnostic.message.as_str()
        })
        .collect::<Vec<_>>();
    // Only the relative `./missing` and `../gone` fire; sass:/https/bare-partial/resolved stay silent.
    assert_eq!(module_messages.len(), 2, "got {module_messages:?}");
    assert!(
        module_messages
            .iter()
            .any(|message| message.contains("'./missing'"))
    );
    assert!(
        module_messages
            .iter()
            .any(|message| message.contains("'../gone'"))
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"unresolvedSassImportDiagnostics")
    );
    Ok(())
}

// RFC-0007-E4 (#45): nested `@at-root <selector> {}` is included in cascade analysis.
#[test]
fn at_root_selector_block_is_included_in_cascade_analysis() {
    let candidates = crate::summarize_omena_query_style_hover_candidates("App.module.scss", "")
        .map(|summary| summary.candidates)
        .unwrap_or_default();
    let fired = crate::summarize_omena_query_style_diagnostics_for_file(
        "App.module.scss",
        ".a { @at-root .b { color: red; color: blue; } }",
        candidates.as_slice(),
    );
    let codes = fired
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&"unreachableDeclaration"),
        "duplicate `color` inside @at-root .b must be analyzed: {codes:?}"
    );

    // Control: a non-duplicate @at-root selector block must NOT fire.
    let clean = crate::summarize_omena_query_style_diagnostics_for_file(
        "App.module.scss",
        ".a { @at-root .b { color: red; } }",
        candidates.as_slice(),
    );
    assert!(
        clean
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "unreachableDeclaration"),
        "non-duplicate @at-root block must stay clean: {:?}",
        clean
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    );
}

#[test]
fn cascade_smt_violation_surfaces_for_unsatisfiable_box_shorthand_obligation()
-> Result<(), &'static str> {
    // #38 / L8: the SMT precision-checker family is wired onto the real product
    // path. A selector that declares the complete canonical `margin` longhand
    // quartet is a box-shorthand combination candidate; the
    // omena-query-checker-orchestrator smt-gate builds the canonical obligation
    // from the parsed longhands and runs the real evaluate_omena_checker_smt_rules
    // mechanism (default StubSmtBackendV0 propositional backend).
    //
    // Here the last longhand is `!important`, so the obligation's
    // `no-important-longhand` precondition is violated, the backend verdict on the
    // conjunction is Unsat, and `cascadeSmtViolation` is surfaced. (Deep-analysis
    // only — the diagnostic is off on the default surface.)
    let unsat = r#"
.box {
  margin-top: 1px;
  margin-right: 2px;
  margin-bottom: 3px;
  margin-left: 4px !important;
}
"#;
    let unsat_candidates =
        crate::summarize_omena_query_style_hover_candidates("Box.module.css", unsat)
            .ok_or("unsat candidates")?;

    // Default surface: the SMT theory diagnostic must NOT appear.
    let default_diagnostics = crate::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Box.module.css",
        unsat,
        unsat_candidates.candidates.as_slice(),
    );
    assert!(
        default_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "cascadeSmtViolation"),
        "smt violation must be off on the default surface: {:?}",
        default_diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    );

    // Deep-analysis surface: the real Unsat verdict surfaces the violation, with
    // the orchestrator gate + backend-check provenance attached.
    let deep_diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
            "file:///workspace/src/Box.module.css",
            unsat,
            unsat_candidates.candidates.as_slice(),
            true,
        );
    let smt_violation = deep_diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "cascadeSmtViolation")
        .ok_or("cascadeSmtViolation must fire on an unsatisfiable box-shorthand obligation")?;
    assert_eq!(smt_violation.severity, "warning");
    assert!(
        smt_violation
            .provenance
            .contains(&"omena-query-checker-orchestrator.smt-gate"),
        "smt-gate provenance must be attached: {:?}",
        smt_violation.provenance
    );
    assert!(
        smt_violation
            .provenance
            .contains(&"omena-smt.backend-check"),
        "omena-smt.backend-check mechanism provenance must be attached: {:?}",
        smt_violation.provenance
    );

    // Satisfiable counterpart: the SAME canonical `margin` quartet with no
    // `!important` longhand and adjacent source order. Every precondition holds,
    // the backend verdict is Sat, and nothing is surfaced even under deep
    // analysis. If the solver verdict were replaced by a constant the Unsat case
    // would still emit but so would this one — so a satisfiable obligation
    // emitting nothing is the mutation guard.
    let sat = r#"
.box {
  margin-top: 1px;
  margin-right: 2px;
  margin-bottom: 3px;
  margin-left: 4px;
}
"#;
    let sat_candidates = crate::summarize_omena_query_style_hover_candidates("Box.module.css", sat)
        .ok_or("sat candidates")?;
    let sat_diagnostics =
        crate::summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
            "file:///workspace/src/Box.module.css",
            sat,
            sat_candidates.candidates.as_slice(),
            true,
        );
    assert!(
        sat_diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "cascadeSmtViolation"),
        "a satisfiable box-shorthand obligation must surface no smt violation: {:?}",
        sat_diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    );
    Ok(())
}
