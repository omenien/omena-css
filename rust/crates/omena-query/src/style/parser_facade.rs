use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedIcssFactKind, ParsedSassModuleEdgeFactKind,
    ParsedSassSymbolFactKind, ParsedSelectorFactKind, ParsedVariableFactKind, collect_style_facts,
};

use crate::*;

pub fn summarize_omena_query_style_document(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQueryStyleDocumentSummaryV0> {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_style_facts(style_source, dialect);
    let mut selector_names = Vec::new();
    let mut custom_property_decl_names = Vec::new();
    let mut custom_property_ref_names = Vec::new();
    let mut sass_module_use_sources = BTreeSet::new();
    let mut sass_module_forward_sources = BTreeSet::new();

    for selector in facts.selectors {
        if selector.kind == ParsedSelectorFactKind::Class {
            selector_names.push(selector.name);
        }
    }

    for variable in facts.variables {
        match variable.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration => {
                custom_property_decl_names.push(variable.name);
            }
            ParsedVariableFactKind::CustomPropertyReference => {
                custom_property_ref_names.push(variable.name);
            }
            _ => {}
        }
    }

    for edge in facts.sass_module_edges {
        match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => {
                sass_module_use_sources.insert(edge.source);
            }
            ParsedSassModuleEdgeFactKind::Forward => {
                sass_module_forward_sources.insert(edge.source);
            }
            ParsedSassModuleEdgeFactKind::Import => {
                sass_module_use_sources.insert(edge.source);
            }
        }
    }

    Some(OmenaQueryStyleDocumentSummaryV0 {
        schema_version: "0",
        product: "omena-query.style-document-summary",
        language: omena_parser_style_dialect_label(dialect),
        selector_names,
        custom_property_decl_names,
        custom_property_ref_names,
        sass_module_use_sources: sass_module_use_sources.into_iter().collect(),
        sass_module_forward_sources: sass_module_forward_sources.into_iter().collect(),
        diagnostic_count: facts.error_count,
    })
}

pub fn summarize_omena_query_omena_parser_style_facts(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> OmenaQueryOmenaParserStyleFactsV0 {
    let facts = collect_style_facts(style_source, dialect);
    let sass_symbol_resolution =
        summarize_omena_query_sass_symbol_resolution(facts.sass_symbols.as_slice());
    let mut class_selector_names = Vec::new();
    let mut id_selector_names = Vec::new();
    let mut placeholder_selector_names = Vec::new();
    let mut keyframe_names = Vec::new();
    let mut animation_reference_names = Vec::new();
    let mut css_module_value_definition_names = BTreeSet::new();
    let mut css_module_value_reference_names = BTreeSet::new();
    let mut css_module_value_import_sources = BTreeSet::new();
    let mut css_module_composes_target_names = BTreeSet::new();
    let mut css_module_composes_import_sources = BTreeSet::new();
    let mut icss_export_names = BTreeSet::new();
    let mut icss_import_local_names = BTreeSet::new();
    let mut icss_import_remote_names = BTreeSet::new();
    let mut icss_import_sources = BTreeSet::new();
    let mut variable_names = BTreeSet::new();
    let mut sass_symbol_declaration_names = BTreeSet::new();
    let mut sass_symbol_reference_names = BTreeSet::new();
    let mut sass_module_use_sources = BTreeSet::new();
    let mut sass_module_forward_sources = BTreeSet::new();
    let mut sass_module_import_sources = BTreeSet::new();
    let mut custom_property_names = BTreeSet::new();
    let mut custom_property_decl_names = BTreeSet::new();
    let mut custom_property_ref_names = BTreeSet::new();

    for selector in facts.selectors {
        match selector.kind {
            ParsedSelectorFactKind::Class => class_selector_names.push(selector.name),
            ParsedSelectorFactKind::Id => id_selector_names.push(selector.name),
            ParsedSelectorFactKind::Placeholder => placeholder_selector_names.push(selector.name),
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
                custom_property_names.insert(variable.name.clone());
                match variable.kind {
                    ParsedVariableFactKind::CustomPropertyDeclaration => {
                        custom_property_decl_names.insert(variable.name);
                    }
                    ParsedVariableFactKind::CustomPropertyReference => {
                        custom_property_ref_names.insert(variable.name);
                    }
                    _ => {}
                }
            }
        }
    }

    for symbol in &facts.sass_symbols {
        match symbol.role {
            "declaration" => {
                sass_symbol_declaration_names.insert(symbol.name.clone());
            }
            _ => {
                sass_symbol_reference_names.insert(symbol.name.clone());
            }
        }
    }

    for edge in &facts.sass_module_edges {
        match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => {
                sass_module_use_sources.insert(edge.source.clone());
            }
            ParsedSassModuleEdgeFactKind::Forward => {
                sass_module_forward_sources.insert(edge.source.clone());
            }
            ParsedSassModuleEdgeFactKind::Import => {
                sass_module_import_sources.insert(edge.source.clone());
            }
        }
    }

    for animation in facts.animations {
        match animation.kind {
            ParsedAnimationFactKind::KeyframesDeclaration => keyframe_names.push(animation.name),
            ParsedAnimationFactKind::AnimationNameReference => {
                animation_reference_names.push(animation.name);
            }
        }
    }

    for value in facts.css_module_values {
        match value.kind {
            ParsedCssModuleValueFactKind::Definition => {
                css_module_value_definition_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::Reference => {
                css_module_value_reference_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::ImportSource => {
                css_module_value_import_sources.insert(value.name);
            }
        }
    }

    for composes in facts.css_module_composes {
        match composes.kind {
            ParsedCssModuleComposesFactKind::Target => {
                css_module_composes_target_names.insert(composes.name);
            }
            ParsedCssModuleComposesFactKind::ImportSource => {
                css_module_composes_import_sources.insert(composes.name);
            }
        }
    }

    for icss in facts.icss {
        match icss.kind {
            ParsedIcssFactKind::ExportName => {
                icss_export_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportLocalName => {
                icss_import_local_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportRemoteName => {
                icss_import_remote_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportSource => {
                icss_import_sources.insert(icss.name);
            }
        }
    }

    OmenaQueryOmenaParserStyleFactsV0 {
        schema_version: "0",
        product: "omena-query.omena-parser-style-facts",
        dialect: omena_parser_style_dialect_label(dialect),
        class_selector_names,
        id_selector_names,
        placeholder_selector_names,
        keyframe_names,
        animation_reference_names,
        css_module_value_definition_names: css_module_value_definition_names.into_iter().collect(),
        css_module_value_reference_names: css_module_value_reference_names.into_iter().collect(),
        css_module_value_import_sources: css_module_value_import_sources.into_iter().collect(),
        css_module_value_import_edges: facts
            .css_module_value_import_edges
            .into_iter()
            .map(|edge| OmenaQueryCssModuleValueImportEdgeFactV0 {
                remote_name: edge.remote_name,
                local_name: edge.local_name,
                import_source: edge.import_source,
            })
            .collect(),
        css_module_value_definition_edges: facts
            .css_module_value_definition_edges
            .into_iter()
            .map(|edge| OmenaQueryCssModuleValueDefinitionEdgeFactV0 {
                definition_name: edge.definition_name,
                reference_names: edge.reference_names,
            })
            .collect(),
        css_module_composes_target_names: css_module_composes_target_names.into_iter().collect(),
        css_module_composes_import_sources: css_module_composes_import_sources
            .into_iter()
            .collect(),
        css_module_composes_edges: facts
            .css_module_composes_edges
            .into_iter()
            .map(|edge| OmenaQueryCssModuleComposesEdgeFactV0 {
                kind: omena_query_css_module_composes_edge_kind_label(edge.kind),
                owner_selector_names: edge.owner_selector_names,
                target_names: edge.target_names,
                import_source: edge.import_source,
            })
            .collect(),
        icss_export_names: icss_export_names.into_iter().collect(),
        icss_import_local_names: icss_import_local_names.into_iter().collect(),
        icss_import_remote_names: icss_import_remote_names.into_iter().collect(),
        icss_import_sources: icss_import_sources.into_iter().collect(),
        icss_import_edges: facts
            .icss_import_edges
            .into_iter()
            .map(|edge| OmenaQueryIcssImportEdgeFactV0 {
                local_name: edge.local_name,
                remote_name: edge.remote_name,
                import_source: edge.import_source,
            })
            .collect(),
        icss_export_edges: facts
            .icss_export_edges
            .into_iter()
            .map(|edge| OmenaQueryIcssExportEdgeFactV0 {
                export_name: edge.export_name,
                reference_names: edge.reference_names,
            })
            .collect(),
        variable_names: variable_names.into_iter().collect(),
        sass_symbol_declaration_names: sass_symbol_declaration_names.into_iter().collect(),
        sass_symbol_reference_names: sass_symbol_reference_names.into_iter().collect(),
        sass_symbol_facts: facts
            .sass_symbols
            .into_iter()
            .map(|symbol| OmenaQuerySassSymbolFactV0 {
                kind: omena_query_sass_symbol_fact_kind_label(symbol.kind),
                symbol_kind: symbol.symbol_kind,
                name: symbol.name,
                role: symbol.role,
                namespace: symbol.namespace,
            })
            .collect(),
        sass_symbol_resolution,
        sass_module_use_sources: sass_module_use_sources.into_iter().collect(),
        sass_module_forward_sources: sass_module_forward_sources.into_iter().collect(),
        sass_module_import_sources: sass_module_import_sources.into_iter().collect(),
        sass_module_edges: facts
            .sass_module_edges
            .into_iter()
            .map(|edge| OmenaQuerySassModuleEdgeFactV0 {
                kind: omena_query_sass_module_edge_fact_kind_label(edge.kind),
                source: edge.source,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace,
                visibility_filter_kind: edge.visibility_filter_kind,
                visibility_filter_names: edge.visibility_filter_names,
            })
            .collect(),
        custom_property_names: custom_property_names.into_iter().collect(),
        custom_property_decl_names: custom_property_decl_names.into_iter().collect(),
        custom_property_ref_names: custom_property_ref_names.into_iter().collect(),
        at_rule_names: facts
            .at_rules
            .into_iter()
            .map(|at_rule| at_rule.name)
            .collect(),
        parser_error_count: facts.error_count,
    }
}

pub fn summarize_omena_query_omena_parser_css_modules_intermediate(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> omena_parser::ParserIndexSummaryV0 {
    omena_parser::summarize_css_modules_intermediate(style_source, dialect)
}

pub fn summarize_omena_query_omena_parser_lex(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> omena_parser::OmenaParserLexSummaryV0 {
    omena_parser::summarize_omena_parser_lex(style_source, dialect)
}

pub(super) fn omena_parser_dialect_for_style_path(style_path: &str) -> OmenaParserStyleDialect {
    if style_path.ends_with(".sass") {
        OmenaParserStyleDialect::Sass
    } else if style_path.ends_with(".scss") {
        OmenaParserStyleDialect::Scss
    } else if style_path.ends_with(".less") {
        OmenaParserStyleDialect::Less
    } else {
        OmenaParserStyleDialect::Css
    }
}

pub(super) fn omena_parser_style_dialect_label(dialect: OmenaParserStyleDialect) -> &'static str {
    match dialect {
        OmenaParserStyleDialect::Css => "css",
        OmenaParserStyleDialect::Scss => "scss",
        OmenaParserStyleDialect::Sass => "sass",
        OmenaParserStyleDialect::Less => "less",
    }
}

pub(super) fn omena_query_css_module_composes_edge_kind_label(
    kind: ParsedCssModuleComposesEdgeKind,
) -> &'static str {
    match kind {
        ParsedCssModuleComposesEdgeKind::Local => "local",
        ParsedCssModuleComposesEdgeKind::Global => "global",
        ParsedCssModuleComposesEdgeKind::External => "external",
    }
}

pub(super) fn omena_query_sass_symbol_fact_kind_label(
    kind: ParsedSassSymbolFactKind,
) -> &'static str {
    match kind {
        ParsedSassSymbolFactKind::VariableDeclaration => "sassVariableDeclaration",
        ParsedSassSymbolFactKind::VariableReference => "sassVariableReference",
        ParsedSassSymbolFactKind::MixinDeclaration => "sassMixinDeclaration",
        ParsedSassSymbolFactKind::MixinInclude => "sassMixinInclude",
        ParsedSassSymbolFactKind::FunctionDeclaration => "sassFunctionDeclaration",
        ParsedSassSymbolFactKind::FunctionCall => "sassFunctionCall",
    }
}

pub(super) fn omena_query_sass_module_edge_fact_kind_label(
    kind: ParsedSassModuleEdgeFactKind,
) -> &'static str {
    match kind {
        ParsedSassModuleEdgeFactKind::Use => "sassUse",
        ParsedSassModuleEdgeFactKind::Forward => "sassForward",
        ParsedSassModuleEdgeFactKind::Import => "sassImport",
    }
}

fn summarize_omena_query_sass_symbol_resolution(
    symbols: &[omena_parser::ParsedSassSymbolFact],
) -> OmenaQuerySassSymbolResolutionV0 {
    let mut declaration_by_symbol: BTreeMap<
        (&'static str, Option<String>, String),
        (usize, &'static str),
    > = BTreeMap::new();
    let mut declaration_count = 0usize;
    let mut reference_count = 0usize;
    let mut edges = Vec::new();

    for (source_order, symbol) in symbols.iter().enumerate() {
        let kind = omena_query_sass_symbol_fact_kind_label(symbol.kind);
        if omena_query_sass_symbol_fact_kind_is_declaration(symbol.kind) {
            declaration_count += 1;
            declaration_by_symbol.insert(
                (
                    symbol.symbol_kind,
                    symbol.namespace.clone(),
                    symbol.name.clone(),
                ),
                (source_order, kind),
            );
            continue;
        }
        if !omena_query_sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }

        reference_count += 1;
        let declaration = declaration_by_symbol.get(&(
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
        ));
        edges.push(OmenaQuerySassSymbolResolutionEdgeV0 {
            symbol_kind: symbol.symbol_kind,
            name: symbol.name.clone(),
            namespace: symbol.namespace.clone(),
            reference_kind: kind,
            reference_role: symbol.role,
            reference_source_order: source_order,
            declaration_kind: declaration.map(|(_, declaration_kind)| *declaration_kind),
            declaration_source_order: declaration.map(|(declaration_order, _)| *declaration_order),
            status: if declaration.is_some() {
                "resolved"
            } else {
                "unresolved"
            },
        });
    }

    let resolved_reference_count = edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .count();
    OmenaQuerySassSymbolResolutionV0 {
        schema_version: "0",
        product: "omena-query.sass-symbol-same-file-resolution",
        resolution_scope: "same-file",
        declaration_count,
        reference_count,
        resolved_reference_count,
        unresolved_reference_count: reference_count.saturating_sub(resolved_reference_count),
        edges,
        capabilities: OmenaQuerySassSymbolResolutionCapabilitiesV0 {
            same_file_lexical_resolution_ready: true,
            declaration_before_reference_ready: true,
            unresolved_reference_reporting_ready: true,
            cross_file_module_resolution_ready: false,
        },
    }
}

pub(super) fn omena_query_sass_symbol_fact_kind_is_declaration(
    kind: ParsedSassSymbolFactKind,
) -> bool {
    matches!(
        kind,
        ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::MixinDeclaration
            | ParsedSassSymbolFactKind::FunctionDeclaration
    )
}

pub(super) fn omena_query_sass_symbol_fact_kind_is_reference(
    kind: ParsedSassSymbolFactKind,
) -> bool {
    matches!(
        kind,
        ParsedSassSymbolFactKind::VariableReference
            | ParsedSassSymbolFactKind::MixinInclude
            | ParsedSassSymbolFactKind::FunctionCall
    )
}
