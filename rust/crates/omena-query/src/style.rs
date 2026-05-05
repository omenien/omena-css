use super::*;

pub fn summarize_omena_query_style_semantic_graph_from_source(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<StyleSemanticGraphSummaryV0> {
    summarize_omena_bridge_style_semantic_graph_from_source(style_path, style_source, input)
}

pub fn summarize_omena_query_style_document(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQueryStyleDocumentSummaryV0> {
    let sheet = parse_style_module(style_path, style_source)?;
    let index = summarize_css_modules_intermediate(&sheet);
    Some(OmenaQueryStyleDocumentSummaryV0 {
        schema_version: "0",
        product: "omena-query.style-document-summary",
        language: style_language_label(sheet.language),
        selector_names: index.selectors.names,
        custom_property_decl_names: index.custom_properties.decl_names,
        custom_property_ref_names: index.custom_properties.ref_names,
        sass_module_use_sources: index.sass.module_use_sources,
        sass_module_forward_sources: index.sass.module_forward_sources,
        diagnostic_count: sheet.diagnostics.len(),
    })
}

pub fn summarize_omena_query_omena_parser_style_facts(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> OmenaQueryOmenaParserStyleFactsV0 {
    let facts = collect_style_facts(style_source, dialect);
    let mut class_selector_names = Vec::new();
    let mut id_selector_names = Vec::new();
    let mut variable_names = BTreeSet::new();
    let mut custom_property_names = BTreeSet::new();

    for selector in facts.selectors {
        match selector.kind {
            ParsedSelectorFactKind::Class => class_selector_names.push(selector.name),
            ParsedSelectorFactKind::Id => id_selector_names.push(selector.name),
            ParsedSelectorFactKind::Placeholder => {}
        }
    }

    for variable in facts.variables {
        match variable.kind {
            ParsedVariableFactKind::ScssDeclaration
            | ParsedVariableFactKind::ScssReference
            | ParsedVariableFactKind::LessDeclaration
            | ParsedVariableFactKind::LessReference => {
                variable_names.insert(variable.name);
            }
            ParsedVariableFactKind::CustomPropertyDeclaration
            | ParsedVariableFactKind::CustomPropertyReference => {
                custom_property_names.insert(variable.name);
            }
        }
    }

    OmenaQueryOmenaParserStyleFactsV0 {
        schema_version: "0",
        product: "omena-query.omena-parser-style-facts",
        dialect: omena_parser_style_dialect_label(dialect),
        class_selector_names,
        id_selector_names,
        variable_names: variable_names.into_iter().collect(),
        custom_property_names: custom_property_names.into_iter().collect(),
        at_rule_names: facts
            .at_rules
            .into_iter()
            .map(|at_rule| at_rule.name)
            .collect(),
        parser_error_count: facts.error_count,
    }
}

pub fn summarize_omena_query_style_hover_candidates(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQueryStyleHoverCandidatesV0> {
    let sheet = parse_style_module(style_path, style_source)?;
    let index = summarize_css_modules_intermediate(&sheet);
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    collect_style_selector_hover_candidates_from_parser_facts(
        index.selectors.definition_facts.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_custom_property_hover_candidates(
        sheet.source.as_str(),
        index.custom_properties.decl_facts.as_slice(),
        index.custom_properties.ref_names.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_symbol_hover_candidates(
        index.sass.symbol_decl_facts.as_slice(),
        index.sass.selector_symbol_facts.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_partial_evaluator_selector_candidates(
        sheet.source.as_str(),
        sheet.nodes.as_slice(),
        &mut seen,
        &mut candidates,
    );
    candidates.sort();
    Some(OmenaQueryStyleHoverCandidatesV0 {
        schema_version: "0",
        product: "omena-query.style-hover-candidates",
        language: style_language_label(sheet.language),
        candidates,
    })
}

pub fn summarize_omena_query_style_hover_render_parts(
    source: &str,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> OmenaQueryStyleHoverRenderPartsV0 {
    let mut parts = OmenaQueryStyleHoverRenderPartsV0 {
        schema_version: "0",
        product: "omena-query.style-hover-render-parts",
        snippet: String::new(),
        value: None,
        signature: None,
        render_source: "lineSnippet",
    };

    match kind {
        "selector" => {
            parts.snippet = rule_snippet_around_position(source, position).unwrap_or_else(|| {
                parts.render_source = "selectorFallback";
                format!(".{name} {{ ... }}")
            });
            if parts.render_source != "selectorFallback" {
                parts.render_source = "ruleSnippet";
            }
        }
        "customPropertyReference" | "customPropertyDeclaration" => {
            parts.snippet = line_snippet_at_position(source, position).unwrap_or_default();
        }
        kind if is_sass_symbol_candidate_kind(kind) => {
            parts.snippet = line_snippet_at_position(source, position).unwrap_or_default();
            if sass_symbol_kind_from_candidate_kind(kind) == Some("variable")
                && is_sass_symbol_declaration_kind(kind)
            {
                parts.value = sass_variable_value_from_declaration_line(parts.snippet.as_str());
            } else if matches!(
                sass_symbol_kind_from_candidate_kind(kind),
                Some("mixin" | "function")
            ) && is_sass_symbol_declaration_kind(kind)
                && let Some((signature, snippet)) =
                    sass_callable_definition_render_parts(source, position)
            {
                parts.signature = Some(signature);
                parts.snippet = snippet;
                parts.render_source = "callableBlockSnippet";
            }
        }
        _ => {
            parts.snippet = name.to_string();
            parts.render_source = "candidateNameFallback";
        }
    }

    parts
}

pub fn summarize_omena_query_missing_custom_property_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declaration_names = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| candidate.name.as_str())
        .collect::<BTreeSet<_>>();
    if declaration_names.is_empty() {
        return Vec::new();
    }

    let insertion_range = end_of_source_range(source);
    candidates
        .iter()
        .filter(|candidate| {
            candidate.kind == "customPropertyReference"
                && !declaration_names.contains(candidate.name.as_str())
        })
        .map(|candidate| OmenaQueryStyleDiagnosticV0 {
            code: "missingCustomProperty",
            range: candidate.range,
            message: format!(
                "CSS custom property '{}' not found in indexed style tokens.",
                candidate.name
            ),
            create_custom_property: Some(OmenaQueryCreateCustomPropertyActionV0 {
                uri: style_uri.to_string(),
                range: insertion_range,
                new_text: format!("\n\n:root {{\n  {}: ;\n}}\n", candidate.name),
                property_name: candidate.name.clone(),
            }),
        })
        .collect()
}

pub fn read_omena_query_cascade_at_position(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
    position: ParserPositionV0,
) -> Option<OmenaQueryCascadeAtPositionV0> {
    let graph =
        summarize_omena_query_style_semantic_graph_from_source(style_path, style_source, input)?;
    Some(read_omena_query_cascade_at_position_from_graph(
        style_path,
        style_source,
        &graph,
        position,
    ))
}

pub fn read_omena_query_cascade_at_position_from_graph(
    style_path: &str,
    style_source: &str,
    graph: &StyleSemanticGraphSummaryV0,
    position: ParserPositionV0,
) -> OmenaQueryCascadeAtPositionV0 {
    let positioned_references = positioned_custom_property_reference_facts(
        style_source,
        graph.parser_facts.custom_properties.ref_facts.as_slice(),
    );
    let reference = positioned_references
        .iter()
        .find(|(_, range)| parser_range_contains_position(range, position));

    let Some((reference, reference_range)) = reference else {
        return OmenaQueryCascadeAtPositionV0 {
            schema_version: "0",
            product: "omena-query.read-cascade-at-position",
            style_path: style_path.to_string(),
            query_position: position,
            status: "noCustomPropertyReference",
            cascade_engine: "omena-cascade",
            reference_name: None,
            reference_range: None,
            winner_declaration_source_order: None,
            winner_declaration_file_path: None,
            winner_declaration_range: None,
            winner_context_kind: None,
            candidate_declaration_count: 0,
            shadowed_declaration_source_orders: Vec::new(),
        };
    };

    let ranking = graph
        .design_token_semantics
        .cascade_ranking_signal
        .ranked_references
        .iter()
        .find(|ranking| {
            ranking.reference_name == reference.name
                && ranking.reference_source_order == reference.source_order
        });

    OmenaQueryCascadeAtPositionV0 {
        schema_version: "0",
        product: "omena-query.read-cascade-at-position",
        style_path: style_path.to_string(),
        query_position: position,
        status: if ranking.is_some() {
            "resolved"
        } else {
            "unresolved"
        },
        cascade_engine: "omena-cascade",
        reference_name: Some(reference.name.clone()),
        reference_range: Some(*reference_range),
        winner_declaration_source_order: ranking
            .map(|ranking| ranking.winner_declaration_source_order),
        winner_declaration_file_path: ranking
            .and_then(|ranking| ranking.winner_declaration_file_path.clone()),
        winner_declaration_range: ranking.and_then(|ranking| ranking.winner_declaration_range),
        winner_context_kind: ranking.map(|ranking| ranking.winner_context_kind),
        candidate_declaration_count: ranking
            .map(|ranking| ranking.candidate_declaration_count)
            .unwrap_or(0),
        shadowed_declaration_source_orders: ranking
            .map(|ranking| ranking.shadowed_declaration_source_orders.clone())
            .unwrap_or_default(),
    }
}

pub fn summarize_omena_query_missing_selector_diagnostic(
    target_style_uri: &str,
    target_style_source: &str,
    selector_name: &str,
    source_reference_range: ParserRangeV0,
) -> OmenaQuerySourceDiagnosticV0 {
    let insertion_range = end_of_source_range(target_style_source);
    let has_existing_style_content = !target_style_source.trim().is_empty();
    OmenaQuerySourceDiagnosticV0 {
        code: "missingSelector",
        range: source_reference_range,
        message: format!(
            "CSS Module selector '.{selector_name}' not found in indexed style tokens."
        ),
        create_selector: Some(OmenaQueryCreateSelectorActionV0 {
            uri: target_style_uri.to_string(),
            range: insertion_range,
            new_text: if has_existing_style_content {
                format!("\n\n.{selector_name} {{\n}}\n")
            } else {
                format!(".{selector_name} {{\n}}\n")
            },
            selector_name: selector_name.to_string(),
        }),
    }
}

pub fn resolve_omena_query_source_provider_candidates(
    source_candidates: Vec<OmenaQuerySourceSelectorCandidateV0>,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> OmenaQuerySourceProviderCandidateResolutionV0 {
    if definitions.is_empty() {
        return OmenaQuerySourceProviderCandidateResolutionV0 {
            schema_version: "0",
            product: "omena-query.source-provider-candidate-resolution",
            matched: Vec::new(),
            unresolved: Vec::new(),
        };
    }

    let (mut matched, mut unresolved): (Vec<_>, Vec<_>) =
        source_candidates.into_iter().partition(|candidate| {
            definitions.iter().any(|definition| {
                source_selector_candidate_matches_definition(candidate, definition)
            })
        });
    matched.sort();
    unresolved.sort();
    OmenaQuerySourceProviderCandidateResolutionV0 {
        schema_version: "0",
        product: "omena-query.source-provider-candidate-resolution",
        matched,
        unresolved,
    }
}

pub fn resolve_omena_query_style_selector_definitions_for_source_candidate(
    candidate: &OmenaQuerySourceSelectorCandidateV0,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> Vec<OmenaQueryStyleSelectorDefinitionV0> {
    let mut matched = definitions
        .iter()
        .filter(|definition| source_selector_candidate_matches_definition(candidate, definition))
        .cloned()
        .collect::<Vec<_>>();
    matched.sort_by_key(|definition| {
        (
            definition.uri.clone(),
            definition.range.start.line,
            definition.range.start.character,
            definition.name.clone(),
        )
    });
    matched.dedup();
    matched
}

pub fn resolve_omena_query_source_candidate_selector_names(
    candidate: &OmenaQuerySourceSelectorCandidateV0,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    target_style_uri: Option<&str>,
) -> Vec<String> {
    if candidate.kind != "sourceSelectorPrefixReference" {
        return vec![candidate.name.clone()];
    }

    let mut names = definitions
        .iter()
        .filter(|definition| source_selector_candidate_matches_definition(candidate, definition))
        .filter(|definition| {
            candidate
                .target_style_uri
                .as_deref()
                .or(target_style_uri)
                .is_none_or(|target_uri| target_uri == definition.uri)
        })
        .map(|definition| definition.name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

pub fn resolve_omena_query_selector_rename_edits(
    selector_name: &str,
    new_name: &str,
    target_style_uri: Option<&str>,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    references: &[OmenaQuerySourceSelectorReferenceEditTargetV0],
) -> Vec<OmenaQueryWorkspaceTextEditV0> {
    let replacement = new_name.trim_start_matches('.');
    if replacement.is_empty() {
        return Vec::new();
    }

    let mut edits = definitions
        .iter()
        .filter(|definition| definition.name == selector_name)
        .filter(|definition| target_style_uri.is_none_or(|target_uri| target_uri == definition.uri))
        .map(|definition| OmenaQueryWorkspaceTextEditV0 {
            uri: definition.uri.clone(),
            range: definition.range,
            new_text: replacement.to_string(),
        })
        .chain(
            references
                .iter()
                .filter(|reference| reference.name == selector_name)
                .filter(|reference| {
                    source_reference_matches_target_style(reference, target_style_uri)
                })
                .map(|reference| OmenaQueryWorkspaceTextEditV0 {
                    uri: reference.uri.clone(),
                    range: reference.range,
                    new_text: replacement.to_string(),
                }),
        )
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| {
        (
            edit.uri.clone(),
            edit.range.start.line,
            edit.range.start.character,
            edit.range.end.line,
            edit.range.end.character,
        )
    });
    edits
}

pub fn is_omena_query_sass_symbol_candidate_kind(kind: &str) -> bool {
    omena_query_sass_symbol_kind_from_candidate_kind(kind).is_some()
}

pub fn is_omena_query_sass_symbol_reference_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableReference"
            | "sassMixinInclude"
            | "sassFunctionCall"
            | "sassMixinReference"
            | "sassFunctionReference"
            | "sassSymbolReference"
    )
}

pub fn is_omena_query_sass_symbol_declaration_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableDeclaration"
            | "sassMixinDeclaration"
            | "sassFunctionDeclaration"
            | "sassSymbolDeclaration"
    )
}

pub fn omena_query_sass_symbol_kind_from_candidate_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "sassVariableDeclaration" | "sassVariableReference" => Some("variable"),
        "sassMixinDeclaration" | "sassMixinInclude" | "sassMixinReference" => Some("mixin"),
        "sassFunctionDeclaration" | "sassFunctionCall" | "sassFunctionReference" => {
            Some("function")
        }
        "sassSymbolDeclaration" | "sassSymbolReference" => Some("symbol"),
        _ => None,
    }
}

pub fn omena_query_sass_symbol_target_matches(
    candidate_kind: &str,
    candidate_name: &str,
    candidate_namespace: Option<&str>,
    target_kind: &str,
    target_name: &str,
    target_namespace: Option<&str>,
) -> bool {
    candidate_name == target_name
        && candidate_namespace == target_namespace
        && omena_query_sass_symbol_kind_from_candidate_kind(candidate_kind)
            == omena_query_sass_symbol_kind_from_candidate_kind(target_kind)
}

pub fn resolve_omena_query_sass_symbol_declarations(
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    symbol_kind: &str,
    name: &str,
) -> Vec<OmenaQueryStyleHoverCandidateV0> {
    candidates
        .iter()
        .filter(|target| {
            is_omena_query_sass_symbol_declaration_kind(target.kind)
                && omena_query_sass_symbol_kind_from_candidate_kind(target.kind)
                    == Some(symbol_kind)
                && target.name == name
        })
        .cloned()
        .collect()
}

pub fn resolve_omena_query_sass_module_use_sources_for_candidate(
    sources: &OmenaQuerySassModuleSourcesV0,
    namespace: Option<&str>,
) -> Vec<String> {
    let mut selected = sources
        .module_use_edges
        .iter()
        .filter(|edge| {
            if let Some(namespace) = namespace {
                edge.namespace.as_deref() == Some(namespace)
            } else {
                edge.namespace_kind == "wildcard"
            }
        })
        .filter(|edge| !is_sass_builtin_module_source(edge.source.as_str()))
        .map(|edge| edge.source.clone())
        .collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    selected
}

pub fn resolve_omena_query_sass_forward_sources(
    sources: &OmenaQuerySassModuleSourcesV0,
) -> Vec<String> {
    let mut selected = sources
        .module_forward_sources
        .iter()
        .filter(|source| !is_sass_builtin_module_source(source.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    selected
}

pub fn summarize_omena_query_sass_module_sources(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQuerySassModuleSourcesV0> {
    let sheet = parse_style_module(style_path, style_source)?;
    let index = summarize_css_modules_intermediate(&sheet);
    Some(OmenaQuerySassModuleSourcesV0 {
        schema_version: "0",
        product: "omena-query.sass-module-sources",
        module_use_edges: index
            .sass
            .module_use_edges
            .into_iter()
            .map(|edge| OmenaQuerySassModuleUseEdgeV0 {
                source: edge.source,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace,
            })
            .collect(),
        module_forward_sources: index.sass.module_forward_sources,
    })
}

pub fn summarize_omena_query_style_semantic_graph_batch_from_sources<'a>(
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    input: &EngineInputV2,
) -> OmenaQueryStyleSemanticGraphBatchOutputV0 {
    summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests(
        styles,
        input,
        &[],
    )
}

pub fn summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests<'a>(
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    input: &EngineInputV2,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryStyleSemanticGraphBatchOutputV0 {
    let style_sources = styles.into_iter().collect::<Vec<_>>();
    let parsed_styles = style_sources
        .iter()
        .filter_map(|(style_path, style_source)| {
            parse_style_module(style_path, style_source)
                .map(|sheet| ((*style_path).to_string(), sheet))
        })
        .collect::<Vec<_>>();
    let workspace_declarations = parsed_styles
        .iter()
        .flat_map(|(style_path, sheet)| {
            collect_omena_bridge_design_token_workspace_declarations(style_path, sheet)
        })
        .collect::<Vec<_>>();
    let graphs = style_sources
        .into_iter()
        .map(
            |(style_path, _style_source)| OmenaQueryStyleSemanticGraphBatchEntryV0 {
                style_path: style_path.to_string(),
                graph: parsed_style_by_path(&parsed_styles, style_path).map(|sheet| {
                    let import_reachable_declarations =
                        filter_import_reachable_design_token_workspace_declarations(
                            style_path,
                            &parsed_styles,
                            &workspace_declarations,
                            package_manifests,
                        );
                    summarize_omena_bridge_style_semantic_graph_for_path_with_scoped_workspace_declarations(
                        sheet,
                        input,
                        Some(style_path),
                        &import_reachable_declarations,
                        DesignTokenExternalDeclarationCandidateScopeV0::CrossFileImportGraph,
                    )
                }),
            },
        )
        .collect::<Vec<_>>();

    OmenaQueryStyleSemanticGraphBatchOutputV0 {
        schema_version: "0",
        product: "omena-semantic.style-semantic-graph-batch",
        graphs,
    }
}

fn parsed_style_by_path<'a>(
    parsed_styles: &'a [(String, Stylesheet)],
    style_path: &str,
) -> Option<&'a Stylesheet> {
    parsed_styles
        .iter()
        .find(|(parsed_style_path, _sheet)| parsed_style_path == style_path)
        .map(|(_style_path, sheet)| sheet)
}

fn filter_import_reachable_design_token_workspace_declarations(
    target_style_path: &str,
    parsed_styles: &[(String, Stylesheet)],
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<DesignTokenWorkspaceDeclarationFactV0> {
    let reachable_style_paths = collect_import_reachable_style_path_metadata(
        target_style_path,
        parsed_styles,
        package_manifests,
    );
    workspace_declarations
        .iter()
        .filter_map(|declaration| {
            if declaration.file_path == target_style_path {
                return Some(declaration.clone());
            }
            let reachability = reachable_style_paths.get(declaration.file_path.as_str())?;
            let mut declaration = declaration.clone();
            declaration.import_graph_distance = Some(reachability.distance);
            declaration.import_graph_order = Some(reachability.order);
            Some(declaration)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImportReachability {
    distance: usize,
    order: usize,
}

fn collect_import_reachable_style_path_metadata(
    target_style_path: &str,
    parsed_styles: &[(String, Stylesheet)],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<String, ImportReachability> {
    let mut reachable_style_paths = BTreeMap::new();
    let available_style_paths = parsed_styles
        .iter()
        .map(|(style_path, _sheet)| style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut pending_style_paths = collect_import_reachable_direct_style_paths(
        target_style_path,
        parsed_styles,
        &available_style_paths,
        package_manifests,
    )
    .into_iter()
    .map(|style_path| (style_path, 1usize))
    .collect::<VecDeque<_>>();
    let style_by_path = parsed_styles
        .iter()
        .map(|(style_path, sheet)| (style_path.as_str(), sheet))
        .collect::<BTreeMap<_, _>>();
    let mut visit_order = 0usize;

    while let Some((style_path, distance)) = pending_style_paths.pop_front() {
        if style_path == target_style_path || reachable_style_paths.contains_key(&style_path) {
            continue;
        }
        reachable_style_paths.insert(
            style_path.clone(),
            ImportReachability {
                distance,
                order: visit_order,
            },
        );
        visit_order += 1;

        let Some(sheet) = style_by_path.get(style_path.as_str()) else {
            continue;
        };
        for source in collect_sass_module_sources(sheet) {
            if let Some(next_style_path) = resolve_style_module_source(
                &style_path,
                &source,
                &available_style_paths,
                package_manifests,
            ) {
                pending_style_paths.push_back((next_style_path, distance + 1));
            }
        }
    }

    reachable_style_paths
}

fn collect_import_reachable_direct_style_paths(
    target_style_path: &str,
    parsed_styles: &[(String, Stylesheet)],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<String> {
    let Some(target_sheet) = parsed_style_by_path(parsed_styles, target_style_path) else {
        return Vec::new();
    };
    collect_sass_module_sources(target_sheet)
        .into_iter()
        .filter_map(|source| {
            resolve_style_module_source(
                target_style_path,
                &source,
                available_style_paths,
                package_manifests,
            )
        })
        .collect()
}

fn collect_sass_module_sources(sheet: &Stylesheet) -> Vec<String> {
    let summary = summarize_css_modules_intermediate(sheet);
    let mut sources = Vec::new();
    for edge in summary.sass.module_use_edges {
        push_unique_string(&mut sources, edge.source);
    }
    for source in summary.sass.module_forward_sources {
        push_unique_string(&mut sources, source);
    }
    for source in summary.sass.module_import_sources {
        push_unique_string(&mut sources, source);
    }
    sources
}

fn resolve_style_module_source(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Option<String> {
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    resolve_omena_resolver_style_module_source(
        from_style_path,
        source,
        available_style_paths,
        &resolver_package_manifests,
    )
}

fn collect_style_selector_hover_candidates_from_parser_facts(
    definition_facts: &[engine_style_parser::ParserIndexSelectorDefinitionFactV0],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in definition_facts {
        if seen.insert((fact.byte_span.start, fact.byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: "selector",
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSelectorDefinitionFacts",
                namespace: None,
            });
        }
    }
}

fn collect_custom_property_hover_candidates(
    source: &str,
    decl_facts: &[engine_style_parser::ParserIndexCustomPropertyDeclFactV0],
    ref_names: &[String],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in decl_facts {
        if seen.insert((fact.byte_span.start, fact.byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: "customPropertyDeclaration",
                name: fact.name.clone(),
                range: fact.range,
                source: "openedStyleDocumentIndex",
                namespace: None,
            });
        }
    }

    for name in ref_names {
        for byte_span in custom_property_ref_byte_spans(source, name) {
            if seen.insert((byte_span.start, byte_span.end, name.clone())) {
                candidates.push(OmenaQueryStyleHoverCandidateV0 {
                    kind: "customPropertyReference",
                    name: name.clone(),
                    range: parser_range_for_byte_span(source, byte_span),
                    source: "openedStyleDocumentIndex",
                    namespace: None,
                });
            }
        }
    }
}

fn collect_sass_symbol_hover_candidates(
    decl_facts: &[engine_style_parser::ParserIndexSassSymbolDeclFactV0],
    ref_facts: &[engine_style_parser::ParserIndexSassSelectorSymbolFactV0],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in decl_facts {
        if seen.insert((
            fact.byte_span.start,
            fact.byte_span.end,
            format!("{}:{}", fact.symbol_kind, fact.name),
        )) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: sass_symbol_declaration_candidate_kind(fact.symbol_kind),
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSassSymbolFacts",
                namespace: None,
            });
        }
    }

    for fact in ref_facts {
        if seen.insert((
            fact.byte_span.start,
            fact.byte_span.end,
            format!(
                "{}:{}:{}",
                fact.symbol_kind,
                fact.namespace.as_deref().unwrap_or_default(),
                fact.name
            ),
        )) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: sass_symbol_reference_candidate_kind(fact.symbol_kind, fact.role),
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSassSymbolFacts",
                namespace: fact.namespace.clone(),
            });
        }
    }
}

fn collect_sass_partial_evaluator_selector_candidates(
    source: &str,
    nodes: &[engine_style_parser::SyntaxNode],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for node in nodes {
        if let Some(SyntaxNodePayload::AtRule(at_rule)) = &node.payload
            && at_rule.kind == AtRuleKind::Include
        {
            let range_span = ParserByteSpanV0 {
                start: node.header_span.unwrap_or(node.span).start,
                end: node.header_span.unwrap_or(node.span).end,
            };
            for selector_name in infer_sass_include_generated_selector_names(&at_rule.params) {
                if seen.insert((range_span.start, range_span.end, selector_name.clone())) {
                    candidates.push(OmenaQueryStyleHoverCandidateV0 {
                        kind: "selector",
                        name: selector_name,
                        range: parser_range_for_byte_span(source, range_span),
                        source: "sassPartialEvaluatorGeneratedSelectors",
                        namespace: None,
                    });
                }
            }
        }
        collect_sass_partial_evaluator_selector_candidates(
            source,
            &node.children,
            seen,
            candidates,
        );
    }
}

fn infer_sass_include_generated_selector_names(params: &str) -> Vec<String> {
    let Some(prefix) = sass_named_argument_string_value(params, "prefix") else {
        return Vec::new();
    };
    if prefix.is_empty() || !prefix.chars().all(is_css_identifier_continue) {
        return Vec::new();
    }
    let mut selectors = sass_first_map_string_keys(params)
        .into_iter()
        .filter(|key| !key.is_empty() && key.chars().all(is_css_identifier_continue))
        .map(|key| format!("{prefix}-{key}"))
        .collect::<Vec<_>>();
    selectors.sort();
    selectors.dedup();
    selectors
}

fn sass_named_argument_string_value(params: &str, name: &str) -> Option<String> {
    let needle = format!("${name}");
    let mut cursor = 0usize;
    while let Some(relative_match) = params[cursor..].find(needle.as_str()) {
        let name_start = cursor + relative_match;
        let name_end = name_start + needle.len();
        if !sass_identifier_boundary(params, name_start, name_end) {
            cursor = name_end;
            continue;
        }
        let colon_offset = skip_ascii_whitespace(params, name_end);
        if params.as_bytes().get(colon_offset) != Some(&b':') {
            cursor = name_end;
            continue;
        }
        let value_start = skip_ascii_whitespace(params, colon_offset + 1);
        return sass_string_literal_value(params, value_start).map(|(value, _)| value);
    }
    None
}

fn sass_first_map_string_keys(params: &str) -> Vec<String> {
    let mut cursor = 0usize;
    while cursor < params.len() {
        let Some(open_relative) = params[cursor..].find('(') else {
            break;
        };
        let open = cursor + open_relative;
        let Some(close) = matching_style_block_end(params, open, b'(', b')') else {
            break;
        };
        let keys = sass_map_string_keys(params, open + 1, close);
        if !keys.is_empty() {
            return keys;
        }
        cursor = open + 1;
    }
    Vec::new()
}

fn sass_map_string_keys(params: &str, start: usize, end: usize) -> Vec<String> {
    split_top_level_style_segments(params, start, end, b',')
        .into_iter()
        .filter_map(|(entry_start, entry_end)| {
            let key_start = skip_ascii_whitespace(params, entry_start);
            let (key, key_end) = sass_string_literal_value(params, key_start)?;
            let colon_offset = skip_ascii_whitespace(params, key_end);
            (colon_offset < entry_end && params.as_bytes().get(colon_offset) == Some(&b':'))
                .then_some(key)
        })
        .collect()
}

fn sass_string_literal_value(source: &str, quote_offset: usize) -> Option<(String, usize)> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let literal_end = skip_style_string_literal(source, quote_offset, source.len())?;
    let value_end = literal_end.saturating_sub(1);
    source
        .get(quote_offset + 1..value_end)
        .map(|value| (value.to_string(), literal_end))
}

fn sass_identifier_boundary(source: &str, start: usize, end: usize) -> bool {
    let before = source
        .get(..start)
        .and_then(|prefix| prefix.chars().next_back())
        .is_none_or(|ch| !is_css_identifier_continue(ch) && ch != '$');
    let after = source
        .get(end..)
        .and_then(|suffix| suffix.chars().next())
        .is_none_or(|ch| !is_css_identifier_continue(ch));
    before && after
}

fn sass_symbol_declaration_candidate_kind(symbol_kind: &str) -> &'static str {
    match symbol_kind {
        "variable" => "sassVariableDeclaration",
        "mixin" => "sassMixinDeclaration",
        "function" => "sassFunctionDeclaration",
        _ => "sassSymbolDeclaration",
    }
}

fn is_sass_symbol_candidate_kind(kind: &str) -> bool {
    sass_symbol_kind_from_candidate_kind(kind).is_some()
}

fn is_sass_symbol_declaration_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableDeclaration"
            | "sassMixinDeclaration"
            | "sassFunctionDeclaration"
            | "sassSymbolDeclaration"
    )
}

fn sass_symbol_kind_from_candidate_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "sassVariableDeclaration" | "sassVariableReference" => Some("variable"),
        "sassMixinDeclaration" | "sassMixinInclude" | "sassMixinReference" => Some("mixin"),
        "sassFunctionDeclaration" | "sassFunctionCall" | "sassFunctionReference" => {
            Some("function")
        }
        "sassSymbolDeclaration" | "sassSymbolReference" => Some("symbol"),
        _ => None,
    }
}

fn sass_symbol_reference_candidate_kind(symbol_kind: &str, role: &str) -> &'static str {
    match (symbol_kind, role) {
        ("variable", _) => "sassVariableReference",
        ("mixin", "include") => "sassMixinInclude",
        ("function", "call") => "sassFunctionCall",
        ("mixin", _) => "sassMixinReference",
        ("function", _) => "sassFunctionReference",
        _ => "sassSymbolReference",
    }
}

fn sass_variable_value_from_declaration_line(line: &str) -> Option<String> {
    let (_, value) = line.split_once(':')?;
    let value = value
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches("!default")
        .trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn sass_callable_definition_render_parts(
    source: &str,
    position: ParserPositionV0,
) -> Option<(String, String)> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let open_brace = source[line_start..].find('{')? + line_start;
    let close_brace = matching_style_block_end(source, open_brace, b'{', b'}')?;
    let signature = source[line_start..open_brace].trim().to_string();
    let body = source[open_brace + 1..close_brace].trim();
    if signature.is_empty() || body.is_empty() {
        return None;
    }
    Some((signature, trim_hover_snippet(body)))
}

fn rule_snippet_around_position(source: &str, position: ParserPositionV0) -> Option<String> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let open_brace = source[line_start..].find('{')? + line_start;
    let mut depth = 0usize;
    let mut cursor = open_brace;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let snippet = source[line_start..=cursor].trim();
                    return Some(trim_hover_snippet(snippet));
                }
            }
            _ => {}
        }
        cursor = advance_style_scan_cursor(source, cursor, source.len());
    }
    None
}

fn line_snippet_at_position(source: &str, position: ParserPositionV0) -> Option<String> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    Some(source[line_start..line_end].trim().to_string())
}

fn trim_hover_snippet(snippet: &str) -> String {
    const MAX_SNIPPET_LEN: usize = 1200;
    if snippet.len() <= MAX_SNIPPET_LEN {
        return snippet.to_string();
    }
    let end = char_boundary_floor(snippet, MAX_SNIPPET_LEN);
    format!("{}...", snippet[..end].trim_end())
}

fn custom_property_ref_byte_spans(source: &str, name: &str) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut search_offset = 0usize;

    while let Some(relative_match) = source[search_offset..].find(name) {
        let name_start = search_offset + relative_match;
        let name_end = name_start + name.len();
        if source[..name_start].trim_end().ends_with("var(")
            && is_selector_name_boundary(source, name_end)
        {
            spans.push(ParserByteSpanV0 {
                start: name_start,
                end: name_end,
            });
        }
        search_offset += relative_match + name.len();
    }

    spans
}

fn positioned_custom_property_reference_facts<'a>(
    source: &str,
    ref_facts: &'a [engine_style_parser::ParserIndexCustomPropertyRefFactV0],
) -> Vec<(
    &'a engine_style_parser::ParserIndexCustomPropertyRefFactV0,
    ParserRangeV0,
)> {
    let mut ranges_by_name = BTreeMap::<&str, std::collections::VecDeque<ParserRangeV0>>::new();
    for name in ref_facts
        .iter()
        .map(|fact| fact.name.as_str())
        .collect::<BTreeSet<_>>()
    {
        ranges_by_name.insert(
            name,
            custom_property_ref_byte_spans(source, name)
                .into_iter()
                .map(|span| parser_range_for_byte_span(source, span))
                .collect(),
        );
    }

    let mut ordered_ref_facts = ref_facts.iter().collect::<Vec<_>>();
    ordered_ref_facts.sort_by_key(|fact| fact.source_order);
    ordered_ref_facts
        .into_iter()
        .filter_map(|fact| {
            ranges_by_name
                .get_mut(fact.name.as_str())
                .and_then(std::collections::VecDeque::pop_front)
                .map(|range| (fact, range))
        })
        .collect()
}

fn is_selector_name_boundary(source: &str, byte_offset: usize) -> bool {
    source[byte_offset..]
        .chars()
        .next()
        .is_none_or(|ch| !is_css_identifier_continue(ch))
}

fn is_css_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}

fn parser_range_for_byte_span(source: &str, span: ParserByteSpanV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: parser_position_for_byte_offset(source, span.start),
        end: parser_position_for_byte_offset(source, span.end),
    }
}

fn end_of_source_range(source: &str) -> ParserRangeV0 {
    let position = parser_position_for_byte_offset(source, source.len());
    ParserRangeV0 {
        start: position,
        end: position,
    }
}

fn parser_range_contains_position(range: &ParserRangeV0, position: ParserPositionV0) -> bool {
    parser_position_is_after_or_equal(position, range.start)
        && parser_position_is_before(position, range.end)
}

fn parser_position_is_after_or_equal(position: ParserPositionV0, start: ParserPositionV0) -> bool {
    position.line > start.line
        || (position.line == start.line && position.character >= start.character)
}

fn parser_position_is_before(position: ParserPositionV0, end: ParserPositionV0) -> bool {
    position.line < end.line || (position.line == end.line && position.character < end.character)
}

fn parser_position_for_byte_offset(source: &str, offset: usize) -> ParserPositionV0 {
    let clamped_offset = offset.min(source.len());
    let mut line = 0usize;
    let mut character = 0usize;

    for (byte_index, ch) in source.char_indices() {
        if byte_index >= clamped_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }

    ParserPositionV0 { line, character }
}

fn byte_offset_for_parser_position(source: &str, position: ParserPositionV0) -> Option<usize> {
    let mut current_line = 0usize;
    let mut current_character = 0usize;

    if position.line == 0 && position.character == 0 {
        return Some(0);
    }

    for (byte_index, ch) in source.char_indices() {
        if current_line == position.line && current_character == position.character {
            return Some(byte_index);
        }
        if ch == '\n' {
            current_line += 1;
            current_character = 0;
            if current_line == position.line && position.character == 0 {
                return Some(byte_index + ch.len_utf8());
            }
        } else if current_line == position.line {
            current_character += ch.len_utf16();
        }
    }

    (current_line == position.line && current_character == position.character)
        .then_some(source.len())
}

fn skip_ascii_whitespace(source: &str, mut offset: usize) -> usize {
    while source
        .as_bytes()
        .get(offset)
        .is_some_and(u8::is_ascii_whitespace)
    {
        offset += 1;
    }
    offset
}

fn matching_style_block_end(
    source: &str,
    open_offset: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    if source.as_bytes().get(open_offset) != Some(&open) {
        return None;
    }
    let mut cursor = advance_style_scan_cursor(source, open_offset, source.len());
    let mut depth = 1usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_style_string_literal(source, cursor, source.len())?;
            }
            byte if byte == open => {
                depth += 1;
                cursor = advance_style_scan_cursor(source, cursor, source.len());
            }
            byte if byte == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor);
                }
                cursor = advance_style_scan_cursor(source, cursor, source.len());
            }
            _ => cursor = advance_style_scan_cursor(source, cursor, source.len()),
        }
    }
    None
}

fn split_top_level_style_segments(
    source: &str,
    start: usize,
    end: usize,
    delimiter: u8,
) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let end = char_boundary_floor(source, end);
    let mut segment_start = char_boundary_ceil(source, start).min(end);
    let mut cursor = segment_start;
    let mut depth = 0usize;
    while cursor < end {
        match source.as_bytes().get(cursor).copied() {
            Some(b'\'' | b'"' | b'`') => {
                cursor = skip_style_string_literal(source, cursor, end).unwrap_or(end);
            }
            Some(b'(' | b'[' | b'{') => {
                depth += 1;
                cursor = advance_style_scan_cursor(source, cursor, end);
            }
            Some(b')' | b']' | b'}') => {
                depth = depth.saturating_sub(1);
                cursor = advance_style_scan_cursor(source, cursor, end);
            }
            Some(byte) if byte == delimiter && depth == 0 => {
                segments.push((segment_start, cursor));
                cursor = advance_style_scan_cursor(source, cursor, end);
                segment_start = cursor;
            }
            Some(_) => cursor = advance_style_scan_cursor(source, cursor, end),
            None => break,
        }
    }
    if segment_start <= end {
        segments.push((segment_start, end));
    }
    segments
}

fn skip_style_string_literal(source: &str, quote_offset: usize, limit: usize) -> Option<usize> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    let limit = char_boundary_floor(source, limit);
    let mut cursor = quote_offset + 1;
    while cursor < limit {
        let byte = source.as_bytes().get(cursor).copied()?;
        if byte == b'\\' {
            cursor = advance_style_escaped_char(source, cursor, limit);
            continue;
        }
        if byte == quote {
            return Some(cursor + 1);
        }
        cursor = advance_style_scan_cursor(source, cursor, limit);
    }
    None
}

fn advance_style_escaped_char(source: &str, slash_offset: usize, limit: usize) -> usize {
    let after_slash = advance_style_scan_cursor(source, slash_offset, limit);
    advance_style_scan_cursor(source, after_slash, limit)
}

fn advance_style_scan_cursor(source: &str, cursor: usize, limit: usize) -> usize {
    let cursor = char_boundary_ceil(source, cursor);
    let limit = char_boundary_floor(source, limit);
    if cursor >= limit {
        return limit;
    }
    char_boundary_ceil(source, cursor + 1).min(limit)
}

fn char_boundary_floor(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index > 0 && !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn char_boundary_ceil(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index < source.len() && !source.is_char_boundary(index) {
        index += 1;
    }
    index
}

fn source_selector_candidate_matches_definition(
    candidate: &OmenaQuerySourceSelectorCandidateV0,
    definition: &OmenaQueryStyleSelectorDefinitionV0,
) -> bool {
    let selector_matches = if candidate.kind == "sourceSelectorPrefixReference" {
        definition.name.starts_with(candidate.name.as_str())
    } else {
        definition.name == candidate.name
    };
    selector_matches
        && candidate
            .target_style_uri
            .as_deref()
            .is_none_or(|target_uri| target_uri == definition.uri)
}

fn source_reference_matches_target_style(
    reference: &OmenaQuerySourceSelectorReferenceEditTargetV0,
    target_style_uri: Option<&str>,
) -> bool {
    target_style_uri.is_none_or(|target_uri| {
        reference
            .target_style_uri
            .as_deref()
            .is_none_or(|candidate_target_uri| candidate_target_uri == target_uri)
    })
}

fn is_sass_builtin_module_source(source: &str) -> bool {
    source.starts_with("sass:")
}

fn style_language_label(language: StyleLanguage) -> &'static str {
    match language {
        StyleLanguage::Css => "css",
        StyleLanguage::Scss => "scss",
        StyleLanguage::Less => "less",
    }
}

fn omena_parser_style_dialect_label(dialect: OmenaParserStyleDialect) -> &'static str {
    match dialect {
        OmenaParserStyleDialect::Css => "css",
        OmenaParserStyleDialect::Scss => "scss",
        OmenaParserStyleDialect::Sass => "sass",
        OmenaParserStyleDialect::Less => "less",
    }
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}
