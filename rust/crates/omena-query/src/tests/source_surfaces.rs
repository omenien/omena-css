use crate::{
    OmenaQuerySourceImportedStyleBindingV0, OmenaQuerySourceSelectorCandidateV0,
    OmenaQuerySourceSelectorReferenceEditTargetV0, OmenaQueryStyleSelectorDefinitionV0,
    ParserPositionV0, ParserRangeV0, canonicalize_omena_query_source_selector_references,
    is_omena_query_sass_symbol_candidate_kind, is_omena_query_sass_symbol_reference_kind,
    omena_query_sass_symbol_kind_from_candidate_kind, omena_query_sass_symbol_target_matches,
    resolve_omena_query_sass_forward_sources,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    resolve_omena_query_sass_symbol_declarations, resolve_omena_query_selector_rename_edits,
    resolve_omena_query_source_candidate_selector_names,
    resolve_omena_query_style_uri_for_specifier, summarize_omena_query_sass_module_sources,
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
