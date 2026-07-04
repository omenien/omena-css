use super::*;
use omena_parser::{ParsedSassIncludeFact, ParsedSelectorFact, ParsedVariableFact};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

mod cascade_position;
mod code_actions;
mod completion;
mod cross_file_hypergraph;
mod cross_file_summary;
mod diagnostic_suppressions;
mod diagnostics;
mod dynamic_classname;
mod insights;
mod parser_facade;
#[cfg(feature = "salsa-memo")]
mod salsa_memo;
mod sass;
mod source_refs;
mod stylesheet_evaluation;
mod substrate;
mod transform;

#[cfg(test)]
pub(crate) use cascade_checker::cascade_declarations_collect_probe;
pub use cascade_position::*;
pub use code_actions::*;
pub use completion::*;
#[cfg(feature = "hypergraph-ifds")]
pub use cross_file_hypergraph::*;
use cross_file_summary::summarize_omena_query_cross_file_summary;
#[cfg(any(test, feature = "test-support"))]
pub use cross_file_summary::{
    read_workspace_cross_file_summary_direct_recompute_count_for_test,
    reset_workspace_cross_file_summary_direct_recompute_count_for_test,
};
pub use cross_file_summary::{
    summarize_omena_query_categorical_design_system_cross_project_summary,
    summarize_omena_query_m4_axis_c_readiness,
    summarize_omena_query_source_selector_reference_cross_file_summary,
    summarize_omena_query_workspace_cross_file_summary,
};
#[cfg(test)]
pub(crate) use diagnostics::collect_omena_query_visible_sass_symbol_keys_for_workspace_file;
pub use diagnostics::*;
pub use dynamic_classname::*;
pub use insights::*;
#[cfg(test)]
pub(crate) use parser_facade::style_facts_collect_probe;
pub use parser_facade::{
    OmenaQueryStyleFrameRefreshFactsV0, OmenaQueryStyleFrameRefreshParseCacheV0,
    summarize_omena_query_omena_parser_css_modules_intermediate,
    summarize_omena_query_omena_parser_lex, summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_style_document,
    summarize_omena_query_style_frame_refresh_facts_with_reuse,
};
use parser_facade::{
    collect_omena_query_omena_parser_style_facts_raw, omena_parser_dialect_for_style_path,
    omena_parser_style_dialect_label, omena_query_sass_symbol_fact_kind_is_declaration,
    omena_query_sass_symbol_fact_kind_is_reference,
};
#[cfg(feature = "salsa-memo")]
pub use salsa_memo::*;
pub use sass::*;
pub use source_refs::*;
pub use substrate::*;
pub use transform::*;

mod cascade_checker;

pub fn summarize_omena_query_style_semantic_graph_from_source(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<StyleSemanticGraphSummaryV0> {
    summarize_omena_bridge_style_semantic_graph_from_source(style_path, style_source, input)
}

pub fn read_omena_query_style_context_index(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<OmenaQueryStyleContextIndexV0> {
    let graph =
        summarize_omena_query_style_semantic_graph_from_source(style_path, style_source, input)?;
    Some(OmenaQueryStyleContextIndexV0 {
        schema_version: "0",
        product: "omena-query.style-context-index",
        style_path: style_path.to_string(),
        language: graph.language,
        context_index_source: graph.semantic_facts.context_index.product,
        context_index: graph.semantic_facts.context_index,
    })
}

pub fn summarize_omena_query_style_hover_candidates(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQueryStyleHoverCandidatesV0> {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    collect_style_selector_hover_candidates_from_omena_parser_facts(
        style_source,
        facts.selectors.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_custom_property_hover_candidates_from_omena_parser_facts(
        style_source,
        facts.variables.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_symbol_hover_candidates_from_omena_parser_facts(
        style_source,
        facts.sass_symbols.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts(
        style_source,
        facts.sass_includes.as_slice(),
        &mut seen,
        &mut candidates,
    );
    candidates.sort();
    Some(OmenaQueryStyleHoverCandidatesV0 {
        schema_version: "0",
        product: "omena-query.style-hover-candidates",
        language: omena_parser_style_dialect_label(dialect),
        candidates,
    })
}

pub fn summarize_omena_query_style_hover_render_parts(
    source: &str,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> OmenaQueryStyleHoverRenderPartsV0 {
    summarize_omena_query_style_hover_render_parts_with_branch_scope(
        source, kind, name, position, None, None,
    )
}

pub fn summarize_omena_query_style_hover_render_parts_for_hover_position(
    source: &str,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> OmenaQueryStyleHoverRenderPartsV0 {
    let branch_scope = (kind == "selector")
        .then(|| selector_hover_branch_scope_at_position(source, name, position))
        .flatten();
    summarize_omena_query_style_hover_render_parts_with_branch_scope(
        source,
        kind,
        name,
        position,
        branch_scope,
        None,
    )
}

fn summarize_omena_query_style_hover_render_parts_with_branch_scope(
    source: &str,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
    selector_branch_scope: Option<HoverCascadeBranchScope>,
    precollected_target_declarations: Option<&[cascade_checker::QueryCheckerCascadeDeclaration]>,
) -> OmenaQueryStyleHoverRenderPartsV0 {
    let mut parts = OmenaQueryStyleHoverRenderPartsV0 {
        schema_version: "0",
        product: "omena-query.style-hover-render-parts",
        snippet: String::new(),
        value: None,
        signature: None,
        property_value_narrowings: Vec::new(),
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
            parts.property_value_narrowings = match precollected_target_declarations {
                Some(declarations) => selector_property_value_narrowings_from_declarations(
                    declarations,
                    name,
                    selector_branch_scope.as_ref(),
                ),
                None => selector_property_value_narrowings_for_hover(
                    source,
                    name,
                    selector_branch_scope.as_ref(),
                ),
            };
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

pub fn summarize_omena_query_style_hover_render_parts_for_workspace_file(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> Option<OmenaQueryStyleHoverRenderPartsV0> {
    let target = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)?;
    let mut parts =
        summarize_omena_query_style_hover_render_parts(&target.style_source, kind, name, position);
    if kind == "selector" {
        let module_graph_narrowings = selector_property_value_narrowings_for_hover_module_graph(
            target_style_path,
            style_sources,
            package_manifests,
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
            name,
            None,
        );
        if !module_graph_narrowings.is_empty() {
            parts.property_value_narrowings = module_graph_narrowings;
        }
    }
    Some(parts)
}

pub fn summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> Option<OmenaQueryStyleHoverRenderPartsV0> {
    let target = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)?;
    let branch_scope = (kind == "selector")
        .then(|| selector_hover_branch_scope_at_position(&target.style_source, name, position))
        .flatten();
    let mut parts = summarize_omena_query_style_hover_render_parts_with_branch_scope(
        &target.style_source,
        kind,
        name,
        position,
        branch_scope.clone(),
        None,
    );
    if kind == "selector" {
        let module_graph_narrowings = selector_property_value_narrowings_for_hover_module_graph(
            target_style_path,
            style_sources,
            package_manifests,
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
            name,
            branch_scope.as_ref(),
        );
        if !module_graph_narrowings.is_empty() {
            parts.property_value_narrowings = module_graph_narrowings;
        }
    }
    Some(parts)
}

/// Substrate-backed variant of
/// [`summarize_omena_query_style_hover_render_parts_for_workspace_file`]: the
/// name-independent collection (per-file cascade declarations + cross-file resolution)
/// comes precollected, so only the per-name narrowing runs here. The substrate MUST
/// have been built from the same `style_sources` (rfcs#63 E-ii).
pub fn summarize_omena_query_style_hover_render_parts_for_workspace_file_with_substrate(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryStyleCascadeNarrowingSubstrateV0,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> Option<OmenaQueryStyleHoverRenderPartsV0> {
    let target = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)?;
    summarize_omena_query_style_hover_render_parts_for_target_with_substrate(
        target_style_path,
        &target.style_source,
        substrate,
        kind,
        name,
        position,
        // Mirror the non-substrate workspace-file variant: no hovered-branch narrowing.
        None,
    )
}

/// Substrate-backed variant of
/// [`summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position`].
pub fn summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position_with_substrate(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryStyleCascadeNarrowingSubstrateV0,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> Option<OmenaQueryStyleHoverRenderPartsV0> {
    let target = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)?;
    let branch_scope = (kind == "selector")
        .then(|| selector_hover_branch_scope_at_position(&target.style_source, name, position))
        .flatten();
    summarize_omena_query_style_hover_render_parts_for_target_with_substrate(
        target_style_path,
        &target.style_source,
        substrate,
        kind,
        name,
        position,
        branch_scope,
    )
}

fn summarize_omena_query_style_hover_render_parts_for_target_with_substrate(
    target_style_path: &str,
    target_style_source: &str,
    substrate: &OmenaQueryStyleCascadeNarrowingSubstrateV0,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
    branch_scope: Option<HoverCascadeBranchScope>,
) -> Option<OmenaQueryStyleHoverRenderPartsV0> {
    let mut parts = summarize_omena_query_style_hover_render_parts_with_branch_scope(
        target_style_source,
        kind,
        name,
        position,
        branch_scope.clone(),
        substrate.declarations_for_style_path(target_style_path),
    );
    if kind == "selector" {
        let module_graph_narrowings =
            selector_property_value_narrowings_for_hover_module_graph_with_substrate(
                target_style_path,
                substrate,
                name,
                branch_scope.as_ref(),
            );
        if !module_graph_narrowings.is_empty() {
            parts.property_value_narrowings = module_graph_narrowings;
        }
    }
    Some(parts)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct HoverCascadeBranchScope {
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct HoverCascadeBranchMatch {
    span_len: usize,
    scope: HoverCascadeBranchScope,
}

type SelectorPropertyBranchKey = (String, Vec<String>, Option<String>, Option<i32>);

/// Name-independent cascade-narrowing inputs precollected over a fixed style corpus
/// (rfcs#63 E-ii): per-file cascade declarations in `style_sources` order plus the
/// cross-file resolution. Building it costs one collection pass over the corpus; every
/// subsequent per-name narrowing (hover, completion documentation) is a cheap filter.
/// Only valid for the exact `(style_sources, package_manifests, resolution_inputs)` it
/// was built from — callers own that cache-key discipline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryStyleCascadeNarrowingSubstrateV0 {
    entries: Vec<StyleCascadeNarrowingSubstrateEntry>,
    resolution: OmenaQuerySassModuleCrossFileResolutionV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StyleCascadeNarrowingSubstrateEntry {
    style_path: String,
    facts: OmenaQueryOmenaParserStyleFactsV0,
    declarations: Vec<cascade_checker::QueryCheckerCascadeDeclaration>,
}

impl OmenaQueryStyleCascadeNarrowingSubstrateV0 {
    fn declarations_for_style_path(
        &self,
        style_path: &str,
    ) -> Option<&[cascade_checker::QueryCheckerCascadeDeclaration]> {
        self.entries
            .iter()
            .find(|entry| entry.style_path == style_path)
            .map(|entry| entry.declarations.as_slice())
    }

    pub(crate) fn visible_sass_symbol_keys_for_workspace_file(
        &self,
        target_style_path: &str,
        package_manifests: &[OmenaQueryStylePackageManifestV0],
        external_sifs: &[OmenaQueryExternalSifInputV0],
        resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    ) -> BTreeSet<diagnostics::SassSymbolKey> {
        let facts_by_path = self
            .entries
            .iter()
            .map(|entry| (entry.style_path.as_str(), &entry.facts))
            .collect::<BTreeMap<_, _>>();
        diagnostics::collect_visible_sass_symbol_keys(
            target_style_path,
            &facts_by_path,
            &self.resolution,
            diagnostics::OmenaQueryExternalSifResolutionContext {
                package_manifests,
                bundler_path_mappings: resolution_inputs.bundler_path_mappings.as_slice(),
                tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.as_slice(),
                external_sifs,
            },
        )
    }
}

pub fn collect_omena_query_style_cascade_narrowing_substrate(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryStyleCascadeNarrowingSubstrateV0 {
    collect_omena_query_style_cascade_narrowing_substrate_with_external_sifs(
        style_sources,
        package_manifests,
        &[],
        resolution_inputs,
    )
}

pub fn collect_omena_query_style_cascade_narrowing_substrate_with_external_sifs(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryStyleCascadeNarrowingSubstrateV0 {
    #[cfg(feature = "salsa-memo")]
    {
        let mut host = OmenaQueryStyleMemoHostV0::new();
        if let Some(selector) = host.workspace_revision_selector(
            style_sources,
            &[],
            package_manifests,
            external_sifs,
            resolution_inputs,
        ) {
            return selector.style_cascade_narrowing_substrate();
        }
    }

    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let mut resolution = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    diagnostics::promote_sif_backed_external_edges(
        &mut resolution,
        diagnostics::OmenaQueryExternalSifResolutionContext {
            package_manifests,
            bundler_path_mappings: resolution_inputs.bundler_path_mappings.as_slice(),
            tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.as_slice(),
            external_sifs,
        },
    );
    let entries = style_sources
        .iter()
        .filter_map(|source| {
            let facts = style_fact_entries
                .iter()
                .find(|entry| entry.style_path == source.style_path)
                .map(|entry| entry.facts.clone())?;
            Some(StyleCascadeNarrowingSubstrateEntry {
                style_path: source.style_path.clone(),
                facts,
                declarations: cascade_checker::collect_query_checker_cascade_declarations(
                    source.style_source.as_str(),
                ),
            })
        })
        .collect();
    OmenaQueryStyleCascadeNarrowingSubstrateV0 {
        entries,
        resolution,
    }
}

fn selector_property_value_narrowings_for_hover(
    source: &str,
    name: &str,
    hovered_branch_scope: Option<&HoverCascadeBranchScope>,
) -> Vec<AbstractPropertyValueNarrowingV0> {
    let declarations = cascade_checker::collect_query_checker_cascade_declarations(source);
    selector_property_value_narrowings_from_declarations(
        declarations.as_slice(),
        name,
        hovered_branch_scope,
    )
}

fn selector_property_value_narrowings_from_declarations(
    declarations: &[cascade_checker::QueryCheckerCascadeDeclaration],
    name: &str,
    hovered_branch_scope: Option<&HoverCascadeBranchScope>,
) -> Vec<AbstractPropertyValueNarrowingV0> {
    let selector = format!(".{name}");
    let matching_declarations = declarations
        .iter()
        .filter(|declaration| declaration.input.selector.as_str() == selector)
        .collect::<Vec<_>>();
    let mut branch_keys = matching_declarations
        .iter()
        .map(|declaration| {
            (
                declaration.input.property.clone(),
                declaration.input.condition_context.clone(),
                declaration.input.layer_name.clone(),
                declaration.input.layer_order,
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|(_, condition_context, _, _)| {
            cascade_checker::query_condition_context_static_supports_pruning_evidence(
                condition_context.as_slice(),
                hovered_branch_scope.map(|scope| scope.condition_context.as_slice()),
            )
            .is_none_or(|evidence| !evidence.pruned)
        })
        .collect::<Vec<_>>();
    branch_keys.sort();
    if let Some(hovered_branch_scope) = hovered_branch_scope {
        let filtered_branch_keys =
            filter_hovered_branch_keys(branch_keys.as_slice(), hovered_branch_scope);
        if !filtered_branch_keys.is_empty() {
            branch_keys = filtered_branch_keys;
        }
    }

    branch_keys
        .into_iter()
        .map(
            |(property_name, condition_context, layer_name, layer_order)| {
                let property_candidates = matching_declarations
                    .iter()
                    .filter(|declaration| declaration.input.property == property_name)
                    .map(|declaration| AbstractPropertyValueCandidateV0 {
                        property_name: declaration.input.property.clone(),
                        value: declaration.input.value.clone(),
                        pseudo_state: None,
                        condition_context: declaration.input.condition_context.clone(),
                        layer_name: declaration.input.layer_name.clone(),
                        layer_order: declaration.input.layer_order,
                        source_order: Some(declaration.input.source_order),
                        important: declaration.input.important,
                        same_selector_ordering: true,
                    })
                    .collect::<Vec<_>>();
                narrow_abstract_property_value_for_cascade_branch(
                    property_name.as_str(),
                    None,
                    condition_context.as_slice(),
                    layer_name.as_deref(),
                    layer_order,
                    true,
                    property_candidates.as_slice(),
                )
            },
        )
        .collect()
}

fn selector_property_value_narrowings_for_hover_module_graph(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    name: &str,
    hovered_branch_scope: Option<&HoverCascadeBranchScope>,
) -> Vec<AbstractPropertyValueNarrowingV0> {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let resolution = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    let reachable_style_paths = diagnostics::collect_sass_module_graph_reachable_style_paths(
        target_style_path,
        &resolution,
    );
    if reachable_style_paths.len() <= 1 {
        return Vec::new();
    }

    let selector = format!(".{name}");
    let collected_declarations = style_sources
        .iter()
        .filter(|source| reachable_style_paths.contains(source.style_path.as_str()))
        .flat_map(|source| {
            cascade_checker::collect_query_checker_cascade_declarations(
                source.style_source.as_str(),
            )
        })
        .filter(|declaration| declaration.input.selector.as_str() == selector)
        .collect::<Vec<_>>();
    let matching_declarations = collected_declarations.iter().collect::<Vec<_>>();
    module_graph_narrowings_from_matching_declarations(
        matching_declarations.as_slice(),
        hovered_branch_scope,
    )
}

fn selector_property_value_narrowings_for_hover_module_graph_with_substrate(
    target_style_path: &str,
    substrate: &OmenaQueryStyleCascadeNarrowingSubstrateV0,
    name: &str,
    hovered_branch_scope: Option<&HoverCascadeBranchScope>,
) -> Vec<AbstractPropertyValueNarrowingV0> {
    let reachable_style_paths = diagnostics::collect_sass_module_graph_reachable_style_paths(
        target_style_path,
        &substrate.resolution,
    );
    if reachable_style_paths.len() <= 1 {
        return Vec::new();
    }

    let selector = format!(".{name}");
    let matching_declarations = substrate
        .entries
        .iter()
        .filter(|entry| reachable_style_paths.contains(entry.style_path.as_str()))
        .flat_map(|entry| entry.declarations.iter())
        .filter(|declaration| declaration.input.selector.as_str() == selector)
        .collect::<Vec<_>>();
    module_graph_narrowings_from_matching_declarations(
        matching_declarations.as_slice(),
        hovered_branch_scope,
    )
}

fn module_graph_narrowings_from_matching_declarations(
    matching_declarations: &[&cascade_checker::QueryCheckerCascadeDeclaration],
    hovered_branch_scope: Option<&HoverCascadeBranchScope>,
) -> Vec<AbstractPropertyValueNarrowingV0> {
    if matching_declarations.is_empty() {
        return Vec::new();
    }

    let mut branch_keys = matching_declarations
        .iter()
        .map(|declaration| {
            (
                declaration.input.property.clone(),
                declaration.input.condition_context.clone(),
                declaration.input.layer_name.clone(),
                declaration.input.layer_order,
            )
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|(_, condition_context, _, _)| {
            cascade_checker::query_condition_context_static_supports_pruning_evidence(
                condition_context.as_slice(),
                hovered_branch_scope.map(|scope| scope.condition_context.as_slice()),
            )
            .is_none_or(|evidence| !evidence.pruned)
        })
        .collect::<Vec<_>>();
    branch_keys.sort();
    if let Some(hovered_branch_scope) = hovered_branch_scope {
        let filtered_branch_keys =
            filter_hovered_branch_keys(branch_keys.as_slice(), hovered_branch_scope);
        if !filtered_branch_keys.is_empty() {
            branch_keys = filtered_branch_keys;
        }
    }

    branch_keys
        .into_iter()
        .map(
            |(property_name, condition_context, layer_name, layer_order)| {
                let property_candidates = matching_declarations
                    .iter()
                    .filter(|declaration| declaration.input.property == property_name)
                    .map(|declaration| AbstractPropertyValueCandidateV0 {
                        property_name: declaration.input.property.clone(),
                        value: declaration.input.value.clone(),
                        pseudo_state: None,
                        condition_context: declaration.input.condition_context.clone(),
                        layer_name: declaration.input.layer_name.clone(),
                        layer_order: declaration.input.layer_order,
                        source_order: Some(declaration.input.source_order),
                        important: declaration.input.important,
                        same_selector_ordering: false,
                    })
                    .collect::<Vec<_>>();
                let mut narrowed = narrow_abstract_property_value_for_cascade_branch(
                    property_name.as_str(),
                    None,
                    condition_context.as_slice(),
                    layer_name.as_deref(),
                    layer_order,
                    true,
                    property_candidates.as_slice(),
                );
                narrowed.stylesheet_scope = "moduleGraph";
                narrowed
            },
        )
        .collect()
}

fn filter_hovered_branch_keys(
    branch_keys: &[SelectorPropertyBranchKey],
    hovered_branch_scope: &HoverCascadeBranchScope,
) -> Vec<SelectorPropertyBranchKey> {
    branch_keys
        .iter()
        .filter(|(_, condition_context, layer_name, layer_order)| {
            condition_context == &hovered_branch_scope.condition_context
                && layer_name == &hovered_branch_scope.layer_name
                && layer_order == &hovered_branch_scope.layer_order
        })
        .cloned()
        .collect()
}

fn selector_hover_branch_scope_at_position(
    source: &str,
    name: &str,
    position: ParserPositionV0,
) -> Option<HoverCascadeBranchScope> {
    let offset = byte_offset_for_parser_position(source, position)?;
    let selector = format!(".{name}");
    let mut layer_orders = BTreeMap::new();
    let mut next_layer_order = 0i32;
    let mut matches = Vec::new();
    collect_hover_selector_branch_scopes(
        source,
        0,
        source.len(),
        None,
        Vec::new(),
        None,
        None,
        &mut layer_orders,
        &mut next_layer_order,
        selector.as_str(),
        offset,
        &mut matches,
    );
    matches.sort();
    matches.into_iter().next().map(|matched| matched.scope)
}

#[allow(clippy::too_many_arguments)]
fn collect_hover_selector_branch_scopes(
    source: &str,
    start: usize,
    end: usize,
    parent_selector: Option<String>,
    condition_context: Vec<String>,
    layer_name: Option<String>,
    layer_order: Option<i32>,
    layer_orders: &mut BTreeMap<String, i32>,
    next_layer_order: &mut i32,
    target_selector: &str,
    hover_offset: usize,
    matches: &mut Vec<HoverCascadeBranchMatch>,
) {
    let mut index = start;
    while let Some(open_index) = find_hover_style_top_level_byte(source, index, end, b'{') {
        let Some(close_index) = matching_style_block_end(source, open_index, b'{', b'}') else {
            break;
        };
        if close_index > end {
            break;
        }
        let prelude_start = hover_style_prelude_start(source, start, open_index);
        let prelude = source[prelude_start..open_index].trim();
        let body_start = open_index + 1;

        if let Some(layer) = hover_layer_name_from_prelude(prelude) {
            let order = *layer_orders.entry(layer.clone()).or_insert_with(|| {
                let order = *next_layer_order;
                *next_layer_order += 1;
                order
            });
            collect_hover_selector_branch_scopes(
                source,
                body_start,
                close_index,
                parent_selector.clone(),
                condition_context.clone(),
                Some(layer),
                Some(order),
                layer_orders,
                next_layer_order,
                target_selector,
                hover_offset,
                matches,
            );
        } else if prelude.starts_with('@') {
            let mut nested_condition_context = condition_context.clone();
            nested_condition_context.push(normalize_hover_condition_prelude(prelude));
            collect_hover_selector_branch_scopes(
                source,
                body_start,
                close_index,
                parent_selector.clone(),
                nested_condition_context,
                layer_name.clone(),
                layer_order,
                layer_orders,
                next_layer_order,
                target_selector,
                hover_offset,
                matches,
            );
        } else if !prelude.is_empty() {
            let canonical_members = split_hover_selector_list(prelude)
                .into_iter()
                .map(|member| canonical_hover_selector(parent_selector.as_deref(), member.as_str()))
                .collect::<Vec<_>>();
            if canonical_members
                .iter()
                .any(|member| member == target_selector)
                && hover_offset >= prelude_start
                && hover_offset <= close_index
            {
                matches.push(HoverCascadeBranchMatch {
                    span_len: close_index.saturating_sub(prelude_start),
                    scope: HoverCascadeBranchScope {
                        condition_context: condition_context.clone(),
                        layer_name: layer_name.clone(),
                        layer_order,
                    },
                });
            }
            for canonical_selector in canonical_members {
                collect_hover_selector_branch_scopes(
                    source,
                    body_start,
                    close_index,
                    Some(canonical_selector),
                    condition_context.clone(),
                    layer_name.clone(),
                    layer_order,
                    layer_orders,
                    next_layer_order,
                    target_selector,
                    hover_offset,
                    matches,
                );
            }
        }

        index = close_index + 1;
    }
}

fn find_hover_style_top_level_byte(
    source: &str,
    start: usize,
    end: usize,
    needle: u8,
) -> Option<usize> {
    let mut index = start;
    let mut quote: Option<u8> = None;
    let mut paren_depth = 0usize;
    while index < end {
        let byte = source.as_bytes().get(index).copied()?;
        if let Some(quote_byte) = quote {
            if byte == b'\\' {
                index = advance_style_escaped_char(source, index, end);
            } else if byte == quote_byte {
                quote = None;
                index = advance_style_scan_cursor(source, index, end);
            } else {
                index = advance_style_scan_cursor(source, index, end);
            }
            continue;
        }
        if source[index..end].starts_with("/*")
            && let Some(close_offset) = source[index + 2..end].find("*/")
        {
            index += close_offset + 4;
            continue;
        }
        if byte == needle && paren_depth == 0 {
            return Some(index);
        }
        match byte {
            b'"' | b'\'' | b'`' => {
                quote = Some(byte);
                index = advance_style_scan_cursor(source, index, end);
            }
            b'(' => {
                paren_depth += 1;
                index = advance_style_scan_cursor(source, index, end);
            }
            b')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index = advance_style_scan_cursor(source, index, end);
            }
            _ => index = advance_style_scan_cursor(source, index, end),
        }
    }
    None
}

fn hover_style_prelude_start(source: &str, search_start: usize, open_index: usize) -> usize {
    source[search_start..open_index]
        .rfind(['{', '}', ';'])
        .map(|offset| search_start + offset + 1)
        .unwrap_or(search_start)
}

fn hover_layer_name_from_prelude(prelude: &str) -> Option<String> {
    let rest = prelude.trim_start().strip_prefix("@layer")?.trim();
    let name = rest
        .split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, ',' | '{' | ';'))
        .next()
        .unwrap_or_default()
        .trim_matches(['"', '\'']);
    if name.is_empty() {
        Some("(anonymous-layer)".to_string())
    } else {
        Some(name.to_string())
    }
}

fn normalize_hover_condition_prelude(prelude: &str) -> String {
    prelude.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn split_hover_selector_list(prelude: &str) -> Vec<String> {
    let mut members = split_top_level_style_segments(prelude, 0, prelude.len(), b',')
        .into_iter()
        .filter_map(|(start, end)| {
            let member = prelude[start..end].trim();
            (!member.is_empty()).then(|| member.to_string())
        })
        .collect::<Vec<_>>();
    if members.is_empty() {
        members.push(prelude.trim().to_string());
    }
    members
}

fn canonical_hover_selector(parent_selector: Option<&str>, selector: &str) -> String {
    let selector = selector.trim();
    match parent_selector {
        Some(parent_selector) if selector.contains('&') => selector.replace('&', parent_selector),
        Some(parent_selector) => format!("{parent_selector} {selector}"),
        None => selector.to_string(),
    }
}

fn source_reference_text_selector_name(source: &str, span: ParserByteSpanV0) -> Option<String> {
    let text = source.get(span.start..span.end)?;
    if text.is_empty() {
        return None;
    }
    text.chars()
        .all(is_css_identifier_continue)
        .then(|| text.to_string())
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
    let style_sources = styles
        .into_iter()
        .map(|(style_path, style_source)| OmenaQueryStyleSourceInputV0 {
            style_path: style_path.to_string(),
            style_source: style_source.to_string(),
        })
        .collect::<Vec<_>>();
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();

    #[cfg(feature = "salsa-memo")]
    {
        let mut host = OmenaQueryStyleMemoHostV0::new();
        if let Some(selector) = host.workspace_revision_selector(
            style_sources.as_slice(),
            &[],
            package_manifests,
            &[],
            &OmenaQueryStyleResolutionInputsV0::default(),
        ) {
            return selector.style_semantic_graph_batch(input, package_manifests);
        }
    }

    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    let css_modules_resolution =
        summarize_css_modules_cross_file_resolution(&style_fact_entries, package_manifests);
    let sass_module_resolution = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        &[],
        &[],
    );
    let cross_file_summary = summarize_omena_query_cross_file_summary(
        &style_fact_entries,
        &css_modules_resolution,
        &sass_module_resolution,
    );
    summarize_omena_query_style_semantic_graph_batch_from_committed_parts(
        style_sources.as_slice(),
        style_fact_entries.as_slice(),
        input,
        package_manifests,
        cross_file_summary,
        css_modules_resolution,
        sass_module_resolution,
    )
}

pub(in crate::style) fn summarize_omena_query_style_semantic_graph_batch_from_committed_parts(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    input: &EngineInputV2,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    cross_file_summary: OmenaQueryCrossFileSummaryV0,
    css_modules_resolution: OmenaQueryCssModulesCrossFileResolutionV0,
    sass_module_resolution: OmenaQuerySassModuleCrossFileResolutionV0,
) -> OmenaQueryStyleSemanticGraphBatchOutputV0 {
    let workspace_declarations = style_fact_entries
        .iter()
        .flat_map(|entry| {
            collect_omena_bridge_design_token_workspace_declarations_from_source(
                entry.style_path.as_str(),
                entry.style_source.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let graphs = style_sources
        .iter()
        .map(|source| OmenaQueryStyleSemanticGraphBatchEntryV0 {
                style_path: source.style_path.clone(),
                graph: {
                    let import_reachable_declarations =
                        filter_import_reachable_design_token_workspace_declarations(
                            source.style_path.as_str(),
                            style_fact_entries,
                            &workspace_declarations,
                            package_manifests,
                        );
                    summarize_omena_bridge_style_semantic_graph_from_source_with_scoped_workspace_declarations(
                        source.style_path.as_str(),
                        source.style_source.as_str(),
                        input,
                        &import_reachable_declarations,
                        DesignTokenExternalDeclarationCandidateScopeV0::CrossFileImportGraph,
                    )
                },
            })
        .collect::<Vec<_>>();

    OmenaQueryStyleSemanticGraphBatchOutputV0 {
        schema_version: "0",
        product: "omena-semantic.style-semantic-graph-batch",
        cross_file_summary,
        css_modules_resolution,
        sass_module_resolution,
        graphs,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OmenaQueryStyleFactEntry {
    style_path: String,
    style_source: String,
    facts: OmenaQueryOmenaParserStyleFactsV0,
    semantic_runtime_index: Option<omena_semantic::StyleRuntimeIndexFactsV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryModuleInterfaceProjectionV0 {
    pub style_path: String,
    pub css_modules_style_facts: omena_semantic::CssModulesCrossFileStyleFactsV0,
    pub style_dependency_sources: Vec<String>,
    pub sass_module_edges: Vec<OmenaQuerySassModuleEdgeFactV0>,
    pub sass_module_configurable_variable_names: BTreeSet<String>,
    pub sass_module_rule_configurations: Vec<OmenaQuerySassModuleRuleConfigurationSurfaceV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQuerySassModuleRuleConfigurationSurfaceV0 {
    pub edge_kind: &'static str,
    pub rule_ordinal: usize,
    pub variable_overrides: BTreeMap<String, String>,
    pub forward_variable_overrides: BTreeMap<String, omena_semantic::SassModuleVariableOverrideV0>,
}

pub fn summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    #[cfg(feature = "salsa-memo")]
    {
        let mut host = OmenaQueryStyleMemoHostV0::new();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
            package_manifests: package_manifests.to_vec(),
            tsconfig_path_mappings: tsconfig_path_mappings.to_vec(),
            bundler_path_mappings: bundler_path_mappings.to_vec(),
            ..OmenaQueryStyleResolutionInputsV0::default()
        };
        if let Some(selector) = host.workspace_revision_selector(
            style_sources,
            &[],
            package_manifests,
            &[],
            &resolution_inputs,
        ) {
            return selector.sass_module_cross_file_resolution().clone();
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    record_sass_module_resolution_direct_recompute_for_test();

    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    )
}

fn collect_omena_query_style_fact_entries(
    style_sources: &[(&str, &str)],
) -> Vec<OmenaQueryStyleFactEntry> {
    style_sources
        .iter()
        .map(|(style_path, style_source)| {
            collect_omena_query_style_fact_entry(style_path, style_source)
        })
        .collect()
}

fn collect_omena_query_style_fact_entry(
    style_path: &str,
    style_source: &str,
) -> OmenaQueryStyleFactEntry {
    let facts = summarize_omena_query_omena_parser_style_facts(
        style_source,
        omena_parser_dialect_for_style_path(style_path),
    );
    let semantic_runtime_index = semantic_runtime_index_from_query_style_facts(style_path, &facts);
    OmenaQueryStyleFactEntry {
        style_path: style_path.to_string(),
        style_source: style_source.to_string(),
        semantic_runtime_index,
        facts,
    }
}

fn module_interface_projection_for_query(
    entry: &OmenaQueryStyleFactEntry,
) -> OmenaQueryModuleInterfaceProjectionV0 {
    OmenaQueryModuleInterfaceProjectionV0 {
        style_path: entry.style_path.clone(),
        css_modules_style_facts: css_modules_cross_file_style_fact_for_query(entry),
        style_dependency_sources: collect_style_module_dependency_sources_from_facts(&entry.facts),
        sass_module_edges: entry.facts.sass_module_edges.clone(),
        sass_module_configurable_variable_names:
            stylesheet_evaluation::derive_static_scss_stylesheet_module_configurable_variable_names(
                &entry.style_source,
            ),
        sass_module_rule_configurations: sass_module_rule_configuration_surfaces_for_query(entry),
    }
}

fn sass_module_rule_configuration_surfaces_for_query(
    entry: &OmenaQueryStyleFactEntry,
) -> Vec<OmenaQuerySassModuleRuleConfigurationSurfaceV0> {
    let mut surfaces = Vec::new();
    let mut sass_use_rule_ordinal = 0usize;
    let mut sass_forward_rule_ordinal = 0usize;
    for edge in &entry.facts.sass_module_edges {
        match edge.kind {
            "sassUse" => {
                surfaces.push(OmenaQuerySassModuleRuleConfigurationSurfaceV0 {
                    edge_kind: edge.kind,
                    rule_ordinal: sass_use_rule_ordinal,
                    variable_overrides:
                        omena_semantic::derive_sass_module_rule_variable_overrides_at_ordinal(
                            entry.style_source.as_str(),
                            "@use",
                            sass_use_rule_ordinal,
                        ),
                    forward_variable_overrides: BTreeMap::new(),
                });
                sass_use_rule_ordinal += 1;
            }
            "sassForward" => {
                let forward_variable_overrides =
                    omena_semantic::derive_sass_module_forward_variable_overrides_at_ordinal(
                        entry.style_source.as_str(),
                        sass_forward_rule_ordinal,
                    );
                surfaces.push(OmenaQuerySassModuleRuleConfigurationSurfaceV0 {
                    edge_kind: edge.kind,
                    rule_ordinal: sass_forward_rule_ordinal,
                    variable_overrides: forward_variable_overrides
                        .iter()
                        .map(|(name, override_entry)| (name.clone(), override_entry.value.clone()))
                        .collect(),
                    forward_variable_overrides,
                });
                sass_forward_rule_ordinal += 1;
            }
            _ => {}
        }
    }
    surfaces
}

fn semantic_runtime_index_from_query_style_facts(
    style_path: &str,
    facts: &OmenaQueryOmenaParserStyleFactsV0,
) -> Option<omena_semantic::StyleRuntimeIndexFactsV0> {
    let language = semantic_runtime_index_language_for_style_path(style_path)?;
    Some(omena_semantic::StyleRuntimeIndexFactsV0 {
        schema_version: "0",
        product: "omena-semantic.style-runtime-index-facts",
        style_path: style_path.to_string(),
        language,
        class_selector_names: facts.class_selector_names.clone(),
        custom_property_names: facts.custom_property_names.clone(),
        custom_property_decl_names: facts.custom_property_decl_names.clone(),
        custom_property_ref_names: facts.custom_property_ref_names.clone(),
        keyframe_names: facts.keyframe_names.clone(),
        animation_reference_names: facts.animation_reference_names.clone(),
        ready_surfaces: vec![
            "semanticRuntimeIndexFacts",
            "customPropertyRuntimeIndex",
            "keyframeRuntimeIndex",
        ],
    })
}

fn semantic_runtime_index_language_for_style_path(style_path: &str) -> Option<&'static str> {
    if style_path.ends_with(".module.css") || style_path.ends_with(".css") {
        Some("css")
    } else if style_path.ends_with(".module.scss") || style_path.ends_with(".scss") {
        Some("scss")
    } else if style_path.ends_with(".module.sass") || style_path.ends_with(".sass") {
        Some("sass")
    } else if style_path.ends_with(".module.less") || style_path.ends_with(".less") {
        Some("less")
    } else {
        None
    }
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static SASS_MODULE_RESOLUTION_DIRECT_RECOMPUTES: std::cell::Cell<u64> =
        const { std::cell::Cell::new(0) };
    static SASS_MODULE_RESOLUTION_INTERNAL_COMPUTES: std::cell::Cell<u64> =
        const { std::cell::Cell::new(0) };
}

#[cfg(any(test, feature = "test-support"))]
pub fn reset_sass_module_resolution_direct_recompute_count_for_test() {
    SASS_MODULE_RESOLUTION_DIRECT_RECOMPUTES.with(|count| count.set(0));
}

#[cfg(any(test, feature = "test-support"))]
pub fn reset_sass_module_resolution_internal_compute_count_for_test() {
    SASS_MODULE_RESOLUTION_INTERNAL_COMPUTES.with(|count| count.set(0));
}

#[cfg(any(test, feature = "test-support"))]
pub fn read_sass_module_resolution_direct_recompute_count_for_test() -> u64 {
    SASS_MODULE_RESOLUTION_DIRECT_RECOMPUTES.with(|count| count.get())
}

#[cfg(any(test, feature = "test-support"))]
pub fn read_sass_module_resolution_internal_compute_count_for_test() -> u64 {
    SASS_MODULE_RESOLUTION_INTERNAL_COMPUTES.with(|count| count.get())
}

#[cfg(any(test, feature = "test-support"))]
fn record_sass_module_resolution_direct_recompute_for_test() {
    SASS_MODULE_RESOLUTION_DIRECT_RECOMPUTES.with(|count| {
        count.set(count.get() + 1);
    });
}

#[cfg(any(test, feature = "test-support"))]
fn record_sass_module_resolution_internal_compute_for_test() {
    SASS_MODULE_RESOLUTION_INTERNAL_COMPUTES.with(|count| {
        count.set(count.get() + 1);
    });
}

/// Derive the load-path roots to try when joining a load-path-rooted `@use` (dart-sass
/// `--load-path`). Each in-graph style file contributes its ancestor directories: a path-shaped
/// specifier `src/scss/design-system.scss` is then joinable under any root `<R>` for which
/// `<R>/src/scss/design-system.scss` is itself in-graph. The resolver accepts only such existing
/// candidates, so over-collecting roots cannot fabricate a spurious edge. (RFC-0007-I, #49)
fn collect_load_path_roots(available_style_paths: &BTreeSet<&str>) -> Vec<String> {
    let mut roots = BTreeSet::new();
    for path in available_style_paths {
        let mut current = *path;
        // Walk up the directory chain on the normalized `/` separator. Style paths flowing
        // through the query layer are already forward-slash normalized by the resolver.
        while let Some(parent_end) = current.rfind('/') {
            if parent_end == 0 {
                // Keep the filesystem root (`/`) as a candidate load-path root.
                roots.insert("/".to_string());
                break;
            }
            let parent = &current[..parent_end];
            if !roots.insert(parent.to_string()) {
                // This ancestor (and therefore all of its ancestors) is already recorded.
                break;
            }
            current = parent;
        }
    }
    roots.into_iter().collect()
}

fn summarize_sass_module_cross_file_resolution(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_sass_module_resolution_internal_compute_for_test();

    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let resolver_available_style_paths = style_fact_entries
        .iter()
        .flat_map(|entry| {
            [
                entry.style_path.clone(),
                resolver_style_path(entry.style_path.as_str()),
            ]
        })
        .collect::<BTreeSet<_>>();
    let resolver_available_style_path_refs = resolver_available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    // Load-path roots are the ancestor directories of the in-graph style files. A
    // load-path-rooted `@use 'src/scss/design-system.scss'` (dart-sass `--load-path`) is joined
    // only when `<root>/src/scss/design-system.scss` is itself an in-graph file, so deriving
    // roots from `available_style_paths` keeps the join sound without new configuration input,
    // and never shadows the file-relative or bare-package routes. (RFC-0007-I, #49)
    let load_path_roots = collect_load_path_roots(&resolver_available_style_path_refs);
    let load_path_root_refs = load_path_roots
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    let source_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.clone(), entry.style_source.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();

    for entry in style_fact_entries {
        let mut sass_use_rule_ordinal = 0usize;
        let mut sass_forward_rule_ordinal = 0usize;
        for edge in &entry.facts.sass_module_edges {
            let rule_ordinal = match edge.kind {
                "sassUse" => {
                    let rule_ordinal = sass_use_rule_ordinal;
                    sass_use_rule_ordinal += 1;
                    rule_ordinal
                }
                "sassForward" => {
                    let rule_ordinal = sass_forward_rule_ordinal;
                    sass_forward_rule_ordinal += 1;
                    rule_ordinal
                }
                _ => 0,
            };
            let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
                resolver_style_path(entry.style_path.as_str()).as_str(),
                edge.source.as_str(),
                &resolver_available_style_path_refs,
                &resolver_package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
                &load_path_root_refs,
            );
            let status = if resolution.resolution_kind == "externalIgnored" {
                "external"
            } else if resolution.resolved_style_path.is_some() {
                "resolved"
            } else {
                "unresolved"
            };
            let resolved_style_path =
                resolution
                    .resolved_style_path
                    .and_then(|resolved_style_path| {
                        canonical_available_style_path(
                            resolved_style_path.as_str(),
                            &available_style_paths,
                        )
                        .or(Some(resolved_style_path))
                    });
            let symlink_chain_link_count = resolution.symlink_chain.link_count;
            let symlink_chain_links = resolution
                .symlink_chain
                .links
                .into_iter()
                .map(|link| OmenaQuerySymlinkChainLinkV0 {
                    link_path: link.link_path,
                    target_path: link.target_path,
                    target_was_absolute: link.target_was_absolute,
                })
                .collect::<Vec<_>>();
            let configuration_evidence =
                transform::derive_static_scss_module_resolution_configuration_evidence(
                    entry.style_source.as_str(),
                    edge.kind,
                    rule_ordinal,
                    resolved_style_path.as_deref(),
                );
            let invalid_configuration_variable_names =
                resolved_style_path
                    .as_deref()
                    .and_then(|target_path| {
                        source_by_path.get(target_path).map(|target_source| {
                            let configurable_names = transform::derive_static_scss_module_configurable_variable_names_for_resolution(
                                target_path,
                                target_source,
                                &available_style_paths,
                                &source_by_path,
                                package_manifests,
                                bundler_path_mappings,
                                tsconfig_path_mappings,
                            );
                            configuration_evidence
                                .configuration_variable_names
                                .iter()
                                .filter(|name| !configurable_names.contains(*name))
                                .cloned()
                                .collect::<Vec<_>>()
                        })
                    })
                    .unwrap_or_default();
            edges.push(OmenaQuerySassModuleEdgeResolutionV0 {
                from_style_path: entry.style_path.clone(),
                edge_kind: edge.kind,
                source: edge.source.clone(),
                rule_ordinal,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                forward_prefix: edge.forward_prefix.clone(),
                visibility_filter_kind: edge.visibility_filter_kind,
                visibility_filter_names: edge.visibility_filter_names.clone(),
                resolved_style_path,
                status,
                resolution_kind: resolution.resolution_kind,
                candidate_count: resolution.candidate_count,
                symlink_chain_link_count,
                symlink_chain_links,
                configuration_signature: configuration_evidence.configuration_signature,
                configuration_variable_count: configuration_evidence.configuration_variable_count,
                invalid_configuration_variable_names,
                module_instance_identity_key: configuration_evidence.module_instance_identity_key,
            });
        }
    }

    edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.edge_kind,
            edge.rule_ordinal,
            edge.source.clone(),
        )
    });
    let configurable_names_memo: RefCell<BTreeMap<String, BTreeSet<String>>> =
        RefCell::new(BTreeMap::new());
    let semantic_edges = sass_module_graph_edge_facts_for_query(&edges);
    let semantic_resolution = omena_semantic::summarize_sass_module_graph_resolution(
        style_fact_entries.len(),
        semantic_edges.as_slice(),
        &QuerySassModuleGraphConfigurationResolver {
            source_by_path: &source_by_path,
            available_style_paths: &available_style_paths,
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            configurable_names_memo: &configurable_names_memo,
        },
    );
    let graph_closure_edges = semantic_resolution
        .graph_closure_edges
        .into_iter()
        .map(|edge| OmenaQuerySassModuleGraphClosureEdgeV0 {
            from_style_path: edge.from_style_path,
            target_style_path: edge.target_style_path,
            edge_kind: edge.edge_kind,
            depth: edge.depth,
            path: edge.path,
            namespace_kind: edge.namespace_kind,
            namespace: edge.namespace,
            forward_prefix: edge.forward_prefix,
            visibility_filter_kind: edge.visibility_filter_kind,
            visibility_filter_names: edge.visibility_filter_names,
            configuration_signature: edge.configuration_signature,
            configuration_variable_count: edge.configuration_variable_count,
            invalid_configuration_variable_names: edge.invalid_configuration_variable_names,
            module_instance_identity_key: edge.module_instance_identity_key,
        })
        .collect::<Vec<_>>();
    let cycles = semantic_resolution
        .cycles
        .into_iter()
        .map(|cycle| OmenaQuerySassModuleCycleV0 { path: cycle.path })
        .collect::<Vec<_>>();
    let symlink_chain_edge_count = edges
        .iter()
        .filter(|edge| edge.symlink_chain_link_count > 0)
        .count();
    let symlink_chain_link_count = edges.iter().map(|edge| edge.symlink_chain_link_count).sum();

    OmenaQuerySassModuleCrossFileResolutionV0 {
        schema_version: "0",
        product: "omena-query.sass-module-cross-file-resolution",
        status: "moduleGraphClosureResolved",
        resolution_scope: "batchModuleGraph",
        style_count: semantic_resolution.style_count,
        module_edge_count: semantic_resolution.module_edge_count,
        resolved_module_edge_count: semantic_resolution.resolved_module_edge_count,
        unresolved_module_edge_count: semantic_resolution.unresolved_module_edge_count,
        external_module_edge_count: semantic_resolution.external_module_edge_count,
        symlink_chain_edge_count,
        symlink_chain_link_count,
        configured_module_instance_count: semantic_resolution.configured_module_instance_count,
        edges,
        graph_closure_edge_count: semantic_resolution.graph_closure_edge_count,
        cycle_count: semantic_resolution.cycle_count,
        visibility_filter_count: semantic_resolution.visibility_filter_count,
        graph_closure_edges,
        cycles,
        capabilities: OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0 {
            omena_parser_module_edge_consumption_ready: true,
            resolver_backed_source_resolution_ready: true,
            package_manifest_resolution_ready: true,
            external_module_filtering_ready: true,
            graph_closure_ready: true,
            cycle_detection_ready: true,
            namespace_show_hide_filter_ready: true,
            configured_module_instance_identity_ready: true,
            symlink_chain_metadata_ready: true,
        },
        next_priorities: Vec::new(),
    }
}

fn summarize_sass_module_cross_file_resolution_from_module_interfaces(
    module_interfaces: &[OmenaQueryModuleInterfaceProjectionV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    #[cfg(any(test, feature = "test-support"))]
    record_sass_module_resolution_internal_compute_for_test();

    let available_style_paths = module_interfaces
        .iter()
        .map(|projection| projection.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let resolver_available_style_paths = module_interfaces
        .iter()
        .flat_map(|projection| {
            [
                projection.style_path.clone(),
                resolver_style_path(projection.style_path.as_str()),
            ]
        })
        .collect::<BTreeSet<_>>();
    let resolver_available_style_path_refs = resolver_available_style_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let load_path_roots = collect_load_path_roots(&resolver_available_style_path_refs);
    let load_path_root_refs = load_path_roots
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    let module_interface_by_path = module_interfaces
        .iter()
        .map(|projection| (projection.style_path.clone(), projection))
        .collect::<BTreeMap<_, _>>();
    let configurable_names_by_path = sass_configurable_variable_names_from_module_interfaces(
        module_interfaces,
        &resolver_available_style_path_refs,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    let mut edges = Vec::new();

    for projection in module_interfaces {
        let mut sass_use_rule_ordinal = 0usize;
        let mut sass_forward_rule_ordinal = 0usize;
        for edge in &projection.sass_module_edges {
            let rule_ordinal = match edge.kind {
                "sassUse" => {
                    let rule_ordinal = sass_use_rule_ordinal;
                    sass_use_rule_ordinal += 1;
                    rule_ordinal
                }
                "sassForward" => {
                    let rule_ordinal = sass_forward_rule_ordinal;
                    sass_forward_rule_ordinal += 1;
                    rule_ordinal
                }
                _ => 0,
            };
            let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
                resolver_style_path(projection.style_path.as_str()).as_str(),
                edge.source.as_str(),
                &resolver_available_style_path_refs,
                &resolver_package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
                &load_path_root_refs,
            );
            let status = if resolution.resolution_kind == "externalIgnored" {
                "external"
            } else if resolution.resolved_style_path.is_some() {
                "resolved"
            } else {
                "unresolved"
            };
            let resolved_style_path =
                resolution
                    .resolved_style_path
                    .and_then(|resolved_style_path| {
                        canonical_available_style_path(
                            resolved_style_path.as_str(),
                            &available_style_paths,
                        )
                        .or(Some(resolved_style_path))
                    });
            let symlink_chain_link_count = resolution.symlink_chain.link_count;
            let symlink_chain_links = resolution
                .symlink_chain
                .links
                .into_iter()
                .map(|link| OmenaQuerySymlinkChainLinkV0 {
                    link_path: link.link_path,
                    target_path: link.target_path,
                    target_was_absolute: link.target_was_absolute,
                })
                .collect::<Vec<_>>();
            let variable_overrides = sass_module_rule_variable_overrides_from_interface(
                projection,
                edge.kind,
                rule_ordinal,
            );
            let invalid_configuration_variable_names = resolved_style_path
                .as_deref()
                .and_then(|target_path| configurable_names_by_path.get(target_path))
                .map(|configurable_names| {
                    variable_overrides
                        .keys()
                        .filter(|name| !configurable_names.contains(*name))
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let module_instance_identity_key = match edge.kind {
                "sassUse" | "sassForward" => resolved_style_path.as_deref().map(|target_path| {
                    omena_semantic::summarize_sass_module_instance_identity_key(
                        target_path,
                        &variable_overrides,
                    )
                }),
                _ => None,
            };
            edges.push(OmenaQuerySassModuleEdgeResolutionV0 {
                from_style_path: projection.style_path.clone(),
                edge_kind: edge.kind,
                source: edge.source.clone(),
                rule_ordinal,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                forward_prefix: edge.forward_prefix.clone(),
                visibility_filter_kind: edge.visibility_filter_kind,
                visibility_filter_names: edge.visibility_filter_names.clone(),
                resolved_style_path,
                status,
                resolution_kind: resolution.resolution_kind,
                candidate_count: resolution.candidate_count,
                symlink_chain_link_count,
                symlink_chain_links,
                configuration_signature:
                    omena_semantic::summarize_sass_module_configuration_signature(
                        &variable_overrides,
                    ),
                configuration_variable_count: variable_overrides.len(),
                invalid_configuration_variable_names,
                module_instance_identity_key,
            });
        }
    }

    edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.edge_kind,
            edge.rule_ordinal,
            edge.source.clone(),
        )
    });
    let semantic_edges = sass_module_graph_edge_facts_for_query(&edges);
    let semantic_resolution = omena_semantic::summarize_sass_module_graph_resolution(
        module_interfaces.len(),
        semantic_edges.as_slice(),
        &ModuleInterfaceSassModuleGraphConfigurationResolver {
            module_interface_by_path: &module_interface_by_path,
            configurable_names_by_path: &configurable_names_by_path,
        },
    );
    let graph_closure_edges = semantic_resolution
        .graph_closure_edges
        .into_iter()
        .map(|edge| OmenaQuerySassModuleGraphClosureEdgeV0 {
            from_style_path: edge.from_style_path,
            target_style_path: edge.target_style_path,
            edge_kind: edge.edge_kind,
            depth: edge.depth,
            path: edge.path,
            namespace_kind: edge.namespace_kind,
            namespace: edge.namespace,
            forward_prefix: edge.forward_prefix,
            visibility_filter_kind: edge.visibility_filter_kind,
            visibility_filter_names: edge.visibility_filter_names,
            configuration_signature: edge.configuration_signature,
            configuration_variable_count: edge.configuration_variable_count,
            invalid_configuration_variable_names: edge.invalid_configuration_variable_names,
            module_instance_identity_key: edge.module_instance_identity_key,
        })
        .collect::<Vec<_>>();
    let cycles = semantic_resolution
        .cycles
        .into_iter()
        .map(|cycle| OmenaQuerySassModuleCycleV0 { path: cycle.path })
        .collect::<Vec<_>>();
    let symlink_chain_edge_count = edges
        .iter()
        .filter(|edge| edge.symlink_chain_link_count > 0)
        .count();
    let symlink_chain_link_count = edges.iter().map(|edge| edge.symlink_chain_link_count).sum();

    OmenaQuerySassModuleCrossFileResolutionV0 {
        schema_version: "0",
        product: "omena-query.sass-module-cross-file-resolution",
        status: "moduleGraphClosureResolved",
        resolution_scope: "batchModuleGraph",
        style_count: semantic_resolution.style_count,
        module_edge_count: semantic_resolution.module_edge_count,
        resolved_module_edge_count: semantic_resolution.resolved_module_edge_count,
        unresolved_module_edge_count: semantic_resolution.unresolved_module_edge_count,
        external_module_edge_count: semantic_resolution.external_module_edge_count,
        symlink_chain_edge_count,
        symlink_chain_link_count,
        configured_module_instance_count: semantic_resolution.configured_module_instance_count,
        edges,
        graph_closure_edge_count: semantic_resolution.graph_closure_edge_count,
        cycle_count: semantic_resolution.cycle_count,
        visibility_filter_count: semantic_resolution.visibility_filter_count,
        graph_closure_edges,
        cycles,
        capabilities: OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0 {
            omena_parser_module_edge_consumption_ready: true,
            resolver_backed_source_resolution_ready: true,
            package_manifest_resolution_ready: true,
            external_module_filtering_ready: true,
            graph_closure_ready: true,
            cycle_detection_ready: true,
            namespace_show_hide_filter_ready: true,
            configured_module_instance_identity_ready: true,
            symlink_chain_metadata_ready: true,
        },
        next_priorities: Vec::new(),
    }
}

fn canonical_available_style_path(
    candidate: &str,
    available_style_paths: &BTreeSet<&str>,
) -> Option<String> {
    if available_style_paths.contains(candidate) {
        return Some(candidate.to_string());
    }
    let candidate_path = style_path_equivalence_key(candidate)?;
    available_style_paths
        .iter()
        .find(|available| {
            style_path_equivalence_key(available).as_deref() == Some(candidate_path.as_path())
        })
        .map(|available| (*available).to_string())
}

fn sass_module_rule_variable_overrides_from_interface(
    projection: &OmenaQueryModuleInterfaceProjectionV0,
    edge_kind: &'static str,
    rule_ordinal: usize,
) -> BTreeMap<String, String> {
    projection
        .sass_module_rule_configurations
        .iter()
        .find(|surface| surface.edge_kind == edge_kind && surface.rule_ordinal == rule_ordinal)
        .map(|surface| surface.variable_overrides.clone())
        .unwrap_or_default()
}

fn sass_module_forward_variable_overrides_from_interface(
    projection: &OmenaQueryModuleInterfaceProjectionV0,
    rule_ordinal: usize,
) -> BTreeMap<String, omena_semantic::SassModuleVariableOverrideV0> {
    sass_module_forward_variable_overrides_from_rule_configurations(
        projection.sass_module_rule_configurations.as_slice(),
        rule_ordinal,
    )
}

fn sass_module_forward_variable_overrides_from_rule_configurations(
    rule_configurations: &[OmenaQuerySassModuleRuleConfigurationSurfaceV0],
    rule_ordinal: usize,
) -> BTreeMap<String, omena_semantic::SassModuleVariableOverrideV0> {
    rule_configurations
        .iter()
        .find(|surface| surface.edge_kind == "sassForward" && surface.rule_ordinal == rule_ordinal)
        .map(|surface| surface.forward_variable_overrides.clone())
        .unwrap_or_default()
}

fn sass_configurable_variable_names_from_module_interfaces(
    module_interfaces: &[OmenaQueryModuleInterfaceProjectionV0],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let module_interface_by_path = module_interfaces
        .iter()
        .map(|projection| (projection.style_path.clone(), projection))
        .collect::<BTreeMap<_, _>>();
    let mut memo = BTreeMap::new();
    let mut context = ModuleInterfaceSassConfigurableNamesContext {
        module_interface_by_path: &module_interface_by_path,
        available_style_paths,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        memo: &mut memo,
    };
    for projection in module_interfaces {
        let mut visiting = BTreeSet::new();
        let names = sass_configurable_variable_names_for_module_interface(
            projection.style_path.as_str(),
            &mut context,
            &mut visiting,
        );
        context.memo.insert(projection.style_path.clone(), names);
    }
    memo
}

struct ModuleInterfaceSassConfigurableNamesContext<'a> {
    module_interface_by_path: &'a BTreeMap<String, &'a OmenaQueryModuleInterfaceProjectionV0>,
    available_style_paths: &'a BTreeSet<&'a str>,
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &'a [OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &'a [OmenaResolverTsconfigPathMappingV0],
    memo: &'a mut BTreeMap<String, BTreeSet<String>>,
}

fn sass_configurable_variable_names_for_module_interface(
    style_path: &str,
    context: &mut ModuleInterfaceSassConfigurableNamesContext<'_>,
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    if let Some(cached) = context.memo.get(style_path) {
        return cached.clone();
    }
    if !visiting.insert(style_path.to_string()) {
        return BTreeSet::new();
    }
    let Some(projection) = context.module_interface_by_path.get(style_path) else {
        visiting.remove(style_path);
        return BTreeSet::new();
    };
    let projection_style_path = projection.style_path.clone();
    let rule_configurations = projection.sass_module_rule_configurations.clone();
    let mut names = projection.sass_module_configurable_variable_names.clone();
    let forward_edges = projection
        .sass_module_edges
        .iter()
        .filter(|edge| edge.kind == "sassForward")
        .cloned()
        .enumerate()
        .collect::<Vec<_>>();
    for (forward_rule_ordinal, edge) in forward_edges {
        let Some(resolved) = resolve_style_module_source(
            projection_style_path.as_str(),
            edge.source.as_str(),
            context.available_style_paths,
            context.package_manifests,
        )
        .or_else(|| {
            let resolver_package_manifests = context
                .package_manifests
                .iter()
                .map(|manifest| OmenaResolverStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let load_path_roots = collect_load_path_roots(context.available_style_paths);
            let load_path_root_refs = load_path_roots
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>();
            summarize_omena_resolver_style_module_resolution_with_load_path_roots(
                resolver_style_path(projection_style_path.as_str()).as_str(),
                edge.source.as_str(),
                context.available_style_paths,
                &resolver_package_manifests,
                context.bundler_path_mappings,
                context.tsconfig_path_mappings,
                &load_path_root_refs,
            )
            .resolved_style_path
        }) else {
            continue;
        };
        let Some(resolved) =
            canonical_available_style_path(resolved.as_str(), context.available_style_paths)
        else {
            continue;
        };
        let child_names = sass_configurable_variable_names_for_module_interface(
            resolved.as_str(),
            context,
            visiting,
        );
        let non_default_forward_overrides =
            sass_module_forward_variable_overrides_from_rule_configurations(
                rule_configurations.as_slice(),
                forward_rule_ordinal,
            )
            .into_iter()
            .filter_map(|(name, override_entry)| (!override_entry.is_default).then_some(name))
            .collect::<BTreeSet<_>>();
        let child_names = child_names
            .into_iter()
            .filter(|name| !non_default_forward_overrides.contains(name))
            .collect::<BTreeSet<_>>();
        names.extend(
            omena_semantic::filter_sass_forward_configurable_variable_names(
                child_names,
                edge.forward_prefix.as_deref(),
                edge.visibility_filter_kind,
                &edge.visibility_filter_names,
            ),
        );
    }
    visiting.remove(style_path);
    context.memo.insert(style_path.to_string(), names.clone());
    names
}

#[derive(Debug, Clone, Copy)]
struct ModuleInterfaceSassModuleGraphConfigurationResolver<'a> {
    module_interface_by_path: &'a BTreeMap<String, &'a OmenaQueryModuleInterfaceProjectionV0>,
    configurable_names_by_path: &'a BTreeMap<String, BTreeSet<String>>,
}

impl omena_semantic::SassModuleGraphConfigurationResolverV0
    for ModuleInterfaceSassModuleGraphConfigurationResolver<'_>
{
    fn use_variable_overrides(
        &self,
        request: omena_semantic::SassModuleUseConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String> {
        self.module_interface_by_path
            .get(request.from_style_path)
            .map(|projection| {
                sass_module_rule_variable_overrides_from_interface(
                    projection,
                    "sassUse",
                    request.rule_ordinal,
                )
            })
            .unwrap_or_default()
    }

    fn forward_effective_variable_overrides(
        &self,
        request: omena_semantic::SassModuleForwardConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String> {
        let Some(projection) = self.module_interface_by_path.get(request.from_style_path) else {
            return BTreeMap::new();
        };
        let explicit_variable_overrides =
            sass_module_forward_variable_overrides_from_interface(projection, request.rule_ordinal);
        omena_semantic::derive_sass_forward_effective_variable_overrides(
            &explicit_variable_overrides,
            request.inherited_variable_overrides,
            request.forward_prefix,
            request.visibility_filter_kind,
            request.visibility_filter_names,
            request.configurable_names,
        )
    }

    fn configurable_names(&self, target_style_path: &str) -> BTreeSet<String> {
        self.configurable_names_by_path
            .get(target_style_path)
            .cloned()
            .unwrap_or_default()
    }
}

fn style_path_equivalence_key(path_or_uri: &str) -> Option<PathBuf> {
    let path = path_or_uri.strip_prefix("file://").unwrap_or(path_or_uri);
    Some(Path::new(path).components().collect())
}

fn resolver_style_path(path_or_uri: &str) -> String {
    path_or_uri
        .strip_prefix("file://")
        .unwrap_or(path_or_uri)
        .to_string()
}

#[derive(Debug, Clone, Copy)]
struct QuerySassModuleGraphConfigurationResolver<'a> {
    source_by_path: &'a BTreeMap<String, String>,
    available_style_paths: &'a BTreeSet<&'a str>,
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &'a [OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &'a [OmenaResolverTsconfigPathMappingV0],
    configurable_names_memo: &'a RefCell<BTreeMap<String, BTreeSet<String>>>,
}

impl omena_semantic::SassModuleGraphConfigurationResolverV0
    for QuerySassModuleGraphConfigurationResolver<'_>
{
    fn use_variable_overrides(
        &self,
        request: omena_semantic::SassModuleUseConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String> {
        let Some(style_source) = self.source_by_path.get(request.from_style_path) else {
            return BTreeMap::new();
        };
        omena_semantic::derive_sass_module_rule_variable_overrides_at_ordinal(
            style_source,
            "@use",
            request.rule_ordinal,
        )
    }

    fn forward_effective_variable_overrides(
        &self,
        request: omena_semantic::SassModuleForwardConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String> {
        let Some(style_source) = self.source_by_path.get(request.from_style_path) else {
            return BTreeMap::new();
        };
        omena_semantic::derive_sass_module_forward_effective_variable_overrides_at_ordinal(
            style_source,
            request.rule_ordinal,
            request.inherited_variable_overrides,
            request.forward_prefix,
            request.visibility_filter_kind,
            request.visibility_filter_names,
            request.configurable_names,
        )
    }

    fn configurable_names(&self, target_style_path: &str) -> BTreeSet<String> {
        memoized_configurable_names(target_style_path, self)
    }
}

fn sass_module_graph_edge_facts_for_query(
    edges: &[OmenaQuerySassModuleEdgeResolutionV0],
) -> Vec<omena_semantic::SassModuleGraphEdgeFactV0> {
    edges
        .iter()
        .map(|edge| omena_semantic::SassModuleGraphEdgeFactV0 {
            from_style_path: edge.from_style_path.clone(),
            edge_kind: edge.edge_kind,
            source: edge.source.clone(),
            rule_ordinal: edge.rule_ordinal,
            namespace_kind: edge.namespace_kind,
            namespace: edge.namespace.clone(),
            forward_prefix: edge.forward_prefix.clone(),
            visibility_filter_kind: edge.visibility_filter_kind,
            visibility_filter_names: edge.visibility_filter_names.clone(),
            resolved_style_path: edge.resolved_style_path.clone(),
            status: edge.status,
            configuration_signature: edge.configuration_signature.clone(),
            configuration_variable_count: edge.configuration_variable_count,
            invalid_configuration_variable_names: edge.invalid_configuration_variable_names.clone(),
            module_instance_identity_key: edge.module_instance_identity_key.clone(),
        })
        .collect()
}

// Test-only counter of ACTUAL configurable-name derivations (memo misses that run the parse +
// disk-resolution work). With the L1 memo this is O(distinct modules); without it the same
// derivation runs per enumerated closure path = O(paths) (super-polynomial). The end-to-end
// growth gate (tests) asserts this stays ~linear, catching a regression of the L1 memo that the
// output-only equivalence oracle cannot see. Compiled out of non-test builds (zero overhead).
#[cfg(test)]
thread_local! {
    static CONFIGURABLE_NAMES_DERIVATIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_configurable_names_derivation_count() {
    CONFIGURABLE_NAMES_DERIVATIONS.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn configurable_names_derivation_count() -> u64 {
    CONFIGURABLE_NAMES_DERIVATIONS.with(|count| count.get())
}

#[cfg(test)]
pub(crate) fn with_rawallpaths_closure<R>(body: impl FnOnce() -> R) -> R {
    omena_semantic::with_sass_module_rawallpaths_closure_for_test(body)
}

fn memoized_configurable_names(
    target_style_path: &str,
    context: &QuerySassModuleGraphConfigurationResolver<'_>,
) -> BTreeSet<String> {
    {
        let cache = context.configurable_names_memo.borrow();
        if let Some(cached) = cache.get(target_style_path) {
            return cached.clone();
        }
    }
    let computed = context
        .source_by_path
        .get(target_style_path)
        .map(|target_source| {
            #[cfg(test)]
            CONFIGURABLE_NAMES_DERIVATIONS.with(|count| count.set(count.get() + 1));
            transform::derive_static_scss_module_configurable_variable_names_for_resolution(
                target_style_path,
                target_source,
                context.available_style_paths,
                context.source_by_path,
                context.package_manifests,
                context.bundler_path_mappings,
                context.tsconfig_path_mappings,
            )
        })
        .unwrap_or_default();
    context
        .configurable_names_memo
        .borrow_mut()
        .insert(target_style_path.to_string(), computed.clone());
    computed
}

fn summarize_css_modules_cross_file_resolution(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCssModulesCrossFileResolutionV0 {
    let semantic_facts = css_modules_cross_file_style_facts_for_query(style_fact_entries);
    let style_import_edges =
        style_import_reachability_edges_for_query(style_fact_entries, package_manifests);
    summarize_css_modules_cross_file_resolution_from_semantic_inputs(
        semantic_facts.as_slice(),
        style_import_edges.as_slice(),
        package_manifests,
    )
}

fn summarize_css_modules_cross_file_resolution_from_module_interfaces(
    module_interfaces: &[OmenaQueryModuleInterfaceProjectionV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCssModulesCrossFileResolutionV0 {
    let semantic_facts = module_interfaces
        .iter()
        .map(|projection| projection.css_modules_style_facts.clone())
        .collect::<Vec<_>>();
    let style_import_edges = style_import_reachability_edges_from_module_interfaces(
        module_interfaces,
        package_manifests,
    );
    summarize_css_modules_cross_file_resolution_from_semantic_inputs(
        semantic_facts.as_slice(),
        style_import_edges.as_slice(),
        package_manifests,
    )
}

fn summarize_css_modules_cross_file_resolution_from_semantic_inputs(
    semantic_facts: &[omena_semantic::CssModulesCrossFileStyleFactsV0],
    style_import_edges: &[omena_semantic::StyleImportReachabilityEdgeFactV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCssModulesCrossFileResolutionV0 {
    let semantic_package_manifests = semantic_package_manifests_for_query(package_manifests);
    let semantic_resolution = omena_semantic::summarize_css_modules_cross_file_resolution(
        semantic_facts,
        style_import_edges,
        semantic_package_manifests.as_slice(),
    );
    let composes_closure_edges = semantic_resolution
        .composes_closure_edges
        .into_iter()
        .map(|edge| OmenaQueryCssModulesComposesClosureEdgeV0 {
            from_style_path: edge.from_style_path,
            owner_selector_name: edge.owner_selector_name,
            target_style_path: edge.target_style_path,
            target_selector_name: edge.target_selector_name,
            depth: edge.depth,
            path: edge.path,
        })
        .collect::<Vec<_>>();
    let value_closure_edges = semantic_resolution
        .value_closure_edges
        .into_iter()
        .map(|edge| OmenaQueryCssModulesValueClosureEdgeV0 {
            from_style_path: edge.from_style_path,
            value_name: edge.value_name,
            target_style_path: edge.target_style_path,
            target_value_name: edge.target_value_name,
            depth: edge.depth,
            path: edge.path,
        })
        .collect::<Vec<_>>();
    let icss_closure_edges = semantic_resolution
        .icss_closure_edges
        .into_iter()
        .map(|edge| OmenaQueryCssModulesIcssClosureEdgeV0 {
            from_style_path: edge.from_style_path,
            name: edge.name,
            target_style_path: edge.target_style_path,
            target_name: edge.target_name,
            depth: edge.depth,
            path: edge.path,
        })
        .collect::<Vec<_>>();
    let edges = semantic_resolution
        .edges
        .into_iter()
        .map(|edge| OmenaQueryCssModulesImportEdgeResolutionV0 {
            from_style_path: edge.from_style_path,
            import_kind: edge.import_kind,
            source: edge.source,
            resolved_style_path: edge.resolved_style_path,
            status: edge.status,
            import_graph_distance: edge.import_graph_distance,
            import_graph_order: edge.import_graph_order,
            imported_names: edge.imported_names,
            exported_names: edge.exported_names,
            matched_names: edge.matched_names,
        })
        .collect::<Vec<_>>();
    let cycles = semantic_resolution
        .cycles
        .into_iter()
        .map(|cycle| OmenaQueryCssModulesCycleV0 {
            kind: cycle.kind,
            path: cycle.path,
        })
        .collect::<Vec<_>>();

    OmenaQueryCssModulesCrossFileResolutionV0 {
        schema_version: "0",
        product: "omena-query.css-modules-cross-file-resolution",
        status: "semanticLayerOwnedResolutionAdapter",
        resolution_scope: "batchImportGraph",
        style_count: semantic_resolution.style_count,
        import_edge_count: semantic_resolution.import_edge_count,
        resolved_import_edge_count: semantic_resolution.resolved_import_edge_count,
        unresolved_import_edge_count: semantic_resolution.unresolved_import_edge_count,
        matched_name_count: semantic_resolution.matched_name_count,
        edges,
        composes_closure_edge_count: composes_closure_edges.len(),
        value_closure_edge_count: value_closure_edges.len(),
        icss_closure_edge_count: icss_closure_edges.len(),
        composes_cycle_count: semantic_resolution.composes_cycle_count,
        value_cycle_count: semantic_resolution.value_cycle_count,
        icss_cycle_count: semantic_resolution.icss_cycle_count,
        composes_closure_edges,
        value_closure_edges,
        icss_closure_edges,
        cycles,
        capabilities: OmenaQueryCssModulesCrossFileResolutionCapabilitiesV0 {
            semantic_layer_owned: semantic_resolution.capabilities.semantic_layer_owned,
            import_source_resolution_ready: semantic_resolution
                .capabilities
                .import_source_resolution_ready,
            cross_file_resolution_ready: true,
            composes_closure_ready: semantic_resolution.capabilities.transitive_closure_ready,
            composes_name_match_ready: semantic_resolution.capabilities.composes_name_match_ready,
            value_name_match_ready: semantic_resolution.capabilities.value_name_match_ready,
            icss_name_match_ready: semantic_resolution.capabilities.icss_name_match_ready,
            transitive_closure_ready: semantic_resolution.capabilities.transitive_closure_ready,
            value_graph_closure_ready: semantic_resolution.capabilities.value_graph_closure_ready,
            icss_export_import_closure_ready: semantic_resolution
                .capabilities
                .icss_export_import_closure_ready,
            cycle_detection_ready: semantic_resolution.capabilities.cycle_detection_ready,
        },
        next_priorities: vec![],
    }
}

fn css_modules_cross_file_style_facts_for_query(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
) -> Vec<omena_semantic::CssModulesCrossFileStyleFactsV0> {
    style_fact_entries
        .iter()
        .map(css_modules_cross_file_style_fact_for_query)
        .collect()
}

fn css_modules_cross_file_style_fact_for_query(
    entry: &OmenaQueryStyleFactEntry,
) -> omena_semantic::CssModulesCrossFileStyleFactsV0 {
    omena_semantic::CssModulesCrossFileStyleFactsV0 {
        style_path: entry.style_path.clone(),
        class_selector_names: entry.facts.class_selector_names.clone(),
        css_module_value_definition_names: entry.facts.css_module_value_definition_names.clone(),
        css_module_value_import_edges: entry
            .facts
            .css_module_value_import_edges
            .iter()
            .map(|edge| omena_semantic::CssModulesValueImportEdgeFactV0 {
                remote_name: edge.remote_name.clone(),
                local_name: edge.local_name.clone(),
                import_source: edge.import_source.clone(),
            })
            .collect(),
        css_module_value_definition_edges: entry
            .facts
            .css_module_value_definition_edges
            .iter()
            .map(|edge| omena_semantic::CssModulesValueDefinitionEdgeFactV0 {
                definition_name: edge.definition_name.clone(),
                reference_names: edge.reference_names.clone(),
            })
            .collect(),
        css_module_composes_edges: entry
            .facts
            .css_module_composes_edges
            .iter()
            .map(|edge| omena_semantic::CssModulesComposesEdgeFactV0 {
                kind: edge.kind,
                owner_selector_names: edge.owner_selector_names.clone(),
                target_names: edge.target_names.clone(),
                import_source: edge.import_source.clone(),
            })
            .collect(),
        icss_export_names: entry.facts.icss_export_names.clone(),
        icss_import_edges: entry
            .facts
            .icss_import_edges
            .iter()
            .map(|edge| omena_semantic::CssModulesIcssImportEdgeFactV0 {
                local_name: edge.local_name.clone(),
                remote_name: edge.remote_name.clone(),
                import_source: edge.import_source.clone(),
            })
            .collect(),
        icss_export_edges: entry
            .facts
            .icss_export_edges
            .iter()
            .map(|edge| omena_semantic::CssModulesIcssExportEdgeFactV0 {
                export_name: edge.export_name.clone(),
                reference_names: edge.reference_names.clone(),
            })
            .collect(),
    }
}

fn semantic_package_manifests_for_query(
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaResolverStylePackageManifestV0> {
    package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect()
}

fn style_import_reachability_edges_for_query(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<omena_semantic::StyleImportReachabilityEdgeFactV0> {
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for entry in style_fact_entries {
        let targets = collect_style_module_dependency_sources_from_facts(&entry.facts)
            .into_iter()
            .filter_map(|source| {
                resolve_style_module_source(
                    entry.style_path.as_str(),
                    &source,
                    &available_style_paths,
                    package_manifests,
                )
            })
            .collect::<BTreeSet<_>>();
        for target in targets {
            edges.push(omena_semantic::StyleImportReachabilityEdgeFactV0 {
                from_style_path: entry.style_path.clone(),
                target_style_path: target,
            });
        }
    }
    edges
}

fn style_import_reachability_edges_from_module_interfaces(
    module_interfaces: &[OmenaQueryModuleInterfaceProjectionV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<omena_semantic::StyleImportReachabilityEdgeFactV0> {
    let available_style_paths = module_interfaces
        .iter()
        .map(|projection| projection.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for projection in module_interfaces {
        let targets = projection
            .style_dependency_sources
            .iter()
            .filter_map(|source| {
                resolve_style_module_source(
                    projection.style_path.as_str(),
                    source,
                    &available_style_paths,
                    package_manifests,
                )
            })
            .collect::<BTreeSet<_>>();
        for target in targets {
            edges.push(omena_semantic::StyleImportReachabilityEdgeFactV0 {
                from_style_path: projection.style_path.clone(),
                target_style_path: target,
            });
        }
    }
    edges
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssModulesComposesNode {
    style_path: String,
    selector_name: String,
}

fn collect_css_modules_composes_adjacency(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<CssModulesComposesNode, BTreeSet<CssModulesComposesNode>> {
    collect_css_modules_composes_adjacency_with_path_mappings(
        facts_by_path,
        available_style_paths,
        package_manifests,
        &[],
        &[],
        &[],
    )
}

fn collect_css_modules_composes_adjacency_with_path_mappings(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
) -> BTreeMap<CssModulesComposesNode, BTreeSet<CssModulesComposesNode>> {
    let mut graph = BTreeMap::new();
    for (style_path, facts) in facts_by_path {
        let class_names = facts
            .class_selector_names
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for edge in &facts.css_module_composes_edges {
            if edge.kind == "global" {
                continue;
            }
            let target_style_path = if edge.kind == "external" {
                edge.import_source.as_deref().and_then(|source| {
                    resolve_style_module_source_with_path_mappings(
                        style_path,
                        source,
                        available_style_paths,
                        package_manifests,
                        bundler_path_mappings,
                        tsconfig_path_mappings,
                        disk_style_path_identities,
                    )
                })
            } else {
                Some((*style_path).to_string())
            };
            let Some(target_style_path) = target_style_path else {
                continue;
            };
            let target_class_names = if target_style_path == *style_path {
                class_names.clone()
            } else {
                facts_by_path
                    .get(target_style_path.as_str())
                    .map(|facts| {
                        facts
                            .class_selector_names
                            .iter()
                            .map(String::as_str)
                            .collect::<BTreeSet<_>>()
                    })
                    .unwrap_or_default()
            };
            for owner_selector_name in &edge.owner_selector_names {
                if !class_names.contains(owner_selector_name.as_str()) {
                    continue;
                }
                let owner = CssModulesComposesNode {
                    style_path: (*style_path).to_string(),
                    selector_name: owner_selector_name.clone(),
                };
                for target_selector_name in &edge.target_names {
                    if !target_class_names.contains(target_selector_name.as_str()) {
                        continue;
                    }
                    graph
                        .entry(owner.clone())
                        .or_insert_with(BTreeSet::new)
                        .insert(CssModulesComposesNode {
                            style_path: target_style_path.clone(),
                            selector_name: target_selector_name.clone(),
                        });
                }
            }
        }
    }
    graph
}

fn filter_import_reachable_design_token_workspace_declarations(
    target_style_path: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<DesignTokenWorkspaceDeclarationFactV0> {
    let reachable_style_paths = collect_import_reachable_style_path_metadata(
        target_style_path,
        style_fact_entries,
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
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<String, ImportReachability> {
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for entry in style_fact_entries {
        let targets = collect_style_module_dependency_sources_from_facts(&entry.facts)
            .into_iter()
            .filter_map(|source| {
                resolve_style_module_source(
                    entry.style_path.as_str(),
                    &source,
                    &available_style_paths,
                    package_manifests,
                )
            })
            .collect::<BTreeSet<_>>();
        for target in targets {
            edges.push(omena_semantic::StyleImportReachabilityEdgeFactV0 {
                from_style_path: entry.style_path.clone(),
                target_style_path: target,
            });
        }
    }

    omena_semantic::summarize_style_import_reachability(target_style_path, edges.as_slice())
        .reachable_style_paths
        .into_iter()
        .map(|fact| {
            (
                fact.style_path,
                ImportReachability {
                    distance: fact.distance,
                    order: fact.order,
                },
            )
        })
        .collect()
}

fn collect_style_module_dependency_sources_from_facts(
    facts: &OmenaQueryOmenaParserStyleFactsV0,
) -> Vec<String> {
    let mut sources = facts
        .sass_module_edges
        .iter()
        .map(|edge| edge.source.clone())
        .collect::<Vec<_>>();
    sources.extend(
        facts
            .css_module_value_import_edges
            .iter()
            .map(|edge| edge.import_source.clone()),
    );
    sources.extend(
        facts
            .css_module_composes_edges
            .iter()
            .filter_map(|edge| edge.import_source.clone()),
    );
    sources.extend(
        facts
            .icss_import_edges
            .iter()
            .map(|edge| edge.import_source.clone()),
    );
    sources.sort();
    sources.dedup();
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

/// Alias-aware style-module resolution: the same routing as `resolve_style_module_source`, plus
/// tsconfig/bundler path-mapping resolution so a workspace-alias specifier (`@/styles/a.module.scss`)
/// resolves when the workspace's `paths`/`alias` config is wired in. RFC-0007-J (#50): the
/// unused-selector usage collector must use this so it agrees with the reference/goto path, which
/// already resolves aliases — otherwise an alias import leaves every selector dimmed `unusedSelector`.
fn resolve_style_module_source_with_path_mappings(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
) -> Option<String> {
    resolve_style_module_source_with_path_mappings_and_identity_index(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        disk_style_path_identities,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn resolve_style_module_source_with_path_mappings_and_identity_index(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    identity_index: Option<&OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
) -> Option<String> {
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    summarize_omena_resolver_style_module_resolution_with_confirmation_inputs(
        from_style_path,
        source,
        available_style_paths,
        disk_style_path_identities,
        &resolver_package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        &[],
        OmenaResolverStyleModuleConfirmationOptionsV0 {
            allow_disk_confirmation: true,
            identity_index,
            ..OmenaResolverStyleModuleConfirmationOptionsV0::default()
        },
    )
    .resolved_style_path
}

fn collect_style_selector_hover_candidates_from_omena_parser_facts(
    source: &str,
    definition_facts: &[ParsedSelectorFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in definition_facts {
        if fact.kind != ParsedSelectorFactKind::Class {
            continue;
        }
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if seen.insert((byte_span.start, byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: "selector",
                name: fact.name.clone(),
                range: parser_range_for_byte_span(source, byte_span),
                source: "omenaParserSelectorFacts",
                namespace: None,
            });
        }
    }
}

fn collect_custom_property_hover_candidates_from_omena_parser_facts(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in variable_facts {
        let kind = match fact.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration => "customPropertyDeclaration",
            ParsedVariableFactKind::CustomPropertyReference => "customPropertyReference",
            _ => continue,
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if seen.insert((byte_span.start, byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind,
                name: fact.name.clone(),
                range: parser_range_for_byte_span(source, byte_span),
                source: "omenaParserVariableFacts",
                namespace: None,
            });
        }
    }
}

fn collect_sass_symbol_hover_candidates_from_omena_parser_facts(
    source: &str,
    symbol_facts: &[omena_parser::ParsedSassSymbolFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in symbol_facts {
        let kind = match fact.kind {
            ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::MixinDeclaration
            | ParsedSassSymbolFactKind::FunctionDeclaration => {
                sass_symbol_declaration_candidate_kind(fact.symbol_kind)
            }
            ParsedSassSymbolFactKind::VariableReference
            | ParsedSassSymbolFactKind::MixinInclude
            | ParsedSassSymbolFactKind::FunctionCall => {
                sass_symbol_reference_candidate_kind(fact.symbol_kind, fact.role)
            }
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if seen.insert((
            byte_span.start,
            byte_span.end,
            format!(
                "{}:{}:{}",
                fact.symbol_kind,
                fact.namespace.as_deref().unwrap_or_default(),
                fact.name
            ),
        )) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind,
                name: fact.name.clone(),
                range: parser_range_for_byte_span(source, byte_span),
                source: "omenaParserSassSymbolFacts",
                namespace: fact.namespace.clone(),
            });
        }
    }
}

fn collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts(
    source: &str,
    includes: &[ParsedSassIncludeFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for include in includes {
        let start: u32 = include.range.start().into();
        let end: u32 = include.range.end().into();
        let range_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        for selector_name in infer_sass_include_generated_selector_names(&include.params) {
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

fn style_completion_context_at_position(
    source: &str,
    position: ParserPositionV0,
) -> Option<(&'static str, Option<String>)> {
    let cursor = byte_offset_for_parser_position(source, position)?;
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let line_prefix = source.get(line_start..cursor)?;
    if let Some(var_start) = line_prefix.rfind("var(") {
        let var_prefix = &line_prefix[var_start + "var(".len()..];
        if !var_prefix.contains(')') {
            let prefix = var_prefix
                .rsplit(|ch: char| ch == ',' || ch.is_ascii_whitespace())
                .next()
                .unwrap_or_default();
            let prefix = (!prefix.is_empty()).then(|| prefix.to_string());
            return Some(("styleCustomPropertyReference", prefix));
        }
    }
    if let Some(prefix) = sass_variable_completion_prefix(line_prefix) {
        return Some(("sassVariableReference", Some(prefix)));
    }
    if let Some(prefix) = sass_mixin_completion_prefix(line_prefix) {
        return Some(("sassMixinReference", prefix));
    }
    if let Some(prefix) = sass_member_completion_prefix(line_prefix) {
        return Some(("sassMemberReference", Some(prefix)));
    }

    Some(("styleDocument", None))
}

fn sass_variable_completion_prefix(line_prefix: &str) -> Option<String> {
    let token = sass_completion_trailing_token(line_prefix)?;
    let dollar_offset = token.rfind('$')?;
    let suffix = token.get(dollar_offset + 1..)?;
    if !suffix.chars().all(is_sass_completion_identifier_continue) {
        return None;
    }
    let prefix = token.get(..)?;
    (!prefix.is_empty()).then(|| prefix.to_string())
}

fn sass_mixin_completion_prefix(line_prefix: &str) -> Option<Option<String>> {
    let include_offset = line_prefix.rfind("@include")?;
    let after_include = line_prefix.get(include_offset + "@include".len()..)?;
    if after_include.contains(';') || after_include.contains('{') || after_include.contains('}') {
        return None;
    }
    let token = sass_completion_trailing_token(after_include.trim_start())?;
    if token.contains('$') || !token.chars().all(is_sass_completion_member_continue) {
        return None;
    }
    Some((!token.is_empty()).then(|| token.to_string()))
}

fn sass_member_completion_prefix(line_prefix: &str) -> Option<String> {
    let token = sass_completion_trailing_token(line_prefix)?;
    if token.starts_with('.') || token.contains('$') || !token.contains('.') {
        return None;
    }
    if !token.chars().all(is_sass_completion_member_continue) {
        return None;
    }
    let (namespace, _) = token.split_once('.')?;
    (!namespace.is_empty()).then(|| token.to_string())
}

fn sass_completion_trailing_token(text: &str) -> Option<&str> {
    text.rsplit(|ch: char| {
        ch.is_ascii_whitespace()
            || matches!(ch, ':' | ';' | '{' | '}' | '(' | ')' | ',' | '[' | ']')
    })
    .next()
    .filter(|token| !token.is_empty())
}

fn is_sass_completion_identifier_continue(ch: char) -> bool {
    is_css_identifier_continue(ch)
}

fn is_sass_completion_member_continue(ch: char) -> bool {
    is_css_identifier_continue(ch) || ch == '.' || ch == '$'
}

fn trim_hover_snippet(snippet: &str) -> String {
    const MAX_SNIPPET_LEN: usize = 1200;
    if snippet.len() <= MAX_SNIPPET_LEN {
        return snippet.to_string();
    }
    let end = char_boundary_floor(snippet, MAX_SNIPPET_LEN);
    format!("{}...", snippet[..end].trim_end())
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

fn push_omena_query_ready_surface(ready_surfaces: &mut Vec<&'static str>, surface: &'static str) {
    if !ready_surfaces.contains(&surface) {
        ready_surfaces.push(surface);
    }
}

fn end_of_source_range(source: &str) -> ParserRangeV0 {
    let position = parser_position_for_byte_offset(source, source.len());
    ParserRangeV0 {
        start: position,
        end: position,
    }
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

fn is_sass_builtin_module_source(source: &str) -> bool {
    source.starts_with("sass:")
}

fn format_query_sass_symbol_label(symbol_kind: &str, name: &str) -> String {
    match symbol_kind {
        "variable" => format!("Sass variable '${name}'"),
        "mixin" => format!("Sass mixin '@mixin {name}'"),
        "function" => format!("Sass function '{name}()'"),
        _ => format!("Sass symbol '{name}'"),
    }
}

#[cfg(test)]
mod runtime_index_tests {
    use super::*;

    #[test]
    fn semantic_runtime_index_from_query_facts_matches_source_parser() {
        let style_path = "/workspace/src/App.module.scss";
        let style_source = r#"
@keyframes fade { to { opacity: 1; } }
.card {
  --brand: red;
  color: var(--brand);
  animation: fade 1s;
}
"#;
        let facts = summarize_omena_query_omena_parser_style_facts(
            style_source,
            omena_parser_dialect_for_style_path(style_path),
        );

        assert_eq!(
            semantic_runtime_index_from_query_style_facts(style_path, &facts),
            omena_semantic::summarize_style_runtime_index_facts_from_source(
                style_path,
                style_source,
            ),
        );
    }
}
