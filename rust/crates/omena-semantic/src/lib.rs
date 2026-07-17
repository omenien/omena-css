//! Semantic fact layer for parsed omena-css style modules.
//!
//! This crate lifts parser facts into selector, custom-property, Sass module,
//! design-token, and source-evidence summaries. It is the bridge between the
//! lossless parser substrate and query/LSP consumers that need stable semantic
//! contracts rather than raw CST traversal.

use engine_input_producers::EngineInputV2;
use omena_cascade::{SelectorMatchVerdict, selector_context_witness_for_declaration};
use omena_interner::{
    intern_class_name, intern_css_ident, intern_custom_property_name, intern_keyframes_name,
    intern_mixin_name,
};
use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedCst, ParsedSassModuleEdgeFactKind,
    ParsedSassSymbolFactKind, ParsedSelectorFactKind, ParsedStyleFacts, ParsedVariableFactKind,
    StyleDialect, facts_from_cst, parse,
};
use omena_syntax::{SyntaxKind, SyntaxNode};
use serde::Serialize;
use std::collections::BTreeSet;

mod css_modules;
mod css_modules_cross_file;
mod design_tokens;
mod evidence;
mod layer_tree;
mod lossless_cst;
mod observation;
mod sass_module_graph;
mod selector_identity;
mod selector_references;
mod source_evidence;
mod types;

pub use css_modules::{
    CssModulesSemanticCapabilitiesV0, CssModulesSemanticSummaryV0, summarize_css_modules_semantics,
    summarize_css_modules_semantics_from_source,
};
pub use css_modules_cross_file::{
    CssModulesComposesClosureEdgeV0, CssModulesComposesEdgeFactV0,
    CssModulesCrossFileClosureCapabilitiesV0, CssModulesCrossFileClosureSummaryV0,
    CssModulesCrossFileResolutionCapabilitiesV0, CssModulesCrossFileResolutionSummaryV0,
    CssModulesCrossFileStyleFactsV0, CssModulesCycleV0, CssModulesIcssClosureEdgeV0,
    CssModulesIcssExportEdgeFactV0, CssModulesIcssImportEdgeFactV0,
    CssModulesImportEdgeResolutionV0, CssModulesValueClosureEdgeV0,
    CssModulesValueDefinitionEdgeFactV0, CssModulesValueImportEdgeFactV0,
    summarize_css_modules_cross_file_closure, summarize_css_modules_cross_file_resolution,
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
pub use sass_module_graph::{
    SassModuleConfigurableNamesResolverV0, SassModuleCycleV0,
    SassModuleForwardConfigurationRequestV0, SassModuleGraphClosureCapabilitiesV0,
    SassModuleGraphClosureEdgeV0, SassModuleGraphClosureSummaryV0,
    SassModuleGraphConfigurationResolverV0, SassModuleGraphEdgeFactV0,
    SassModuleGraphResolutionCapabilitiesV0, SassModuleGraphResolutionSummaryV0,
    SassModuleUseConfigurationRequestV0, SassModuleVariableOverrideV0,
    SassModuleVisibleSymbolsResolverV0, SassSymbolKeyV0, StyleImportReachabilityCapabilitiesV0,
    StyleImportReachabilityEdgeFactV0, StyleImportReachabilityFactV0,
    StyleImportReachabilitySummaryV0, apply_sass_forward_prefix, collect_visible_sass_symbol_keys,
    derive_sass_forward_effective_variable_overrides, derive_sass_forward_export_prefix_at_ordinal,
    derive_sass_module_configurable_variable_names,
    derive_sass_module_forward_effective_variable_overrides_at_ordinal,
    derive_sass_module_forward_variable_override_values_at_ordinal,
    derive_sass_module_forward_variable_overrides_at_ordinal,
    derive_sass_module_rule_variable_overrides_at_ordinal,
    filter_sass_forward_configurable_variable_names, filter_sass_forward_exports,
    fold_sass_symbol_name, prefix_sass_forward_exports,
    resolve_sass_module_effective_variable_overrides, sass_forward_filter_name_matches_symbol,
    sass_module_configuration_variables_are_valid, sass_symbol_key,
    summarize_sass_module_configuration_signature, summarize_sass_module_graph_closure,
    summarize_sass_module_graph_resolution, summarize_sass_module_instance_identity_key,
    summarize_style_import_reachability, with_sass_module_rawallpaths_closure_for_test,
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
    StyleContextSelectorMembershipV0, StyleCustomPropertySemanticFactsV0, StyleLayerBlockBindingV0,
    StyleLayerIndexV0, StyleLayerOrderNodeV0, StyleLayerStatementV0, StyleSassSemanticFactsV0,
    StyleScopeIndexV0, StyleSelectorIdentityFactsV0, StyleSemanticFactsV0, Stylesheet,
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
pub struct StyleRuntimeIndexFactsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub language: &'static str,
    pub class_selector_names: Vec<String>,
    pub custom_property_names: Vec<String>,
    pub custom_property_decl_names: Vec<String>,
    pub custom_property_ref_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub animation_reference_names: Vec<String>,
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
    let (boundary, facts) = summarize_omena_parser_style_semantic_boundary_with_facts_from_source(
        &sheet.path,
        &sheet.source,
    );
    let parser_facts = boundary.parser_facts;
    let semantic_facts = boundary.semantic_facts;
    let effective_style_path = style_path.or(Some(sheet.path.as_str()));
    let design_token_semantics = summarize_design_token_semantics_with_workspace_declarations(
        &parser_facts,
        &semantic_facts,
        effective_style_path,
        workspace_declarations,
    );
    let css_modules_semantics = css_modules::summarize_css_modules_semantics_from_facts(&facts);
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
    dialect_for_style_path(style_path)?;
    let (boundary, facts) = summarize_omena_parser_style_semantic_boundary_with_facts_from_source(
        style_path,
        style_source,
    );
    let css_modules_semantics = css_modules::summarize_css_modules_semantics_from_facts(&facts);
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

pub fn summarize_style_runtime_index_facts_from_source(
    style_path: &str,
    style_source: &str,
) -> Option<StyleRuntimeIndexFactsV0> {
    dialect_for_style_path(style_path)?;
    let boundary =
        summarize_omena_parser_style_semantic_boundary_from_source(style_path, style_source);
    let custom_property_names = boundary
        .semantic_facts
        .custom_properties
        .decl_names
        .iter()
        .chain(boundary.semantic_facts.custom_properties.ref_names.iter())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    Some(StyleRuntimeIndexFactsV0 {
        schema_version: "0",
        product: "omena-semantic.style-runtime-index-facts",
        style_path: style_path.to_string(),
        language: boundary.language,
        class_selector_names: boundary.parser_facts.selectors.names,
        custom_property_names,
        custom_property_decl_names: boundary.semantic_facts.custom_properties.decl_names,
        custom_property_ref_names: boundary.semantic_facts.custom_properties.ref_names,
        keyframe_names: boundary.parser_facts.keyframes.names,
        animation_reference_names: boundary.parser_facts.keyframes.animation_ref_names,
        ready_surfaces: vec![
            "semanticRuntimeIndexFacts",
            "customPropertyRuntimeIndex",
            "keyframeRuntimeIndex",
        ],
    })
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
    summarize_omena_parser_style_semantic_boundary_with_facts_from_source(style_path, style_source)
        .0
}

fn summarize_omena_parser_style_semantic_boundary_with_facts_from_source(
    style_path: &str,
    style_source: &str,
) -> (StyleSemanticBoundarySummaryV0, ParsedStyleFacts) {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let parsed = parse(style_source, dialect);
    let facts = facts_from_cst(style_source, &parsed);
    let cst = parsed.cst();
    let parser_facts = summarize_omena_parser_contract_facts(
        style_source,
        parsed.token_count(),
        parsed.syntax().children().count(),
        parsed.errors().len(),
        &facts,
        &cst,
    );
    let semantic_facts =
        summarize_omena_parser_semantic_facts(style_source, &facts, &parser_facts, &cst);
    let design_token_semantics = summarize_design_token_semantics(&parser_facts, &semantic_facts);
    let selector_identity_engine =
        summarize_selector_identity_engine(&semantic_facts.selector_identity);
    let promotion_evidence = summarize_semantic_promotion_evidence(&parser_facts, &semantic_facts);
    let lossless_cst_contract = summarize_lossless_cst_contract(&parser_facts.lossless_cst);

    (
        StyleSemanticBoundarySummaryV0 {
            schema_version: "0",
            language: omena_parser_dialect_label(dialect),
            parser_facts,
            semantic_facts,
            design_token_semantics,
            selector_identity_engine,
            promotion_evidence,
            lossless_cst_contract,
        },
        facts,
    )
}

fn summarize_omena_parser_contract_facts(
    source: &str,
    token_count: usize,
    root_node_count: usize,
    diagnostic_count: usize,
    facts: &ParsedStyleFacts,
    cst: &ParsedCst,
) -> ParserBoundarySyntaxFactsV0 {
    let (all_token_spans_within_source, all_node_spans_within_source) =
        cst_span_bounds_within_source(cst, source.len());
    ParserBoundarySyntaxFactsV0 {
        lossless_cst: ParserLosslessCstFactsV0 {
            source_byte_len: source.len(),
            token_count,
            root_node_count,
            diagnostic_count,
            all_token_spans_within_source,
            all_node_spans_within_source,
        },
        selectors: summarize_omena_parser_selector_facts(source, facts),
        values: summarize_omena_parser_value_facts(facts),
        custom_properties: summarize_omena_parser_custom_property_facts(source, facts, cst),
        sass: summarize_omena_parser_sass_syntax_facts(facts),
        keyframes: summarize_omena_parser_keyframe_facts(facts),
        composes: summarize_omena_parser_composes_facts(facts),
        wrappers: ParserIndexWrapperFactsV0::default(),
    }
}

fn cst_span_bounds_within_source(cst: &ParsedCst, source_byte_len: usize) -> (bool, bool) {
    let all_token_spans_within_source = cst
        .root()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .all(|token| {
            let range = token.text_range();
            byte_offsets_within_source(
                u32::from(range.start()) as usize,
                u32::from(range.end()) as usize,
                source_byte_len,
            )
        });
    let all_node_spans_within_source = std::iter::once(cst.root())
        .chain(cst.root().descendants())
        .all(|node| {
            let range = node.text_range();
            byte_offsets_within_source(
                u32::from(range.start()) as usize,
                u32::from(range.end()) as usize,
                source_byte_len,
            )
        });
    (all_token_spans_within_source, all_node_spans_within_source)
}

fn byte_offsets_within_source(start: usize, end: usize, source_byte_len: usize) -> bool {
    start <= end && end <= source_byte_len
}

fn summarize_omena_parser_semantic_facts(
    source: &str,
    facts: &ParsedStyleFacts,
    parser_facts: &ParserBoundarySyntaxFactsV0,
    cst: &ParsedCst,
) -> StyleSemanticFactsV0 {
    let custom_properties =
        summarize_omena_parser_custom_property_semantic_facts(&parser_facts.custom_properties);
    let sass_same_file_resolution =
        summarize_omena_parser_sass_same_file_resolution(&parser_facts.sass);
    let sass_selector_resolution =
        summarize_omena_parser_sass_selector_resolution(facts, &sass_same_file_resolution, cst);
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
        context_index: summarize_style_context_index(source, cst),
    }
}

fn summarize_style_context_index(source: &str, cst: &ParsedCst) -> StyleContextIndexV0 {
    let layer_statements = layer_statement_facts_from_cst(source, cst);
    let (context_blocks, memberships) = style_context_blocks_and_memberships_from_cst(source, cst);
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
    let layer_order = layer_tree::summarize_layer_order_from_cst(source, cst);

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
            named_layer_count: layer_order.order_nodes.len(),
            order_nodes: layer_order.order_nodes,
            block_bindings: layer_order.block_bindings,
            unresolved_topology_count: layer_order.unresolved_topology_count,
            topology_complete: layer_order.topology_complete,
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

/// Build the canonical nested cascade-layer order from the parser CST.
pub fn summarize_style_layer_order_from_source(
    source: &str,
    dialect: StyleDialect,
) -> StyleLayerIndexV0 {
    let parsed = parse(source, dialect);
    let cst = parsed.cst();
    let context = summarize_style_context_index(source, &cst);
    context.layer_index
}

fn layer_statement_facts_from_cst(source: &str, cst: &ParsedCst) -> Vec<StyleLayerStatementV0> {
    let mut statements = Vec::new();
    for node in cst
        .root()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::LayerRule)
    {
        if cst_node_has_block(node) {
            continue;
        }
        let range = node.text_range();
        let byte_span = ParserByteSpanV0 {
            start: u32::from(range.start()) as usize,
            end: u32::from(range.end()) as usize,
        };
        for layer_name in node
            .descendants()
            .filter(|child| child.kind() == SyntaxKind::LayerName)
            .flat_map(|child| split_layer_names(&syntax_node_text(child)))
        {
            statements.push(StyleLayerStatementV0 {
                name: layer_name,
                source_order: statements.len(),
                byte_span,
                range: parser_range_for_byte_span(source, byte_span),
            });
        }
    }
    statements
}

fn style_context_blocks_and_memberships_from_cst(
    source: &str,
    cst: &ParsedCst,
) -> (
    Vec<StyleContextBlockV0>,
    Vec<StyleContextSelectorMembershipV0>,
) {
    let mut context_nodes = Vec::new();
    let mut blocks = Vec::new();
    for node in cst
        .root()
        .descendants()
        .filter(|node| cst_context_kind(node.kind()).is_some() && cst_node_has_block(node))
    {
        let Some(context) = style_context_block_for_cst_node(source, node, blocks.len()) else {
            continue;
        };
        context_nodes.push((node, context.clone()));
        blocks.push(context);
    }

    let mut memberships = Vec::new();
    for rule in cst
        .root()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::Rule)
    {
        let selector_names = class_names_from_rule_node(rule);
        if selector_names.is_empty() {
            continue;
        }
        for context in cst_context_blocks_for_rule(rule, &context_nodes) {
            for selector_name in &selector_names {
                memberships.push(StyleContextSelectorMembershipV0 {
                    selector_name: selector_name.clone(),
                    context_id: context.id.clone(),
                    context_kind: context.kind,
                    source_order: memberships.len(),
                });
            }
        }
    }

    (blocks, memberships)
}

fn style_context_block_for_cst_node(
    source: &str,
    node: &SyntaxNode,
    source_order: usize,
) -> Option<StyleContextBlockV0> {
    let kind = cst_context_kind(node.kind())?;
    let prelude = cst_context_prelude(node);
    let name = match kind {
        "layer" => split_layer_names(&prelude).into_iter().next(),
        "container" => container_name_from_prelude(&prelude),
        "scope" => None,
        _ => None,
    };
    let header_end = cst_node_block_open_end(node)?;
    let byte_span = ParserByteSpanV0 {
        start: u32::from(node.text_range().start()) as usize,
        end: header_end,
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

fn cst_context_kind(kind: SyntaxKind) -> Option<&'static str> {
    match kind {
        SyntaxKind::LayerRule => Some("layer"),
        SyntaxKind::ContainerRule => Some("container"),
        SyntaxKind::ScopeRule => Some("scope"),
        _ => None,
    }
}

fn cst_context_prelude(node: &SyntaxNode) -> String {
    if node.kind() == SyntaxKind::LayerRule {
        return layer_rule_prelude(node);
    }
    let prelude_kind = match node.kind() {
        SyntaxKind::ContainerRule => SyntaxKind::ContainerCondition,
        SyntaxKind::ScopeRule => SyntaxKind::ScopeRange,
        _ => return String::new(),
    };
    node.descendants()
        .find(|child| child.kind() == prelude_kind)
        .map(|child| syntax_node_text(child).trim().to_string())
        .unwrap_or_default()
}

fn layer_rule_prelude(node: &SyntaxNode) -> String {
    let text = syntax_node_text(node);
    let Some(rest) = text.trim_start().strip_prefix("@layer") else {
        return String::new();
    };
    rest.split(['{', ';', '\n'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn cst_node_has_block(node: &SyntaxNode) -> bool {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| matches!(token.kind(), SyntaxKind::LeftBrace | SyntaxKind::SassIndent))
}

fn cst_node_block_open_end(node: &SyntaxNode) -> Option<usize> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| matches!(token.kind(), SyntaxKind::LeftBrace | SyntaxKind::SassIndent))
        .map(|token| u32::from(token.text_range().end()) as usize)
}

fn cst_context_blocks_for_rule<'a>(
    rule: &SyntaxNode,
    contexts: &'a [(&'a SyntaxNode, StyleContextBlockV0)],
) -> Vec<&'a StyleContextBlockV0> {
    contexts
        .iter()
        .filter(|(node, _)| {
            rule.text_range().start() > node.text_range().start()
                && rule.text_range().end() < node.text_range().end()
        })
        .map(|(_, context)| context)
        .collect()
}

fn class_names_from_rule_node(rule: &SyntaxNode) -> Vec<String> {
    let mut names = BTreeSet::new();
    for child in rule.children() {
        if matches!(
            child.kind(),
            SyntaxKind::DeclarationList | SyntaxKind::RuleList | SyntaxKind::SassIndentedBlock
        ) {
            break;
        }
        for class_node in child
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::ClassSelector)
        {
            if let Some(name) = class_selector_name_from_cst_node(class_node) {
                names.insert(name);
            }
        }
    }
    names.into_iter().collect()
}

fn class_selector_name_from_cst_node(node: &SyntaxNode) -> Option<String> {
    syntax_node_text(node)
        .trim()
        .strip_prefix('.')
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
}

fn syntax_node_text(node: &SyntaxNode) -> String {
    node.try_resolved()
        .map(|resolved| resolved.text().to_string())
        .unwrap_or_default()
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
    cst: &ParsedCst,
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
                let context = style_context_for_cst_offset(source, cst, byte_span.start);
                decl_facts.push(ParserIndexCustomPropertyDeclFactV0 {
                    name: variable.name.clone(),
                    value: declaration_value_text(source, byte_span.start),
                    source_order: decl_facts.len(),
                    byte_span,
                    range: parser_range_for_byte_span(source, byte_span),
                    selector_contexts: context.selector_contexts,
                    condition_context: context.condition_context,
                    layer_names: context.layer_names,
                    under_media: context.under_media,
                    under_supports: context.under_supports,
                    under_layer: context.under_layer,
                });
            }
            ParsedVariableFactKind::CustomPropertyReference => {
                let byte_offset = u32::from(variable.range.start()) as usize;
                let context = style_context_for_cst_offset(source, cst, byte_offset);
                ref_names.insert(variable.name.clone());
                ref_facts.push(ParserIndexCustomPropertyRefFactV0 {
                    name: variable.name.clone(),
                    source_order: ref_facts.len(),
                    selector_contexts: context.selector_contexts,
                    condition_context: context.condition_context,
                    layer_names: context.layer_names,
                    under_media: context.under_media,
                    under_supports: context.under_supports,
                    under_layer: context.under_layer,
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
    facts: &ParsedStyleFacts,
    resolution: &ParserIndexSassSameFileResolutionFactsV0,
    cst: &ParsedCst,
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
                let selector = semantic_selector_name_for_cst_offset(
                    cst,
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
                let selector = semantic_selector_name_for_cst_offset(
                    cst,
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
        !matches!(
            selector_context_witness_for_declaration(selector, &reference.selector_contexts)
                .verdict,
            SelectorMatchVerdict::No
        )
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

#[derive(Debug, Clone, Default)]
struct StyleOffsetContext {
    selector_contexts: Vec<String>,
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
    layer_names: Vec<String>,
    condition_context: Vec<String>,
}

fn style_context_for_cst_offset(
    source: &str,
    cst: &ParsedCst,
    byte_offset: usize,
) -> StyleOffsetContext {
    let mut context = StyleOffsetContext::default();
    for node in cst
        .root()
        .descendants()
        .filter(|node| cst_node_contains_byte_offset(node, byte_offset))
    {
        match node.kind() {
            SyntaxKind::Rule => {
                if let Some(selector) = rule_selector_text_from_cst_node(node) {
                    context.selector_contexts.push(selector);
                }
            }
            SyntaxKind::MediaRule => {
                context.under_media = true;
                if let Some(header) = cst_at_rule_header_text(source, node) {
                    context.condition_context.push(header);
                }
            }
            SyntaxKind::SupportsRule => {
                context.under_supports = true;
                if let Some(header) = cst_at_rule_header_text(source, node) {
                    context.condition_context.push(header);
                }
            }
            SyntaxKind::LayerRule => {
                if cst_node_has_block(node) {
                    context.under_layer = true;
                    context
                        .layer_names
                        .extend(split_layer_names(&cst_context_prelude(node)));
                }
            }
            kind if cst_non_layer_condition_kind(kind) => {
                if let Some(header) = cst_at_rule_header_text(source, node)
                    && !header.starts_with("@layer")
                {
                    context.condition_context.push(header);
                }
            }
            _ => {}
        }
    }
    context
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

fn semantic_selector_name_for_cst_offset(cst: &ParsedCst, byte_offset: usize) -> Option<String> {
    cst.root()
        .descendants()
        .filter(|node| {
            node.kind() == SyntaxKind::Rule && cst_node_contains_byte_offset(node, byte_offset)
        })
        .filter_map(last_class_selector_name_from_rule_node)
        .last()
}

fn cst_node_contains_byte_offset(node: &SyntaxNode, byte_offset: usize) -> bool {
    let range = node.text_range();
    let start = u32::from(range.start()) as usize;
    let end = u32::from(range.end()) as usize;
    start <= byte_offset && byte_offset < end
}

fn cst_non_layer_condition_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ContainerRule
            | SyntaxKind::ScopeRule
            | SyntaxKind::AtRule
            | SyntaxKind::KeyframesRule
            | SyntaxKind::FontFaceRule
            | SyntaxKind::PageRule
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::PageMarginRule
            | SyntaxKind::CounterStyleRule
            | SyntaxKind::FontPaletteValuesRule
            | SyntaxKind::ColorProfileRule
            | SyntaxKind::PositionTryRule
            | SyntaxKind::FontFeatureValuesRule
            | SyntaxKind::FontFeatureValuesStylisticRule
            | SyntaxKind::FontFeatureValuesStylesetRule
            | SyntaxKind::FontFeatureValuesCharacterVariantRule
            | SyntaxKind::FontFeatureValuesSwashRule
            | SyntaxKind::FontFeatureValuesOrnamentsRule
            | SyntaxKind::FontFeatureValuesAnnotationRule
            | SyntaxKind::FontFeatureValuesHistoricalFormsRule
            | SyntaxKind::ViewTransitionRule
            | SyntaxKind::WhenRule
            | SyntaxKind::ElseRule
            | SyntaxKind::IfRule
    )
}

fn cst_at_rule_header_text(source: &str, node: &SyntaxNode) -> Option<String> {
    let start = u32::from(node.text_range().start()) as usize;
    let end = cst_node_block_open_start(node)?;
    source
        .get(start..end)
        .map(normalized_condition_header)
        .filter(|header| !header.is_empty())
}

fn cst_node_block_open_start(node: &SyntaxNode) -> Option<usize> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| matches!(token.kind(), SyntaxKind::LeftBrace | SyntaxKind::SassIndent))
        .map(|token| u32::from(token.text_range().start()) as usize)
}

fn rule_selector_text_from_cst_node(node: &SyntaxNode) -> Option<String> {
    let mut selector = String::new();
    for child in node.children() {
        if matches!(
            child.kind(),
            SyntaxKind::DeclarationList | SyntaxKind::RuleList | SyntaxKind::SassIndentedBlock
        ) {
            break;
        }
        selector.push_str(&syntax_node_text(child));
    }
    let selector = selector.trim();
    (!selector.is_empty()).then(|| selector.to_string())
}

fn last_class_selector_name_from_rule_node(node: &SyntaxNode) -> Option<String> {
    let mut last = None;
    for child in node.children() {
        if matches!(
            child.kind(),
            SyntaxKind::DeclarationList | SyntaxKind::RuleList | SyntaxKind::SassIndentedBlock
        ) {
            break;
        }
        for class_node in child
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::ClassSelector)
        {
            last = class_selector_name_from_cst_node(class_node);
        }
    }
    last
}

fn normalized_condition_header(header: &str) -> String {
    header.split_whitespace().collect::<Vec<_>>().join(" ")
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
    // LSP positions count UTF-16 code units per line, not raw bytes. Walk chars
    // and accumulate `len_utf16()` so non-ASCII source produces correct columns,
    // matching the canonical helpers in omena-query/src/style.rs and
    // omena-lsp-server/src/protocol.rs (previously this used a raw byte offset,
    // which diverged on multi-byte characters).
    let clamped_offset = byte_offset.min(source.len());
    let mut line = 0usize;
    let mut character = 0usize;

    for (index, ch) in source.char_indices() {
        if index >= clamped_offset {
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
mod tests;
