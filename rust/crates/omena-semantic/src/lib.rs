use engine_input_producers::EngineInputV2;
use engine_style_parser::{
    NestedSafetyCountsV0, ParserBoundarySyntaxFactsV0, ParserIndexComposesFactsV0,
    ParserIndexCustomPropertyDeclFactV0, ParserIndexCustomPropertyFactsV0,
    ParserIndexCustomPropertyRefFactV0, ParserIndexKeyframesFactsV0,
    ParserIndexSassModuleUseFactV0, ParserIndexSassSameFileResolutionFactsV0,
    ParserIndexSelectorDefinitionFactV0, ParserIndexSelectorFactsV0, ParserIndexValueFactsV0,
    ParserIndexWrapperFactsV0, ParserLosslessCstFactsV0, ParserSassSyntaxFactsV0,
    StyleCustomPropertySemanticFactsV0, StyleSassSemanticFactsV0, StyleSelectorIdentityFactsV0,
    StyleSemanticFactsV0, Stylesheet, summarize_semantic_boundary,
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

pub fn summarize_style_semantic_boundary(sheet: &Stylesheet) -> StyleSemanticBoundarySummaryV0 {
    let boundary = summarize_semantic_boundary(sheet);
    let parser_facts = boundary.parser_facts;
    let semantic_facts = boundary.semantic_facts;
    let design_token_semantics = summarize_design_token_semantics(&parser_facts, &semantic_facts);
    let selector_identity_engine =
        summarize_selector_identity_engine(&semantic_facts.selector_identity);
    let promotion_evidence = summarize_semantic_promotion_evidence(&parser_facts, &semantic_facts);
    let lossless_cst_contract = summarize_lossless_cst_contract(&parser_facts.lossless_cst);

    StyleSemanticBoundarySummaryV0 {
        schema_version: "0",
        language: boundary.language,
        parser_facts,
        semantic_facts,
        design_token_semantics,
        selector_identity_engine,
        promotion_evidence,
        lossless_cst_contract,
    }
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
    let boundary = summarize_semantic_boundary(sheet);
    let parser_facts = boundary.parser_facts;
    let semantic_facts = boundary.semantic_facts;
    let design_token_semantics = summarize_design_token_semantics_with_workspace_declarations(
        &parser_facts,
        &semantic_facts,
        style_path,
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

pub fn summarize_parser_contract_facts(sheet: &Stylesheet) -> ParserBoundarySyntaxFactsV0 {
    summarize_style_semantic_boundary(sheet).parser_facts
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
    let semantic_facts = summarize_omena_parser_semantic_facts(&parser_facts);
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
    parser_facts: &ParserBoundarySyntaxFactsV0,
) -> StyleSemanticFactsV0 {
    let custom_properties =
        summarize_omena_parser_custom_property_semantic_facts(&parser_facts.custom_properties);
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
            selectors_with_resolved_variable_refs_names: Vec::new(),
            selectors_with_unresolved_variable_refs_names: Vec::new(),
            selectors_with_resolved_mixin_includes_names: Vec::new(),
            selectors_with_unresolved_mixin_includes_names: Vec::new(),
            selectors_with_function_calls_names: parser_facts.sass.function_call_names.clone(),
            same_file_resolution: ParserIndexSassSameFileResolutionFactsV0::default(),
        },
    }
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
                decl_facts.push(ParserIndexCustomPropertyDeclFactV0 {
                    name: variable.name.clone(),
                    source_order: decl_facts.len(),
                    byte_span,
                    range: parser_range_for_byte_span(source, byte_span),
                    selector_contexts: Vec::new(),
                    under_media: false,
                    under_supports: false,
                    under_layer: false,
                });
            }
            ParsedVariableFactKind::CustomPropertyReference => {
                ref_names.insert(variable.name.clone());
                ref_facts.push(ParserIndexCustomPropertyRefFactV0 {
                    name: variable.name.clone(),
                    source_order: ref_facts.len(),
                    selector_contexts: Vec::new(),
                    under_media: false,
                    under_supports: false,
                    under_layer: false,
                });
            }
            _ => {}
        }
    }
    ParserIndexCustomPropertyFactsV0 {
        decl_names: decl_names.into_iter().collect(),
        decl_facts,
        ref_names: ref_names.into_iter().collect(),
        ref_facts,
        ..ParserIndexCustomPropertyFactsV0::default()
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
    let decl_names = facts.decl_names.iter().cloned().collect::<BTreeSet<_>>();
    let mut resolved_ref_names = BTreeSet::new();
    let mut unresolved_ref_names = BTreeSet::new();
    for name in &facts.ref_names {
        if decl_names.contains(name) {
            resolved_ref_names.insert(name.clone());
        } else {
            unresolved_ref_names.insert(name.clone());
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

fn previous_non_whitespace_byte_index(bytes: &[u8], before: usize) -> Option<usize> {
    let mut index = before.checked_sub(1)?;
    loop {
        if !bytes[index].is_ascii_whitespace() {
            return Some(index);
        }
        index = index.checked_sub(1)?;
    }
}

fn parser_byte_span_for_offsets(start: usize, end: usize) -> engine_style_parser::ParserByteSpanV0 {
    engine_style_parser::ParserByteSpanV0 { start, end }
}

fn parser_range_for_byte_span(
    source: &str,
    span: engine_style_parser::ParserByteSpanV0,
) -> engine_style_parser::ParserRangeV0 {
    engine_style_parser::ParserRangeV0 {
        start: parser_position_for_byte_offset(source, span.start),
        end: parser_position_for_byte_offset(source, span.end),
    }
}

fn parser_position_for_byte_offset(
    source: &str,
    byte_offset: usize,
) -> engine_style_parser::ParserPositionV0 {
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
    engine_style_parser::ParserPositionV0 {
        line,
        character: offset.saturating_sub(line_start),
    }
}

fn omena_parser_dialect_for_style_path(style_path: &str) -> StyleDialect {
    if style_path.ends_with(".sass") {
        StyleDialect::Sass
    } else if style_path.ends_with(".scss") {
        StyleDialect::Scss
    } else if style_path.ends_with(".less") {
        StyleDialect::Less
    } else {
        StyleDialect::Css
    }
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
    use engine_input_producers::{
        ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
        SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2,
        StyleSelectorV2, TypeFactEntryV2,
    };
    use engine_style_parser::parse_style_module;

    use super::{
        TheoryObservationHarnessInput, summarize_lossless_cst_contract,
        summarize_omena_parser_style_semantic_boundary_from_source,
        summarize_parser_contract_facts, summarize_selector_identity_engine,
        summarize_semantic_promotion_evidence,
        summarize_semantic_promotion_evidence_with_source_input, summarize_source_input_evidence,
        summarize_style_semantic_boundary, summarize_style_semantic_facts,
        summarize_style_semantic_graph, summarize_style_semantic_graph_from_source,
        summarize_theory_observation_contract, summarize_theory_observation_harness,
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
