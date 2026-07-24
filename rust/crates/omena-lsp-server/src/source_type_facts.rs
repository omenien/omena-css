use crate::protocol::{file_uri_to_path, is_css_identifier_continue, workspace_folder_compatible};
use crate::source_type_fact_cache::{
    load_source_type_fact_sidecar, store_source_type_fact_sidecar,
};
use crate::{
    LspShellState, LspTextDocumentState, ensure_style_document_loaded_from_disk,
    parser_range_for_byte_span, source_selector_candidates_from_index,
};
use omena_query::{
    OmenaQueryEngineInputV2,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSelectorReferenceSurfaceV0 as SourceSelectorReferenceSurface,
    OmenaQuerySourceTypeFactProviderUnavailableFactV0 as SourceTypeFactProviderUnavailableFact,
    OmenaQuerySourceTypeFactTargetV0 as SourceTypeFactTarget, ParserByteSpanV0,
    canonicalize_omena_query_source_selector_references,
    summarize_omena_query_expression_domain_selector_projection,
};
use omena_sif::compute_omena_sif_leaf_hash_v1;
use omena_tsgo_client::{
    TsgoJsonRpcTypeFactProviderV0, TsgoResolvedTypeV0, TsgoTypeFactRequestV0,
    TsgoTypeFactResultEntryV0, TsgoTypeFactTargetV0, build_tsgo_process_command,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const SOURCE_TYPE_FACT_CACHE_MAX_ENTRIES: usize = 128;
const TSGO_PROVIDER_ID: &str = "tsgo";
const TSGO_PROVIDER_PROJECT_MISS: &str = "projectMiss";
const TSGO_PROVIDER_NO_TRANSPORT: &str = "noTransport";
const TSGO_PROVIDER_PROCESS_UNAVAILABLE: &str = "processUnavailable";
const TSGO_PROVIDER_REQUEST_FAILED: &str = "requestFailed";
const TSGO_PROVIDER_MISSING_RESULT: &str = "missingResult";
const TSGO_PROVIDER_UNRESOLVABLE: &str = "unresolvable";

pub(crate) fn refresh_source_type_fact_candidates_for_document(
    state: &mut LspShellState,
    uri: &str,
) {
    let Some(document) = state.document(uri).cloned() else {
        return;
    };
    if crate::protocol::is_style_document_uri(document.uri.as_str()) {
        return;
    }
    let type_fact_targets = document.source_syntax_index.type_fact_targets.clone();
    if type_fact_targets.is_empty() {
        return;
    }
    let Some(request) =
        tsgo_type_fact_request_for_document(&document, type_fact_targets.as_slice())
    else {
        replace_tsgo_provider_unavailable_for_document(
            state,
            uri,
            type_fact_targets.as_slice(),
            TSGO_PROVIDER_PROJECT_MISS,
        );
        return;
    };
    let cache_key =
        source_type_fact_cache_key(state, &document, &request, type_fact_targets.as_slice());
    if let Some(entries) = cache_key
        .as_ref()
        .and_then(|key| state.source_type_fact_cache.get(key))
        .cloned()
    {
        apply_source_type_fact_results_to_document(state, uri, entries.as_slice());
        return;
    }
    if let Some((cache_key, entries)) = cache_key.as_ref().and_then(|key| {
        load_source_type_fact_sidecar(
            state,
            document.workspace_folder_uri.as_deref(),
            document.uri.as_str(),
            key,
        )
        .map(|entries| (key.clone(), entries))
    }) {
        state
            .source_type_fact_cache
            .insert(cache_key, entries.clone());
        trim_source_type_fact_cache(&mut state.source_type_fact_cache);
        apply_source_type_fact_results_to_document(state, uri, entries.as_slice());
        return;
    }

    let Some(tsgo_command) = tsgo_process_command_for_workspace(request.workspace_root.as_str())
    else {
        replace_tsgo_provider_unavailable_for_document(
            state,
            uri,
            type_fact_targets.as_slice(),
            TSGO_PROVIDER_NO_TRANSPORT,
        );
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
        replace_tsgo_provider_unavailable_for_document(
            state,
            uri,
            type_fact_targets.as_slice(),
            TSGO_PROVIDER_PROCESS_UNAVAILABLE,
        );
        return;
    }

    let pool = std::mem::take(&mut state.tsgo_workspace_process_pool);
    let mut provider = TsgoJsonRpcTypeFactProviderV0::new(pool);
    let entries = provider.collect_type_facts(&request).ok();
    state.tsgo_workspace_process_pool = provider.into_transport();
    let Some(entries) = entries else {
        replace_tsgo_provider_unavailable_for_document(
            state,
            uri,
            type_fact_targets.as_slice(),
            TSGO_PROVIDER_REQUEST_FAILED,
        );
        return;
    };
    if let Some(cache_key) = cache_key {
        store_source_type_fact_sidecar(
            state,
            document.workspace_folder_uri.as_deref(),
            document.uri.as_str(),
            cache_key.as_str(),
            entries.as_slice(),
        );
        state
            .source_type_fact_cache
            .insert(cache_key, entries.clone());
        trim_source_type_fact_cache(&mut state.source_type_fact_cache);
    }
    apply_source_type_fact_results_to_document(state, uri, entries.as_slice());
}

fn source_type_fact_cache_key(
    state: &LspShellState,
    document: &LspTextDocumentState,
    request: &TsgoTypeFactRequestV0,
    type_fact_targets: &[SourceTypeFactTarget],
) -> Option<String> {
    let key = json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.source-type-fact-cache-key",
        "documentUri": document.uri,
        "documentHash": document_text_hash(document),
        "workspaceRoot": request.workspace_root,
        "configPath": request.config_path,
        "configSignature": source_type_fact_tsconfig_signature(request.config_path.as_str()),
        "workspaceSourceSignature": source_type_fact_workspace_signature(
            state,
            document.workspace_folder_uri.as_deref(),
        ),
        "requestTargets": request.targets,
        "sourceTargets": type_fact_targets,
    });
    let bytes = serde_json::to_vec(&key).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn source_type_fact_tsconfig_signature(config_path: &str) -> String {
    std::fs::read(config_path)
        .map(|bytes| {
            compute_omena_sif_leaf_hash_v1(bytes.as_slice())
                .as_str()
                .to_string()
        })
        .unwrap_or_else(|_| format!("unreadable:{config_path}"))
}

fn source_type_fact_workspace_signature(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> String {
    let source_inputs = state
        .documents
        .values()
        .filter(|document| !crate::protocol::is_style_document_uri(document.uri.as_str()))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| {
            json!({
                "uri": document.uri,
                "workspaceFolderUri": document.workspace_folder_uri,
                "languageId": document.language_id,
                "textHash": document_text_hash(document),
            })
        })
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&source_inputs).unwrap_or_default();
    compute_omena_sif_leaf_hash_v1(bytes.as_slice())
        .as_str()
        .to_string()
}

fn document_text_hash(document: &LspTextDocumentState) -> String {
    if document.text_hash.is_empty() {
        return compute_omena_sif_leaf_hash_v1(document.text.as_bytes())
            .as_str()
            .to_string();
    }
    document.text_hash.clone()
}

fn trim_source_type_fact_cache(cache: &mut BTreeMap<String, Vec<TsgoTypeFactResultEntryV0>>) {
    while cache.len() > SOURCE_TYPE_FACT_CACHE_MAX_ENTRIES {
        let Some(key) = cache.keys().next().cloned() else {
            break;
        };
        cache.remove(key.as_str());
    }
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
    let targets = document.source_syntax_index.type_fact_targets.clone();
    let mut references = document.source_syntax_index.selector_references.clone();
    remove_source_type_fact_selector_references(
        &mut references,
        document.source_type_fact_selector_references.as_slice(),
    );
    let unavailable_facts =
        tsgo_provider_unavailable_facts_for_type_targets(targets.as_slice(), entries);
    ensure_referenced_style_documents_loaded_for_type_facts(state, targets.as_slice());
    let mut next_type_fact_references = Vec::new();
    for (target, selector_name) in
        project_source_type_fact_targets_with_query(state, &document, targets.as_slice(), entries)
    {
        let reference = source_selector_reference(
            target.byte_span,
            Some(selector_name),
            SourceSelectorReferenceMatchKind::Exact,
            target.target_style_uri.as_deref(),
            SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection,
        );
        references.push(reference.clone());
        next_type_fact_references.push(reference);
    }
    canonicalize_omena_query_source_selector_references(&mut references);
    let Some(document) = state.document_mut(uri) else {
        return;
    };
    document.source_syntax_index.selector_references = references;
    document.source_type_fact_selector_references = next_type_fact_references;
    document
        .source_syntax_index
        .type_fact_provider_unavailable
        .retain(|fact| fact.provider_id != TSGO_PROVIDER_ID);
    document
        .source_syntax_index
        .type_fact_provider_unavailable
        .extend(unavailable_facts);
    let source_syntax_index = document.source_syntax_index.clone();
    document.source_selector_candidates =
        source_selector_candidates_from_index(document, &source_syntax_index);
}

fn replace_tsgo_provider_unavailable_for_document(
    state: &mut LspShellState,
    uri: &str,
    targets: &[SourceTypeFactTarget],
    reason: &'static str,
) {
    let Some(document) = state.document_mut(uri) else {
        return;
    };
    remove_source_type_fact_selector_references(
        &mut document.source_syntax_index.selector_references,
        document.source_type_fact_selector_references.as_slice(),
    );
    document.source_type_fact_selector_references.clear();
    document
        .source_syntax_index
        .type_fact_provider_unavailable
        .retain(|fact| fact.provider_id != TSGO_PROVIDER_ID);
    document
        .source_syntax_index
        .type_fact_provider_unavailable
        .extend(
            targets
                .iter()
                .map(|target| SourceTypeFactProviderUnavailableFact {
                    byte_span: target.byte_span,
                    expression_id: target.expression_id.clone(),
                    target_style_uri: target.target_style_uri.clone(),
                    provider_id: TSGO_PROVIDER_ID,
                    reason,
                }),
        );
    let source_syntax_index = document.source_syntax_index.clone();
    document.source_selector_candidates =
        source_selector_candidates_from_index(document, &source_syntax_index);
}

fn remove_source_type_fact_selector_references(
    references: &mut Vec<SourceSelectorReferenceFact>,
    type_fact_references: &[SourceSelectorReferenceFact],
) {
    references.retain(|reference| !type_fact_references.contains(reference));
}

fn tsgo_provider_unavailable_facts_for_type_targets(
    targets: &[SourceTypeFactTarget],
    entries: &[TsgoTypeFactResultEntryV0],
) -> Vec<SourceTypeFactProviderUnavailableFact> {
    let entries_by_id = entries
        .iter()
        .map(|entry| (entry.expression_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    targets
        .iter()
        .filter_map(|target| {
            let reason = match entries_by_id.get(target.expression_id.as_str()) {
                None => TSGO_PROVIDER_MISSING_RESULT,
                Some(entry) if entry.resolved_type.kind != "union" => TSGO_PROVIDER_UNRESOLVABLE,
                Some(_) => return None,
            };
            Some(SourceTypeFactProviderUnavailableFact {
                byte_span: target.byte_span,
                expression_id: target.expression_id.clone(),
                target_style_uri: target.target_style_uri.clone(),
                provider_id: TSGO_PROVIDER_ID,
                reason,
            })
        })
        .collect()
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
            let target_style_uri = target
                .target_style_uri
                .as_deref()
                .map(canonical_type_fact_query_uri)?;
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
                "filePath": canonical_type_fact_query_uri(style_document.uri.as_str()),
                "source": style_document.text,
                "document": {
                    "selectors": selectors,
                },
            })
        })
        .collect::<Vec<_>>();
    let workspace_root = document
        .workspace_folder_uri
        .as_deref()
        .and_then(file_uri_to_path)
        .or_else(|| {
            file_uri_to_path(document.uri.as_str())
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| document.workspace_folder_uri.clone().unwrap_or_default());
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
        "workspace": {
            "root": workspace_root,
            "classnameTransform": "asIs",
            "settingsKey": "lsp-source-type-facts",
        },
        "sources": [{
            "filePath": source_file_path,
            "document": {
                "classExpressions": class_expressions,
            },
        }],
        "styles": styles,
        "typeFacts": type_facts,
    }))
    .ok()
}

fn canonical_type_fact_query_uri(uri: &str) -> String {
    crate::protocol::canonical_file_uri(uri).unwrap_or_else(|| uri.to_string())
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

fn source_selector_reference(
    byte_span: ParserByteSpanV0,
    selector_name: Option<String>,
    match_kind: SourceSelectorReferenceMatchKind,
    target_style_uri: Option<&str>,
    surface: SourceSelectorReferenceSurface,
) -> SourceSelectorReferenceFact {
    SourceSelectorReferenceFact {
        byte_span,
        selector_name,
        match_kind,
        target_style_uri: target_style_uri.map(ToString::to_string),
        surface,
    }
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
        std::env::var("OMENA_TSGO_CHECKERS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok()),
    ))
}

fn resolve_tsgo_binary_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("OMENA_TSGO_PATH")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{handle_lsp_message, protocol::path_to_file_uri};
    use omena_tsgo_client::TsgoResolvedTypeV0;
    use serde_json::json;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn persisted_source_type_facts_project_without_tsgo_transport() -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-source-type-fact-cache-{}",
            std::process::id()
        ));
        let src_dir = workspace_root.join("src");
        let source_path = src_dir.join("App.tsx");
        let style_path = src_dir.join("App.module.scss");
        let _ = std::fs::remove_dir_all(&workspace_root);
        std::fs::create_dir_all(&src_dir)?;
        std::fs::write(workspace_root.join("tsconfig.json"), "{}")?;
        std::fs::write(
            &style_path,
            ".small { color: red; }\n.medium { color: blue; }",
        )?;
        let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
interface BadgeProps { size: "small" | "medium"; }
export function Badge({ size }: BadgeProps) {
  return <span className={cx(size)} />;
}"#;
        std::fs::write(&source_path, source_text)?;

        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        let source_uri = path_to_file_uri(source_path.as_path());
        let style_uri = path_to_file_uri(style_path.as_path());
        let mut state = LspShellState::default();
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "workspaceFolders": [
                        {
                            "uri": workspace_uri,
                            "name": "source-type-fact-cache",
                        },
                    ],
                },
            }),
        );
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": style_uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": ".small { color: red; }\n.medium { color: blue; }",
                    },
                },
            }),
        );
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": source_uri,
                        "languageId": "typescriptreact",
                        "version": 1,
                        "text": source_text,
                    },
                },
            }),
        );

        let (cache_key, entry) = {
            let document = state
                .document(source_uri.as_str())
                .ok_or_else(|| std::io::Error::other("source document should be open"))?;
            let type_fact_targets = document.source_syntax_index.type_fact_targets.clone();
            let size_target = type_fact_targets
                .iter()
                .find(|target| {
                    source_text.get(target.byte_span.start..target.byte_span.end) == Some("size")
                })
                .ok_or_else(|| std::io::Error::other("size type fact target should exist"))?;
            let request =
                tsgo_type_fact_request_for_document(document, type_fact_targets.as_slice())
                    .ok_or_else(|| std::io::Error::other("type fact request should build"))?;
            let cache_key = source_type_fact_cache_key(
                &state,
                document,
                &request,
                type_fact_targets.as_slice(),
            )
            .ok_or_else(|| std::io::Error::other("cache key should build"))?;
            (
                cache_key,
                TsgoTypeFactResultEntryV0 {
                    file_path: request
                        .targets
                        .first()
                        .map(|target| target.file_path.clone())
                        .unwrap_or_default(),
                    expression_id: size_target.expression_id.clone(),
                    resolved_type: TsgoResolvedTypeV0 {
                        kind: "union",
                        values: vec!["medium".to_string(), "small".to_string()],
                    },
                },
            )
        };
        crate::source_type_fact_cache::store_source_type_fact_sidecar(
            &state,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            cache_key.as_str(),
            &[entry],
        );
        let sidecar_path =
            crate::source_type_fact_cache::source_type_fact_sidecar_file_path_for_test(
                &state,
                Some(workspace_uri.as_str()),
                source_uri.as_str(),
            )
            .ok_or_else(|| std::io::Error::other("source type fact sidecar path should resolve"))?;
        assert!(
            sidecar_path.exists(),
            "fixture should persist a source type fact sidecar: {sidecar_path:?}"
        );
        assert!(
            state.source_type_fact_cache.is_empty(),
            "test must prove disk rehydration, not the in-memory source type fact cache"
        );

        refresh_source_type_fact_candidates_for_document(&mut state, source_uri.as_str());

        let selector_names = state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?
            .source_syntax_index
            .selector_references
            .iter()
            .filter_map(|reference| reference.selector_name.as_deref())
            .collect::<Vec<_>>();
        assert!(
            selector_names.contains(&"small") && selector_names.contains(&"medium"),
            "persisted type facts should project class references without starting tsgo: {selector_names:?}"
        );
        assert!(
            state
                .source_type_fact_cache
                .contains_key(cache_key.as_str()),
            "disk-loaded source type facts should repopulate the in-memory cache"
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
        Ok(())
    }

    #[test]
    fn unresolvable_source_type_facts_surface_unknown_precision_diagnostics() -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-source-type-fact-unresolvable-{}",
            std::process::id()
        ));
        let src_dir = workspace_root.join("src");
        let source_path = src_dir.join("App.tsx");
        let style_path = src_dir.join("App.module.scss");
        let _ = std::fs::remove_dir_all(&workspace_root);
        std::fs::create_dir_all(&src_dir)?;
        std::fs::write(workspace_root.join("tsconfig.json"), "{}")?;
        std::fs::write(
            &style_path,
            ".small { color: red; }\n.medium { color: blue; }",
        )?;
        let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
interface BadgeProps { size: "small" | "medium"; }
export function Badge({ size }: BadgeProps) {
  return <span className={cx(size)} />;
}"#;
        std::fs::write(&source_path, source_text)?;

        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        let source_uri = path_to_file_uri(source_path.as_path());
        let style_uri = path_to_file_uri(style_path.as_path());
        let mut state = LspShellState::default();
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "workspaceFolders": [
                        {
                            "uri": workspace_uri,
                            "name": "source-type-fact-unresolvable",
                        },
                    ],
                },
            }),
        );
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": style_uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": ".small { color: red; }\n.medium { color: blue; }",
                    },
                },
            }),
        );
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": source_uri,
                        "languageId": "typescriptreact",
                        "version": 1,
                        "text": source_text,
                    },
                },
            }),
        );

        let (cache_key, resolved_entry, unresolved_entry) = {
            let document = state
                .document(source_uri.as_str())
                .ok_or_else(|| std::io::Error::other("source document should be open"))?;
            let type_fact_targets = document.source_syntax_index.type_fact_targets.clone();
            let size_target = type_fact_targets
                .iter()
                .find(|target| {
                    source_text.get(target.byte_span.start..target.byte_span.end) == Some("size")
                })
                .ok_or_else(|| std::io::Error::other("size type fact target should exist"))?;
            let request =
                tsgo_type_fact_request_for_document(document, type_fact_targets.as_slice())
                    .ok_or_else(|| std::io::Error::other("type fact request should build"))?;
            let cache_key = source_type_fact_cache_key(
                &state,
                document,
                &request,
                type_fact_targets.as_slice(),
            )
            .ok_or_else(|| std::io::Error::other("cache key should build"))?;
            let file_path = request
                .targets
                .first()
                .map(|target| target.file_path.clone())
                .unwrap_or_default();
            let expression_id = size_target.expression_id.clone();
            (
                cache_key,
                TsgoTypeFactResultEntryV0 {
                    file_path: file_path.clone(),
                    expression_id: expression_id.clone(),
                    resolved_type: TsgoResolvedTypeV0 {
                        kind: "union",
                        values: vec!["small".to_string()],
                    },
                },
                TsgoTypeFactResultEntryV0 {
                    file_path,
                    expression_id,
                    resolved_type: TsgoResolvedTypeV0 {
                        kind: "unresolvable",
                        values: Vec::new(),
                    },
                },
            )
        };
        crate::source_type_fact_cache::store_source_type_fact_sidecar(
            &state,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            cache_key.as_str(),
            &[resolved_entry],
        );
        refresh_source_type_fact_candidates_for_document(&mut state, source_uri.as_str());
        let document = state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?;
        assert!(
            document
                .source_syntax_index
                .selector_references
                .iter()
                .any(|reference| reference.selector_name.as_deref() == Some("small")),
            "resolved tsgo union should project the concrete selector before the unavailable refresh",
        );
        assert!(
            document
                .source_syntax_index
                .type_fact_provider_unavailable
                .is_empty(),
            "resolved tsgo union should not produce unavailable facts",
        );

        crate::source_type_fact_cache::store_source_type_fact_sidecar(
            &state,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            cache_key.as_str(),
            std::slice::from_ref(&unresolved_entry),
        );
        state
            .source_type_fact_cache
            .insert(cache_key.clone(), vec![unresolved_entry]);

        refresh_source_type_fact_candidates_for_document(&mut state, source_uri.as_str());

        let unavailable = &state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?
            .source_syntax_index
            .type_fact_provider_unavailable;
        assert_eq!(unavailable.len(), 1);
        assert_eq!(unavailable[0].provider_id, "tsgo");
        assert_eq!(unavailable[0].reason, "unresolvable");
        assert!(
            !state
                .document(source_uri.as_str())
                .ok_or_else(|| std::io::Error::other("source document should remain open"))?
                .source_syntax_index
                .selector_references
                .iter()
                .any(|reference| reference.selector_name.as_deref() == Some("small")),
            "unresolvable tsgo refresh must drop the previous concrete projection",
        );

        let diagnostics = crate::source_diagnostics::resolve_source_diagnostics_for_uri(
            &state,
            source_uri.as_str(),
        );
        let unknown = diagnostics
            .as_array()
            .and_then(|diagnostics| {
                diagnostics.iter().find(|diagnostic| {
                    diagnostic.get("code").and_then(serde_json::Value::as_str)
                        == Some("unknownClassValueDomain")
                })
            })
            .ok_or_else(|| {
                std::io::Error::other(format!(
                    "unknown precision diagnostic should be emitted: {diagnostics}"
                ))
            })?;
        assert_eq!(
            unknown
                .pointer("/data/precision/valueDomain")
                .and_then(serde_json::Value::as_str),
            Some("unknown"),
        );
        assert_eq!(
            unknown
                .pointer("/data/precision/flowSensitivity")
                .and_then(serde_json::Value::as_str),
            Some("typeOracleProviderUnavailable"),
        );
        assert!(
            unknown
                .pointer("/data/provenance")
                .and_then(serde_json::Value::as_array)
                .map(|items| {
                    items.iter().any(|item| {
                        item.as_str() == Some("tsgo-provider.unavailable->unknown-precision")
                    })
                })
                .unwrap_or(false),
            "unknown precision diagnostic must record the tsgo downgrade provenance: {unknown}",
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
        Ok(())
    }
}
