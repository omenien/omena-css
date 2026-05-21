use crate::{ParserPositionV0, ParserRangeV0};

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
    assert_eq!(linear_provenance.term_count, 2);

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
