use crate::protocol::{file_uri_to_path, is_css_identifier_continue};
use crate::{
    LspShellState, LspTextDocumentState, ensure_style_document_loaded_from_disk,
    parser_range_for_byte_span, source_selector_candidates_from_index,
};
use omena_query::{
    OmenaQueryEngineInputV2,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceTypeFactTargetV0 as SourceTypeFactTarget, ParserByteSpanV0,
    canonicalize_omena_query_source_selector_references,
    summarize_omena_query_expression_domain_selector_projection,
};
use omena_tsgo_client::{
    TsgoJsonRpcTypeFactProviderV0, TsgoResolvedTypeV0, TsgoTypeFactRequestV0,
    TsgoTypeFactResultEntryV0, TsgoTypeFactTargetV0, build_tsgo_process_command,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) fn refresh_source_type_fact_candidates_for_document(
    state: &mut LspShellState,
    uri: &str,
) {
    let Some(document) = state.document(uri) else {
        return;
    };
    if crate::protocol::is_style_document_uri(document.uri.as_str()) {
        return;
    }
    let type_fact_targets = document.source_syntax_index.type_fact_targets.clone();
    if type_fact_targets.is_empty() {
        return;
    }
    let Some(request) = tsgo_type_fact_request_for_document(document, type_fact_targets.as_slice())
    else {
        return;
    };
    let Some(tsgo_command) = tsgo_process_command_for_workspace(request.workspace_root.as_str())
    else {
        return;
    };
    let config = omena_tsgo_client::TsgoWorkspaceProcessConfigV0 {
        workspace_root: request.workspace_root.clone(),
        command: tsgo_command,
    };
    if state
        .tsgo_workspace_process_pool
        .ensure_workspace_process(config)
        .is_err()
    {
        return;
    }

    let pool = std::mem::take(&mut state.tsgo_workspace_process_pool);
    let mut provider = TsgoJsonRpcTypeFactProviderV0::new(pool);
    let entries = provider.collect_type_facts(&request).ok();
    state.tsgo_workspace_process_pool = provider.into_transport();
    let Some(entries) = entries else {
        return;
    };
    apply_source_type_fact_results_to_document(state, uri, entries.as_slice());
}

fn tsgo_type_fact_request_for_document(
    document: &LspTextDocumentState,
    type_fact_targets: &[SourceTypeFactTarget],
) -> Option<TsgoTypeFactRequestV0> {
    let file_path = file_uri_to_path(document.uri.as_str())?;
    let workspace_root = document
        .workspace_folder_uri
        .as_deref()
        .and_then(file_uri_to_path)
        .or_else(|| file_path.parent().map(Path::to_path_buf))?;
    let config_path = find_tsconfig_for_workspace(workspace_root.as_path())?;
    let file_path = file_path.to_string_lossy().to_string();
    let targets = type_fact_targets
        .iter()
        .filter_map(|target| {
            let position =
                utf16_position_for_byte_offset(document.text.as_str(), target.byte_span.start)?;
            Some(TsgoTypeFactTargetV0 {
                file_path: file_path.clone(),
                expression_id: target.expression_id.clone(),
                position,
            })
        })
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return None;
    }
    Some(TsgoTypeFactRequestV0 {
        workspace_root: workspace_root.to_string_lossy().to_string(),
        config_path: config_path.to_string_lossy().to_string(),
        targets,
    })
}

pub(crate) fn apply_source_type_fact_results_to_document(
    state: &mut LspShellState,
    uri: &str,
    entries: &[TsgoTypeFactResultEntryV0],
) {
    let Some(document) = state.document(uri).cloned() else {
        return;
    };
    let mut references = document.source_syntax_index.selector_references.clone();
    let targets = document.source_syntax_index.type_fact_targets.clone();
    ensure_referenced_style_documents_loaded_for_type_facts(state, targets.as_slice());
    for (target, selector_name) in
        project_source_type_fact_targets_with_query(state, &document, targets.as_slice(), entries)
    {
        push_selector_reference(
            target.byte_span,
            Some(selector_name),
            SourceSelectorReferenceMatchKind::Exact,
            target.target_style_uri.as_deref(),
            &mut references,
        );
    }
    canonicalize_omena_query_source_selector_references(&mut references);
    let Some(document) = state.document_mut(uri) else {
        return;
    };
    document.source_syntax_index.selector_references = references;
    let source_syntax_index = document.source_syntax_index.clone();
    document.source_selector_candidates =
        source_selector_candidates_from_index(document, &source_syntax_index);
}

fn project_source_type_fact_targets_with_query(
    state: &LspShellState,
    document: &LspTextDocumentState,
    targets: &[SourceTypeFactTarget],
    entries: &[TsgoTypeFactResultEntryV0],
) -> Vec<(SourceTypeFactTarget, String)> {
    let Some(input) = query_engine_input_for_source_type_facts(state, document, targets, entries)
    else {
        return Vec::new();
    };
    let projection = summarize_omena_query_expression_domain_selector_projection(&input);
    let targets_by_id = targets
        .iter()
        .cloned()
        .map(|target| (target.expression_id.clone(), target))
        .collect::<BTreeMap<_, _>>();
    let mut projected = Vec::new();
    for entry in projection.projections {
        let Some(target) = targets_by_id.get(entry.node_id.as_str()) else {
            continue;
        };
        for selector_name in entry.selector_names {
            projected.push((target.clone(), selector_name));
        }
    }
    projected.sort_by(|left, right| {
        (
            left.0.expression_id.as_str(),
            left.0.byte_span.start,
            left.1.as_str(),
        )
            .cmp(&(
                right.0.expression_id.as_str(),
                right.0.byte_span.start,
                right.1.as_str(),
            ))
    });
    projected.dedup_by(|left, right| {
        left.0.expression_id == right.0.expression_id
            && left.0.byte_span == right.0.byte_span
            && left.1 == right.1
    });
    projected
}

fn query_engine_input_for_source_type_facts(
    state: &LspShellState,
    document: &LspTextDocumentState,
    targets: &[SourceTypeFactTarget],
    entries: &[TsgoTypeFactResultEntryV0],
) -> Option<OmenaQueryEngineInputV2> {
    let entries_by_id = entries
        .iter()
        .map(|entry| (entry.expression_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let class_expressions = targets
        .iter()
        .filter_map(|target| {
            let target_style_uri = target.target_style_uri.as_ref()?;
            Some(json!({
                "id": target.expression_id,
                "kind": "symbolRef",
                "scssModulePath": target_style_uri,
                "range": parser_range_for_byte_span(document.text.as_str(), target.byte_span),
                "className": null,
                "rootBindingDeclId": null,
                "accessPath": null,
            }))
        })
        .collect::<Vec<_>>();
    let styles = state
        .documents
        .values()
        .filter(|style_document| {
            crate::protocol::is_style_document_uri(style_document.uri.as_str())
        })
        .map(|style_document| {
            let selectors = style_document
                .style_candidates
                .iter()
                .filter(|candidate| candidate.kind == "selector")
                .map(|candidate| {
                    json!({
                        "name": candidate.name,
                        "viewKind": "canonical",
                        "canonicalName": candidate.name,
                        "range": candidate.range,
                        "nestedSafety": null,
                        "composes": null,
                        "bemSuffix": null,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "filePath": style_document.uri,
                "source": style_document.text,
                "document": {
                    "selectors": selectors,
                },
            })
        })
        .collect::<Vec<_>>();
    let source_file_path = file_uri_to_path(document.uri.as_str())
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| document.uri.clone());
    let type_facts = targets
        .iter()
        .filter_map(|target| {
            let entry = entries_by_id.get(target.expression_id.as_str())?;
            let values = project_tsgo_type_fact_target(entry.resolved_type.clone(), target);
            if values.is_empty() {
                return None;
            }
            Some(json!({
                "filePath": source_file_path,
                "expressionId": target.expression_id,
                "facts": {
                    "kind": "finiteSet",
                    "constraintKind": null,
                    "values": values,
                    "prefix": null,
                    "suffix": null,
                    "minLen": null,
                    "maxLen": null,
                    "charMust": null,
                    "charMay": null,
                    "mayIncludeOtherChars": null,
                },
            }))
        })
        .collect::<Vec<_>>();
    if type_facts.is_empty() {
        return None;
    }
    serde_json::from_value(json!({
        "version": "2",
        "sources": [{
            "document": {
                "classExpressions": class_expressions,
            },
        }],
        "styles": styles,
        "typeFacts": type_facts,
    }))
    .ok()
}

fn project_tsgo_type_fact_target(
    resolved_type: TsgoResolvedTypeV0,
    target: &SourceTypeFactTarget,
) -> Vec<String> {
    if resolved_type.kind != "union" {
        return Vec::new();
    }
    let mut names = resolved_type
        .values
        .into_iter()
        .filter(|value| value.chars().all(is_css_identifier_continue))
        .map(|value| format!("{}{}{}", target.prefix, value, target.suffix))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    names
}

fn ensure_referenced_style_documents_loaded_for_type_facts(
    state: &mut LspShellState,
    targets: &[SourceTypeFactTarget],
) {
    let mut referenced_style_uris = targets
        .iter()
        .filter_map(|target| target.target_style_uri.clone())
        .collect::<Vec<_>>();
    referenced_style_uris.sort();
    referenced_style_uris.dedup();
    for style_uri in referenced_style_uris {
        ensure_style_document_loaded_from_disk(state, style_uri.as_str());
    }
}

fn utf16_position_for_byte_offset(source: &str, byte_offset: usize) -> Option<u32> {
    let prefix = source.get(..byte_offset)?;
    let position = prefix.chars().map(char::len_utf16).sum::<usize>();
    u32::try_from(position).ok()
}

fn push_selector_reference(
    byte_span: ParserByteSpanV0,
    selector_name: Option<String>,
    match_kind: SourceSelectorReferenceMatchKind,
    target_style_uri: Option<&str>,
    references: &mut Vec<SourceSelectorReferenceFact>,
) {
    references.push(SourceSelectorReferenceFact {
        byte_span,
        selector_name,
        match_kind,
        target_style_uri: target_style_uri.map(ToString::to_string),
    });
}

fn find_tsconfig_for_workspace(workspace_root: &Path) -> Option<PathBuf> {
    let mut current = Some(workspace_root);
    while let Some(dir) = current {
        for file_name in ["tsconfig.json", "jsconfig.json"] {
            let candidate = dir.join(file_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        current = dir.parent();
    }
    None
}

fn tsgo_process_command_for_workspace(
    workspace_root: &str,
) -> Option<omena_tsgo_client::TsgoProcessCommandV0> {
    let tsgo_path = resolve_tsgo_binary_path()?;
    Some(build_tsgo_process_command(
        tsgo_path.to_string_lossy().as_ref(),
        workspace_root,
        std::env::var("CME_TSGO_CHECKERS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok()),
    ))
}

fn resolve_tsgo_binary_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CME_TSGO_PATH")
        && !path.is_empty()
    {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    let binary_name = if cfg!(windows) { "tsgo.exe" } else { "tsgo" };
    let sibling = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(binary_name)));
    if let Some(path) = sibling
        && path.exists()
    {
        return Some(path);
    }
    None
}
