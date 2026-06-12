use crate::{
    OmenaQuerySourceImportedStyleBindingV0, OmenaQuerySourceMissingSelectorDiagnosticCandidateV0,
    OmenaQuerySourceSelectorCandidateV0, OmenaQuerySourceSelectorReferenceEditTargetV0,
    OmenaQuerySourceSelectorReferenceFactV0, OmenaQuerySourceSelectorReferenceMatchKindV0,
    OmenaQuerySourceSyntaxIndexV0, OmenaQueryStyleSelectorDefinitionV0,
    OmenaQueryStyleSourceInputV0, ParserByteSpanV0, ParserPositionV0, ParserRangeV0,
    canonicalize_omena_query_source_selector_references, is_omena_query_sass_symbol_candidate_kind,
    is_omena_query_sass_symbol_reference_kind, omena_query_sass_symbol_kind_from_candidate_kind,
    omena_query_sass_symbol_target_matches, resolve_omena_query_sass_forward_sources,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    resolve_omena_query_sass_symbol_declarations, resolve_omena_query_selector_rename_edits,
    resolve_omena_query_source_candidate_selector_names,
    resolve_omena_query_source_provider_candidates,
    resolve_omena_query_style_selector_definitions_for_source_candidate,
    resolve_omena_query_style_uri_for_specifier, summarize_omena_query_sass_module_sources,
    summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_source_diagnostics_for_workspace_file_with_context_depth,
    summarize_omena_query_source_diagnostics_for_workspace_file_with_source_syntax_index,
    summarize_omena_query_source_import_declarations, summarize_omena_query_source_syntax_index,
    summarize_omena_query_style_hover_candidates,
};

#[test]
fn source_candidate_matching_normalizes_percent_encoded_file_uris() {
    let source_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 4,
        },
    };
    let definition_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 1,
            character: 1,
        },
        end: ParserPositionV0 {
            line: 1,
            character: 5,
        },
    };
    let candidate = OmenaQuerySourceSelectorCandidateV0 {
        kind: "sourceSelectorPrefixReference",
        name: "btn-".to_string(),
        range: source_range,
        source: "omenaQuerySourceSyntaxIndex",
        target_style_uri: Some(
            "file:///workspace/app/%28marketing%29/Button.module.scss".to_string(),
        ),
    };
    let definitions = vec![OmenaQueryStyleSelectorDefinitionV0 {
        uri: "file:///workspace/app/(marketing)/Button.module.scss".to_string(),
        name: "btn-primary".to_string(),
        range: definition_range,
    }];

    assert_eq!(
        resolve_omena_query_source_candidate_selector_names(
            &candidate,
            definitions.as_slice(),
            None,
        ),
        vec!["btn-primary".to_string()]
    );
}

#[test]
fn source_syntax_index_adapter_is_query_owned_without_changing_product() {
    let style_uri = resolve_omena_query_style_uri_for_specifier(
        "file:///workspace/src/Button.tsx",
        Some("file:///workspace"),
        "./Button.module.scss",
    );
    assert_eq!(
        style_uri.as_deref(),
        Some("file:///workspace/src/Button.module.scss")
    );
    let style_uri = style_uri.unwrap_or_default();
    assert_eq!(style_uri, "file:///workspace/src/Button.module.scss");

    let import_summary = summarize_omena_query_source_import_declarations(
        "import styles from './Button.module.scss';",
    );
    assert_eq!(import_summary.import_count, 1);
    assert_eq!(import_summary.imports[0].binding, "styles");

    let source = "import styles from './Button.module.scss';\nconst el = styles.root;\n";
    let mut index = summarize_omena_query_source_syntax_index(
        source,
        vec![OmenaQuerySourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri,
        }],
        Vec::new(),
    );
    assert_eq!(index.product, "omena-bridge.source-syntax-index");
    assert_eq!(index.selector_references.len(), 1);
    let reference = &index.selector_references[0];
    assert_eq!(
        &source[reference.byte_span.start..reference.byte_span.end],
        "root"
    );

    canonicalize_omena_query_source_selector_references(&mut index.selector_references);
    assert_eq!(index.selector_references.len(), 1);
}

#[test]
fn source_diagnostics_for_file_are_query_owned() {
    let diagnostics = summarize_omena_query_source_diagnostics_for_file(
        "file:///workspace/src/App.tsx",
        &[OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 {
            target_style_uri: "file:///workspace/src/App.module.scss".to_string(),
            target_style_source: ".root {\n}\n".to_string(),
            selector_name: "missing".to_string(),
            source_reference_range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 2,
                    character: 18,
                },
                end: ParserPositionV0 {
                    line: 2,
                    character: 25,
                },
            },
        }],
    );

    assert_eq!(diagnostics.product, "omena-query.diagnostics-for-file");
    assert_eq!(diagnostics.file_kind, "source");
    assert_eq!(diagnostics.diagnostic_count, 1);
    assert_eq!(diagnostics.diagnostics[0].code, "missingSelector");
    assert_eq!(
        diagnostics.diagnostics[0].provenance.as_slice(),
        [
            "omena-query.source-syntax-index",
            "omena-query.style-selector-definitions",
            "omena-query-checker-orchestrator.product-diagnostic-gate",
            "omena-checker.rule-registry",
        ]
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"crossLanguageDiagnostics")
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"checkerProductDiagnosticGate")
    );
}

#[test]
fn source_diagnostics_for_workspace_file_are_query_owned() {
    let diagnostics = summarize_omena_query_source_diagnostics_for_workspace_file(
        "/workspace/src/App.tsx",
        r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
import missing from "./Missing.module.scss";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
const dynamicPrefix = "lost-" + suffix;
export function App({ suffix }) {
  return <div className={cx("ghost", variant, dynamicPrefix, `empty-${suffix}`)} data-x={styles.ghost} />;
}"#,
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: ".root {}\n.chip {}\n".to_string(),
        }],
        &[],
    );

    let codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.product, "omena-query.diagnostics-for-file");
    assert_eq!(diagnostics.file_kind, "source");
    assert!(codes.contains(&"missingModule"));
    assert!(codes.contains(&"missingStaticClass"));
    assert!(codes.contains(&"missingResolvedClassValues"));
    assert!(codes.contains(&"missingResolvedClassDomain"));
    assert!(codes.contains(&"missingTemplatePrefix"));
    let checker_product_diagnostic_provenance = [
        "omena-query.source-syntax-index",
        "omena-query.style-selector-definitions",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "omena-checker.rule-registry",
    ];
    for code in ["missingStaticClass", "missingTemplatePrefix"] {
        assert_eq!(
            diagnostics
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code == code)
                .map(|diagnostic| diagnostic.provenance.as_slice()),
            Some(checker_product_diagnostic_provenance.as_slice())
        );
    }
    assert_eq!(
        diagnostics
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "missingModule")
            .map(|diagnostic| diagnostic.provenance.as_slice()),
        Some(
            [
                "omena-query.source-import-declarations",
                "omena-resolver.style-module-resolution",
                "omena-query-checker-orchestrator.product-diagnostic-gate",
                "omena-checker.rule-registry",
            ]
            .as_slice()
        )
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"sourceResolvedClassDiagnostics")
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"checkerProductDiagnosticGate")
    );
}

#[test]
fn source_diagnostics_consume_precomputed_source_syntax_index() {
    let source_path = "/workspace/src/App.tsx";
    let source = "const view = styles.ghost;";
    assert!(source.contains("ghost"), "fixture should contain selector");
    let selector_start = source.find("ghost").unwrap_or_default();
    let style_uri = "/workspace/src/App.module.scss";
    let diagnostics =
        summarize_omena_query_source_diagnostics_for_workspace_file_with_source_syntax_index(
            source_path,
            source,
            &OmenaQuerySourceSyntaxIndexV0 {
                schema_version: "0",
                product: "omena-bridge.source-syntax-index",
                imported_style_bindings: vec![OmenaQuerySourceImportedStyleBindingV0 {
                    binding: "styles".to_string(),
                    style_uri: style_uri.to_string(),
                }],
                class_string_literals: Vec::new(),
                style_property_accesses: Vec::new(),
                inline_style_declarations: Vec::new(),
                selector_references: vec![OmenaQuerySourceSelectorReferenceFactV0 {
                    byte_span: ParserByteSpanV0 {
                        start: selector_start,
                        end: selector_start + "ghost".len(),
                    },
                    selector_name: Some("ghost".to_string()),
                    match_kind: OmenaQuerySourceSelectorReferenceMatchKindV0::Exact,
                    target_style_uri: Some(style_uri.to_string()),
                }],
                type_fact_targets: Vec::new(),
                class_value_universes: Vec::new(),
                domain_class_references: Vec::new(),
            },
            &[OmenaQueryStyleSourceInputV0 {
                style_path: style_uri.to_string(),
                style_source: ".root {}".to_string(),
            }],
        );

    assert_eq!(diagnostics.product, "omena-query.diagnostics-for-file");
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"sourceIndexedSyntaxDiagnostics")
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missingStaticClass"),
        "precomputed source syntax index should drive source diagnostics without reparsing imports: {:?}",
        diagnostics.diagnostics
    );
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "missingModule"),
        "the precomputed-index path should not synthesize import-resolution diagnostics"
    );
}

#[test]
fn source_diagnostics_surface_variant_recipe_option_universe() {
    let diagnostics = summarize_omena_query_source_diagnostics_for_workspace_file(
        "/workspace/src/App.tsx",
        r#"import { cva } from "class-variance-authority";
const button = cva("btn", {
  variants: {
    intent: {
      primary: "btn-primary",
      secondary: "btn-secondary",
    },
  },
});
button({ intent: "primary" });
button({ intent: "ghost" });
"#,
        &[],
        &[],
    );

    let missing_options = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingClassValueOption")
        .collect::<Vec<_>>();
    assert_eq!(missing_options.len(), 1, "{diagnostics:?}");
    assert!(
        missing_options[0]
            .message
            .contains("Class value option 'ghost' is not defined for button.intent")
    );
    assert_eq!(
        missing_options[0].provenance,
        vec![
            "omena-bridge.class-value-universe-provider",
            "omena-query.source-domain-class-references"
        ]
    );
}

#[test]
fn source_provider_candidate_resolution_is_query_owned() {
    let source_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 4,
        },
    };
    let definition_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 1,
            character: 1,
        },
        end: ParserPositionV0 {
            line: 1,
            character: 5,
        },
    };

    let resolution = resolve_omena_query_source_provider_candidates(
        vec![
            OmenaQuerySourceSelectorCandidateV0 {
                kind: "sourceSelectorReference",
                name: "root".to_string(),
                range: source_range,
                source: "omenaQuerySourceSyntaxIndex",
                target_style_uri: Some("file:///workspace/src/App.module.scss".to_string()),
            },
            OmenaQuerySourceSelectorCandidateV0 {
                kind: "sourceSelectorPrefixReference",
                name: "btn-".to_string(),
                range: source_range,
                source: "omenaQuerySourceSyntaxIndex",
                target_style_uri: Some("file:///workspace/src/App.module.scss".to_string()),
            },
            OmenaQuerySourceSelectorCandidateV0 {
                kind: "sourceSelectorReference",
                name: "ghost".to_string(),
                range: source_range,
                source: "omenaQuerySourceSyntaxIndex",
                target_style_uri: Some("file:///workspace/src/Other.module.scss".to_string()),
            },
        ],
        &[
            OmenaQueryStyleSelectorDefinitionV0 {
                uri: "file:///workspace/src/App.module.scss".to_string(),
                name: "root".to_string(),
                range: definition_range,
            },
            OmenaQueryStyleSelectorDefinitionV0 {
                uri: "file:///workspace/src/App.module.scss".to_string(),
                name: "btn-primary".to_string(),
                range: definition_range,
            },
        ],
    );

    assert_eq!(
        resolution
            .matched
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<Vec<_>>(),
        vec!["btn-", "root"]
    );
    assert_eq!(
        resolution
            .unresolved
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<Vec<_>>(),
        vec!["ghost"]
    );

    let prefix_candidate = &resolution.matched[0];
    let definitions = vec![
        OmenaQueryStyleSelectorDefinitionV0 {
            uri: "file:///workspace/src/App.module.scss".to_string(),
            name: "root".to_string(),
            range: definition_range,
        },
        OmenaQueryStyleSelectorDefinitionV0 {
            uri: "file:///workspace/src/App.module.scss".to_string(),
            name: "btn-primary".to_string(),
            range: definition_range,
        },
    ];
    assert_eq!(
        resolve_omena_query_source_candidate_selector_names(
            prefix_candidate,
            definitions.as_slice(),
            None
        ),
        vec!["btn-primary".to_string()]
    );
    assert_eq!(
        resolve_omena_query_style_selector_definitions_for_source_candidate(
            prefix_candidate,
            definitions.as_slice(),
        )
        .into_iter()
        .map(|definition| definition.name)
        .collect::<Vec<_>>(),
        vec!["btn-primary".to_string()]
    );
}

#[test]
fn selector_rename_edit_planning_is_query_owned() {
    let source_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 3,
            character: 16,
        },
        end: ParserPositionV0 {
            line: 3,
            character: 20,
        },
    };
    let definition_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 1,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 5,
        },
    };

    let edits = resolve_omena_query_selector_rename_edits(
        "root",
        ".shell",
        Some("file:///workspace/src/App.module.scss"),
        &[OmenaQueryStyleSelectorDefinitionV0 {
            uri: "file:///workspace/src/App.module.scss".to_string(),
            name: "root".to_string(),
            range: definition_range,
        }],
        &[OmenaQuerySourceSelectorReferenceEditTargetV0 {
            uri: "file:///workspace/src/App.tsx".to_string(),
            name: "root".to_string(),
            range: source_range,
            target_style_uri: Some("file:///workspace/src/App.module.scss".to_string()),
        }],
    );

    assert_eq!(
        edits
            .iter()
            .map(|edit| (edit.uri.as_str(), edit.new_text.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("file:///workspace/src/App.module.scss", "shell"),
            ("file:///workspace/src/App.tsx", "shell"),
        ]
    );
}

#[test]
fn sass_symbol_matching_is_query_owned() {
    let source = "$accent: red;\n.button { color: $accent; }\n";
    let Some(candidates) =
        summarize_omena_query_style_hover_candidates("Component.module.scss", source)
    else {
        return;
    };

    assert!(is_omena_query_sass_symbol_candidate_kind(
        "sassVariableDeclaration"
    ));
    assert!(is_omena_query_sass_symbol_reference_kind(
        "sassVariableReference"
    ));
    assert_eq!(
        omena_query_sass_symbol_kind_from_candidate_kind("sassVariableReference"),
        Some("variable")
    );
    assert!(omena_query_sass_symbol_target_matches(
        "sassVariableReference",
        "accent",
        None,
        "sassVariableDeclaration",
        "accent",
        None,
    ));

    let declarations = resolve_omena_query_sass_symbol_declarations(
        candidates.candidates.as_slice(),
        "variable",
        "accent",
    );
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].kind, "sassVariableDeclaration");
}

#[test]
fn sass_module_sources_are_query_owned() {
    let sources = summarize_omena_query_sass_module_sources(
        "Component.module.scss",
        r#"
@use "./tokens" as tokens;
@use "./reset" as *;
@use "sass:map";
@import "./legacy";
@forward "./theme";
@forward "sass:color";
"#,
    );
    assert!(sources.is_some());
    let Some(sources) = sources else {
        return;
    };

    assert_eq!(sources.product, "omena-query.sass-module-sources");
    assert!(sources.module_use_edges.iter().any(|edge| {
        edge.source == "./tokens"
            && edge.namespace.as_deref() == Some("tokens")
            && edge.namespace_kind == "alias"
    }));
    assert!(sources.module_use_edges.iter().any(|edge| {
        edge.source == "./reset" && edge.namespace.is_none() && edge.namespace_kind == "wildcard"
    }));
    assert!(sources.module_use_edges.iter().any(|edge| {
        edge.source == "./legacy" && edge.namespace.is_none() && edge.namespace_kind == "wildcard"
    }));
    assert_eq!(
        resolve_omena_query_sass_module_use_sources_for_candidate(&sources, None),
        vec!["./legacy".to_string(), "./reset".to_string()]
    );
    assert_eq!(
        resolve_omena_query_sass_module_use_sources_for_candidate(&sources, Some("tokens"),),
        vec!["./tokens".to_string()]
    );
    assert_eq!(
        resolve_omena_query_sass_forward_sources(&sources),
        vec!["./theme".to_string()]
    );
}

/// Non-tautological mechanism-depth test on the REAL default workspace product
/// path (`summarize_omena_query_source_diagnostics_for_workspace_file`, the
/// function the napi/wasm/CLI consumers forward to).
///
/// The fixture has two dynamic className template projections that interpolate
/// the same `variant` binding: `btn-${variant}` (whose `btn-` prefix matches the
/// indexed `btn-primary` / `btn-secondary` selectors) and `xyz-${variant}`
/// (whose `xyz-` prefix matches no indexed selector). Both call sites are
/// harvested from the syntax index inside the default path and flowed through the
/// real k-limited (k-CFA) M-tier gate.
///
/// At the context-insensitive baseline `k = 0` the two call sites share the
/// `variant` callee and collapse into one `<root>` context: their `btn-` and
/// `xyz-` prefixes join to the longest-common-prefix `""`, i.e. `Top`, which
/// projects across the whole selector universe and never raises
/// `noUnknownDynamicClass`. At the context-sensitive default `k` the call sites
/// stay separate, so the `xyz-` projection narrows to the empty selector set and
/// the harvested default path raises `noUnknownDynamicClass` at the `xyz-`
/// reference range.
///
/// The load-bearing assertion is the differential between the default
/// (context-sensitive) path and the `k = 0` baseline. If `analyze_k_limited_call_site_flows`
/// were replaced by a constant/identity (always k = 0 collapse, or never
/// joining), both runs would emit the same diagnostic set and the differential
/// would fail — so this is not a tautology.
#[test]
fn workspace_source_diagnostics_harvest_context_sensitive_m_tier_flow() {
    let source_path = "/workspace/src/Button.tsx";
    let source = r#"import styles from "./Button.module.scss";
export function Button({ variant }) {
  return (
    <div>
      <span className={`btn-${variant}`} />
      <span className={`xyz-${variant}`} />
    </div>
  );
}"#;
    let style_sources = [OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/Button.module.scss".to_string(),
        style_source: ".btn-primary {}\n.btn-secondary {}\n".to_string(),
    }];

    let context_sensitive = summarize_omena_query_source_diagnostics_for_workspace_file(
        source_path,
        source,
        &style_sources,
        &[],
    );
    let context_insensitive =
        summarize_omena_query_source_diagnostics_for_workspace_file_with_context_depth(
            source_path,
            source,
            &style_sources,
            &[],
            0,
        );

    assert_eq!(context_sensitive.file_kind, "source");

    // The harvested M-tier diagnostics carry the k-limited flow provenance, so
    // they are demonstrably the real mechanism, not a hardcoded source warning.
    let context_sensitive_m_tier = context_sensitive
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic
                .provenance
                .contains(&"omena-abstract-value.k-limited-call-site-flow")
        })
        .collect::<Vec<_>>();
    assert!(
        !context_sensitive_m_tier.is_empty(),
        "default workspace path must emit harvested k-CFA M-tier diagnostics without an external producer"
    );

    // Context-sensitive default k: the `xyz-` site separates and trips
    // noUnknownDynamicClass; the context-insensitive baseline joins it into Top
    // and never does.
    let unknown_class_present = |summary: &crate::OmenaQuerySourceDiagnosticsForFileV0| {
        summary
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "noUnknownDynamicClass")
    };
    assert!(
        unknown_class_present(&context_sensitive),
        "context-sensitive flow must raise noUnknownDynamicClass for the unmatched xyz- projection"
    );
    assert!(
        !unknown_class_present(&context_insensitive),
        "context-insensitive (k=0) collapse joins the prefixes to Top and must NOT raise noUnknownDynamicClass"
    );

    // The differential: the emitted diagnostic set must change with the
    // context-depth bound, not just metadata.
    let context_sensitive_codes = context_sensitive
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.range.start.line))
        .collect::<Vec<_>>();
    let context_insensitive_codes = context_insensitive
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.range.start.line))
        .collect::<Vec<_>>();
    assert_ne!(
        context_sensitive_codes, context_insensitive_codes,
        "k-limiting must change which workspace M-tier diagnostics are emitted"
    );
}

/// Over-correction guard for the M8 k-CFA M-tier FP cleanup (WP7-a).
///
/// `no-imprecise-value` must NOT fire on harvested affix templates: an
/// interpolation is inherently imprecise, so a hint per template is
/// information-free noise. This asserts the demotion is real while the
/// load-bearing `no-unknown-dynamic-class` true positive on the unmatched `xyz-`
/// projection is still preserved (so the demotion did not also silence the real
/// finding).
#[test]
fn workspace_source_diagnostics_suppress_imprecise_value_noise_on_harvested_templates() {
    let source_path = "/workspace/src/Button.tsx";
    let source = r#"import styles from "./Button.module.scss";
export function Button({ variant }) {
  return (
    <div>
      <span className={`btn-${variant}`} />
      <span className={`xyz-${variant}`} />
    </div>
  );
}"#;
    let style_sources = [OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/Button.module.scss".to_string(),
        style_source: ".btn-primary {}\n.btn-secondary {}\n".to_string(),
    }];

    let summary = summarize_omena_query_source_diagnostics_for_workspace_file(
        source_path,
        source,
        &style_sources,
        &[],
    );

    // FP #2 gone: no information-free `noImpreciseValue` hint on either template.
    assert!(
        summary
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "noImpreciseValue"),
        "harvested affix templates must not emit information-free noImpreciseValue hints, got {:?}",
        summary
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>()
    );

    // True positive preserved: the unmatched `xyz-` projection still flags
    // noUnknownDynamicClass at its own reference range, and the matched `btn-`
    // projection (line 4) stays clean.
    let unknown_class = summary
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "noUnknownDynamicClass")
        .collect::<Vec<_>>();
    assert_eq!(
        unknown_class.len(),
        1,
        "exactly the unmatched xyz- projection must flag noUnknownDynamicClass"
    );
    assert_eq!(
        unknown_class[0].range.start.line, 5,
        "noUnknownDynamicClass must anchor to the unmatched xyz- template, not the matched btn- one"
    );
}

/// Over-correction guard for the SOUND module-scoping half of WP7-a.
///
/// A `cx(`btn-${variant}`)` call is bound (via `classnames/bind`) to a SPECIFIC
/// imported CSS Module, so the harvested type-fact target carries that module's
/// resolved `target_style_uri`. `no-unknown-dynamic-class` must therefore be
/// evaluated against ONLY the bound module's selectors, not the union of every
/// imported module:
///
/// - bound to a module that HAS `btn-*` selectors -> provably non-empty -> clean;
/// - bound to a module that has NO `btn-*` selectors -> provably empty ->
///   `no-unknown-dynamic-class` STILL fires, even though a DIFFERENT imported
///   module happens to define `btn-*` (the union would have masked the bug).
#[test]
fn workspace_source_diagnostics_scope_unknown_dynamic_class_to_bound_module() {
    let style_sources = [
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/A.module.scss".to_string(),
            style_source: ".btn-primary {}\n.btn-secondary {}\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/B.module.scss".to_string(),
            style_source: ".card {}\n.panel {}\n".to_string(),
        },
    ];

    let unknown_class_count = |bind_target: &str| {
        let source = format!(
            r#"import bind from "classnames/bind";
import a from "./A.module.scss";
import b from "./B.module.scss";
const cx = bind.bind({bind_target});
export function App({{ variant }}) {{
  return <div className={{cx(`btn-${{variant}}`)}} />;
}}"#
        );
        summarize_omena_query_source_diagnostics_for_workspace_file(
            "/workspace/src/App.tsx",
            &source,
            &style_sources,
            &[],
        )
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "noUnknownDynamicClass")
        .count()
    };

    // Bound to module A, which HAS btn-* selectors: scoped intersection is
    // non-empty -> NO false positive.
    assert_eq!(
        unknown_class_count("a"),
        0,
        "btn- bound to a module that HAS btn-* selectors must not flag noUnknownDynamicClass"
    );

    // Bound to module B, which has NO btn-* selectors: scoped intersection is
    // provably empty -> the genuine bug STILL fires, even though module A (a
    // different import) defines btn-* (the union universe would have masked it).
    assert_eq!(
        unknown_class_count("b"),
        1,
        "btn- bound to a module with NO btn-* selectors must still flag noUnknownDynamicClass \
         (scoped to the bound module, not cross-matched against the union)"
    );
}
