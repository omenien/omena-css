//! Design-token semantic analysis for CSS custom properties.
//!
//! The module ranks declarations, records workspace-scoped candidates, and
//! exposes capability signals for cross-file design-token hover, completion,
//! diagnostics, and cascade-aware resolution.

use omena_cascade::{
    CascadeKey, CascadeLevel, LayerRank, Specificity, select_cascade_winner,
    selector_context_witness, selector_context_witness_for_declaration,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ParserBoundarySyntaxFactsV0, ParserByteSpanV0, ParserIndexCustomPropertyDeclFactV0,
    ParserIndexCustomPropertyRefFactV0, ParserRangeV0, StyleSemanticFactsV0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokenSemanticSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub resolution_scope: &'static str,
    pub declaration_count: usize,
    pub reference_count: usize,
    pub resolved_reference_count: usize,
    pub unresolved_reference_count: usize,
    pub selectors_with_references_count: usize,
    pub context_signal: DesignTokenContextSignalV0,
    pub resolution_signal: DesignTokenResolutionSignalV0,
    pub cascade_ranking_signal: DesignTokenCascadeRankingSignalV0,
    pub declaration_candidates: Vec<DesignTokenDeclarationCandidateV0>,
    pub capabilities: DesignTokenSemanticCapabilitiesV0,
    pub blocking_gaps: Vec<&'static str>,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokenContextSignalV0 {
    pub declaration_context_selector_count: usize,
    pub declaration_wrapper_context_count: usize,
    pub media_context_selector_count: usize,
    pub supports_context_selector_count: usize,
    pub layer_context_selector_count: usize,
    pub wrapper_context_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokenResolutionSignalV0 {
    pub declaration_fact_count: usize,
    pub reference_fact_count: usize,
    pub source_ordered_declaration_count: usize,
    pub source_ordered_reference_count: usize,
    pub occurrence_resolved_reference_count: usize,
    pub occurrence_unresolved_reference_count: usize,
    pub workspace_declaration_fact_count: usize,
    pub cross_file_declaration_fact_count: usize,
    pub workspace_occurrence_resolved_reference_count: usize,
    pub workspace_occurrence_unresolved_reference_count: usize,
    pub context_matched_reference_count: usize,
    pub context_unmatched_reference_count: usize,
    pub root_declaration_count: usize,
    pub selector_scoped_declaration_count: usize,
    pub wrapper_scoped_declaration_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokenCascadeRankingSignalV0 {
    pub ranked_reference_count: usize,
    pub unranked_reference_count: usize,
    pub source_order_winner_declaration_count: usize,
    pub source_order_shadowed_declaration_count: usize,
    pub repeated_name_declaration_count: usize,
    pub theme_context_winner_reference_count: usize,
    pub cross_file_candidate_declaration_count: usize,
    pub cross_file_winner_declaration_count: usize,
    pub cross_file_shadowed_declaration_count: usize,
    pub ranked_references: Vec<DesignTokenRankedReferenceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokenRankedReferenceV0 {
    pub reference_name: String,
    pub reference_source_order: usize,
    pub winner_declaration_source_order: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner_declaration_file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner_declaration_range: Option<ParserRangeV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner_import_graph_distance: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner_import_graph_order: Option<usize>,
    pub shadowed_declaration_source_orders: Vec<usize>,
    pub candidate_declaration_count: usize,
    pub winner_context_kind: &'static str,
    pub cross_file_candidate_declaration_count: usize,
    pub cross_file_shadowed_declaration_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokenSemanticCapabilitiesV0 {
    pub same_file_resolution_ready: bool,
    pub wrapper_context_signal_ready: bool,
    pub source_order_signal_ready: bool,
    pub source_order_cascade_ranking_ready: bool,
    pub workspace_cascade_candidate_signal_ready: bool,
    pub occurrence_resolution_signal_ready: bool,
    pub selector_context_resolution_ready: bool,
    pub theme_override_context_signal_ready: bool,
    pub cross_file_import_graph_ready: bool,
    pub cross_package_cascade_ranking_ready: bool,
    pub theme_override_context_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesignTokenWorkspaceDeclarationFactV0 {
    pub file_path: String,
    pub name: String,
    pub value: String,
    pub source_order: usize,
    pub import_graph_distance: Option<usize>,
    pub import_graph_order: Option<usize>,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
    pub selector_contexts: Vec<String>,
    pub under_media: bool,
    pub under_supports: bool,
    pub under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignTokenDeclarationCandidateV0 {
    pub name: String,
    pub value: String,
    pub source_order: usize,
    pub file_path: String,
    pub range: ParserRangeV0,
    pub selector_contexts: Vec<String>,
    pub under_media: bool,
    pub under_supports: bool,
    pub under_layer: bool,
    pub candidate_scope: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_graph_distance: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_graph_order: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignTokenExternalDeclarationCandidateScopeV0 {
    Workspace,
    CrossFileImportGraph,
}

pub fn summarize_design_token_semantics(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    semantic_facts: &StyleSemanticFactsV0,
) -> DesignTokenSemanticSummaryV0 {
    summarize_design_token_semantics_with_workspace_declarations(
        parser_facts,
        semantic_facts,
        None,
        &[],
    )
}

pub fn summarize_design_token_semantics_with_workspace_declarations(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    semantic_facts: &StyleSemanticFactsV0,
    target_style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
) -> DesignTokenSemanticSummaryV0 {
    summarize_design_token_semantics_with_scoped_workspace_declarations(
        parser_facts,
        semantic_facts,
        target_style_path,
        workspace_declarations,
        DesignTokenExternalDeclarationCandidateScopeV0::Workspace,
    )
}

pub fn summarize_design_token_semantics_with_scoped_workspace_declarations(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    semantic_facts: &StyleSemanticFactsV0,
    target_style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
    candidate_scope: DesignTokenExternalDeclarationCandidateScopeV0,
) -> DesignTokenSemanticSummaryV0 {
    let media_context_selector_count = parser_facts
        .custom_properties
        .selectors_with_refs_under_media_names
        .len();
    let supports_context_selector_count = parser_facts
        .custom_properties
        .selectors_with_refs_under_supports_names
        .len();
    let layer_context_selector_count = parser_facts
        .custom_properties
        .selectors_with_refs_under_layer_names
        .len();
    let declaration_wrapper_context_count =
        parser_facts.custom_properties.decl_names_under_media.len()
            + parser_facts
                .custom_properties
                .decl_names_under_supports
                .len()
            + parser_facts.custom_properties.decl_names_under_layer.len();
    let wrapper_context_count = media_context_selector_count
        + supports_context_selector_count
        + layer_context_selector_count;
    let declaration_context_selector_count =
        parser_facts.custom_properties.decl_context_selectors.len();
    let reference_count = semantic_facts.custom_properties.ref_names.len();
    let declaration_count = semantic_facts.custom_properties.decl_names.len();
    let resolution_signal = summarize_design_token_resolution_signal(
        parser_facts,
        target_style_path,
        workspace_declarations,
    );
    let cascade_ranking_signal = summarize_design_token_cascade_ranking_signal(
        parser_facts,
        target_style_path,
        workspace_declarations,
    );

    let external_candidate_scope_ready = candidate_scope.cross_file_import_graph_ready();
    let status = if reference_count == 0 && declaration_count == 0 {
        "empty"
    } else if cascade_ranking_signal.has_workspace_signal() && external_candidate_scope_ready {
        "cross-file-import-cascade-ranking-seed"
    } else if cascade_ranking_signal.has_workspace_signal() {
        "workspace-cascade-ranking-seed"
    } else if cascade_ranking_signal.has_shadowing_signal() {
        "same-file-cascade-ranking-seed"
    } else if resolution_signal.occurrence_resolution_ready() {
        "context-aware-resolution-seed"
    } else if wrapper_context_count > 0 {
        "context-aware-seed"
    } else {
        "same-file-seed"
    };

    let mut blocking_gaps = Vec::new();
    if reference_count > 0 || declaration_count > 0 {
        if !external_candidate_scope_ready {
            blocking_gaps.push("crossFileImportGraph");
        }
        blocking_gaps.push("crossPackageCascadeRanking");
        if !cascade_ranking_signal.theme_override_context_ready() {
            blocking_gaps.push("themeOverrideContext");
        }
    }
    if !semantic_facts
        .custom_properties
        .unresolved_ref_names
        .is_empty()
    {
        blocking_gaps.push("unresolvedDesignTokenRefs");
    }

    let next_priorities = if reference_count == 0 && declaration_count == 0 {
        vec!["designTokenSeed"]
    } else {
        let mut priorities = Vec::new();
        if !external_candidate_scope_ready {
            priorities.push("crossFileImportGraph");
        }
        priorities.push("crossPackageCascadeRanking");
        if !cascade_ranking_signal.theme_override_context_ready() {
            priorities.push("themeOverrideContext");
        }
        priorities
    };
    let resolution_scope = if cascade_ranking_signal.has_workspace_signal() {
        candidate_scope.resolution_scope()
    } else {
        "same-file"
    };
    let declaration_candidates = summarize_design_token_declaration_candidates(
        parser_facts,
        target_style_path,
        workspace_declarations,
        candidate_scope,
    );

    DesignTokenSemanticSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.design-token-semantics",
        status,
        resolution_scope,
        declaration_count,
        reference_count,
        resolved_reference_count: semantic_facts.custom_properties.resolved_ref_names.len(),
        unresolved_reference_count: semantic_facts.custom_properties.unresolved_ref_names.len(),
        selectors_with_references_count: semantic_facts
            .custom_properties
            .selectors_with_refs_names
            .len(),
        context_signal: DesignTokenContextSignalV0 {
            declaration_context_selector_count,
            declaration_wrapper_context_count,
            media_context_selector_count,
            supports_context_selector_count,
            layer_context_selector_count,
            wrapper_context_count,
        },
        resolution_signal: resolution_signal.clone(),
        cascade_ranking_signal: cascade_ranking_signal.clone(),
        declaration_candidates,
        capabilities: DesignTokenSemanticCapabilitiesV0 {
            same_file_resolution_ready: declaration_count > 0 || reference_count > 0,
            wrapper_context_signal_ready: wrapper_context_count > 0,
            source_order_signal_ready: resolution_signal.source_order_signal_ready(),
            source_order_cascade_ranking_ready: cascade_ranking_signal
                .source_order_cascade_ranking_ready(),
            workspace_cascade_candidate_signal_ready: cascade_ranking_signal.has_workspace_signal(),
            occurrence_resolution_signal_ready: resolution_signal.occurrence_resolution_ready(),
            selector_context_resolution_ready: resolution_signal
                .selector_context_resolution_ready(),
            theme_override_context_signal_ready: declaration_context_selector_count > 0
                || declaration_wrapper_context_count > 0,
            cross_file_import_graph_ready: external_candidate_scope_ready,
            cross_package_cascade_ranking_ready: false,
            theme_override_context_ready: cascade_ranking_signal.theme_override_context_ready(),
        },
        blocking_gaps,
        next_priorities,
    }
}

fn summarize_design_token_declaration_candidates(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    target_style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
    candidate_scope: DesignTokenExternalDeclarationCandidateScopeV0,
) -> Vec<DesignTokenDeclarationCandidateV0> {
    let mut candidates = Vec::new();
    if let Some(file_path) = target_style_path {
        candidates.extend(
            parser_facts
                .custom_properties
                .decl_facts
                .iter()
                .map(|declaration| DesignTokenDeclarationCandidateV0 {
                    name: declaration.name.clone(),
                    value: declaration.value.clone(),
                    source_order: declaration.source_order,
                    file_path: file_path.to_string(),
                    range: declaration.range,
                    selector_contexts: declaration.selector_contexts.clone(),
                    under_media: declaration.under_media,
                    under_supports: declaration.under_supports,
                    under_layer: declaration.under_layer,
                    candidate_scope: "same-file",
                    import_graph_distance: None,
                    import_graph_order: None,
                }),
        );
    }
    candidates.extend(workspace_declarations.iter().map(|declaration| {
        DesignTokenDeclarationCandidateV0 {
            name: declaration.name.clone(),
            value: declaration.value.clone(),
            source_order: declaration.source_order,
            file_path: declaration.file_path.clone(),
            range: declaration.range,
            selector_contexts: declaration.selector_contexts.clone(),
            under_media: declaration.under_media,
            under_supports: declaration.under_supports,
            under_layer: declaration.under_layer,
            candidate_scope: candidate_scope.resolution_scope(),
            import_graph_distance: declaration.import_graph_distance,
            import_graph_order: declaration.import_graph_order,
        }
    }));
    candidates.sort_by(|left, right| {
        left.file_path
            .cmp(&right.file_path)
            .then_with(|| left.source_order.cmp(&right.source_order))
            .then_with(|| left.name.cmp(&right.name))
    });
    candidates.dedup_by(|left, right| {
        left.file_path == right.file_path
            && left.source_order == right.source_order
            && left.name == right.name
            && left.range == right.range
    });
    candidates
}

pub fn collect_design_token_workspace_declarations(
    style_path: &str,
    parser_facts: &ParserBoundarySyntaxFactsV0,
) -> Vec<DesignTokenWorkspaceDeclarationFactV0> {
    parser_facts
        .custom_properties
        .decl_facts
        .iter()
        .map(|declaration| DesignTokenWorkspaceDeclarationFactV0 {
            file_path: style_path.to_string(),
            name: declaration.name.clone(),
            value: declaration.value.clone(),
            source_order: declaration.source_order,
            import_graph_distance: None,
            import_graph_order: None,
            byte_span: declaration.byte_span,
            range: declaration.range,
            selector_contexts: declaration.selector_contexts.clone(),
            under_media: declaration.under_media,
            under_supports: declaration.under_supports,
            under_layer: declaration.under_layer,
        })
        .collect()
}

fn summarize_design_token_cascade_ranking_signal(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    target_style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
) -> DesignTokenCascadeRankingSignalV0 {
    let custom_properties = &parser_facts.custom_properties;
    let mut declaration_name_counts = BTreeMap::<&str, usize>::new();
    let mut winner_declarations = BTreeSet::<(String, usize)>::new();
    let mut shadowed_declarations = BTreeSet::<(String, usize)>::new();
    let mut ranked_reference_count = 0;
    let mut unranked_reference_count = 0;
    let mut cross_file_candidate_declaration_count = 0;
    let mut cross_file_winner_declaration_count = 0;
    let mut cross_file_shadowed_declaration_count = 0;
    let mut theme_context_winner_reference_count = 0;
    let mut ranked_references = Vec::new();

    for declaration in &custom_properties.decl_facts {
        *declaration_name_counts
            .entry(declaration.name.as_str())
            .or_insert(0) += 1;
    }

    for reference in &custom_properties.ref_facts {
        let local_candidates = custom_properties
            .decl_facts
            .iter()
            .filter(|declaration| custom_property_context_matches(declaration, reference))
            .collect::<Vec<_>>();
        let workspace_candidates = workspace_declarations
            .iter()
            .filter(|declaration| {
                target_style_path.is_none_or(|target| declaration.file_path != target)
                    && custom_property_workspace_context_matches(declaration, reference)
            })
            .collect::<Vec<_>>();

        let local_winner = select_cascade_winner(
            local_candidates
                .iter()
                .copied()
                .map(DesignTokenCandidateDeclaration::Local),
            |candidate| candidate.cascade_key(reference, None),
        )
        .map(|(winner, _)| winner);
        let workspace_file_ranks = summarize_workspace_candidate_file_ranks(&workspace_candidates);
        let workspace_winner = select_cascade_winner(
            workspace_candidates
                .iter()
                .copied()
                .map(DesignTokenCandidateDeclaration::Workspace),
            |candidate| candidate.cascade_key(reference, Some(&workspace_file_ranks)),
        )
        .map(|(winner, _)| winner);
        let winner = local_winner.or(workspace_winner);

        let Some(winner) = winner else {
            unranked_reference_count += 1;
            continue;
        };

        ranked_reference_count += 1;
        let candidate_declaration_count = local_candidates.len() + workspace_candidates.len();
        let reference_cross_file_candidate_declaration_count = workspace_candidates.len();
        cross_file_candidate_declaration_count += reference_cross_file_candidate_declaration_count;
        let mut shadowed_declaration_source_orders = Vec::new();
        for candidate in local_candidates {
            if winner.is_local_source_order(candidate.source_order) {
                winner_declarations.insert(custom_property_declaration_key(candidate));
            } else {
                shadowed_declaration_source_orders.push(candidate.source_order);
                shadowed_declarations.insert(custom_property_declaration_key(candidate));
            }
        }
        let reference_cross_file_shadowed_declaration_count = workspace_candidates
            .iter()
            .filter(|candidate| !winner.is_workspace(candidate))
            .count();
        cross_file_shadowed_declaration_count += reference_cross_file_shadowed_declaration_count;
        if winner.is_workspace_winner() {
            cross_file_winner_declaration_count += 1;
        }
        if winner.is_theme_context_winner(reference) {
            theme_context_winner_reference_count += 1;
        }
        shadowed_declaration_source_orders.sort_unstable();
        ranked_references.push(DesignTokenRankedReferenceV0 {
            reference_name: reference.name.clone(),
            reference_source_order: reference.source_order,
            winner_declaration_source_order: winner.source_order(),
            winner_declaration_file_path: winner.file_path().map(ToString::to_string),
            winner_declaration_range: winner.range(),
            winner_import_graph_distance: winner.import_graph_distance(),
            winner_import_graph_order: winner.import_graph_order(),
            shadowed_declaration_source_orders,
            candidate_declaration_count,
            winner_context_kind: winner.context_kind(reference),
            cross_file_candidate_declaration_count:
                reference_cross_file_candidate_declaration_count,
            cross_file_shadowed_declaration_count: reference_cross_file_shadowed_declaration_count,
        });
    }

    DesignTokenCascadeRankingSignalV0 {
        ranked_reference_count,
        unranked_reference_count,
        source_order_winner_declaration_count: winner_declarations.len(),
        source_order_shadowed_declaration_count: shadowed_declarations.len(),
        repeated_name_declaration_count: custom_properties
            .decl_facts
            .iter()
            .filter(|declaration| {
                declaration_name_counts
                    .get(declaration.name.as_str())
                    .is_some_and(|count| *count > 1)
            })
            .count(),
        theme_context_winner_reference_count,
        cross_file_candidate_declaration_count,
        cross_file_winner_declaration_count,
        cross_file_shadowed_declaration_count,
        ranked_references,
    }
}

fn summarize_design_token_resolution_signal(
    parser_facts: &ParserBoundarySyntaxFactsV0,
    target_style_path: Option<&str>,
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
) -> DesignTokenResolutionSignalV0 {
    let custom_properties = &parser_facts.custom_properties;
    let mut occurrence_resolved_reference_count = 0;
    let mut occurrence_unresolved_reference_count = 0;
    let mut workspace_occurrence_resolved_reference_count = 0;
    let mut workspace_occurrence_unresolved_reference_count = 0;
    let cross_file_declaration_fact_count = workspace_declarations
        .iter()
        .filter(|declaration| {
            target_style_path.is_none_or(|target| declaration.file_path != target)
        })
        .count();

    for reference in &custom_properties.ref_facts {
        let has_same_file_match = custom_properties
            .decl_facts
            .iter()
            .any(|declaration| custom_property_context_matches(declaration, reference));
        let has_workspace_match = has_same_file_match
            || workspace_declarations.iter().any(|declaration| {
                target_style_path.is_none_or(|target| declaration.file_path != target)
                    && custom_property_workspace_context_matches(declaration, reference)
            });

        if has_same_file_match {
            occurrence_resolved_reference_count += 1;
        } else {
            occurrence_unresolved_reference_count += 1;
        }
        if has_workspace_match {
            workspace_occurrence_resolved_reference_count += 1;
        } else {
            workspace_occurrence_unresolved_reference_count += 1;
        }
    }

    DesignTokenResolutionSignalV0 {
        declaration_fact_count: custom_properties.decl_facts.len(),
        reference_fact_count: custom_properties.ref_facts.len(),
        source_ordered_declaration_count: custom_properties.decl_facts.len(),
        source_ordered_reference_count: custom_properties.ref_facts.len(),
        occurrence_resolved_reference_count,
        occurrence_unresolved_reference_count,
        workspace_declaration_fact_count: custom_properties.decl_facts.len()
            + cross_file_declaration_fact_count,
        cross_file_declaration_fact_count,
        workspace_occurrence_resolved_reference_count,
        workspace_occurrence_unresolved_reference_count,
        context_matched_reference_count: occurrence_resolved_reference_count,
        context_unmatched_reference_count: occurrence_unresolved_reference_count,
        root_declaration_count: custom_properties
            .decl_facts
            .iter()
            .filter(|declaration| {
                declaration
                    .selector_contexts
                    .iter()
                    .any(|selector| selector == ":root")
            })
            .count(),
        selector_scoped_declaration_count: custom_properties
            .decl_facts
            .iter()
            .filter(|declaration| {
                declaration
                    .selector_contexts
                    .iter()
                    .any(|selector| selector != ":root")
            })
            .count(),
        wrapper_scoped_declaration_count: custom_properties
            .decl_facts
            .iter()
            .filter(|declaration| {
                declaration.under_media || declaration.under_supports || declaration.under_layer
            })
            .count(),
    }
}

impl DesignTokenResolutionSignalV0 {
    fn occurrence_resolution_ready(&self) -> bool {
        self.declaration_fact_count > 0 || self.reference_fact_count > 0
    }

    fn source_order_signal_ready(&self) -> bool {
        self.source_ordered_declaration_count > 0 || self.source_ordered_reference_count > 0
    }

    fn selector_context_resolution_ready(&self) -> bool {
        self.occurrence_resolution_ready()
            && (self.root_declaration_count > 0 || self.selector_scoped_declaration_count > 0)
    }
}

impl DesignTokenCascadeRankingSignalV0 {
    fn source_order_cascade_ranking_ready(&self) -> bool {
        self.ranked_reference_count > 0
    }

    fn has_shadowing_signal(&self) -> bool {
        self.source_order_shadowed_declaration_count > 0
    }

    fn has_workspace_signal(&self) -> bool {
        self.cross_file_candidate_declaration_count > 0
    }

    fn theme_override_context_ready(&self) -> bool {
        self.theme_context_winner_reference_count > 0
    }
}

impl DesignTokenExternalDeclarationCandidateScopeV0 {
    fn cross_file_import_graph_ready(self) -> bool {
        matches!(
            self,
            DesignTokenExternalDeclarationCandidateScopeV0::CrossFileImportGraph
        )
    }

    fn resolution_scope(self) -> &'static str {
        match self {
            DesignTokenExternalDeclarationCandidateScopeV0::Workspace => "workspace-candidate",
            DesignTokenExternalDeclarationCandidateScopeV0::CrossFileImportGraph => {
                "cross-file-import-candidate"
            }
        }
    }
}

#[derive(Clone, Copy)]
enum DesignTokenCandidateDeclaration<'a> {
    Local(&'a ParserIndexCustomPropertyDeclFactV0),
    Workspace(&'a DesignTokenWorkspaceDeclarationFactV0),
}

impl DesignTokenCandidateDeclaration<'_> {
    fn cascade_key(
        &self,
        reference: &ParserIndexCustomPropertyRefFactV0,
        workspace_file_ranks: Option<&BTreeMap<&str, usize>>,
    ) -> CascadeKey {
        let scope_proximity =
            cascade_scope_proximity_for_context_rank(self.context_rank(reference));
        match self {
            DesignTokenCandidateDeclaration::Local(declaration) => CascadeKey::new(
                CascadeLevel::AuthorNormal,
                LayerRank(0),
                scope_proximity,
                Specificity::ZERO,
                cascade_u32_rank(declaration.source_order),
            ),
            DesignTokenCandidateDeclaration::Workspace(declaration) => {
                let file_rank = workspace_file_ranks
                    .and_then(|ranks| ranks.get(declaration.file_path.as_str()).copied())
                    .unwrap_or(usize::MAX);
                CascadeKey::new(
                    CascadeLevel::AuthorNormal,
                    LayerRank(0),
                    scope_proximity,
                    // Import graph tie-breakers are encoded into specificity slots
                    // until selector-match witnesses provide real CSS specificity.
                    Specificity::new(
                        cascade_inverse_rank(
                            declaration.import_graph_distance.unwrap_or(usize::MAX),
                        ),
                        cascade_inverse_rank(declaration.import_graph_order.unwrap_or(usize::MAX)),
                        cascade_inverse_rank(file_rank),
                    ),
                    cascade_u32_rank(declaration.source_order),
                )
            }
        }
    }

    fn source_order(&self) -> usize {
        match self {
            DesignTokenCandidateDeclaration::Local(declaration) => declaration.source_order,
            DesignTokenCandidateDeclaration::Workspace(declaration) => declaration.source_order,
        }
    }

    fn file_path(&self) -> Option<&str> {
        match self {
            DesignTokenCandidateDeclaration::Local(_) => None,
            DesignTokenCandidateDeclaration::Workspace(declaration) => {
                Some(declaration.file_path.as_str())
            }
        }
    }

    fn range(&self) -> Option<ParserRangeV0> {
        match self {
            DesignTokenCandidateDeclaration::Local(_) => None,
            DesignTokenCandidateDeclaration::Workspace(declaration) => Some(declaration.range),
        }
    }

    fn import_graph_distance(&self) -> Option<usize> {
        match self {
            DesignTokenCandidateDeclaration::Local(_) => None,
            DesignTokenCandidateDeclaration::Workspace(declaration) => {
                declaration.import_graph_distance
            }
        }
    }

    fn import_graph_order(&self) -> Option<usize> {
        match self {
            DesignTokenCandidateDeclaration::Local(_) => None,
            DesignTokenCandidateDeclaration::Workspace(declaration) => {
                declaration.import_graph_order
            }
        }
    }

    fn is_local_source_order(&self, source_order: usize) -> bool {
        matches!(
            self,
            DesignTokenCandidateDeclaration::Local(declaration)
                if declaration.source_order == source_order
        )
    }

    fn is_workspace(&self, declaration: &DesignTokenWorkspaceDeclarationFactV0) -> bool {
        matches!(
            self,
            DesignTokenCandidateDeclaration::Workspace(winner)
                if winner.file_path == declaration.file_path
                    && winner.source_order == declaration.source_order
                    && winner.name == declaration.name
        )
    }

    fn is_workspace_winner(&self) -> bool {
        matches!(self, DesignTokenCandidateDeclaration::Workspace(_))
    }

    fn is_theme_context_winner(&self, reference: &ParserIndexCustomPropertyRefFactV0) -> bool {
        self.context_rank(reference) >= 2
    }

    fn context_rank(&self, reference: &ParserIndexCustomPropertyRefFactV0) -> usize {
        match self {
            DesignTokenCandidateDeclaration::Local(declaration) => {
                custom_property_declaration_context_rank(&declaration.selector_contexts, reference)
            }
            DesignTokenCandidateDeclaration::Workspace(declaration) => {
                custom_property_declaration_context_rank(&declaration.selector_contexts, reference)
            }
        }
    }

    fn context_kind(&self, reference: &ParserIndexCustomPropertyRefFactV0) -> &'static str {
        match self.context_rank(reference) {
            2.. => "selector",
            1 => "root",
            _ => "global",
        }
    }
}

fn custom_property_declaration_key(
    declaration: &ParserIndexCustomPropertyDeclFactV0,
) -> (String, usize) {
    (declaration.name.clone(), declaration.source_order)
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
    declaration
        .selector_contexts
        .iter()
        .any(|selector| custom_property_selector_context_matches(selector, reference))
}

fn custom_property_workspace_context_matches(
    declaration: &DesignTokenWorkspaceDeclarationFactV0,
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
    declaration
        .selector_contexts
        .iter()
        .any(|selector| custom_property_selector_context_matches(selector, reference))
}

fn custom_property_selector_context_matches(
    declaration_selector: &str,
    reference: &ParserIndexCustomPropertyRefFactV0,
) -> bool {
    selector_context_witness_for_declaration(declaration_selector, &reference.selector_contexts)
        .matched
}

fn custom_property_declaration_context_rank(
    declaration_selectors: &[String],
    reference: &ParserIndexCustomPropertyRefFactV0,
) -> usize {
    selector_context_witness(declaration_selectors, &reference.selector_contexts).rank
}

fn summarize_workspace_candidate_file_ranks<'a>(
    workspace_candidates: &[&'a DesignTokenWorkspaceDeclarationFactV0],
) -> BTreeMap<&'a str, usize> {
    workspace_candidates
        .iter()
        .map(|candidate| candidate.file_path.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(rank, file_path)| (file_path, rank))
        .collect()
}

fn cascade_scope_proximity_for_context_rank(context_rank: usize) -> u32 {
    match context_rank {
        2.. => 0,
        1 => 1,
        _ => 2,
    }
}

fn cascade_u32_rank(rank: usize) -> u32 {
    rank.min(u32::MAX as usize) as u32
}

fn cascade_inverse_rank(rank: usize) -> u32 {
    u32::MAX - cascade_u32_rank(rank)
}
