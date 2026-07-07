use super::*;
use crate::occurrence_mapping::{
    style_symbol_occurrence_for_candidate, workspace_occurrence_from_style_symbol_occurrence,
    workspace_occurrence_matches_target_style,
};
use crate::style_hover_markdown::render_style_hover_candidate_markdown_from_parts;
use crate::style_symbol_monikers::{
    style_custom_property_moniker, style_external_sif_sass_symbol_moniker,
    style_sass_symbol_moniker_for_document, style_sass_symbol_moniker_for_uri,
    style_symbol_monikers_for_candidate, style_symbol_role_for_candidate,
    style_unresolved_sass_symbol_moniker,
};
use crate::workspace_occurrence_cache::{
    load_workspace_occurrence_shard, store_workspace_occurrence_shard,
};
use crate::workspace_occurrences::{
    source_selector_occurrence_index_from_open_documents,
    workspace_occurrence_indexes_from_documents,
};
use omena_query::{StyleLanguage, summarize_omena_query_sass_module_sources};
use std::collections::BTreeMap;

pub(crate) fn selector_reference_locations_from_open_documents(
    state: &LspShellState,
    selector_name: &str,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> Vec<Value> {
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    summarize_omena_query_refs_for_class_from_occurrence_index(
        selector_name,
        query_target_style_uri.as_deref(),
        false,
        occurrence_index.definitions.as_slice(),
        &occurrence_index.source_selector_index,
    )
    .locations
    .into_iter()
    .map(|location| json!({ "uri": location.uri, "range": location.range }))
    .collect()
}

pub(crate) fn selector_reference_locations_by_name_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> BTreeMap<String, Vec<Value>> {
    let mut locations_by_name: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    for occurrence in occurrence_index
        .workspace_index
        .by_moniker
        .values()
        .flat_map(|occurrences| occurrences.iter())
    {
        if occurrence.family != Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector)
            || !workspace_occurrence_matches_target_style(
                occurrence,
                query_target_style_uri.as_deref(),
            )
        {
            continue;
        }
        locations_by_name
            .entry(occurrence.name.clone())
            .or_default()
            .push(json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            }));
    }
    for locations in locations_by_name.values_mut() {
        locations.sort_by_key(location_sort_key);
        locations
            .dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    }
    locations_by_name
}

pub(crate) fn style_symbol_definition_locations_from_documents(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<Value> {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut locations = occurrences_for_monikers(&occurrence_index, &monikers)
        .into_iter()
        .filter(|occurrence| occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Definition)
        .map(|occurrence| {
            json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            })
        })
        .collect::<Vec<_>>();
    locations.sort_by_key(location_sort_key);
    locations.dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    locations
}

pub(crate) fn style_symbol_reference_locations_from_documents(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    include_declaration: bool,
) -> Vec<Value> {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut locations = occurrences_for_monikers(&occurrence_index, &monikers)
        .into_iter()
        .filter(|occurrence| {
            occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Reference
                || (include_declaration
                    && occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Definition)
        })
        .map(|occurrence| {
            json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            })
        })
        .collect::<Vec<_>>();
    if include_declaration && is_sass_symbol_candidate_kind(candidate.kind) {
        locations.extend(
            sass_symbol_definitions_for_candidate(state, document, candidate)
                .into_iter()
                .map(|(uri, definition)| {
                    json!({
                        "uri": uri,
                        "range": definition.range,
                    })
                }),
        );
    }
    locations.sort_by_key(location_sort_key);
    locations.dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    locations
}

pub(crate) fn resolve_style_symbol_rename(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    new_name: &str,
) -> Value {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut seen = BTreeSet::new();
    let mut changes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for occurrence in occurrences_for_monikers(&occurrence_index, &monikers) {
        if !occurrence.rename_target {
            continue;
        }
        let key = (
            occurrence.uri.clone(),
            occurrence.range.start.line,
            occurrence.range.start.character,
            occurrence.range.end.line,
            occurrence.range.end.character,
        );
        if !seen.insert(key) {
            continue;
        }
        let edit_uri = external_document_uri_for_query_uri(state, occurrence.uri.as_str());
        changes.entry(edit_uri).or_default().push(json!({
            "range": occurrence.range,
            "newText": new_name,
        }));
    }

    if changes.is_empty() {
        return Value::Null;
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(lsp_range_start_sort_key);
    }
    json!({
        "changes": Value::Object(changes.into_iter().map(|(uri, edits)| (uri, json!(edits))).collect()),
    })
}

fn style_symbol_occurrence_index_from_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Arc<OmenaWorkspaceOccurrenceIndexV0> {
    workspace_occurrence_indexes_from_documents(state, workspace_folder_uri).workspace_index
}

pub(crate) fn style_symbol_workspace_occurrences_for_document(
    state: &LspShellState,
    document: &LspTextDocumentState,
    workspace_folder_uri: Option<&str>,
    dependency_digest: Option<&str>,
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    if let Some(shard) = load_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
    ) {
        return shard.occurrences;
    }

    let mut style_occurrences = Vec::new();
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Vec::new();
    };
    for candidate in candidates {
        if candidate.kind.starts_with("customProperty") {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_custom_property_moniker(workspace_folder_uri, candidate.name.as_str()),
                document.uri.as_str(),
                &candidate,
                "customProperty",
                style_symbol_role_for_candidate(&candidate),
            ));
            continue;
        }
        if !is_sass_symbol_candidate_kind(candidate.kind) {
            continue;
        }
        if is_sass_symbol_declaration_kind(candidate.kind) {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_sass_symbol_moniker_for_document(state, document, &candidate),
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "definition",
            ));
            continue;
        }
        let definitions = sass_symbol_definitions_for_candidate(state, document, &candidate);
        if definitions.is_empty() {
            let moniker = if let Some(target) =
                external_sif_sass_symbol_target_for_candidate(state, document, &candidate)
            {
                style_external_sif_sass_symbol_moniker(&target)
            } else {
                style_unresolved_sass_symbol_moniker(workspace_folder_uri, &candidate)
            };
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                moniker,
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "reference",
            ));
            continue;
        }
        for (definition_uri, definition) in definitions {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_sass_symbol_moniker_for_uri(state, definition_uri.as_str(), &definition),
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "reference",
            ));
        }
    }
    style_occurrences.sort();
    style_occurrences.dedup();
    let workspace_occurrences = style_occurrences
        .iter()
        .map(|occurrence| workspace_occurrence_from_style_symbol_occurrence(document, occurrence))
        .collect::<Vec<_>>();
    store_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
        workspace_occurrences.as_slice(),
    );
    workspace_occurrences
}

pub(crate) fn source_candidate_selector_names(
    candidate: &LspStyleHoverCandidate,
    definitions: &[(String, LspStyleHoverCandidate)],
    target_style_uri: Option<&str>,
) -> Vec<String> {
    let query_definitions = definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect::<Vec<_>>();
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    resolve_omena_query_source_candidate_selector_names(
        &query_source_selector_candidate_for_matching(candidate),
        query_definitions.as_slice(),
        query_target_style_uri.as_deref(),
    )
}

pub(crate) fn sass_symbol_definitions_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let Some(symbol_kind) = sass_symbol_kind_from_candidate_kind(candidate.kind) else {
        return Vec::new();
    };
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return vec![(document.uri.clone(), candidate.clone())];
    }

    let mut definitions = if candidate.namespace.is_none() {
        sass_symbol_declarations_in_document(document, symbol_kind, candidate)
    } else {
        Vec::new()
    };
    if candidate.namespace.is_none() && !definitions.is_empty() {
        return definitions;
    }

    for target_uri in sass_module_target_uris_for_candidate(state, document, candidate) {
        definitions.extend(sass_symbol_declarations_for_uri(
            state,
            target_uri.as_str(),
            symbol_kind,
            candidate,
        ));
    }
    definitions.sort_by_key(|(uri, target)| {
        (
            uri.clone(),
            target.range.start.line,
            target.range.start.character,
        )
    });
    definitions.dedup_by(|left, right| {
        left.0 == right.0
            && left.1.kind == right.1.kind
            && left.1.name == right.1.name
            && left.1.range == right.1.range
    });
    definitions
}

fn sass_symbol_declarations_for_uri(
    state: &LspShellState,
    target_uri: &str,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    sass_symbol_declarations_for_uri_with_visited(
        state,
        target_uri,
        symbol_kind,
        candidate,
        &mut BTreeSet::new(),
    )
}

fn sass_symbol_declarations_for_uri_with_visited(
    state: &LspShellState,
    target_uri: &str,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
    visited: &mut BTreeSet<String>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if let Some(target_document) = state.document(target_uri) {
        return sass_symbol_declarations_with_forwards(
            state,
            target_document,
            symbol_kind,
            candidate,
            visited,
        );
    }
    let Some(target_document) = style_document_from_disk_for_uri(state, target_uri) else {
        return Vec::new();
    };
    sass_symbol_declarations_with_forwards(state, &target_document, symbol_kind, candidate, visited)
}

pub(crate) fn style_document_from_disk_for_uri(
    state: &LspShellState,
    uri: &str,
) -> Option<LspTextDocumentState> {
    let text = style_text_for_uri(state, uri)?;
    let workspace_folder_uri = resolve_workspace_folder_uri(state, uri);
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    Some(lsp_text_document_state(
        uri.to_string(),
        workspace_folder_uri,
        StyleLanguage::from_module_path(uri)
            .map(style_language_label)
            .unwrap_or("unknown")
            .to_string(),
        0,
        text,
        &resolution_inputs,
    ))
}

fn sass_symbol_declarations_in_document(
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let query_candidates = document
        .style_candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    resolve_omena_query_sass_symbol_declarations(
        query_candidates.as_slice(),
        symbol_kind,
        candidate.name.as_str(),
    )
    .into_iter()
    .map(lsp_style_hover_candidate_from_query)
    .map(|target| (document.uri.clone(), target))
    .collect()
}

pub(crate) fn sass_module_target_uris_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<String> {
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(
        &sources,
        candidate.namespace.as_deref(),
    ) {
        if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source.as_str()) {
            uris.push(uri);
        }
    }
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        if let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_source.as_str())
        {
            uris.push(uri.clone());
            if let Some(target_document) = state.document(uri.as_str()) {
                uris.extend(sass_forward_module_target_uris(
                    state,
                    target_document,
                    &mut BTreeSet::new(),
                ));
            }
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn sass_symbol_declarations_with_forwards(
    state: &LspShellState,
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
    visited: &mut BTreeSet<String>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let mut definitions = sass_symbol_declarations_in_document(document, symbol_kind, candidate);
    if summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
        .is_none()
    {
        return definitions;
    }
    for forward_edge in sass_forward_edges_for_document(document) {
        let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_edge.source.as_str())
        else {
            continue;
        };
        let Some(target_candidate) =
            forward_edge.private_candidate_for_forwarded_public_candidate(candidate)
        else {
            continue;
        };
        definitions.extend(sass_symbol_declarations_for_uri_with_visited(
            state,
            uri.as_str(),
            symbol_kind,
            &target_candidate,
            visited,
        ));
    }
    definitions
}

pub(crate) fn sass_forward_module_target_uris(
    state: &LspShellState,
    document: &LspTextDocumentState,
    visited: &mut BTreeSet<String>,
) -> Vec<String> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        if let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_source.as_str())
        {
            uris.push(uri.clone());
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SassForwardEdgeForLsp {
    pub(crate) source: String,
    forward_prefix: Option<String>,
}

impl SassForwardEdgeForLsp {
    pub(crate) fn private_candidate_for_forwarded_public_candidate(
        &self,
        candidate: &LspStyleHoverCandidate,
    ) -> Option<LspStyleHoverCandidate> {
        let private_name =
            unapply_sass_forward_prefix(self.forward_prefix.as_deref(), &candidate.name)?;
        let mut target = candidate.clone();
        target.name = private_name;
        target.namespace = None;
        Some(target)
    }
}

pub(crate) fn sass_forward_edges_for_document(
    document: &LspTextDocumentState,
) -> Vec<SassForwardEdgeForLsp> {
    let facts = summarize_omena_query_omena_parser_style_facts(
        document.text.as_str(),
        query_style_dialect_for_uri(document.uri.as_str()),
    );
    facts
        .sass_module_edges
        .into_iter()
        .filter(|edge| edge.kind == "sassForward")
        .map(|edge| SassForwardEdgeForLsp {
            source: edge.source,
            forward_prefix: edge.forward_prefix,
        })
        .collect()
}

pub(crate) fn unapply_sass_forward_prefix(
    prefix: Option<&str>,
    exposed_name: &str,
) -> Option<String> {
    let Some(prefix) = prefix else {
        return Some(exposed_name.to_string());
    };
    if let Some(star_offset) = prefix.find('*') {
        let before = prefix.get(..star_offset).unwrap_or_default();
        let after = prefix
            .get(star_offset + '*'.len_utf8()..)
            .unwrap_or_default();
        let without_before = exposed_name.strip_prefix(before)?;
        let without_after = if after.is_empty() {
            without_before
        } else {
            without_before.strip_suffix(after)?
        };
        return Some(without_after.to_string());
    }
    exposed_name
        .strip_prefix(prefix)
        .map(str::to_string)
        .filter(|name| !name.is_empty())
}

pub(crate) fn reference_lens_title(count: usize) -> String {
    if count == 1 {
        "1 reference".to_string()
    } else {
        format!("{count} references")
    }
}

pub(crate) fn resolve_selector_rename(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
    selector_name: &str,
    new_name: &str,
) -> Value {
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    let rename_plan = summarize_omena_query_rename_plan_from_occurrence_index(
        selector_name,
        new_name,
        query_target_style_uri.as_deref(),
        occurrence_index.definitions.as_slice(),
        &occurrence_index.source_selector_index,
    );
    if rename_plan.edits.is_empty() {
        return Value::Null;
    }

    let mut changes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for edit in rename_plan.edits {
        let edit_uri = external_document_uri_for_query_uri(state, edit.uri.as_str());
        changes.entry(edit_uri).or_default().push(json!({
            "range": edit.range,
            "newText": edit.new_text,
        }));
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(|edit| {
            let line = edit
                .pointer("/range/start/line")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let character = edit
                .pointer("/range/start/character")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            (line, character)
        });
    }

    let mut response_changes = serde_json::Map::new();
    for (uri, edits) in changes {
        response_changes.insert(uri, json!(edits));
    }
    json!({
        "changes": Value::Object(response_changes),
    })
}

pub(crate) fn external_document_uri_for_query_uri(state: &LspShellState, uri: &str) -> String {
    state
        .document(uri)
        .map(|document| document.uri.clone())
        .unwrap_or_else(|| uri.to_string())
}

pub(crate) fn render_style_hover_candidate_markdown_for_workspace(
    state: &LspShellState,
    document_uri: &str,
    source: &str,
    candidate: &LspStyleHoverCandidate,
) -> String {
    let workspace_folder_uri = state
        .document(document_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, document_uri));
    let style_sources = style_sources_for_hover_render(
        state,
        workspace_folder_uri.as_deref(),
        document_uri,
        source,
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    let narrowing_substrate = cascade_narrowing_substrate_for_style_sources(
        state,
        style_sources.as_slice(),
        &resolution_inputs,
    );
    let render_parts =
        summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position_with_substrate(
            document_uri,
            style_sources.as_slice(),
            &narrowing_substrate,
            candidate.kind,
            candidate.name.as_str(),
            candidate.range.start,
        )
        .unwrap_or_else(|| {
            summarize_omena_query_style_hover_render_parts_for_hover_position(
                source,
                candidate.kind,
                candidate.name.as_str(),
                candidate.range.start,
            )
        });
    render_style_hover_candidate_markdown_from_parts(document_uri, candidate, &render_parts)
}
