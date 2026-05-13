//! Semantic fact layer for parsed omena-css style modules.
//!
//! This crate lifts parser facts into selector, custom-property, Sass module,
//! design-token, and source-evidence summaries. It is the bridge between the
//! lossless parser substrate and query/LSP consumers that need stable semantic
//! contracts rather than raw CST traversal.

use engine_input_producers::EngineInputV2;
use omena_cascade::selector_context_witness_for_declaration;
use omena_interner::{
    intern_class_name, intern_css_ident, intern_custom_property_name, intern_keyframes_name,
    intern_mixin_name,
};
use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedSassModuleEdgeFactKind, ParsedSassSymbolFactKind,
    ParsedSelectorFactKind, ParsedStyleFacts, ParsedVariableFactKind, StyleDialect,
    collect_style_facts, parse,
};
use serde::Serialize;
use std::collections::BTreeSet;

mod css_modules;
mod design_tokens;
mod evidence;
mod lossless_cst;
mod observation;
mod selector_identity;
mod selector_references;
mod source_evidence;
mod types;

pub use css_modules::{
    CssModulesSemanticCapabilitiesV0, CssModulesSemanticSummaryV0, summarize_css_modules_semantics,
    summarize_css_modules_semantics_from_source,
};
pub use design_tokens::{
    DesignTokenCascadeRankingSignalV0, DesignTokenContextSignalV0,
    DesignTokenExternalDeclarationCandidateScopeV0, DesignTokenRankedReferenceV0,
    DesignTokenResolutionSignalV0, DesignTokenSemanticCapabilitiesV0, DesignTokenSemanticSummaryV0,
    DesignTokenWorkspaceDeclarationFactV0, collect_design_token_workspace_declarations,
    summarize_design_token_semantics,
    summarize_design_token_semantics_with_scoped_workspace_declarations,
    summarize_design_token_semantics_with_workspace_declarations,
};
pub use evidence::{
    SemanticPromotionEvidenceItemV0, SemanticPromotionEvidenceSummaryV0,
    summarize_semantic_promotion_evidence, summarize_semantic_promotion_evidence_with_source_input,
};
pub use lossless_cst::{
    LosslessCstConsumerReadinessV0, LosslessCstContractV0, LosslessCstSpanInvariantsV0,
    summarize_lossless_cst_contract,
};
pub use observation::{
    SelectorIdentityObservationV0, SemanticCouplingBoundaryObservationV0,
    SemanticGraphDownstreamReadinessV0, SourceEvidenceObservationV0, TheoryObservationContractV0,
    TheoryObservationHarnessInput, TheoryObservationHarnessSummaryV0,
    summarize_theory_observation_contract, summarize_theory_observation_harness,
};
pub use selector_identity::{
    SelectorCanonicalIdentityV0, SelectorIdentityEngineSummaryV0, SelectorIdentityRewriteSafetyV0,
    summarize_selector_identity_engine,
};
pub use selector_references::{
    SelectorEditableDirectReferenceSiteV0, SelectorReferenceEngineSummaryV0,
    SelectorReferenceSiteV0, SelectorReferenceSummaryV0, summarize_selector_reference_engine,
};
pub use source_evidence::{
    BindingOriginEvidenceV0, CertaintyReasonEvidenceV0, ReferenceSiteIdentityEvidenceV0,
    SourceInputPromotionEvidenceSummaryV0, StyleModuleEdgeEvidenceV0,
    ValueDomainExplanationEvidenceV0, summarize_source_input_evidence,
};
pub use types::{
    NestedSafetyCountsV0, ParserBoundarySyntaxFactsV0, ParserByteSpanV0,
    ParserIndexComposesFactsV0, ParserIndexCustomPropertyDeclFactV0,
    ParserIndexCustomPropertyFactsV0, ParserIndexCustomPropertyRefFactV0,
    ParserIndexKeyframesFactsV0, ParserIndexSassModuleUseFactV0,
    ParserIndexSassSameFileResolutionFactsV0, ParserIndexSelectorDefinitionFactV0,
    ParserIndexSelectorFactsV0, ParserIndexValueFactsV0, ParserIndexWrapperFactsV0,
    ParserLosslessCstFactsV0, ParserPositionV0, ParserRangeV0, ParserSassSyntaxFactsV0,
    StyleContainerIndexV0, StyleContextBlockV0, StyleContextIndexV0,
    StyleContextSelectorMembershipV0, StyleCustomPropertySemanticFactsV0, StyleLayerIndexV0,
    StyleLayerStatementV0, StyleSassSemanticFactsV0, StyleScopeIndexV0,
    StyleSelectorIdentityFactsV0, StyleSemanticFactsV0, Stylesheet,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSemanticBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub parser_facts: ParserBoundarySyntaxFactsV0,
    pub semantic_facts: StyleSemanticFactsV0,
    pub design_token_semantics: DesignTokenSemanticSummaryV0,
    pub selector_identity_engine: SelectorIdentityEngineSummaryV0,
    pub promotion_evidence: SemanticPromotionEvidenceSummaryV0,
    pub lossless_cst_contract: LosslessCstContractV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSemanticGraphSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub language: &'static str,
    pub parser_facts: ParserBoundarySyntaxFactsV0,
    pub semantic_facts: StyleSemanticFactsV0,
    pub css_modules_semantics: CssModulesSemanticSummaryV0,
    pub design_token_semantics: DesignTokenSemanticSummaryV0,
    pub selector_identity_engine: SelectorIdentityEngineSummaryV0,
    pub selector_reference_engine: SelectorReferenceEngineSummaryV0,
    pub source_input_evidence: SourceInputPromotionEvidenceSummaryV0,
    pub promotion_evidence: SemanticPromotionEvidenceSummaryV0,
    pub lossless_cst_contract: LosslessCstContractV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSemanticSoaTablesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selector_names: SemanticNameSoaTableV0,
    pub custom_property_names: SemanticNameSoaTableV0,
    pub sass_names: SemanticNameSoaTableV0,
    pub total_row_count: usize,
    pub interned_row_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticNameSoaTableV0 {
    pub table_name: &'static str,
    pub name_kind: &'static str,
    pub row_indices: Vec<usize>,
    pub names: Vec<String>,
    pub interned_row_count: usize,
    pub unique_name_count: usize,
}

pub fn summarize_style_semantic_boundary(sheet: &Stylesheet) -> StyleSemanticBoundarySummaryV0 {
    summarize_omena_parser_style_semantic_boundary_from_source(&sheet.path, &sheet.source)
}

pub fn summarize_style_semantic_graph(
    sheet: &Stylesheet,
    input: &EngineInputV2,
) -> StyleSemanticGraphSummaryV0 {
    summarize_style_semantic_graph_for_path(sheet, input, None)
}

pub fn summarize_style_semantic_graph_for_path(
    sheet: &Stylesheet,
    input: &EngineInputV2,
    style_path: Option<&str>,
) -> StyleSemanticGraphSummaryV0 {
    summarize_style_semantic_graph_for_path_with_workspace_declarations(
        sheet,
        input,
        style_path,
        &[],
    )
}

pub fn summarize_style_semantic_graph_for_path_with_workspace_declarations(
    sheet: &Stylesheet,
    input: &EngineInputV2,
    style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
) -> StyleSemanticGraphSummaryV0 {
    let boundary = summarize_style_semantic_boundary(sheet);
    let parser_facts = boundary.parser_facts;
    let semantic_facts = boundary.semantic_facts;
    let effective_style_path = style_path.or(Some(sheet.path.as_str()));
    let design_token_semantics = summarize_design_token_semantics_with_workspace_declarations(
        &parser_facts,
        &semantic_facts,
        effective_style_path,
        workspace_declarations,
    );
    let css_modules_semantics = summarize_css_modules_semantics(sheet);
    let selector_identity_engine =
        summarize_selector_identity_engine(&semantic_facts.selector_identity);
    let selector_reference_engine = summarize_selector_reference_engine(input, style_path);
    let source_input_evidence = summarize_source_input_evidence(input);
    let promotion_evidence = summarize_semantic_promotion_evidence_with_source_input(
        &parser_facts,
        &semantic_facts,
        input,
    );
    let lossless_cst_contract = summarize_lossless_cst_contract(&parser_facts.lossless_cst);

    StyleSemanticGraphSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.style-semantic-graph",
        language: boundary.language,
        parser_facts,
        semantic_facts,
        css_modules_semantics,
        design_token_semantics,
        selector_identity_engine,
        selector_reference_engine,
        source_input_evidence,
        promotion_evidence,
        lossless_cst_contract,
    }
}

pub fn summarize_style_semantic_graph_from_source(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<StyleSemanticGraphSummaryV0> {
    let css_modules_semantics =
        summarize_css_modules_semantics_from_source(style_path, style_source)?;
    let boundary =
        summarize_omena_parser_style_semantic_boundary_from_source(style_path, style_source);
    let parser_facts = boundary.parser_facts;
    let semantic_facts = boundary.semantic_facts;
    let selector_reference_engine = summarize_selector_reference_engine(input, Some(style_path));
    let source_input_evidence = summarize_source_input_evidence(input);
    let promotion_evidence = summarize_semantic_promotion_evidence_with_source_input(
        &parser_facts,
        &semantic_facts,
        input,
    );

    Some(StyleSemanticGraphSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.style-semantic-graph",
        language: boundary.language,
        parser_facts,
        semantic_facts,
        css_modules_semantics,
        design_token_semantics: boundary.design_token_semantics,
        selector_identity_engine: boundary.selector_identity_engine,
        selector_reference_engine,
        source_input_evidence,
        promotion_evidence,
        lossless_cst_contract: boundary.lossless_cst_contract,
    })
}

pub fn summarize_style_semantic_facts(sheet: &Stylesheet) -> StyleSemanticFactsV0 {
    summarize_style_semantic_boundary(sheet).semantic_facts
}

pub fn summarize_style_semantic_soa_tables(
    semantic_facts: &StyleSemanticFactsV0,
    db: &dyn salsa::Database,
) -> StyleSemanticSoaTablesV0 {
    let selector_names = semantic_name_soa_table(
        "selectors",
        "className",
        semantic_facts.selector_identity.canonical_names.as_slice(),
        |name| intern_class_name(db, name).is_ok(),
    );
    let custom_property_names = semantic_name_soa_table(
        "customProperties",
        "customPropertyName",
        semantic_facts.custom_properties.decl_names.as_slice(),
        |name| intern_custom_property_name(db, name).is_ok(),
    );
    let mut sass_name_sources = Vec::new();
    sass_name_sources.extend(
        semantic_facts
            .sass
            .same_file_resolution
            .resolved_variable_ref_names
            .iter()
            .cloned(),
    );
    sass_name_sources.extend(
        semantic_facts
            .sass
            .same_file_resolution
            .unresolved_variable_ref_names
            .iter()
            .cloned(),
    );
    sass_name_sources.extend(
        semantic_facts
            .sass
            .same_file_resolution
            .resolved_mixin_include_names
            .iter()
            .cloned(),
    );
    sass_name_sources.extend(
        semantic_facts
            .sass
            .same_file_resolution
            .unresolved_mixin_include_names
            .iter()
            .cloned(),
    );
    sass_name_sources.extend(
        semantic_facts
            .sass
            .same_file_resolution
            .resolved_function_call_names
            .iter()
            .cloned(),
    );
    let sass_names =
        semantic_name_soa_table("sass", "cssIdentOrMixinName", &sass_name_sources, |name| {
            intern_css_ident(db, name).is_ok()
                || intern_mixin_name(db, name).is_ok()
                || intern_keyframes_name(db, name).is_ok()
        });
    let total_row_count = selector_names.row_indices.len()
        + custom_property_names.row_indices.len()
        + sass_names.row_indices.len();
    let interned_row_count = selector_names.interned_row_count
        + custom_property_names.interned_row_count
        + sass_names.interned_row_count;

    StyleSemanticSoaTablesV0 {
        schema_version: "0",
        product: "omena-semantic.soa-tables",
        selector_names,
        custom_property_names,
        sass_names,
        total_row_count,
        interned_row_count,
        ready_surfaces: vec!["semanticSoaTables", "semanticSoaNameTables"],
    }
}

fn semantic_name_soa_table(
    table_name: &'static str,
    name_kind: &'static str,
    names: &[String],
    mut intern: impl FnMut(&str) -> bool,
) -> SemanticNameSoaTableV0 {
    let mut unique_names = BTreeSet::new();
    let mut interned_row_count = 0usize;
    for name in names {
        unique_names.insert(name.clone());
        if intern(name) {
            interned_row_count += 1;
        }
    }

    SemanticNameSoaTableV0 {
        table_name,
        name_kind,
        row_indices: (0..names.len()).collect(),
        names: names.to_vec(),
        interned_row_count,
        unique_name_count: unique_names.len(),
    }
}

pub fn summarize_parser_contract_facts(sheet: &Stylesheet) -> ParserBoundarySyntaxFactsV0 {
    summarize_style_semantic_boundary(sheet).parser_facts
}

pub fn parse_style_module(path: &str, source: &str) -> Option<Stylesheet> {
    Some(Stylesheet {
        path: path.to_string(),
        language: dialect_for_style_path(path)?,
        source: source.to_string(),
    })
}

pub fn summarize_omena_parser_style_semantic_boundary_from_source(
    style_path: &str,
    style_source: &str,
) -> StyleSemanticBoundarySummaryV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let parsed = parse(style_source, dialect);
    let facts = collect_style_facts(style_source, dialect);
    let parser_facts = summarize_omena_parser_contract_facts(
        style_source,
        parsed.token_count(),
        parsed.syntax().children().count(),
        parsed.errors().len(),
        &facts,
    );
    let semantic_facts = summarize_omena_parser_semantic_facts(style_source, &facts, &parser_facts);
    let design_token_semantics = summarize_design_token_semantics(&parser_facts, &semantic_facts);
    let selector_identity_engine =
        summarize_selector_identity_engine(&semantic_facts.selector_identity);
    let promotion_evidence = summarize_semantic_promotion_evidence(&parser_facts, &semantic_facts);
    let lossless_cst_contract = summarize_lossless_cst_contract(&parser_facts.lossless_cst);

    StyleSemanticBoundarySummaryV0 {
        schema_version: "0",
        language: omena_parser_dialect_label(dialect),
        parser_facts,
        semantic_facts,
        design_token_semantics,
        selector_identity_engine,
        promotion_evidence,
        lossless_cst_contract,
    }
}

fn summarize_omena_parser_contract_facts(
    source: &str,
    token_count: usize,
    root_node_count: usize,
    diagnostic_count: usize,
    facts: &ParsedStyleFacts,
) -> ParserBoundarySyntaxFactsV0 {
    ParserBoundarySyntaxFactsV0 {
        lossless_cst: ParserLosslessCstFactsV0 {
            source_byte_len: source.len(),
            token_count,
            root_node_count,
            diagnostic_count,
            all_token_spans_within_source: true,
            all_node_spans_within_source: true,
        },
        selectors: summarize_omena_parser_selector_facts(source, facts),
        values: summarize_omena_parser_value_facts(facts),
        custom_properties: summarize_omena_parser_custom_property_facts(source, facts),
        sass: summarize_omena_parser_sass_syntax_facts(facts),
        keyframes: summarize_omena_parser_keyframe_facts(facts),
        composes: summarize_omena_parser_composes_facts(facts),
        wrappers: ParserIndexWrapperFactsV0::default(),
    }
}

fn summarize_omena_parser_semantic_facts(
    source: &str,
    facts: &ParsedStyleFacts,
    parser_facts: &ParserBoundarySyntaxFactsV0,
) -> StyleSemanticFactsV0 {
    let custom_properties =
        summarize_omena_parser_custom_property_semantic_facts(&parser_facts.custom_properties);
    let sass_same_file_resolution =
        summarize_omena_parser_sass_same_file_resolution(&parser_facts.sass);
    let sass_selector_resolution =
        summarize_omena_parser_sass_selector_resolution(source, facts, &sass_same_file_resolution);
    StyleSemanticFactsV0 {
        selector_identity: StyleSelectorIdentityFactsV0 {
            canonical_names: parser_facts.selectors.names.clone(),
            bem_suffix_safe_names: parser_facts.selectors.bem_suffix_safe_names.clone(),
            bem_suffix_parent_names: parser_facts.selectors.bem_suffix_parent_names.clone(),
            nested_unsafe_names: parser_facts.selectors.nested_unsafe_names.clone(),
            nested_safety_counts: parser_facts.selectors.nested_safety_counts.clone(),
        },
        custom_properties,
        sass: StyleSassSemanticFactsV0 {
            selector_symbol_facts: Vec::new(),
            selectors_with_resolved_variable_refs_names: sass_selector_resolution
                .resolved_variable_ref_selectors,
            selectors_with_unresolved_variable_refs_names: sass_selector_resolution
                .unresolved_variable_ref_selectors,
            selectors_with_resolved_mixin_includes_names: sass_selector_resolution
                .resolved_mixin_include_selectors,
            selectors_with_unresolved_mixin_includes_names: sass_selector_resolution
                .unresolved_mixin_include_selectors,
            selectors_with_function_calls_names: parser_facts.sass.function_call_names.clone(),
            same_file_resolution: sass_same_file_resolution,
        },
        context_index: summarize_style_context_index(source),
    }
}

fn summarize_style_context_index(source: &str) -> StyleContextIndexV0 {
    let layer_statements = collect_layer_statement_facts(source);
    let (context_blocks, memberships) = collect_style_context_blocks_and_memberships(source);
    let block_layers = context_blocks
        .iter()
        .filter(|block| block.kind == "layer")
        .cloned()
        .collect::<Vec<_>>();
    let containers = context_blocks
        .iter()
        .filter(|block| block.kind == "container")
        .cloned()
        .collect::<Vec<_>>();
    let scopes = context_blocks
        .iter()
        .filter(|block| block.kind == "scope")
        .cloned()
        .collect::<Vec<_>>();
    let layer_memberships = memberships
        .iter()
        .filter(|membership| membership.context_kind == "layer")
        .cloned()
        .collect::<Vec<_>>();
    let container_memberships = memberships
        .iter()
        .filter(|membership| membership.context_kind == "container")
        .cloned()
        .collect::<Vec<_>>();
    let scope_memberships = memberships
        .iter()
        .filter(|membership| membership.context_kind == "scope")
        .cloned()
        .collect::<Vec<_>>();
    let named_layer_count = layer_statements
        .iter()
        .map(|statement| statement.name.clone())
        .chain(block_layers.iter().filter_map(|block| block.name.clone()))
        .collect::<BTreeSet<_>>()
        .len();

    StyleContextIndexV0 {
        schema_version: "0",
        product: "omena-semantic.style-context-index",
        layer_index: StyleLayerIndexV0 {
            statement_layers: layer_statements,
            anonymous_layer_block_count: block_layers
                .iter()
                .filter(|block| block.name.is_none())
                .count(),
            block_layers,
            selector_memberships: layer_memberships,
            named_layer_count,
        },
        container_index: StyleContainerIndexV0 {
            named_container_count: containers
                .iter()
                .filter(|block| block.name.is_some())
                .count(),
            anonymous_container_count: containers
                .iter()
                .filter(|block| block.name.is_none())
                .count(),
            containers,
            selector_memberships: container_memberships,
        },
        scope_index: StyleScopeIndexV0 {
            scoped_selector_count: scope_memberships
                .iter()
                .map(|membership| membership.selector_name.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            scopes,
            selector_memberships: scope_memberships,
        },
        selector_context_count: memberships.len(),
        ready_surfaces: vec![
            "layerIndex",
            "containerIndex",
            "scopeIndex",
            "selectorContextMembership",
        ],
    }
}

fn collect_layer_statement_facts(source: &str) -> Vec<StyleLayerStatementV0> {
    let mut statements = Vec::new();
    let mut search_start = 0usize;
    while let Some(relative_start) = source
        .get(search_start..)
        .and_then(|tail| tail.find("@layer"))
    {
        let at_index = search_start + relative_start;
        let prelude_start = at_index + "@layer".len();
        let tail = source.get(prelude_start..).unwrap_or_default();
        let semicolon = tail.find(';');
        let open_brace = tail.find('{');
        let Some(semicolon) = semicolon else {
            break;
        };
        if open_brace.is_some_and(|open| open < semicolon) {
            search_start = prelude_start + open_brace.unwrap_or(0) + 1;
            continue;
        }

        let prelude_end = prelude_start + semicolon;
        let prelude = source.get(prelude_start..prelude_end).unwrap_or_default();
        let byte_span = ParserByteSpanV0 {
            start: at_index,
            end: prelude_end + 1,
        };
        for name in split_layer_names(prelude) {
            statements.push(StyleLayerStatementV0 {
                name,
                source_order: statements.len(),
                byte_span,
                range: parser_range_for_byte_span(source, byte_span),
            });
        }
        search_start = prelude_end + 1;
    }
    statements
}

fn collect_style_context_blocks_and_memberships(
    source: &str,
) -> (
    Vec<StyleContextBlockV0>,
    Vec<StyleContextSelectorMembershipV0>,
) {
    let bytes = source.as_bytes();
    let mut blocks = Vec::new();
    let mut memberships = Vec::new();
    let mut active_contexts = Vec::<StyleContextBlockV0>::new();
    let mut block_stack = Vec::<Option<String>>::new();
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                let (header, header_start) =
                    block_header_and_start_before_open_brace(source, index);
                if let Some(context) = style_context_block_for_header(
                    source,
                    &header,
                    header_start,
                    index,
                    blocks.len(),
                ) {
                    block_stack.push(Some(context.id.clone()));
                    active_contexts.push(context.clone());
                    blocks.push(context);
                } else {
                    for selector_name in selector_class_names(&header) {
                        for context in &active_contexts {
                            memberships.push(StyleContextSelectorMembershipV0 {
                                selector_name: selector_name.clone(),
                                context_id: context.id.clone(),
                                context_kind: context.kind,
                                source_order: memberships.len(),
                            });
                        }
                    }
                    block_stack.push(None);
                }
            }
            b'}' => {
                if let Some(Some(context_id)) = block_stack.pop() {
                    if active_contexts
                        .last()
                        .is_some_and(|context| context.id == context_id)
                    {
                        active_contexts.pop();
                    } else {
                        active_contexts.retain(|context| context.id != context_id);
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }

    (blocks, memberships)
}

fn style_context_block_for_header(
    source: &str,
    header: &str,
    header_start: usize,
    open_brace_index: usize,
    source_order: usize,
) -> Option<StyleContextBlockV0> {
    let header = header.trim();
    let (kind, raw_prelude) = if let Some(prelude) = header.strip_prefix("@layer") {
        ("layer", prelude)
    } else if let Some(prelude) = header.strip_prefix("@container") {
        ("container", prelude)
    } else if let Some(prelude) = header.strip_prefix("@scope") {
        ("scope", prelude)
    } else {
        return None;
    };
    let prelude = raw_prelude.trim().to_string();
    let name = match kind {
        "layer" => split_layer_names(&prelude).into_iter().next(),
        "container" => container_name_from_prelude(&prelude),
        "scope" => None,
        _ => None,
    };
    let byte_span = ParserByteSpanV0 {
        start: header_start,
        end: open_brace_index + 1,
    };

    Some(StyleContextBlockV0 {
        id: format!("{kind}:{source_order}"),
        kind,
        name,
        prelude,
        source_order,
        byte_span,
        range: parser_range_for_byte_span(source, byte_span),
    })
}

fn split_layer_names(prelude: &str) -> Vec<String> {
    prelude
        .split(',')
        .filter_map(|name| {
            let name = name.trim();
            if name.is_empty() || name == "{" {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect()
}

fn container_name_from_prelude(prelude: &str) -> Option<String> {
    let trimmed = prelude.trim();
    if trimmed.is_empty() || trimmed.starts_with('(') || trimmed.starts_with("style(") {
        return None;
    }
    let name = trimmed.split_whitespace().next().unwrap_or_default().trim();
    if css_identifier_text_is_plain(name) {
        Some(name.to_string())
    } else {
        None
    }
}

fn css_identifier_text_is_plain(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || matches!(first, '_' | '-'))
        && chars.all(|char| char.is_ascii_alphanumeric() || matches!(char, '_' | '-'))
}

fn block_header_and_start_before_open_brace(
    source: &str,
    open_brace_index: usize,
) -> (String, usize) {
    let bytes = source.as_bytes();
    let mut start = 0usize;
    let mut index = open_brace_index;
    while let Some(previous) = index.checked_sub(1) {
        index = previous;
        if matches!(bytes[index], b'{' | b'}' | b';') {
            start = index + 1;
            break;
        }
        if index == 0 {
            break;
        }
    }
    let raw = source.get(start..open_brace_index).unwrap_or_default();
    let trimmed_start_delta = raw.len().saturating_sub(raw.trim_start().len());
    (raw.trim().to_string(), start + trimmed_start_delta)
}

fn selector_class_names(selector: &str) -> Vec<String> {
    let bytes = selector.as_bytes();
    let mut names = BTreeSet::new();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            let start = index + 1;
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'-'))
            {
                end += 1;
            }
            if end > start
                && let Some(name) = selector.get(start..end)
            {
                names.insert(name.to_string());
            }
            index = end;
            continue;
        }
        index += 1;
    }
    names.into_iter().collect()
}

fn summarize_omena_parser_selector_facts(
    source: &str,
    facts: &ParsedStyleFacts,
) -> ParserIndexSelectorFactsV0 {
    let mut names = Vec::new();
    let mut definition_facts = Vec::new();
    let mut bem_suffix_parent_names = BTreeSet::new();
    let mut bem_suffix_safe_names = BTreeSet::new();
    let mut nested_unsafe_names = BTreeSet::new();
    let mut source_order = 0usize;

    for selector in &facts.selectors {
        if selector.kind != ParsedSelectorFactKind::Class {
            continue;
        }
        let byte_span = parser_byte_span_for_offsets(
            u32::from(selector.range.start()) as usize,
            u32::from(selector.range.end()) as usize,
        );
        let parent_name = bem_suffix_parent_name(selector.name.as_str());
        let nested_safety_kind = if let Some(parent) = parent_name.clone() {
            bem_suffix_parent_names.insert(parent);
            bem_suffix_safe_names.insert(selector.name.clone());
            "bemSuffixSafe"
        } else if selector_has_parent_ampersand_class_prefix(source, byte_span.start) {
            nested_unsafe_names.insert(selector.name.clone());
            "nestedUnsafe"
        } else {
            "flat"
        };
        names.push(selector.name.clone());
        definition_facts.push(ParserIndexSelectorDefinitionFactV0 {
            name: selector.name.clone(),
            source_order,
            byte_span,
            range: parser_range_for_byte_span(source, byte_span),
            nested_safety_kind,
            bem_suffix_parent_name: parent_name,
            under_media: false,
            under_supports: false,
            under_layer: false,
        });
        source_order += 1;
    }

    names.sort();
    names.dedup();
    definition_facts.sort();
    let bem_suffix_safe_names = bem_suffix_safe_names.into_iter().collect::<Vec<_>>();
    let nested_unsafe_names = nested_unsafe_names.into_iter().collect::<Vec<_>>();
    ParserIndexSelectorFactsV0 {
        names,
        definition_facts,
        bem_suffix_parent_names: bem_suffix_parent_names.into_iter().collect(),
        bem_suffix_safe_names: bem_suffix_safe_names.clone(),
        nested_unsafe_names: nested_unsafe_names.clone(),
        selectors_with_value_refs_names: Vec::new(),
        selectors_with_animation_ref_names: Vec::new(),
        selectors_with_animation_name_ref_names: Vec::new(),
        bem_suffix_count: bem_suffix_safe_names.len(),
        nested_safety_counts: NestedSafetyCountsV0 {
            flat: source_order
                .saturating_sub(bem_suffix_safe_names.len())
                .saturating_sub(nested_unsafe_names.len()),
            bem_suffix_safe: bem_suffix_safe_names.len(),
            nested_unsafe: nested_unsafe_names.len(),
        },
    }
}

fn summarize_omena_parser_value_facts(facts: &ParsedStyleFacts) -> ParserIndexValueFactsV0 {
    let mut decl_names = BTreeSet::new();
    let mut ref_names = BTreeSet::new();
    let mut import_sources = BTreeSet::new();
    for value in &facts.css_module_values {
        match value.kind {
            ParsedCssModuleValueFactKind::Definition => {
                decl_names.insert(value.name.clone());
            }
            ParsedCssModuleValueFactKind::Reference => {
                ref_names.insert(value.name.clone());
            }
            ParsedCssModuleValueFactKind::ImportSource => {
                import_sources.insert(value.name.clone());
            }
        }
    }
    ParserIndexValueFactsV0 {
        decl_names: decl_names.into_iter().collect(),
        import_sources: import_sources.into_iter().collect(),
        import_alias_count: facts.css_module_value_import_edge_count,
        ref_names: ref_names.clone().into_iter().collect(),
        local_ref_names: ref_names.into_iter().collect(),
        ..ParserIndexValueFactsV0::default()
    }
}

fn summarize_omena_parser_custom_property_facts(
    source: &str,
    facts: &ParsedStyleFacts,
) -> ParserIndexCustomPropertyFactsV0 {
    let mut decl_names = BTreeSet::new();
    let mut ref_names = BTreeSet::new();
    let mut decl_facts = Vec::new();
    let mut ref_facts = Vec::new();
    for variable in &facts.variables {
        match variable.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration => {
                let byte_span = parser_byte_span_for_offsets(
                    u32::from(variable.range.start()) as usize,
                    u32::from(variable.range.end()) as usize,
                );
                decl_names.insert(variable.name.clone());
                let (selector_contexts, under_media, under_supports, under_layer) =
                    style_context_for_byte_offset(source, byte_span.start);
                decl_facts.push(ParserIndexCustomPropertyDeclFactV0 {
                    name: variable.name.clone(),
                    value: declaration_value_text(source, byte_span.start),
                    source_order: decl_facts.len(),
                    byte_span,
                    range: parser_range_for_byte_span(source, byte_span),
                    selector_contexts,
                    under_media,
                    under_supports,
                    under_layer,
                });
            }
            ParsedVariableFactKind::CustomPropertyReference => {
                let byte_offset = u32::from(variable.range.start()) as usize;
                let (selector_contexts, under_media, under_supports, under_layer) =
                    style_context_for_byte_offset(source, byte_offset);
                ref_names.insert(variable.name.clone());
                ref_facts.push(ParserIndexCustomPropertyRefFactV0 {
                    name: variable.name.clone(),
                    source_order: ref_facts.len(),
                    selector_contexts,
                    under_media,
                    under_supports,
                    under_layer,
                });
            }
            _ => {}
        }
    }
    let selectors_with_refs_names = ref_facts
        .iter()
        .flat_map(|reference| reference.selector_contexts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let selectors_with_refs_under_media_names = ref_facts
        .iter()
        .filter(|reference| reference.under_media)
        .flat_map(|reference| reference.selector_contexts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let selectors_with_refs_under_supports_names = ref_facts
        .iter()
        .filter(|reference| reference.under_supports)
        .flat_map(|reference| reference.selector_contexts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let selectors_with_refs_under_layer_names = ref_facts
        .iter()
        .filter(|reference| reference.under_layer)
        .flat_map(|reference| reference.selector_contexts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let decl_context_selectors = decl_facts
        .iter()
        .flat_map(|declaration| declaration.selector_contexts.iter().cloned())
        .collect::<BTreeSet<_>>();
    let decl_names_under_media = decl_facts
        .iter()
        .filter(|declaration| declaration.under_media)
        .map(|declaration| declaration.name.clone())
        .collect::<BTreeSet<_>>();
    let decl_names_under_supports = decl_facts
        .iter()
        .filter(|declaration| declaration.under_supports)
        .map(|declaration| declaration.name.clone())
        .collect::<BTreeSet<_>>();
    let decl_names_under_layer = decl_facts
        .iter()
        .filter(|declaration| declaration.under_layer)
        .map(|declaration| declaration.name.clone())
        .collect::<BTreeSet<_>>();

    ParserIndexCustomPropertyFactsV0 {
        decl_names: decl_names.into_iter().collect(),
        decl_facts,
        decl_context_selectors: decl_context_selectors.into_iter().collect(),
        decl_names_under_media: decl_names_under_media.into_iter().collect(),
        decl_names_under_supports: decl_names_under_supports.into_iter().collect(),
        decl_names_under_layer: decl_names_under_layer.into_iter().collect(),
        ref_names: ref_names.into_iter().collect(),
        ref_facts,
        selectors_with_refs_names: selectors_with_refs_names.into_iter().collect(),
        selectors_with_refs_under_media_names: selectors_with_refs_under_media_names
            .into_iter()
            .collect(),
        selectors_with_refs_under_supports_names: selectors_with_refs_under_supports_names
            .into_iter()
            .collect(),
        selectors_with_refs_under_layer_names: selectors_with_refs_under_layer_names
            .into_iter()
            .collect(),
    }
}

fn summarize_omena_parser_sass_syntax_facts(facts: &ParsedStyleFacts) -> ParserSassSyntaxFactsV0 {
    let mut variable_decl_names = BTreeSet::new();
    let mut variable_ref_names = BTreeSet::new();
    let mut mixin_decl_names = BTreeSet::new();
    let mut mixin_include_names = BTreeSet::new();
    let mut function_decl_names = BTreeSet::new();
    let mut function_call_names = BTreeSet::new();
    for symbol in &facts.sass_symbols {
        match symbol.kind {
            ParsedSassSymbolFactKind::VariableDeclaration => {
                variable_decl_names.insert(symbol.name.clone());
            }
            ParsedSassSymbolFactKind::VariableReference => {
                variable_ref_names.insert(symbol.name.clone());
            }
            ParsedSassSymbolFactKind::MixinDeclaration => {
                mixin_decl_names.insert(symbol.name.clone());
            }
            ParsedSassSymbolFactKind::MixinInclude => {
                mixin_include_names.insert(symbol.name.clone());
            }
            ParsedSassSymbolFactKind::FunctionDeclaration => {
                function_decl_names.insert(symbol.name.clone());
            }
            ParsedSassSymbolFactKind::FunctionCall => {
                function_call_names.insert(symbol.name.clone());
            }
        }
    }
    let mut module_use_sources = BTreeSet::new();
    let mut module_use_edges = Vec::new();
    let mut module_forward_sources = BTreeSet::new();
    let mut module_import_sources = BTreeSet::new();
    for edge in &facts.sass_module_edges {
        match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => {
                module_use_sources.insert(edge.source.clone());
                module_use_edges.push(ParserIndexSassModuleUseFactV0 {
                    source: edge.source.clone(),
                    namespace_kind: edge.namespace_kind.unwrap_or("default"),
                    namespace: edge.namespace.clone(),
                });
            }
            ParsedSassModuleEdgeFactKind::Forward => {
                module_forward_sources.insert(edge.source.clone());
            }
            ParsedSassModuleEdgeFactKind::Import => {
                module_import_sources.insert(edge.source.clone());
                module_use_edges.push(ParserIndexSassModuleUseFactV0 {
                    source: edge.source.clone(),
                    namespace_kind: "wildcard",
                    namespace: None,
                });
            }
        }
    }
    ParserSassSyntaxFactsV0 {
        variable_decl_names: variable_decl_names.into_iter().collect(),
        variable_parameter_names: Vec::new(),
        variable_ref_names: variable_ref_names.into_iter().collect(),
        mixin_decl_names: mixin_decl_names.into_iter().collect(),
        mixin_include_names: mixin_include_names.into_iter().collect(),
        function_decl_names: function_decl_names.into_iter().collect(),
        function_call_names: function_call_names.into_iter().collect(),
        module_use_sources: module_use_sources.into_iter().collect(),
        module_use_edges,
        module_forward_sources: module_forward_sources.into_iter().collect(),
        module_import_sources: module_import_sources.into_iter().collect(),
    }
}

fn summarize_omena_parser_keyframe_facts(facts: &ParsedStyleFacts) -> ParserIndexKeyframesFactsV0 {
    let mut names = BTreeSet::new();
    let mut animation_ref_names = BTreeSet::new();
    for animation in &facts.animations {
        match animation.kind {
            ParsedAnimationFactKind::KeyframesDeclaration => {
                names.insert(animation.name.clone());
            }
            ParsedAnimationFactKind::AnimationNameReference => {
                animation_ref_names.insert(animation.name.clone());
            }
        }
    }
    ParserIndexKeyframesFactsV0 {
        names: names.into_iter().collect(),
        animation_ref_names: animation_ref_names.clone().into_iter().collect(),
        animation_name_ref_names: animation_ref_names.into_iter().collect(),
        ..ParserIndexKeyframesFactsV0::default()
    }
}

fn summarize_omena_parser_composes_facts(facts: &ParsedStyleFacts) -> ParserIndexComposesFactsV0 {
    let mut local_selector_names = BTreeSet::new();
    let mut imported_selector_names = BTreeSet::new();
    let mut global_selector_names = BTreeSet::new();
    let mut import_sources = BTreeSet::new();
    for edge in &facts.css_module_composes_edges {
        match edge.kind {
            ParsedCssModuleComposesEdgeKind::Local => {
                local_selector_names.extend(edge.target_names.iter().cloned());
            }
            ParsedCssModuleComposesEdgeKind::External => {
                imported_selector_names.extend(edge.target_names.iter().cloned());
                if let Some(source) = &edge.import_source {
                    import_sources.insert(source.clone());
                }
            }
            ParsedCssModuleComposesEdgeKind::Global => {
                global_selector_names.extend(edge.target_names.iter().cloned());
            }
        }
    }
    for composes in &facts.css_module_composes {
        if composes.kind == ParsedCssModuleComposesFactKind::ImportSource {
            import_sources.insert(composes.name.clone());
        }
    }
    let local_selector_names = local_selector_names.into_iter().collect::<Vec<_>>();
    let imported_selector_names = imported_selector_names.into_iter().collect::<Vec<_>>();
    let global_selector_names = global_selector_names.into_iter().collect::<Vec<_>>();
    ParserIndexComposesFactsV0 {
        class_name_count: local_selector_names.len()
            + imported_selector_names.len()
            + global_selector_names.len(),
        local_class_name_count: local_selector_names.len(),
        imported_class_name_count: imported_selector_names.len(),
        global_class_name_count: global_selector_names.len(),
        local_selector_names,
        imported_selector_names,
        global_selector_names,
        import_sources: import_sources.into_iter().collect(),
        ..ParserIndexComposesFactsV0::default()
    }
}

fn summarize_omena_parser_custom_property_semantic_facts(
    facts: &ParserIndexCustomPropertyFactsV0,
) -> StyleCustomPropertySemanticFactsV0 {
    let mut resolved_ref_names = BTreeSet::new();
    let mut unresolved_ref_names = BTreeSet::new();
    for reference in &facts.ref_facts {
        if facts
            .decl_facts
            .iter()
            .any(|declaration| custom_property_context_matches(declaration, reference))
        {
            resolved_ref_names.insert(reference.name.clone());
        } else {
            unresolved_ref_names.insert(reference.name.clone());
        }
    }
    StyleCustomPropertySemanticFactsV0 {
        decl_names: facts.decl_names.clone(),
        ref_names: facts.ref_names.clone(),
        resolved_ref_names: resolved_ref_names.into_iter().collect(),
        unresolved_ref_names: unresolved_ref_names.into_iter().collect(),
        selectors_with_refs_names: facts.selectors_with_refs_names.clone(),
    }
}

struct SassSelectorResolution {
    resolved_variable_ref_selectors: Vec<String>,
    unresolved_variable_ref_selectors: Vec<String>,
    resolved_mixin_include_selectors: Vec<String>,
    unresolved_mixin_include_selectors: Vec<String>,
}

fn summarize_omena_parser_sass_selector_resolution(
    source: &str,
    facts: &ParsedStyleFacts,
    resolution: &ParserIndexSassSameFileResolutionFactsV0,
) -> SassSelectorResolution {
    let resolved_variables = resolution
        .resolved_variable_ref_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let resolved_mixins = resolution
        .resolved_mixin_include_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut resolved_variable_ref_selectors = BTreeSet::new();
    let mut unresolved_variable_ref_selectors = BTreeSet::new();
    let mut resolved_mixin_include_selectors = BTreeSet::new();
    let mut unresolved_mixin_include_selectors = BTreeSet::new();

    for symbol in &facts.sass_symbols {
        match symbol.kind {
            ParsedSassSymbolFactKind::VariableReference => {
                let selector = semantic_selector_name_for_byte_offset(
                    source,
                    u32::from(symbol.range.start()) as usize,
                );
                let Some(selector) = selector else {
                    continue;
                };
                if resolved_variables.contains(&symbol.name) {
                    resolved_variable_ref_selectors.insert(selector);
                } else {
                    unresolved_variable_ref_selectors.insert(selector);
                }
            }
            ParsedSassSymbolFactKind::MixinInclude => {
                let selector = semantic_selector_name_for_byte_offset(
                    source,
                    u32::from(symbol.range.start()) as usize,
                );
                let Some(selector) = selector else {
                    continue;
                };
                if resolved_mixins.contains(&symbol.name) {
                    resolved_mixin_include_selectors.insert(selector);
                } else {
                    unresolved_mixin_include_selectors.insert(selector);
                }
            }
            _ => {}
        }
    }

    SassSelectorResolution {
        resolved_variable_ref_selectors: resolved_variable_ref_selectors.into_iter().collect(),
        unresolved_variable_ref_selectors: unresolved_variable_ref_selectors.into_iter().collect(),
        resolved_mixin_include_selectors: resolved_mixin_include_selectors.into_iter().collect(),
        unresolved_mixin_include_selectors: unresolved_mixin_include_selectors
            .into_iter()
            .collect(),
    }
}

fn custom_property_context_matches(
    declaration: &ParserIndexCustomPropertyDeclFactV0,
    reference: &ParserIndexCustomPropertyRefFactV0,
) -> bool {
    if declaration.name != reference.name {
        return false;
    }
    if declaration.under_media && !reference.under_media {
        return false;
    }
    if declaration.under_supports && !reference.under_supports {
        return false;
    }
    if declaration.under_layer && !reference.under_layer {
        return false;
    }
    if declaration.selector_contexts.is_empty() {
        return true;
    }
    declaration.selector_contexts.iter().any(|selector| {
        selector_context_witness_for_declaration(selector, &reference.selector_contexts).matched
    })
}

fn summarize_omena_parser_sass_same_file_resolution(
    facts: &ParserSassSyntaxFactsV0,
) -> ParserIndexSassSameFileResolutionFactsV0 {
    let variable_targets = facts
        .variable_decl_names
        .iter()
        .chain(facts.variable_parameter_names.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mixin_targets = facts
        .mixin_decl_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let function_targets = facts
        .function_decl_names
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    ParserIndexSassSameFileResolutionFactsV0 {
        resolved_variable_ref_names: names_matching(&facts.variable_ref_names, &variable_targets),
        unresolved_variable_ref_names: names_not_matching(
            &facts.variable_ref_names,
            &variable_targets,
        ),
        resolved_mixin_include_names: names_matching(&facts.mixin_include_names, &mixin_targets),
        unresolved_mixin_include_names: names_not_matching(
            &facts.mixin_include_names,
            &mixin_targets,
        ),
        resolved_function_call_names: names_matching(&facts.function_call_names, &function_targets),
    }
}

fn names_matching(names: &[String], targets: &BTreeSet<String>) -> Vec<String> {
    names
        .iter()
        .filter(|name| targets.contains(*name))
        .cloned()
        .collect()
}

fn names_not_matching(names: &[String], targets: &BTreeSet<String>) -> Vec<String> {
    names
        .iter()
        .filter(|name| !targets.contains(*name))
        .cloned()
        .collect()
}

fn bem_suffix_parent_name(name: &str) -> Option<String> {
    let marker = name.find("__").or_else(|| name.find("--"))?;
    (marker > 0).then(|| name[..marker].to_string())
}

fn selector_has_parent_ampersand_class_prefix(source: &str, selector_start: usize) -> bool {
    let bytes = source.as_bytes();
    if selector_start >= bytes.len() {
        return false;
    }
    let dot_index = if bytes[selector_start] == b'.' {
        selector_start
    } else {
        match previous_non_whitespace_byte_index(bytes, selector_start) {
            Some(index) if bytes[index] == b'.' => index,
            _ => return false,
        }
    };
    matches!(
        previous_non_whitespace_byte_index(bytes, dot_index),
        Some(index) if bytes[index] == b'&'
    )
}

fn style_context_for_byte_offset(
    source: &str,
    byte_offset: usize,
) -> (Vec<String>, bool, bool, bool) {
    let contexts = block_contexts_for_byte_offset(source, byte_offset);
    let selector_contexts = contexts
        .iter()
        .filter_map(|context| match context {
            StyleBlockContext::Selector(selector) => Some(selector.clone()),
            StyleBlockContext::Media
            | StyleBlockContext::Supports
            | StyleBlockContext::Layer
            | StyleBlockContext::OtherAtRule => None,
        })
        .collect::<Vec<_>>();
    let under_media = contexts
        .iter()
        .any(|context| matches!(context, StyleBlockContext::Media));
    let under_supports = contexts
        .iter()
        .any(|context| matches!(context, StyleBlockContext::Supports));
    let under_layer = contexts
        .iter()
        .any(|context| matches!(context, StyleBlockContext::Layer));

    (selector_contexts, under_media, under_supports, under_layer)
}

fn declaration_value_text(source: &str, offset: usize) -> String {
    let span = declaration_statement_byte_span_for_offset(source, offset);
    let Some(statement) = source.get(span.start..span.end) else {
        return String::new();
    };
    let Some(colon) = statement.find(':') else {
        return String::new();
    };
    statement[colon + 1..]
        .trim()
        .trim_end_matches(';')
        .trim()
        .to_string()
}

fn declaration_statement_byte_span_for_offset(source: &str, offset: usize) -> ParserByteSpanV0 {
    let start = source
        .get(..offset)
        .and_then(|before| before.rfind(['{', ';']).map(|index| index + 1))
        .unwrap_or(offset);
    let end = source
        .get(offset..)
        .and_then(|rest| {
            let semicolon = rest.find(';');
            let close = rest.find('}');
            match (semicolon, close) {
                (Some(semicolon), Some(close)) => Some(offset + semicolon.min(close)),
                (Some(semicolon), None) => Some(offset + semicolon + 1),
                (None, Some(close)) => Some(offset + close),
                (None, None) => None,
            }
        })
        .unwrap_or(source.len());
    ParserByteSpanV0 { start, end }
}

fn semantic_selector_name_for_byte_offset(source: &str, byte_offset: usize) -> Option<String> {
    let (selector_contexts, _, _, _) = style_context_for_byte_offset(source, byte_offset);
    selector_contexts
        .last()
        .and_then(|selector| selector_class_name(selector))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StyleBlockContext {
    Selector(String),
    Media,
    Supports,
    Layer,
    OtherAtRule,
}

fn block_contexts_for_byte_offset(source: &str, byte_offset: usize) -> Vec<StyleBlockContext> {
    let bytes = source.as_bytes();
    let mut contexts = Vec::new();
    let limit = byte_offset.min(bytes.len());
    let mut index = 0usize;
    while index < limit {
        match bytes[index] {
            b'{' => {
                let header = block_header_before_open_brace(source, index);
                contexts.push(style_block_context_for_header(&header));
            }
            b'}' => {
                contexts.pop();
            }
            _ => {}
        }
        index += 1;
    }
    contexts
}

fn block_header_before_open_brace(source: &str, open_brace_index: usize) -> String {
    let bytes = source.as_bytes();
    let mut start = 0usize;
    let mut index = open_brace_index;
    while let Some(previous) = index.checked_sub(1) {
        index = previous;
        if matches!(bytes[index], b'{' | b'}' | b';') {
            start = index + 1;
            break;
        }
        if index == 0 {
            break;
        }
    }
    source
        .get(start..open_brace_index)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn style_block_context_for_header(header: &str) -> StyleBlockContext {
    let header = header.trim();
    if header.starts_with("@media") {
        StyleBlockContext::Media
    } else if header.starts_with("@supports") {
        StyleBlockContext::Supports
    } else if header.starts_with("@layer") {
        StyleBlockContext::Layer
    } else if header.starts_with('@') {
        StyleBlockContext::OtherAtRule
    } else {
        StyleBlockContext::Selector(header.to_string())
    }
}

fn selector_class_name(selector: &str) -> Option<String> {
    let bytes = selector.as_bytes();
    let mut index = 0usize;
    let mut last = None;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            let start = index + 1;
            let mut end = start;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'-'))
            {
                end += 1;
            }
            if end > start {
                last = selector.get(start..end).map(ToString::to_string);
            }
            index = end;
        } else {
            index += 1;
        }
    }
    last
}

fn previous_non_whitespace_byte_index(bytes: &[u8], before: usize) -> Option<usize> {
    let mut index = before.checked_sub(1)?;
    loop {
        if !bytes[index].is_ascii_whitespace() {
            return Some(index);
        }
        index = index.checked_sub(1)?;
    }
}

fn parser_byte_span_for_offsets(start: usize, end: usize) -> ParserByteSpanV0 {
    ParserByteSpanV0 { start, end }
}

fn parser_range_for_byte_span(source: &str, span: ParserByteSpanV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: parser_position_for_byte_offset(source, span.start),
        end: parser_position_for_byte_offset(source, span.end),
    }
}

fn parser_position_for_byte_offset(source: &str, byte_offset: usize) -> ParserPositionV0 {
    let mut line = 0usize;
    let mut line_start = 0usize;
    let offset = byte_offset.min(source.len());
    for (index, byte) in source.as_bytes().iter().enumerate() {
        if index >= offset {
            break;
        }
        if *byte == b'\n' {
            line += 1;
            line_start = index + 1;
        }
    }
    ParserPositionV0 {
        line,
        character: offset.saturating_sub(line_start),
    }
}

fn dialect_for_style_path(style_path: &str) -> Option<StyleDialect> {
    if style_path.ends_with(".sass") {
        Some(StyleDialect::Sass)
    } else if style_path.ends_with(".scss") {
        Some(StyleDialect::Scss)
    } else if style_path.ends_with(".less") {
        Some(StyleDialect::Less)
    } else if style_path.ends_with(".css") {
        Some(StyleDialect::Css)
    } else {
        None
    }
}

fn omena_parser_dialect_for_style_path(style_path: &str) -> StyleDialect {
    dialect_for_style_path(style_path).unwrap_or(StyleDialect::Css)
}

fn omena_parser_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TheoryObservationHarnessInput, parse_style_module, summarize_lossless_cst_contract,
        summarize_omena_parser_style_semantic_boundary_from_source,
        summarize_parser_contract_facts, summarize_selector_identity_engine,
        summarize_semantic_promotion_evidence,
        summarize_semantic_promotion_evidence_with_source_input, summarize_source_input_evidence,
        summarize_style_semantic_boundary, summarize_style_semantic_facts,
        summarize_style_semantic_graph, summarize_style_semantic_graph_from_source,
        summarize_style_semantic_soa_tables, summarize_theory_observation_contract,
        summarize_theory_observation_harness,
    };
    use engine_input_producers::{
        ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
        SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2,
        StyleSelectorV2, TypeFactEntryV2,
    };

    #[test]
    fn exposes_omena_parser_backed_semantic_boundary() {
        let summary = summarize_omena_parser_style_semantic_boundary_from_source(
            "Component.module.scss",
            r#"
@use "./tokens" as tokens;
$local: red;

@mixin tone($value) {
  color: $value;
}

.button {
  --brand: red;
  color: var(--brand);
  color: $local;
  @include tone(tokens.$accent);

  &__icon {
    animation: pulse 1s;
  }
}

@keyframes pulse {
  to { opacity: 1; }
}
"#,
        );

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.language, "scss");
        assert_eq!(
            summary.parser_facts.selectors.names,
            vec!["button".to_string(), "button__icon".to_string()]
        );
        assert_eq!(
            summary.parser_facts.custom_properties.decl_names,
            vec!["--brand".to_string()]
        );
        assert_eq!(
            summary.parser_facts.custom_properties.ref_names,
            vec!["--brand".to_string()]
        );
        assert_eq!(
            summary.semantic_facts.custom_properties.resolved_ref_names,
            vec!["--brand".to_string()]
        );
        assert_eq!(
            summary.parser_facts.sass.module_use_sources,
            vec!["./tokens".to_string()]
        );
        assert_eq!(
            summary.parser_facts.sass.mixin_include_names,
            vec!["tone".to_string()]
        );
        assert_eq!(
            summary.parser_facts.sass.variable_ref_names,
            vec![
                "accent".to_string(),
                "local".to_string(),
                "value".to_string()
            ]
        );
        assert_eq!(
            summary.parser_facts.keyframes.names,
            vec!["pulse".to_string()]
        );
        assert_eq!(summary.selector_identity_engine.canonical_id_count, 2);
        assert!(
            summary
                .lossless_cst_contract
                .span_invariants
                .byte_span_contract_ready
        );
    }

    #[test]
    fn indexes_layer_container_and_scope_contexts_for_semantic_consumers() {
        let summary = summarize_omena_parser_style_semantic_boundary_from_source(
            "Component.module.css",
            r#"
@layer reset, components;
@layer components {
  @container card (inline-size > 40rem) {
    @scope (.card) to (.card__body) {
      .card { color: red; }
      .card__body { color: blue; }
    }
  }
}
"#,
        );
        let context_index = summary.semantic_facts.context_index;

        assert_eq!(context_index.product, "omena-semantic.style-context-index");
        assert!(
            context_index
                .ready_surfaces
                .contains(&"selectorContextMembership")
        );
        assert_eq!(
            context_index
                .layer_index
                .statement_layers
                .iter()
                .map(|layer| layer.name.as_str())
                .collect::<Vec<_>>(),
            vec!["reset", "components"]
        );
        assert_eq!(context_index.layer_index.named_layer_count, 2);
        assert_eq!(context_index.layer_index.block_layers.len(), 1);
        assert_eq!(
            context_index.layer_index.block_layers[0].name.as_deref(),
            Some("components")
        );
        assert_eq!(context_index.container_index.containers.len(), 1);
        assert_eq!(
            context_index.container_index.containers[0].name.as_deref(),
            Some("card")
        );
        assert_eq!(context_index.scope_index.scopes.len(), 1);
        assert_eq!(context_index.scope_index.scoped_selector_count, 2);
        assert!(
            context_index
                .scope_index
                .selector_memberships
                .iter()
                .any(|membership| membership.selector_name == "card")
        );
        assert!(
            context_index
                .container_index
                .selector_memberships
                .iter()
                .any(|membership| membership.selector_name == "card__body")
        );
    }

    #[test]
    fn exposes_semantic_soa_tables_with_typed_name_interners() {
        let boundary = summarize_omena_parser_style_semantic_boundary_from_source(
            "Component.module.scss",
            r#"
$local: red;

@mixin tone($value) {
  color: $value;
}

.button {
  --brand: red;
  color: var(--brand);
  color: $local;
  @include tone($local);

  &__icon {}
}
"#,
        );
        let db = salsa::DatabaseImpl::default();
        let tables = summarize_style_semantic_soa_tables(&boundary.semantic_facts, &db);

        assert_eq!(tables.schema_version, "0");
        assert_eq!(tables.product, "omena-semantic.soa-tables");
        assert!(tables.ready_surfaces.contains(&"semanticSoaTables"));
        assert!(tables.ready_surfaces.contains(&"semanticSoaNameTables"));
        assert_eq!(
            tables.selector_names.names,
            vec!["button".to_string(), "button__icon".to_string()]
        );
        assert_eq!(
            tables.custom_property_names.names,
            vec!["--brand".to_string()]
        );
        assert!(tables.sass_names.names.contains(&"local".to_string()));
        assert!(tables.sass_names.names.contains(&"tone".to_string()));
        assert_eq!(tables.total_row_count, tables.interned_row_count);
        assert_eq!(
            tables.total_row_count,
            tables.selector_names.row_indices.len()
                + tables.custom_property_names.row_indices.len()
                + tables.sass_names.row_indices.len()
        );
    }

    #[test]
    fn keeps_omena_parser_nested_compound_selectors_rewrite_blocked() {
        let summary = summarize_omena_parser_style_semantic_boundary_from_source(
            "Component.module.scss",
            ".button { &.active { color: red; } }",
        );

        assert_eq!(
            summary.parser_facts.selectors.names,
            vec!["active".to_string(), "button".to_string()]
        );
        assert_eq!(
            summary.parser_facts.selectors.nested_unsafe_names,
            vec!["active".to_string()]
        );
        assert_eq!(
            summary
                .semantic_facts
                .selector_identity
                .nested_safety_counts
                .nested_unsafe,
            1
        );
        assert_eq!(
            summary
                .selector_identity_engine
                .rewrite_safety
                .blocked_canonical_ids,
            vec!["selector:active".to_string()]
        );
    }

    #[test]
    fn exposes_semantic_summary_without_hiding_parser_contract_facts() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.scss",
            r#"
@use "./tokens" as tokens;
$local: red;

@mixin tone($value) {
  color: $value;
}

.button {
  color: $local;
  @include tone(tokens.$accent);

  &__icon {
    animation: pulse 1s;
  }
}

@keyframes pulse {
  from { opacity: 0; }
  to { opacity: 1; }
}
"#,
        )
        .ok_or_else(|| "SCSS module path should parse".to_string())?;

        let summary = summarize_style_semantic_boundary(&sheet);

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.language, "scss");
        assert!(
            summary
                .parser_facts
                .lossless_cst
                .all_token_spans_within_source
        );
        assert!(
            summary
                .parser_facts
                .lossless_cst
                .all_node_spans_within_source
        );
        assert_eq!(
            summary.parser_facts.sass.module_use_sources,
            vec!["./tokens".to_string()]
        );
        assert_eq!(
            summary.semantic_facts.selector_identity.canonical_names,
            vec!["button".to_string(), "button__icon".to_string()]
        );
        assert_eq!(summary.selector_identity_engine.canonical_id_count, 2);
        assert_eq!(
            summary
                .selector_identity_engine
                .canonical_ids
                .iter()
                .map(|identity| identity.canonical_id.as_str())
                .collect::<Vec<_>>(),
            vec!["selector:button", "selector:button__icon"]
        );
        assert!(
            summary
                .selector_identity_engine
                .rewrite_safety
                .all_canonical_ids_rewrite_safe
        );
        assert_eq!(
            summary
                .semantic_facts
                .selector_identity
                .bem_suffix_safe_names,
            vec!["button__icon".to_string()]
        );
        assert_eq!(
            summary
                .semantic_facts
                .sass
                .selectors_with_resolved_variable_refs_names,
            vec!["button".to_string()]
        );
        assert_eq!(
            summary
                .semantic_facts
                .sass
                .selectors_with_resolved_mixin_includes_names,
            vec!["button".to_string()]
        );
        assert!(
            summary
                .lossless_cst_contract
                .span_invariants
                .byte_span_contract_ready
        );
        assert_eq!(
            summary.promotion_evidence.blocking_gaps,
            vec!["referenceSiteIdentity", "certaintyReason"]
        );
        Ok(())
    }

    #[test]
    fn offers_narrow_semantic_and_parser_contract_accessors() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.scss",
            r#"
$color: red;

.button {
  color: $color;
}
"#,
        )
        .ok_or_else(|| "SCSS module path should parse".to_string())?;

        let parser_facts = summarize_parser_contract_facts(&sheet);
        let semantic_facts = summarize_style_semantic_facts(&sheet);

        assert_eq!(parser_facts.selectors.names, vec!["button".to_string()]);
        assert_eq!(
            parser_facts.sass.variable_decl_names,
            vec!["color".to_string()]
        );
        assert_eq!(
            semantic_facts
                .sass
                .selectors_with_resolved_variable_refs_names,
            vec!["button".to_string()]
        );
        assert!(
            semantic_facts
                .sass
                .selectors_with_unresolved_variable_refs_names
                .is_empty()
        );
        Ok(())
    }

    #[test]
    fn exposes_selector_identity_as_dedicated_semantic_sub_engine() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.scss",
            r#"
.button {
  &__icon {}
  &.active {}
}
"#,
        )
        .ok_or_else(|| "SCSS module path should parse".to_string())?;

        let semantic_facts = summarize_style_semantic_facts(&sheet);
        let selector_identity =
            summarize_selector_identity_engine(&semantic_facts.selector_identity);

        assert_eq!(
            selector_identity.product,
            "omena-semantic.selector-identity"
        );
        assert_eq!(
            selector_identity
                .canonical_ids
                .iter()
                .map(|identity| {
                    (
                        identity.canonical_id.as_str(),
                        identity.identity_kind,
                        identity.rewrite_safety,
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                ("selector:active", "localClass", "blocked"),
                ("selector:button", "localClass", "safe"),
                ("selector:button__icon", "bemSuffix", "safe")
            ]
        );
        assert_eq!(
            selector_identity.rewrite_safety.blocked_canonical_ids,
            vec!["selector:active".to_string()]
        );
        assert_eq!(
            selector_identity.rewrite_safety.blockers,
            vec!["nested-expansion"]
        );
        Ok(())
    }

    #[test]
    fn exposes_promotion_evidence_gaps_without_hiding_ready_contracts() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.scss",
            r#"
@use "./tokens" as tokens;

.button {
  color: tokens.$accent;
}
"#,
        )
        .ok_or_else(|| "SCSS module path should parse".to_string())?;

        let parser_facts = summarize_parser_contract_facts(&sheet);
        let semantic_facts = summarize_style_semantic_facts(&sheet);
        let evidence = summarize_semantic_promotion_evidence(&parser_facts, &semantic_facts);

        assert_eq!(evidence.product, "omena-semantic.promotion-evidence");
        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "selectorCanonicalId")
                .map(|item| item.status),
            Some("ready")
        );
        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "sourceSpan")
                .map(|item| item.status),
            Some("ready")
        );
        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "referenceSiteIdentity")
                .map(|item| item.status),
            Some("gap")
        );
        assert_eq!(
            evidence.next_priorities,
            vec!["referenceSiteIdentity", "certaintyReason", "bindingOrigin"]
        );
        Ok(())
    }

    #[test]
    fn exposes_design_token_seed_promotion_evidence() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.css",
            r#"
:root {
  --color-gray-700: #767678;
}

.button {
  color: var(--color-gray-700);
  border-color: var(--missing);
}
"#,
        )
        .ok_or_else(|| "CSS module path should parse".to_string())?;

        let parser_facts = summarize_parser_contract_facts(&sheet);
        let semantic_facts = summarize_style_semantic_facts(&sheet);
        let evidence = summarize_semantic_promotion_evidence(&parser_facts, &semantic_facts);
        let design_token_seed = evidence
            .items
            .iter()
            .find(|item| item.evidence == "designTokenSeed")
            .ok_or_else(|| "expected design token seed evidence".to_string())?;

        assert_eq!(design_token_seed.status, "ready");
        assert_eq!(
            design_token_seed.provider,
            "ParserIndexCustomPropertyFactsV0"
        );
        assert_eq!(design_token_seed.observed_count, 3);
        Ok(())
    }

    #[test]
    fn exposes_design_token_semantic_readiness_surface() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.css",
            r#"
:root {
  --color-gray-700: #767678;
}

@media (min-width: 600px) {
  .button {
    color: var(--color-gray-700);
  }
}

.ghost {
  border-color: var(--missing);
}
"#,
        )
        .ok_or_else(|| "CSS module path should parse".to_string())?;

        let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

        assert_eq!(summary.product, "omena-semantic.design-token-semantics");
        assert_eq!(summary.status, "context-aware-resolution-seed");
        assert_eq!(summary.resolution_scope, "same-file");
        assert_eq!(summary.declaration_count, 1);
        assert_eq!(summary.reference_count, 2);
        assert_eq!(summary.resolved_reference_count, 1);
        assert_eq!(summary.unresolved_reference_count, 1);
        assert_eq!(summary.selectors_with_references_count, 2);
        assert_eq!(summary.context_signal.declaration_context_selector_count, 1);
        assert_eq!(summary.context_signal.declaration_wrapper_context_count, 0);
        assert_eq!(summary.context_signal.media_context_selector_count, 1);
        assert_eq!(summary.context_signal.wrapper_context_count, 1);
        assert_eq!(summary.resolution_signal.declaration_fact_count, 1);
        assert_eq!(summary.resolution_signal.reference_fact_count, 2);
        assert_eq!(
            summary.resolution_signal.source_ordered_declaration_count,
            1
        );
        assert_eq!(summary.resolution_signal.source_ordered_reference_count, 2);
        assert_eq!(
            summary
                .resolution_signal
                .occurrence_resolved_reference_count,
            1
        );
        assert_eq!(
            summary
                .resolution_signal
                .occurrence_unresolved_reference_count,
            1
        );
        assert_eq!(summary.resolution_signal.root_declaration_count, 1);
        assert_eq!(
            summary.resolution_signal.selector_scoped_declaration_count,
            0
        );
        assert_eq!(
            summary.resolution_signal.wrapper_scoped_declaration_count,
            0
        );
        assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 1);
        assert_eq!(summary.cascade_ranking_signal.unranked_reference_count, 1);
        assert_eq!(
            summary
                .cascade_ranking_signal
                .source_order_winner_declaration_count,
            1
        );
        assert_eq!(
            summary
                .cascade_ranking_signal
                .source_order_shadowed_declaration_count,
            0
        );
        assert_eq!(
            summary
                .cascade_ranking_signal
                .repeated_name_declaration_count,
            0
        );
        assert!(summary.capabilities.same_file_resolution_ready);
        assert!(summary.capabilities.wrapper_context_signal_ready);
        assert!(summary.capabilities.source_order_signal_ready);
        assert!(summary.capabilities.source_order_cascade_ranking_ready);
        assert!(summary.capabilities.occurrence_resolution_signal_ready);
        assert!(summary.capabilities.selector_context_resolution_ready);
        assert!(summary.capabilities.theme_override_context_signal_ready);
        assert!(!summary.capabilities.cross_package_cascade_ranking_ready);
        assert!(!summary.capabilities.theme_override_context_ready);
        assert_eq!(
            summary.blocking_gaps,
            vec![
                "crossFileImportGraph",
                "crossPackageCascadeRanking",
                "themeOverrideContext",
                "unresolvedDesignTokenRefs"
            ]
        );
        assert_eq!(
            summary.next_priorities,
            vec![
                "crossFileImportGraph",
                "crossPackageCascadeRanking",
                "themeOverrideContext"
            ]
        );
        Ok(())
    }

    #[test]
    fn exposes_design_token_occurrence_context_resolution_signal() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.css",
            r#"
:root {
  --surface: white;
}

.theme {
  --brand: #222;
}

.button {
  color: var(--brand);
  background: var(--surface);
}

.theme .button {
  border-color: var(--brand);
}
"#,
        )
        .ok_or_else(|| "CSS module path should parse".to_string())?;

        let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

        assert_eq!(summary.resolved_reference_count, 2);
        assert_eq!(summary.unresolved_reference_count, 1);
        assert_eq!(summary.resolution_signal.declaration_fact_count, 2);
        assert_eq!(summary.resolution_signal.reference_fact_count, 3);
        assert_eq!(
            summary.resolution_signal.source_ordered_declaration_count,
            2
        );
        assert_eq!(summary.resolution_signal.source_ordered_reference_count, 3);
        assert_eq!(
            summary
                .resolution_signal
                .occurrence_resolved_reference_count,
            2
        );
        assert_eq!(
            summary
                .resolution_signal
                .occurrence_unresolved_reference_count,
            1
        );
        assert_eq!(summary.resolution_signal.context_matched_reference_count, 2);
        assert_eq!(
            summary.resolution_signal.context_unmatched_reference_count,
            1
        );
        assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 2);
        assert_eq!(summary.cascade_ranking_signal.unranked_reference_count, 1);
        assert_eq!(
            summary
                .cascade_ranking_signal
                .source_order_winner_declaration_count,
            2
        );
        assert_eq!(
            summary
                .cascade_ranking_signal
                .source_order_shadowed_declaration_count,
            0
        );
        assert_eq!(summary.resolution_signal.root_declaration_count, 1);
        assert_eq!(
            summary.resolution_signal.selector_scoped_declaration_count,
            1
        );
        assert!(summary.capabilities.occurrence_resolution_signal_ready);
        assert!(summary.capabilities.source_order_signal_ready);
        assert!(summary.capabilities.selector_context_resolution_ready);
        Ok(())
    }

    #[test]
    fn exposes_design_token_source_order_cascade_ranking_signal() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.css",
            r#"
:root {
  --surface: white;
}

:root {
  --surface: black;
}

.theme {
  --surface: gray;
}

.button {
  color: var(--surface);
}

.theme .button {
  background: var(--surface);
}
"#,
        )
        .ok_or_else(|| "CSS module path should parse".to_string())?;

        let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

        assert_eq!(summary.status, "same-file-cascade-ranking-seed");
        assert_eq!(summary.declaration_count, 1);
        assert_eq!(summary.reference_count, 1);
        assert_eq!(summary.resolved_reference_count, 1);
        assert_eq!(summary.unresolved_reference_count, 0);
        assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 2);
        assert_eq!(summary.cascade_ranking_signal.unranked_reference_count, 0);
        assert_eq!(
            summary
                .cascade_ranking_signal
                .source_order_winner_declaration_count,
            2
        );
        assert_eq!(
            summary
                .cascade_ranking_signal
                .source_order_shadowed_declaration_count,
            2
        );
        assert_eq!(
            summary
                .cascade_ranking_signal
                .repeated_name_declaration_count,
            3
        );
        assert_eq!(summary.cascade_ranking_signal.ranked_references.len(), 2);
        let first_ranked_reference = &summary.cascade_ranking_signal.ranked_references[0];
        assert_eq!(first_ranked_reference.reference_name, "--surface");
        assert_eq!(first_ranked_reference.reference_source_order, 0);
        assert_eq!(first_ranked_reference.winner_declaration_source_order, 1);
        assert_eq!(
            first_ranked_reference.shadowed_declaration_source_orders,
            vec![0]
        );
        assert_eq!(first_ranked_reference.candidate_declaration_count, 2);
        let second_ranked_reference = &summary.cascade_ranking_signal.ranked_references[1];
        assert_eq!(second_ranked_reference.reference_name, "--surface");
        assert_eq!(second_ranked_reference.reference_source_order, 1);
        assert_eq!(second_ranked_reference.winner_declaration_source_order, 2);
        assert_eq!(
            second_ranked_reference.shadowed_declaration_source_orders,
            vec![0, 1]
        );
        assert_eq!(second_ranked_reference.candidate_declaration_count, 3);
        assert!(summary.capabilities.source_order_cascade_ranking_ready);
        assert!(!summary.capabilities.cross_package_cascade_ranking_ready);
        Ok(())
    }

    #[test]
    fn ranks_theme_context_declarations_ahead_of_later_root_tokens() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.css",
            r#"
:root {
  --surface: white;
}

[data-theme="dark"] {
  --surface: black;
}

:root {
  --surface: beige;
}

[data-theme="dark"] .button {
  color: var(--surface);
}
"#,
        )
        .ok_or_else(|| "CSS module path should parse".to_string())?;

        let summary = summarize_style_semantic_boundary(&sheet).design_token_semantics;

        assert_eq!(summary.status, "same-file-cascade-ranking-seed");
        assert_eq!(summary.cascade_ranking_signal.ranked_reference_count, 1);
        assert_eq!(
            summary
                .cascade_ranking_signal
                .theme_context_winner_reference_count,
            1
        );
        let ranked_reference = &summary.cascade_ranking_signal.ranked_references[0];
        assert_eq!(ranked_reference.reference_name, "--surface");
        assert_eq!(ranked_reference.winner_declaration_source_order, 1);
        assert_eq!(
            ranked_reference.shadowed_declaration_source_orders,
            vec![0, 2]
        );
        assert_eq!(ranked_reference.winner_context_kind, "selector");
        assert!(summary.capabilities.theme_override_context_ready);
        assert!(!summary.blocking_gaps.contains(&"themeOverrideContext"));
        assert!(!summary.next_priorities.contains(&"themeOverrideContext"));
        Ok(())
    }

    #[test]
    fn exposes_lossless_cst_contract_for_precise_consumers() -> Result<(), String> {
        let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
            .ok_or_else(|| "SCSS module path should parse".to_string())?;

        let parser_facts = summarize_parser_contract_facts(&sheet);
        let contract = summarize_lossless_cst_contract(&parser_facts.lossless_cst);

        assert_eq!(contract.product, "omena-semantic.lossless-cst-contract");
        assert!(contract.span_invariants.byte_span_contract_ready);
        assert!(contract.consumer_readiness.precise_rename_base_ready);
        assert!(contract.consumer_readiness.formatter_base_ready);
        assert!(!contract.consumer_readiness.recovery_diagnostics_observed);
        Ok(())
    }

    #[test]
    fn exposes_source_input_evidence_for_reference_identity_and_certainty_reasons() {
        let evidence = summarize_source_input_evidence(&sample_engine_input());

        assert_eq!(evidence.product, "omena-semantic.source-input-evidence");
        assert_eq!(evidence.reference_site_identity.status, "ready");
        assert_eq!(evidence.reference_site_identity.reference_site_count, 2);
        assert_eq!(
            evidence.reference_site_identity.direct_reference_site_count,
            1
        );
        assert_eq!(
            evidence
                .reference_site_identity
                .expanded_reference_site_count,
            1
        );
        assert_eq!(
            evidence.reference_site_identity.editable_direct_site_count,
            1
        );
        assert_eq!(evidence.certainty_reason.status, "ready");
        assert_eq!(evidence.certainty_reason.expression_count, 2);
        assert_eq!(evidence.certainty_reason.exact_count, 1);
        assert_eq!(evidence.certainty_reason.inferred_count, 1);
        assert_eq!(evidence.binding_origin.status, "ready");
        assert_eq!(evidence.binding_origin.expression_count, 2);
        assert_eq!(evidence.binding_origin.direct_class_name_count, 1);
        assert_eq!(evidence.binding_origin.root_binding_count, 1);
        assert_eq!(
            evidence
                .binding_origin
                .expression_kind_counts
                .get("literal"),
            Some(&1)
        );
        assert_eq!(evidence.style_module_edge.status, "ready");
        assert_eq!(evidence.style_module_edge.source_style_edge_count, 2);
        assert_eq!(evidence.style_module_edge.distinct_style_module_count, 1);
        assert_eq!(
            evidence.style_module_edge.missing_style_document_edge_count,
            0
        );
        assert_eq!(evidence.value_domain_explanation.status, "ready");
        assert_eq!(evidence.value_domain_explanation.expression_count, 2);
        assert_eq!(evidence.value_domain_explanation.exact_expression_count, 1);
        assert_eq!(
            evidence
                .value_domain_explanation
                .constrained_expression_count,
            1
        );
        assert_eq!(evidence.value_domain_explanation.finite_value_count, 1);
        assert_eq!(evidence.value_domain_explanation.derivation_count, 2);
        assert_eq!(evidence.value_domain_explanation.derivation_step_count, 2);
        assert_eq!(
            evidence
                .value_domain_explanation
                .derivation_product_counts
                .get("omena-abstract-value.reduced-class-value-derivation"),
            Some(&2)
        );
        assert_eq!(
            evidence
                .value_domain_explanation
                .derivation_reduced_kind_counts
                .get("exact"),
            Some(&1)
        );
        assert_eq!(
            evidence
                .value_domain_explanation
                .derivation_reduced_kind_counts
                .get("prefix"),
            Some(&1)
        );
        assert_eq!(
            evidence
                .value_domain_explanation
                .derivation_operation_counts
                .get("baseFromFacts"),
            Some(&2)
        );
        assert_eq!(
            evidence
                .certainty_reason
                .reason_counts
                .get("single selector matched"),
            Some(&1)
        );
        assert_eq!(
            evidence
                .certainty_reason
                .reason_counts
                .get("constrained runtime shape matched a bounded selector set"),
            Some(&1)
        );
    }

    #[test]
    fn source_input_evidence_upgrades_promotion_evidence_gaps() -> Result<(), String> {
        let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
            .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let parser_facts = summarize_parser_contract_facts(&sheet);
        let semantic_facts = summarize_style_semantic_facts(&sheet);
        let evidence = summarize_semantic_promotion_evidence_with_source_input(
            &parser_facts,
            &semantic_facts,
            &sample_engine_input(),
        );

        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "referenceSiteIdentity")
                .map(|item| item.status),
            Some("ready")
        );
        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "bindingOrigin")
                .map(|item| item.status),
            Some("ready")
        );
        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "styleModuleEdge")
                .map(|item| item.status),
            Some("ready")
        );
        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "valueDomainExplanation")
                .map(|item| item.status),
            Some("ready")
        );
        assert_eq!(
            evidence
                .items
                .iter()
                .find(|item| item.evidence == "certaintyReason")
                .map(|item| item.status),
            Some("ready")
        );
        assert!(evidence.blocking_gaps.is_empty());
        assert!(evidence.next_priorities.is_empty());
        Ok(())
    }

    #[test]
    fn exposes_style_semantic_graph_with_source_backed_promotion_evidence() -> Result<(), String> {
        let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
            .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let graph = summarize_style_semantic_graph(&sheet, &sample_engine_input());

        assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
        assert_eq!(graph.language, "scss");
        assert_eq!(
            graph.selector_reference_engine.product,
            "omena-semantic.selector-references"
        );
        assert_eq!(graph.selector_reference_engine.selector_count, 2);
        assert_eq!(graph.selector_reference_engine.referenced_selector_count, 2);
        assert_eq!(graph.selector_reference_engine.total_reference_sites, 2);
        assert_eq!(graph.source_input_evidence.binding_origin.status, "ready");
        assert_eq!(
            graph
                .promotion_evidence
                .items
                .iter()
                .filter(|item| item.status == "gap")
                .count(),
            0
        );
        assert!(graph.promotion_evidence.blocking_gaps.is_empty());
        assert!(
            graph
                .lossless_cst_contract
                .span_invariants
                .byte_span_contract_ready
        );
        Ok(())
    }

    #[test]
    fn summarizes_style_semantic_graph_from_source_for_host_consumers() -> Result<(), String> {
        let graph = summarize_style_semantic_graph_from_source(
            "/tmp/Component.module.scss",
            ".button { color: red; }",
            &sample_engine_input(),
        )
        .ok_or_else(|| "expected style semantic graph".to_string())?;

        assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
        assert_eq!(graph.language, "scss");
        assert_eq!(
            graph.selector_identity_engine.product,
            "omena-semantic.selector-identity"
        );
        assert_eq!(
            graph.selector_reference_engine.style_path,
            Some("/tmp/Component.module.scss".to_string())
        );
        assert_eq!(graph.selector_reference_engine.selector_count, 2);

        Ok(())
    }

    #[test]
    fn style_semantic_graph_includes_css_modules_parser_fact_seed() -> Result<(), String> {
        let graph = summarize_style_semantic_graph_from_source(
            "/tmp/Component.module.scss",
            "@value primary: #fff; @value accent: primary; @value secondary as localSecondary from \"./tokens.module.scss\"; :export { primary: #fff; forwarded: imported; } :import(\"./tokens.css\") { imported: primary; } @keyframes fade { to { opacity: 1; } } .card { composes: base utility from \"./base.module.scss\"; animation: fade 1s; }",
            &sample_engine_input(),
        )
        .ok_or_else(|| "expected style semantic graph".to_string())?;

        let css_modules = graph.css_modules_semantics;
        assert_eq!(css_modules.product, "omena-semantic.css-modules-semantics");
        assert_eq!(css_modules.status, "parserFactSeed");
        assert_eq!(css_modules.resolution_scope, "perFileFactSummary");
        assert_eq!(css_modules.class_export_names, vec!["card"]);
        assert_eq!(css_modules.composes_edge_seed_count, 1);
        assert_eq!(css_modules.composes_external_edge_count, 1);
        assert_eq!(css_modules.composes_local_edge_count, 0);
        assert_eq!(css_modules.composes_global_edge_count, 0);
        assert_eq!(css_modules.composes_target_names, vec!["base", "utility"]);
        assert_eq!(
            css_modules.composes_import_sources,
            vec!["./base.module.scss"]
        );
        assert_eq!(
            css_modules.value_definition_names,
            vec!["accent", "localSecondary", "primary"]
        );
        assert_eq!(
            css_modules.value_reference_names,
            vec!["primary", "secondary"]
        );
        assert_eq!(
            css_modules.value_import_sources,
            vec!["./tokens.module.scss"]
        );
        assert_eq!(css_modules.value_import_edge_count, 1);
        assert_eq!(css_modules.value_definition_edge_count, 1);
        assert_eq!(css_modules.value_edge_seed_count, 2);
        assert_eq!(css_modules.icss_export_names, vec!["forwarded", "primary"]);
        assert_eq!(css_modules.icss_import_local_names, vec!["imported"]);
        assert_eq!(css_modules.icss_import_remote_names, vec!["primary"]);
        assert_eq!(css_modules.icss_import_sources, vec!["./tokens.css"]);
        assert_eq!(css_modules.icss_import_edge_count, 1);
        assert_eq!(css_modules.icss_export_edge_count, 1);
        assert_eq!(css_modules.icss_edge_seed_count, 2);
        assert_eq!(css_modules.keyframe_names, vec!["fade"]);
        assert_eq!(css_modules.animation_reference_names, vec!["fade"]);
        assert!(css_modules.capabilities.parser_fact_surface_ready);
        assert!(css_modules.capabilities.per_file_symbol_summary_ready);
        assert!(!css_modules.capabilities.cross_file_resolution_ready);
        assert!(!css_modules.capabilities.composes_closure_ready);

        Ok(())
    }

    #[test]
    fn theory_observation_harness_reports_ready_semantic_graph() -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.scss",
            ".button { &__icon { color: red; } }",
        )
        .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let graph = summarize_style_semantic_graph(&sheet, &sample_engine_input());
        let observation = summarize_theory_observation_harness(&graph);

        assert_eq!(
            observation.product,
            "omena-semantic.theory-observation-harness"
        );
        assert_eq!(
            observation.graph_product,
            "omena-semantic.style-semantic-graph"
        );
        assert_eq!(observation.selector_identity.status, "ready");
        assert_eq!(observation.selector_identity.observed_selector_count, 2);
        assert_eq!(
            observation.selector_identity.rewrite_blocked_selector_count,
            0
        );
        assert!(observation.selector_identity.rename_safe);
        assert_eq!(observation.source_evidence.status, "ready");
        assert_eq!(observation.source_evidence.reference_site_count, 2);
        assert_eq!(
            observation
                .source_evidence
                .certainty_reason_counts
                .get("single selector matched"),
            Some(&1)
        );
        assert_eq!(observation.downstream_readiness.status, "ready");
        assert!(observation.downstream_readiness.downstream_check_ready);
        assert!(observation.downstream_readiness.precise_rename_ready);
        assert_eq!(observation.coupling_boundary.generic_observation_count, 4);
        assert_eq!(
            observation.coupling_boundary.cme_coupled_observation_count,
            2
        );
        assert_eq!(
            observation.coupling_boundary.split_recommendation,
            "keep-integrated-observe-boundary"
        );
        assert!(observation.blocking_gaps.is_empty());
        assert_eq!(
            observation.next_priorities,
            vec!["externalCorpus", "traitDogfooding"]
        );

        let contract = summarize_theory_observation_contract(&graph);
        assert_eq!(
            contract.product,
            "omena-semantic.theory-observation-contract"
        );
        assert_eq!(
            contract.observation_product,
            "omena-semantic.theory-observation-harness"
        );
        assert!(contract.ready);
        assert!(contract.publish_ready);
        assert_eq!(contract.selector_identity_status, "ready");
        assert_eq!(contract.source_evidence_status, "ready");
        assert_eq!(contract.downstream_readiness_status, "ready");
        assert!(contract.blocking_gaps.is_empty());
        assert!(contract.publish_blocking_gaps.is_empty());
        assert!(contract.observation_gaps.is_empty());
        assert_eq!(contract, graph.summarize_theory_observation_contract());
        Ok(())
    }

    #[test]
    fn theory_observation_harness_marks_rewrite_blockers_without_hiding_graph_readiness()
    -> Result<(), String> {
        let sheet = parse_style_module(
            "Component.module.scss",
            r#"
.button {
  &.active {}
}
"#,
        )
        .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let graph = summarize_style_semantic_graph(&sheet, &sample_engine_input());
        let observation = summarize_theory_observation_harness(&graph);

        assert_eq!(observation.selector_identity.status, "partial");
        assert_eq!(
            observation.selector_identity.rewrite_blocked_selector_count,
            1
        );
        assert_eq!(
            observation.selector_identity.blockers,
            vec!["nested-expansion"]
        );
        assert!(observation.downstream_readiness.downstream_check_ready);
        assert!(!observation.downstream_readiness.precise_rename_ready);
        assert_eq!(observation.downstream_readiness.status, "partial");
        assert_eq!(
            observation.blocking_gaps,
            vec!["selectorRewriteSafety", "downstreamReadiness"]
        );

        let contract = graph.summarize_theory_observation_contract();
        assert!(!contract.ready);
        assert!(!contract.publish_ready);
        assert_eq!(
            contract.blocking_gaps,
            vec!["selectorRewriteSafety", "downstreamReadiness"]
        );
        assert_eq!(
            contract.publish_blocking_gaps,
            vec!["selectorRewriteSafety"]
        );
        assert_eq!(contract.observation_gaps, vec!["downstreamReadiness"]);
        Ok(())
    }

    #[test]
    fn theory_observation_harness_exposes_cme_coupling_gaps() -> Result<(), String> {
        let sheet = parse_style_module("Component.module.scss", ".button { color: red; }")
            .ok_or_else(|| "SCSS module path should parse".to_string())?;
        let graph = summarize_style_semantic_graph(&sheet, &empty_engine_input());
        let observation = summarize_theory_observation_harness(&graph);

        assert_eq!(observation.selector_identity.status, "ready");
        assert_eq!(observation.source_evidence.status, "gap");
        assert_eq!(observation.source_evidence.reference_site_count, 0);
        assert_eq!(
            observation
                .source_evidence
                .explainable_certainty_reason_count,
            0
        );
        assert_eq!(observation.downstream_readiness.status, "gap");
        assert_eq!(
            observation.blocking_gaps,
            vec!["sourceEvidence", "downstreamReadiness"]
        );
        assert_eq!(
            observation.coupling_boundary.generic_surfaces,
            vec![
                "parserSemanticFacts",
                "designTokenSemantics",
                "selectorIdentity",
                "losslessCstContract"
            ]
        );
        assert_eq!(
            observation.coupling_boundary.cme_coupled_surfaces,
            vec!["sourceInputEvidence", "promotionEvidenceWithSourceInput"]
        );
        let contract = graph.summarize_theory_observation_contract();
        assert!(!contract.ready);
        assert!(contract.publish_ready);
        assert!(contract.publish_blocking_gaps.is_empty());
        assert_eq!(
            contract.observation_gaps,
            vec!["sourceEvidence", "downstreamReadiness"]
        );
        Ok(())
    }

    fn sample_engine_input() -> EngineInputV2 {
        EngineInputV2 {
            version: "2".to_string(),
            sources: vec![SourceAnalysisInputV2 {
                document: SourceDocumentV2 {
                    class_expressions: vec![
                        ClassExpressionInputV2 {
                            id: "expr-literal".to_string(),
                            kind: "literal".to_string(),
                            scss_module_path: "/tmp/Component.module.scss".to_string(),
                            range: range(4, 12, 4, 18),
                            class_name: Some("button".to_string()),
                            root_binding_decl_id: None,
                            access_path: None,
                        },
                        ClassExpressionInputV2 {
                            id: "expr-prefix".to_string(),
                            kind: "symbolRef".to_string(),
                            scss_module_path: "/tmp/Component.module.scss".to_string(),
                            range: range(5, 12, 5, 24),
                            class_name: None,
                            root_binding_decl_id: Some("decl-prefix".to_string()),
                            access_path: None,
                        },
                    ],
                },
            }],
            styles: vec![StyleAnalysisInputV2 {
                file_path: "/tmp/Component.module.scss".to_string(),
                source: None,
                document: StyleDocumentV2 {
                    selectors: vec![
                        StyleSelectorV2 {
                            name: "button".to_string(),
                            view_kind: "canonical".to_string(),
                            canonical_name: Some("button".to_string()),
                            range: range(0, 1, 0, 7),
                            nested_safety: Some("flat".to_string()),
                            composes: None,
                            bem_suffix: None,
                        },
                        StyleSelectorV2 {
                            name: "button--primary".to_string(),
                            view_kind: "canonical".to_string(),
                            canonical_name: Some("button--primary".to_string()),
                            range: range(1, 1, 1, 16),
                            nested_safety: Some("flat".to_string()),
                            composes: None,
                            bem_suffix: None,
                        },
                    ],
                },
            }],
            type_facts: vec![
                TypeFactEntryV2 {
                    file_path: "/tmp/Component.tsx".to_string(),
                    expression_id: "expr-literal".to_string(),
                    facts: StringTypeFactsV2 {
                        kind: "exact".to_string(),
                        constraint_kind: None,
                        values: Some(vec!["button".to_string()]),
                        prefix: None,
                        suffix: None,
                        min_len: None,
                        max_len: None,
                        char_must: None,
                        char_may: None,
                        may_include_other_chars: None,
                    },
                },
                TypeFactEntryV2 {
                    file_path: "/tmp/Component.tsx".to_string(),
                    expression_id: "expr-prefix".to_string(),
                    facts: StringTypeFactsV2 {
                        kind: "constrained".to_string(),
                        constraint_kind: Some("prefix".to_string()),
                        values: None,
                        prefix: Some("button--".to_string()),
                        suffix: None,
                        min_len: None,
                        max_len: None,
                        char_must: None,
                        char_may: None,
                        may_include_other_chars: None,
                    },
                },
            ],
        }
    }

    fn empty_engine_input() -> EngineInputV2 {
        EngineInputV2 {
            version: "2".to_string(),
            sources: Vec::new(),
            styles: Vec::new(),
            type_facts: Vec::new(),
        }
    }

    fn range(
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> RangeV2 {
        RangeV2 {
            start: PositionV2 {
                line: start_line,
                character: start_character,
            },
            end: PositionV2 {
                line: end_line,
                character: end_character,
            },
        }
    }
}
