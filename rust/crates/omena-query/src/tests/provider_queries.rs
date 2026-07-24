use crate::{
    AbstractPropertyValueV0, OmenaQueryCompletionCandidateV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceImportedStyleBindingV0, OmenaQuerySourceSelectorReferenceCandidateV0,
    OmenaQuerySourceSelectorReferenceEditTargetV0, OmenaQuerySourceSelectorReferenceFactV0,
    OmenaQuerySourceSelectorReferenceMatchKindV0, OmenaQuerySourceSelectorReferenceSurfaceV0,
    OmenaQuerySourceSyntaxIndexV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSelectorDefinitionV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTsconfigPathMappingV0, OmenaWorkspaceOccurrenceSurfaceV0, ParserByteSpanV0,
    ParserPositionV0, ParserRangeV0, resolve_omena_query_style_uri_for_specifier,
    summarize_omena_query_missing_selector_diagnostic, summarize_omena_query_refs_for_class,
    summarize_omena_query_refs_for_class_from_occurrence_index,
    summarize_omena_query_refs_for_workspace_class,
    summarize_omena_query_refs_for_workspace_class_with_resolution_inputs,
    summarize_omena_query_rename_plan, summarize_omena_query_rename_plan_from_occurrence_index,
    summarize_omena_query_source_completion_at_position,
    summarize_omena_query_source_selector_occurrence_index,
    summarize_omena_query_style_completion_at_position,
    summarize_omena_query_style_extract_code_actions, summarize_omena_query_style_hover_candidates,
    summarize_omena_query_style_hover_render_parts,
    summarize_omena_query_style_hover_render_parts_for_hover_position,
    summarize_omena_query_style_inline_code_actions,
    summarize_omena_query_style_inline_code_actions_with_resolution_inputs,
    summarize_omena_query_style_insight_code_actions, summarize_omena_query_style_insights,
};

#[test]
fn style_hover_candidates_are_query_owned() {
    let candidates = summarize_omena_query_style_hover_candidates(
        "Component.module.scss",
        r#"
@mixin variants($prefix, $map) {
  @each $name, $value in $map {
.#{$prefix}-#{$name} { color: $value; }
  }
}

@use "./tokens" as tokens;
$accent: red;
.button { color: var(--brand); }
:root { --brand: blue; }
@include variants($prefix: "tone", $map: ("warm": red));
.alert { color: tokens.$brand; @include tokens.tone(red); width: tokens.double(2px); }
"#,
    );
    assert!(candidates.is_some());
    let Some(candidates) = candidates else {
        return;
    };

    assert_eq!(candidates.product, "omena-query.style-hover-candidates");
    assert!(
        candidates
            .candidates
            .iter()
            .any(|candidate| candidate.kind == "selector" && candidate.name == "button")
    );
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "customPropertyReference" && candidate.name == "--brand"
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassVariableDeclaration" && candidate.name == "accent"
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassVariableReference"
            && candidate.name == "brand"
            && candidate.namespace.as_deref() == Some("tokens")
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassMixinInclude"
            && candidate.name == "tone"
            && candidate.namespace.as_deref() == Some("tokens")
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassFunctionCall"
            && candidate.name == "double"
            && candidate.namespace.as_deref() == Some("tokens")
    }));
    assert!(
        candidates
            .candidates
            .iter()
            .any(
                |candidate| candidate.source == "sassPartialEvaluatorGeneratedSelectors"
                    && candidate.name == "tone-warm"
            )
    );
}

#[test]
fn style_hover_render_parts_are_query_owned() {
    let source = r#"$accent: red !default;
@mixin tone($color) {
  color: $color;
}
.button { color: var(--brand); }
@media (min-width: 40rem) { @layer theme { .button { color: blue; } } }
"#;

    let variable = summarize_omena_query_style_hover_render_parts(
        source,
        "sassVariableDeclaration",
        "accent",
        ParserPositionV0 {
            line: 0,
            character: 1,
        },
    );
    assert_eq!(variable.product, "omena-query.style-hover-render-parts");
    assert_eq!(variable.value.as_deref(), Some("red"));
    assert_eq!(variable.snippet, "$accent: red !default;");

    let mixin = summarize_omena_query_style_hover_render_parts(
        source,
        "sassMixinDeclaration",
        "tone",
        ParserPositionV0 {
            line: 1,
            character: 7,
        },
    );
    assert_eq!(mixin.signature.as_deref(), Some("@mixin tone($color)"));
    assert_eq!(mixin.snippet, "color: $color;");
    assert_eq!(mixin.render_source, "callableBlockSnippet");

    // A multi-line body renders flush left: the extraction trims the first
    // line, so the continuation lines' common indent must be removed too —
    // while RELATIVE indentation (nested rules) survives.
    let multi_line_source = "@mixin typography {\n    font-size: 12px;\n    line-height: 18px;\n    .fill {\n        padding-left: 3px;\n    }\n}\n";
    let multi_line = summarize_omena_query_style_hover_render_parts(
        multi_line_source,
        "sassMixinDeclaration",
        "typography",
        ParserPositionV0 {
            line: 0,
            character: 7,
        },
    );
    assert_eq!(
        multi_line.snippet,
        "font-size: 12px;\nline-height: 18px;\n.fill {\n    padding-left: 3px;\n}",
    );

    let selector = summarize_omena_query_style_hover_render_parts(
        source,
        "selector",
        "button",
        ParserPositionV0 {
            line: 4,
            character: 1,
        },
    );
    assert_eq!(selector.snippet, ".button { color: var(--brand); }");
    assert_eq!(selector.render_source, "ruleSnippet");
    assert!(selector.property_value_narrowings.iter().any(|narrowing| {
        narrowing.property_name == "color"
            && narrowing.requested_condition_context.is_empty()
            && narrowing.requested_layer_scope == "exactLayer"
            && narrowing.matched_candidate_count == 1
    }));
    assert!(selector.property_value_narrowings.iter().any(|narrowing| {
        narrowing.property_name == "color"
            && narrowing.requested_condition_context
                == vec!["@media (min-width: 40rem)".to_string()]
            && narrowing.requested_layer_name.as_deref() == Some("theme")
            && narrowing.matched_candidate_count == 1
    }));
}

#[test]
fn style_hover_render_parts_narrow_same_selector_values_by_source_order() {
    let selector = summarize_omena_query_style_hover_render_parts(
        ".button { color: red; }\n.button { color: blue; }",
        "selector",
        "button",
        ParserPositionV0 {
            line: 1,
            character: 1,
        },
    );

    let color = selector
        .property_value_narrowings
        .iter()
        .find(|narrowing| narrowing.property_name == "color");
    assert_eq!(
        color.map(|narrowing| narrowing.matched_candidate_count),
        Some(2)
    );
    assert_eq!(
        color.map(|narrowing| &narrowing.value),
        Some(&AbstractPropertyValueV0::Exact {
            property_name: "color".to_string(),
            value: "#00f".to_string(),
            pseudo_state: None,
        })
    );
}

#[test]
fn style_hover_render_parts_for_hover_position_prefers_active_condition_layer_branch() {
    let source = r#".button { color: red; }
@media (min-width: 40rem) {
  @layer theme {
    .button { color: blue; }
  }
}
"#;
    let selector = summarize_omena_query_style_hover_render_parts_for_hover_position(
        source,
        "selector",
        "button",
        ParserPositionV0 {
            line: 3,
            character: 5,
        },
    );

    let color_narrowings = selector
        .property_value_narrowings
        .iter()
        .filter(|narrowing| narrowing.property_name == "color")
        .collect::<Vec<_>>();
    assert_eq!(color_narrowings.len(), 1);
    let color = color_narrowings[0];
    assert_eq!(
        color.requested_condition_context,
        vec!["@media (min-width: 40rem)".to_string()]
    );
    assert_eq!(color.requested_layer_name.as_deref(), Some("theme"));
    assert_eq!(
        color.value,
        AbstractPropertyValueV0::Exact {
            property_name: "color".to_string(),
            value: "#00f".to_string(),
            pseudo_state: None,
        }
    );
}

#[test]
fn style_selector_completion_includes_cascade_narrowed_values() -> Result<(), &'static str> {
    let source = r#".button { color: var(--brand); }
@media (min-width: 40rem) { @layer theme { .button { color: blue; } } }
"#;
    let candidates = summarize_omena_query_style_hover_candidates("Component.module.scss", source)
        .ok_or("style candidates")?;

    let completion = summarize_omena_query_style_completion_at_position(
        "file:///workspace/src/Component.module.scss",
        source,
        ParserPositionV0 {
            line: 0,
            character: 0,
        },
        candidates.candidates.as_slice(),
    );

    let button = completion
        .items
        .iter()
        .find(|item| item.label == ".button")
        .ok_or("button completion")?;
    let documentation = button
        .documentation
        .as_deref()
        .ok_or("button documentation")?;
    assert!(documentation.contains("Cascade narrowed values:"));
    assert!(documentation.contains("- `color`: `var(--brand)`"));
    assert!(documentation.contains("@layer theme"));
    assert!(documentation.contains("`blue`"));
    Ok(())
}

#[test]
fn completion_at_position_is_query_owned_for_style_and_source() -> Result<(), &'static str> {
    let source = ":root { --brand: red; }\n.root { color: var(--br); }\n.row { display: flex; }";
    let candidates = summarize_omena_query_style_hover_candidates("Component.module.scss", source)
        .ok_or("style candidates")?;

    let style_completion = summarize_omena_query_style_completion_at_position(
        "file:///workspace/src/Component.module.scss",
        source,
        ParserPositionV0 {
            line: 1,
            character: 23,
        },
        candidates.candidates.as_slice(),
    );
    assert_eq!(style_completion.product, "omena-query.completion-at");
    assert_eq!(
        style_completion.context_kind,
        "styleCustomPropertyReference"
    );
    assert_eq!(style_completion.prefix.as_deref(), Some("--br"));
    assert_eq!(
        style_completion
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["--brand"]
    );

    let source_completion = summarize_omena_query_source_completion_at_position(
        "file:///workspace/src/App.tsx",
        ParserPositionV0 {
            line: 1,
            character: 22,
        },
        &[
            OmenaQueryCompletionCandidateV0 {
                file_uri: "file:///workspace/src/Component.module.scss".to_string(),
                name: "root".to_string(),
                kind: "selector",
                range: ParserRangeV0 {
                    start: ParserPositionV0 {
                        line: 1,
                        character: 1,
                    },
                    end: ParserPositionV0 {
                        line: 1,
                        character: 5,
                    },
                },
                source: "omenaQueryStyleHoverCandidates",
                documentation: Some("Cascade narrowed values:\n- `display`: `block`".to_string()),
            },
            OmenaQueryCompletionCandidateV0 {
                file_uri: "file:///workspace/src/Other.module.scss".to_string(),
                name: "rootOther".to_string(),
                kind: "selector",
                range: ParserRangeV0 {
                    start: ParserPositionV0 {
                        line: 0,
                        character: 1,
                    },
                    end: ParserPositionV0 {
                        line: 0,
                        character: 10,
                    },
                },
                source: "omenaQueryStyleHoverCandidates",
                documentation: None,
            },
        ],
        Some("file:///workspace/src/Component.module.scss"),
        Some("ro"),
        &[],
    );
    assert_eq!(source_completion.context_kind, "sourceCssModuleTarget");
    assert_eq!(source_completion.item_count, 1);
    assert_eq!(source_completion.items[0].label, "root");
    assert_eq!(
        source_completion.items[0].ranking_source,
        "targetAndPrefixNarrowing"
    );
    assert_eq!(
        source_completion.items[0].documentation.as_deref(),
        Some("Cascade narrowed values:\n- `display`: `block`")
    );
    assert!(
        source_completion
            .ready_surfaces
            .contains(&"bridgeAwareSelectorCompletion")
    );
    Ok(())
}

#[test]
fn source_completion_ranking_prefers_value_domain_projection() {
    let candidates = [
        OmenaQueryCompletionCandidateV0 {
            file_uri: "file:///workspace/src/Component.module.scss".to_string(),
            name: "item--large".to_string(),
            kind: "selector",
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 0,
                    character: 1,
                },
                end: ParserPositionV0 {
                    line: 0,
                    character: 12,
                },
            },
            source: "omenaQueryStyleHoverCandidates",
            documentation: None,
        },
        OmenaQueryCompletionCandidateV0 {
            file_uri: "file:///workspace/src/Component.module.scss".to_string(),
            name: "item--primary".to_string(),
            kind: "selector",
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 1,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 14,
                },
            },
            source: "omenaQueryStyleHoverCandidates",
            documentation: None,
        },
        OmenaQueryCompletionCandidateV0 {
            file_uri: "file:///workspace/src/Component.module.scss".to_string(),
            name: "item--secondary".to_string(),
            kind: "selector",
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 2,
                    character: 1,
                },
                end: ParserPositionV0 {
                    line: 2,
                    character: 16,
                },
            },
            source: "omenaQueryStyleHoverCandidates",
            documentation: None,
        },
    ];
    let completion = summarize_omena_query_source_completion_at_position(
        "file:///workspace/src/App.tsx",
        ParserPositionV0 {
            line: 0,
            character: 42,
        },
        &candidates,
        Some("file:///workspace/src/Component.module.scss"),
        Some("item--"),
        &["item--secondary".to_string(), "item--primary".to_string()],
    );

    assert_eq!(completion.context_kind, "sourceCssModuleValueDomainTarget");
    assert_eq!(
        completion
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["item--primary", "item--secondary", "item--large"]
    );
    assert_eq!(
        completion.items[0].ranking_source,
        "valueDomainSelectorProjection"
    );
    assert!(
        completion
            .ready_surfaces
            .contains(&"valueDomainAwareSelectorCompletion")
    );
}

#[test]
fn completion_ranking_uses_query_facts() -> Result<(), &'static str> {
    let source =
        ":root { --alpha: red; }\n.card { --zeta: blue; color: var(--); }\n.next { --omega: red; }";
    let candidates = summarize_omena_query_style_hover_candidates("Component.module.scss", source)
        .ok_or("style candidates")?;

    let completion = summarize_omena_query_style_completion_at_position(
        "file:///workspace/src/Component.module.scss",
        source,
        ParserPositionV0 {
            line: 1,
            character: 35,
        },
        candidates.candidates.as_slice(),
    );

    assert_eq!(completion.context_kind, "styleCustomPropertyReference");
    assert_eq!(
        completion
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["--zeta", "--alpha", "--omega"]
    );
    assert_eq!(
        completion.items[0].ranking_source,
        "sameFileSourceOrderCascade"
    );
    assert!(completion.items[0].sort_text.starts_with("00-"));
    Ok(())
}

#[test]
fn style_extract_code_actions_are_query_owned() {
    let source = ".button { color: #ff0000; margin: 1rem; }";
    let plan = summarize_omena_query_style_extract_code_actions(
        "file:///workspace/src/App.module.scss",
        source,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 17,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 24,
            },
        },
    );

    assert_eq!(plan.product, "omena-query.code-actions");
    assert_eq!(plan.file_kind, "style");
    assert_eq!(plan.action_count, 2);
    assert_eq!(
        plan.actions
            .iter()
            .map(|action| action.title.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Extract CSS custom property '--extracted-color'",
            "Extract @value 'extractedColor'",
        ]
    );
    assert_eq!(plan.actions[0].kind, "refactor.extract");
    assert_eq!(plan.actions[0].source, "omenaQueryStyleExtractCodeActions");
    assert_eq!(
        plan.actions[0]
            .edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .collect::<Vec<_>>(),
        vec![
            ":root {\n  --extracted-color: #ff0000;\n}\n\n",
            "var(--extracted-color)"
        ]
    );
    assert_eq!(
        plan.actions[1]
            .edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .collect::<Vec<_>>(),
        vec!["@value extractedColor: #ff0000;\n\n", "extractedColor"]
    );
    assert!(plan.ready_surfaces.contains(&"styleExtractRefactorActions"));
}

#[test]
fn style_inline_code_actions_are_query_owned() {
    let source = ".button {\n  composes: base;\n  color: red;\n}\n.base {\n  color: blue;\n  margin: 1rem;\n}";
    let style_uri = "file:///workspace/src/App.module.scss";
    let plan = summarize_omena_query_style_inline_code_actions(
        style_uri,
        &[OmenaQueryStyleSourceInputV0 {
            style_path: style_uri.to_string(),
            style_source: source.to_string(),
        }],
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 12,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 16,
            },
        },
        &[],
    );

    assert_eq!(plan.product, "omena-query.code-actions");
    assert_eq!(plan.file_kind, "style");
    assert_eq!(plan.action_count, 1);
    assert_eq!(plan.actions[0].title, "Inline composed class 'base'");
    assert_eq!(plan.actions[0].kind, "refactor.inline");
    assert_eq!(plan.actions[0].source, "omenaQueryStyleInlineCodeActions");
    assert_eq!(
        plan.actions[0].edits[0].range,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 2,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 17,
            },
        }
    );
    assert_eq!(
        plan.actions[0].edits[0].new_text,
        "color: blue;\n  margin: 1rem;"
    );
    assert!(plan.ready_surfaces.contains(&"styleInlineRefactorActions"));
}

#[test]
fn style_inline_code_actions_resolve_workspace_aliases() {
    let style_uri = "/workspace/src/App.module.scss";
    let target_uri = "/workspace/src/styles/Base.module.scss";
    let plan = summarize_omena_query_style_inline_code_actions_with_resolution_inputs(
        style_uri,
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: style_uri.to_string(),
                style_source: ".button {\n  composes: base from \"@styles/Base.module.scss\";\n}"
                    .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: target_uri.to_string(),
                style_source: ".base { color: blue; }".to_string(),
            },
        ],
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 12,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 16,
            },
        },
        &[],
        &OmenaQueryStyleResolutionInputsV0 {
            tsconfig_path_mappings: vec![OmenaQueryTsconfigPathMappingV0 {
                base_path: "/workspace".to_string(),
                pattern: "@styles/*".to_string(),
                target_patterns: vec!["src/styles/*".to_string()],
            }],
            ..OmenaQueryStyleResolutionInputsV0::default()
        },
    );

    assert_eq!(plan.action_count, 1, "{plan:?}");
    assert_eq!(plan.actions[0].title, "Inline composed class 'base'");
    assert_eq!(plan.actions[0].edits[0].new_text, "color: blue;");
}

#[test]
fn style_insights_surface_shorthand_combinable_facts() {
    let source = ".button {\n  margin-top: 1px;\n  margin-right: 2px;\n  margin-bottom: 3px;\n  margin-left: 4px;\n}";
    let insights =
        summarize_omena_query_style_insights("file:///workspace/src/App.module.scss", source);

    assert_eq!(insights.product, "omena-query.style-insights");
    assert_eq!(insights.insight_count, 1);
    assert!(insights.ready_surfaces.contains(&"styleInsightSurface"));
    let insight = &insights.insights[0];
    assert_eq!(insight.kind, "shorthandCombinable");
    assert_eq!(insight.title, "Combine margin longhands into shorthand");
    assert_eq!(insight.source, "omenaQueryStyleInsights");
    assert_eq!(insight.confidence, "high");
    assert_eq!(insight.scope, "singleSelector");
    assert_eq!(
        insight.range,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 2,
            },
            end: ParserPositionV0 {
                line: 4,
                character: 18,
            },
        }
    );
    assert_eq!(
        insight
            .primary_edit
            .as_ref()
            .map(|edit| edit.new_text.as_str()),
        Some("margin: 1px 2px 3px 4px")
    );
    assert!(
        insight
            .shorthand_combinable
            .as_ref()
            .is_some_and(|shorthand| {
                shorthand.shorthand_property == "margin"
                    && shorthand
                        .longhand_properties
                        .iter()
                        .map(String::as_str)
                        .eq(["margin-top", "margin-right", "margin-bottom", "margin-left"])
                    && shorthand.combined_value == "1px 2px 3px 4px"
            })
    );
    assert!(
        insight
            .cascade_insight
            .as_ref()
            .is_some_and(|cascade| cascade.relationship == "replaceLonghandQuartetWithShorthand")
    );
}

#[test]
fn style_insight_code_actions_consume_shorthand_combinable_facts() {
    let source = ".button {\n  margin-top: 1px;\n  margin-right: 2px;\n  margin-bottom: 3px;\n  margin-left: 4px;\n}";
    let plan = summarize_omena_query_style_insight_code_actions(
        "file:///workspace/src/App.module.scss",
        source,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 2,
                character: 4,
            },
            end: ParserPositionV0 {
                line: 2,
                character: 4,
            },
        },
    );

    assert_eq!(plan.product, "omena-query.code-actions");
    assert_eq!(plan.action_count, 1);
    assert_eq!(
        plan.ready_surfaces,
        vec![
            "styleInsightCodeActions",
            "styleInsightSurface",
            "productFacingCodeActions",
        ]
    );
    assert_eq!(plan.actions[0].kind, "quickfix");
    assert_eq!(plan.actions[0].source, "omenaQueryStyleInsightCodeActions");
    assert_eq!(
        plan.actions[0].title,
        "Combine margin longhands into shorthand"
    );
    assert_eq!(plan.actions[0].edits[0].new_text, "margin: 1px 2px 3px 4px");
}

#[test]
fn style_insights_surface_cascade_relationship_facts() {
    let source = ".button {\n  margin: 1px;\n  margin-left: 2px;\n  border-color: red;\n  border-color: red;\n}\n.primary { color: blue; }\n.secondary { color: green; }";
    let insights =
        summarize_omena_query_style_insights("file:///workspace/src/App.module.scss", source);
    let kinds = insights
        .insights
        .iter()
        .map(|insight| insight.kind)
        .collect::<Vec<_>>();

    assert!(
        insights
            .ready_surfaces
            .contains(&"cascadeRelationshipInsights")
    );
    assert!(insights.ready_surfaces.contains(&"insightConfidenceScope"));
    assert!(kinds.contains(&"partialShorthandOverride"));
    assert!(kinds.contains(&"longhandRedundant"));
    assert!(kinds.contains(&"specificityTie"));

    let partial = insights
        .insights
        .iter()
        .find(|insight| insight.kind == "partialShorthandOverride");
    assert!(partial.is_some());
    assert_eq!(partial.map(|insight| insight.confidence), Some("high"));
    assert_eq!(partial.map(|insight| insight.scope), Some("singleSelector"));
    assert_eq!(
        partial.map(|insight| insight.primary_edit.is_none()),
        Some(true)
    );
    assert_eq!(
        partial
            .and_then(|insight| insight.cascade_insight.as_ref())
            .map(|cascade| {
                cascade.relationship == "longhandOverridesEarlierShorthand"
                    && cascade.property == "margin"
                    && cascade.related_property.as_deref() == Some("margin-left")
            }),
        Some(true)
    );

    let redundant = insights
        .insights
        .iter()
        .find(|insight| insight.kind == "longhandRedundant");
    assert!(redundant.is_some());
    assert_eq!(redundant.map(|insight| insight.confidence), Some("high"));
    assert_eq!(
        redundant.map(|insight| insight.primary_edit.is_none()),
        Some(true)
    );

    let tie = insights
        .insights
        .iter()
        .find(|insight| insight.kind == "specificityTie");
    assert!(tie.is_some());
    assert_eq!(
        tie.map(|insight| insight.scope),
        Some("crossSelectorSameStylesheet")
    );
    assert_eq!(
        tie.and_then(|insight| insight.cascade_insight.as_ref())
            .map(|cascade| cascade.relationship == "sourceOrderDecidesEqualSpecificity"),
        Some(true)
    );

    let plan = summarize_omena_query_style_insight_code_actions(
        "file:///workspace/src/App.module.scss",
        source,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 2,
                character: 4,
            },
            end: ParserPositionV0 {
                line: 2,
                character: 4,
            },
        },
    );
    assert_eq!(plan.action_count, 0);
}

#[test]
fn refs_for_class_is_query_owned_and_workspace_scoped() {
    let definition = OmenaQueryStyleSelectorDefinitionV0 {
        uri: "file:///workspace/src/Component.module.scss".to_string(),
        name: "root".to_string(),
        range: ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 1,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 5,
            },
        },
    };
    let references = vec![
        OmenaQuerySourceSelectorReferenceCandidateV0 {
            uri: "file:///workspace/src/App.tsx".to_string(),
            kind: "sourceSelectorReference",
            name: "root".to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 31,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 35,
                },
            },
            source: "omenaQuerySourceSyntaxIndex",
            target_style_uri: Some("file:///workspace/src/Component.module.scss".to_string()),
        },
        OmenaQuerySourceSelectorReferenceCandidateV0 {
            uri: "file:///workspace/src/Other.tsx".to_string(),
            kind: "sourceSelectorReference",
            name: "root".to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 31,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 35,
                },
            },
            source: "omenaQuerySourceSyntaxIndex",
            target_style_uri: Some("file:///workspace/src/Other.module.scss".to_string()),
        },
    ];

    let refs = summarize_omena_query_refs_for_class(
        "root",
        Some("file:///workspace/src/Component.module.scss"),
        true,
        &[definition],
        references.as_slice(),
    );
    assert_eq!(refs.product, "omena-query.refs-for-class");
    assert_eq!(refs.location_count, 2);
    assert_eq!(refs.locations[0].role, "definition");
    assert_eq!(refs.locations[1].role, "reference");
    assert_eq!(refs.locations[1].uri, "file:///workspace/src/App.tsx");
    assert!(
        refs.ready_surfaces
            .contains(&"workspaceWideSelectorReferences")
    );
}

#[test]
fn rename_plan_is_query_owned_and_workspace_scoped() {
    let definition = OmenaQueryStyleSelectorDefinitionV0 {
        uri: "file:///workspace/src/Component.module.scss".to_string(),
        name: "root".to_string(),
        range: ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 1,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 5,
            },
        },
    };
    let reference = OmenaQuerySourceSelectorReferenceEditTargetV0 {
        uri: "file:///workspace/src/App.tsx".to_string(),
        name: "root".to_string(),
        range: ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 31,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 35,
            },
        },
        target_style_uri: Some("file:///workspace/src/Component.module.scss".to_string()),
    };

    let plan = summarize_omena_query_rename_plan(
        "root",
        "button",
        Some("file:///workspace/src/Component.module.scss"),
        &[definition],
        &[reference],
    );
    assert_eq!(plan.product, "omena-query.rename-plan");
    assert_eq!(plan.edit_count, 2);
    assert_eq!(plan.edits[0].new_text, "button");
    assert_eq!(
        plan.edits
            .iter()
            .map(|edit| edit.uri.as_str())
            .collect::<Vec<_>>(),
        vec![
            "file:///workspace/src/App.tsx",
            "file:///workspace/src/Component.module.scss"
        ]
    );
    assert!(plan.ready_surfaces.contains(&"workspaceWideSelectorRename"));
}

#[test]
fn source_selector_occurrence_index_feeds_refs_and_rename() {
    let definition = OmenaQueryStyleSelectorDefinitionV0 {
        uri: "file:///workspace/src/Component.module.scss".to_string(),
        name: "root".to_string(),
        range: ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 1,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 5,
            },
        },
    };
    let references = vec![
        OmenaQuerySourceSelectorReferenceCandidateV0 {
            uri: "file:///workspace/src/App.tsx".to_string(),
            kind: "sourceSelectorReference",
            name: "root".to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 31,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 35,
                },
            },
            source: "omenaTsgoTypeFactProjection",
            target_style_uri: Some("file:///workspace/src/Component.module.scss".to_string()),
        },
        OmenaQuerySourceSelectorReferenceCandidateV0 {
            uri: "file:///workspace/src/Other.tsx".to_string(),
            kind: "sourceSelectorReference",
            name: "root".to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 31,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 35,
                },
            },
            source: "omenaQuerySourceSyntaxIndex",
            target_style_uri: Some("file:///workspace/src/Other.module.scss".to_string()),
        },
    ];

    let index = summarize_omena_query_source_selector_occurrence_index(
        std::slice::from_ref(&definition),
        &references,
    );
    assert_eq!(
        index.product,
        "omena-query.source-selector-occurrence-index"
    );
    assert_eq!(index.moniker_count, 2);
    assert_eq!(index.occurrence_count, 2);
    assert!(index.occurrences.iter().any(|occurrence| occurrence.moniker
        == "css-module-selector:file:///workspace/src/Component.module.scss#.root"));
    assert!(index.occurrences.iter().any(|occurrence| {
        occurrence.uri == "file:///workspace/src/App.tsx"
            && occurrence.source == OmenaWorkspaceOccurrenceSurfaceV0::OmenaQuerySourceSyntaxIndex
    }));
    assert_eq!(
        references[0].projection_surface(),
        OmenaQuerySourceSelectorReferenceSurfaceV0::OmenaTsgoTypeFactProjection
    );
    assert_eq!(
        references[1].projection_surface(),
        OmenaQuerySourceSelectorReferenceSurfaceV0::OmenaQuerySourceSyntaxIndex
    );
    assert_eq!(
        index.workspace_index.product,
        "omena-query.workspace-occurrence-index"
    );
    assert_eq!(index.workspace_index.moniker_count, 2);
    assert_eq!(index.workspace_index.occurrence_count, 2);
    assert_eq!(
        index
            .workspace_index
            .by_moniker
            .get("css-module-selector:file:///workspace/src/Component.module.scss#.root")
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        index
            .workspace_index
            .by_file
            .get("file:///workspace/src/App.tsx")
            .cloned()
            .unwrap_or_default(),
        vec!["css-module-selector:file:///workspace/src/Component.module.scss#.root"]
    );

    let refs = summarize_omena_query_refs_for_class_from_occurrence_index(
        "root",
        Some("file:///workspace/src/Component.module.scss"),
        true,
        std::slice::from_ref(&definition),
        &index,
    );
    assert_eq!(refs.location_count, 2);
    assert_eq!(refs.locations[0].role, "definition");
    assert_eq!(refs.locations[1].uri, "file:///workspace/src/App.tsx");
    let workspace_refs = summarize_omena_query_refs_for_class_from_occurrence_index(
        "root",
        None,
        false,
        std::slice::from_ref(&definition),
        &index,
    );
    assert_eq!(workspace_refs.location_count, 2);

    let rename = summarize_omena_query_rename_plan_from_occurrence_index(
        "root",
        "button",
        Some("file:///workspace/src/Component.module.scss"),
        std::slice::from_ref(&definition),
        &index,
    );
    assert_eq!(
        rename
            .edits
            .iter()
            .map(|edit| edit.uri.as_str())
            .collect::<Vec<_>>(),
        vec![
            "file:///workspace/src/App.tsx",
            "file:///workspace/src/Component.module.scss"
        ]
    );
    assert!(
        rename
            .ready_surfaces
            .contains(&"sourceSelectorOccurrenceIndex")
    );
}

#[test]
fn workspace_refs_consume_precomputed_source_syntax_index() -> Result<(), &'static str> {
    let source = "const local = root;\n";
    let root_start = source
        .find("root")
        .ok_or("fixture should contain the selector token")?;
    let refs = summarize_omena_query_refs_for_workspace_class(
        "root",
        Some("file:///workspace/src/Component.module.scss"),
        false,
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "file:///workspace/src/Component.module.scss".to_string(),
            style_source: ".root { color: red; }".to_string(),
        }],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: "file:///workspace/src/App.tsx".to_string(),
            source_source: source.to_string(),
            source_syntax_index: Some(OmenaQuerySourceSyntaxIndexV0 {
                schema_version: "0",
                product: "omena-bridge.source-syntax-index",
                imported_style_bindings: vec![OmenaQuerySourceImportedStyleBindingV0 {
                    binding: "styles".to_string(),
                    style_uri: "file:///workspace/src/Component.module.scss".to_string(),
                }],
                class_string_literals: Vec::new(),
                style_property_accesses: Vec::new(),
                inline_style_declarations: Vec::new(),
                selector_references: vec![OmenaQuerySourceSelectorReferenceFactV0 {
                    byte_span: ParserByteSpanV0 {
                        start: root_start,
                        end: root_start + "root".len(),
                    },
                    selector_name: Some("root".to_string()),
                    match_kind: OmenaQuerySourceSelectorReferenceMatchKindV0::Exact,
                    target_style_uri: Some(
                        "file:///workspace/src/Component.module.scss".to_string(),
                    ),
                    surface: Default::default(),
                }],
                type_fact_targets: Vec::new(),
                type_fact_target_skipped: Vec::new(),
                type_fact_target_skipped_count: 0,
                type_fact_provider_unavailable: Vec::new(),
                class_value_universes: Vec::new(),
                domain_class_references: Vec::new(),
                source_elements: Vec::new(),
                element_parent_edges: Vec::new(),
            }),
            has_unresolved_style_import: false,
        }],
        &[],
    );

    assert_eq!(refs.location_count, 1);
    assert_eq!(refs.locations[0].uri, "file:///workspace/src/App.tsx");
    assert_eq!(refs.locations[0].name, "root");
    assert_eq!(
        refs.locations[0].source,
        "omenaQuerySourceSelectorReferences"
    );
    Ok(())
}

#[test]
fn workspace_refs_resolve_alias_imports_from_explicit_inputs() {
    let style_path = "/workspace/src/styles/Component.module.scss";
    let source_path = "/workspace/src/App.tsx";
    let refs = summarize_omena_query_refs_for_workspace_class_with_resolution_inputs(
        "root",
        Some(style_path),
        false,
        &[OmenaQueryStyleSourceInputV0 {
            style_path: style_path.to_string(),
            style_source: ".root { color: red; }".to_string(),
        }],
        &[OmenaQuerySourceDocumentInputV0 {
            source_path: source_path.to_string(),
            source_source: r#"import styles from "@styles/Component.module.scss";
export const app = <div className={styles.root} />;"#
                .to_string(),
            source_syntax_index: None,
            has_unresolved_style_import: false,
        }],
        &[],
        &OmenaQueryStyleResolutionInputsV0 {
            tsconfig_path_mappings: vec![OmenaQueryTsconfigPathMappingV0 {
                base_path: "/workspace".to_string(),
                pattern: "@styles/*".to_string(),
                target_patterns: vec!["src/styles/*".to_string()],
            }],
            ..OmenaQueryStyleResolutionInputsV0::default()
        },
    );

    assert_eq!(refs.location_count, 1, "{refs:?}");
    assert_eq!(refs.locations[0].uri, source_path);
    assert_eq!(refs.locations[0].name, "root");
    assert_eq!(
        refs.locations[0].source,
        "omenaQuerySourceSelectorReferences"
    );
}

#[test]
fn missing_selector_diagnostics_are_query_owned() {
    let diagnostic = summarize_omena_query_missing_selector_diagnostic(
        "file:///workspace/src/App.module.scss",
        ".root {\n}\n",
        "missing",
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 2,
                character: 18,
            },
            end: ParserPositionV0 {
                line: 2,
                character: 25,
            },
        },
    );

    assert_eq!(diagnostic.code, "missingSelector");
    assert_eq!(
        diagnostic.message,
        "CSS Module selector '.missing' not found in indexed style tokens."
    );
    assert_eq!(
        diagnostic
            .create_selector
            .as_ref()
            .map(|action| action.new_text.as_str()),
        Some("\n\n.missing {\n}\n")
    );
    assert_eq!(
        diagnostic
            .create_selector
            .as_ref()
            .map(|action| action.range),
        Some(ParserRangeV0 {
            start: ParserPositionV0 {
                line: 2,
                character: 0,
            },
            end: ParserPositionV0 {
                line: 2,
                character: 0,
            },
        })
    );
    let linear_provenance = diagnostic.linear_provenance();
    assert_eq!(
        linear_provenance.labels(),
        vec![
            "omena-query.source-syntax-index",
            "omena-query.style-selector-definitions"
        ]
    );
}

#[cfg(unix)]
#[test]
fn query_resolves_symlinked_package_style_uri_to_canonical_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!(
        "omena_query_symlinked_package_identity_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let source = root.join("src/App.module.scss");
    let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let linked_scope = root.join("node_modules/@design");
    let linked_package = linked_scope.join("tokens");
    let style = real_package.join("src/index.scss");
    std::fs::create_dir_all(
        source
            .parent()
            .ok_or_else(|| std::io::Error::other("source"))?,
    )?;
    std::fs::create_dir_all(
        style
            .parent()
            .ok_or_else(|| std::io::Error::other("style"))?,
    )?;
    std::fs::create_dir_all(linked_scope.as_path())?;
    std::fs::write(&source, r#"@use "@design/tokens" as tokens;"#)?;
    std::fs::write(
        real_package.join("package.json"),
        r#"{"sass":"src/index.scss"}"#,
    )?;
    std::fs::write(&style, "$brand: #fff;")?;
    std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;

    let resolved_uri = resolve_omena_query_style_uri_for_specifier(
        test_file_uri(source.as_path()).as_str(),
        Some(test_file_uri(root.as_path()).as_str()),
        "@design/tokens",
    );
    let expected_uri = test_file_uri(std::fs::canonicalize(style)?.as_path());

    assert_eq!(resolved_uri.as_deref(), Some(expected_uri.as_str()));
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn query_resolves_vite_bundler_alias_style_uri() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!(
        "omena_query_vite_alias_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let source = root.join("src/App.tsx");
    let style = root.join("src/styles/Button.module.scss");
    std::fs::create_dir_all(
        style
            .parent()
            .ok_or_else(|| std::io::Error::other("style"))?,
    )?;
    std::fs::write(&source, "")?;
    std::fs::write(&style, ".button { color: red; }")?;
    std::fs::write(
        root.join("vite.config.ts"),
        r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
    )?;

    let resolved_uri = resolve_omena_query_style_uri_for_specifier(
        test_file_uri(source.as_path()).as_str(),
        Some(test_file_uri(root.as_path()).as_str()),
        "@styles/Button.module.scss",
    );
    let expected_uri = test_file_uri(std::fs::canonicalize(style)?.as_path());

    assert_eq!(resolved_uri.as_deref(), Some(expected_uri.as_str()));
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

fn test_file_uri(path: &std::path::Path) -> String {
    format!("file://{}", path.to_string_lossy())
}
