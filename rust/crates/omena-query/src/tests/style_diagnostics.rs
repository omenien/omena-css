use crate::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStyleDiagnosticsForFileV0,
    OmenaQueryStyleSourceInputV0, ParserPositionV0, ParserRangeV0,
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
                        "omena-query.source-selector-usage"
                    ])
    );
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

fn unused_selector_messages(summary: &OmenaQueryStyleDiagnosticsForFileV0) -> Vec<&str> {
    summary
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "unusedSelector")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}
