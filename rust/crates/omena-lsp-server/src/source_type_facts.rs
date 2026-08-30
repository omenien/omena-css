use crate::protocol::{file_uri_to_path, workspace_folder_compatible};
use crate::source_type_fact_cache::{
    SourceTypeFactSidecarFreshnessV0, SourceTypeFactSidecarLoadV0,
    load_source_type_fact_sidecar_with_freshness, store_source_type_fact_sidecar_with_freshness,
};
use crate::{
    LspShellState, LspSourceTypeFactCacheEntryV0, LspSourceTypeFactTierAttemptV0,
    LspTextDocumentState, ensure_style_document_loaded_from_disk,
    source_selector_candidates_from_index,
};
use omena_query::{
    OmenaQueryEngineInputV2,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSelectorReferenceSurfaceV0 as SourceSelectorReferenceSurface,
    OmenaQuerySourceTypeFactExpressionShapeV0 as SourceTypeFactExpressionShape,
    OmenaQuerySourceTypeFactLexicalAttemptV0 as SourceTypeFactLexicalAttempt,
    OmenaQuerySourceTypeFactLexicalDispositionV0 as SourceTypeFactLexicalDisposition,
    OmenaQuerySourceTypeFactProviderUnavailableFactV0 as SourceTypeFactProviderUnavailableFact,
    OmenaQuerySourceTypeFactTargetV0 as SourceTypeFactTarget, ParserByteSpanV0,
    canonicalize_omena_query_source_selector_references,
    summarize_omena_query_expression_domain_selector_projection,
};
use omena_sif::compute_omena_sif_leaf_hash_v1;
use omena_syntax::ident::{is_ascii_word_continue, is_css_name_continue, is_safe_css_identifier};
use omena_tsgo_client::{
    TsgoJsonRpcTypeFactProviderV0, TsgoResolvedTypeV0, TsgoSpanTypeFactRequestV0,
    TsgoSpanTypeFactResultEntryV0, TsgoSpanTypeFactTargetV0, TsgoTypeFactRequestV0,
    TsgoTypeFactResultEntryV0, TsgoTypeFactTargetV0, build_tsgo_process_command,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const SOURCE_TYPE_FACT_CACHE_MAX_ENTRIES: usize = 128;
const TSGO_PROVIDER_ID: &str = "tsgo";
const TSGO_PROVIDER_PROJECT_MISS: &str = "projectMiss";
const TSGO_PROVIDER_NO_TRANSPORT: &str = "noTransport";
const TSGO_PROVIDER_PROCESS_UNAVAILABLE: &str = "processUnavailable";
const TSGO_PROVIDER_REQUEST_FAILED: &str = "requestFailed";
const TSGO_PROVIDER_MISSING_RESULT: &str = "missingResult";
const TSGO_PROVIDER_UNRESOLVABLE: &str = "unresolvable";
const SOURCE_TYPE_FACT_TIER_LEXICAL: &str = "lexical";
const SOURCE_TYPE_FACT_TIER_TSGO: &str = "tsgo";
const SOURCE_TYPE_FACT_OUTCOME_RESOLVED: &str = "resolved";
pub(crate) const SOURCE_TYPE_FACT_OUTCOME_NOT_ATTEMPTED: &str = "notAttempted";
const SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE: &str = "unavailable";
const SOURCE_TYPE_FACT_OUTCOME_UNRESOLVED: &str = "unresolved";
const SOURCE_TYPE_FACT_OUTCOME_REFUSED: &str = "refused";
const SOURCE_TYPE_FACT_REASON_FINITE_EXACT_DOMAIN: &str = "finiteExactDomain";
pub(crate) const SOURCE_TYPE_FACT_REASON_PROVIDER_NOT_REQUESTED: &str = "providerNotRequested";
const SOURCE_TYPE_FACT_REASON_NON_EXACT_DOMAIN: &str = "nonExactDomain";
const SOURCE_TYPE_FACT_REASON_OUTCOME_NOT_RESOLVED: &str = "outcomeNotResolved";
const SOURCE_TYPE_FACT_REASON_SPAN_NOT_EXACT: &str = "spanNotExact";
const SOURCE_TYPE_FACT_REASON_EMPTY_EXACT_DOMAIN: &str = "emptyExactDomain";
const SOURCE_TYPE_FACT_REASON_MEMBER_COUNT_MISMATCH: &str = "memberCountMismatch";
const SOURCE_TYPE_FACT_REASON_NON_UNION_RESOLVED_TYPE: &str = "nonUnionResolvedType";
const SOURCE_TYPE_FACT_REASON_EMPTY_RESOLVED_VALUES: &str = "emptyResolvedValues";
const SOURCE_TYPE_FACT_REASON_INVALID_CSS_IDENTIFIER_CHARACTER: &str =
    "invalidCssIdentifierCharacter";
const SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER: &str = "unsafeCssIdentifier";
const SOURCE_TYPE_FACT_CLOSURE_INDEX_BUDGET: &str = "indexBudgetExhausted";
const SOURCE_TYPE_FACT_CLOSURE_MODULE_SPECIFIER: &str = "unindexedModuleSpecifier";
const SOURCE_TYPE_FACT_CLOSURE_PACKAGE_EXTENDS: &str = "packageFormExtends";
const SOURCE_TYPE_FACT_CLOSURE_TSCONFIG_PARSE_FAILED: &str = "tsconfigParseFailed";
const SOURCE_TYPE_FACT_CLOSURE_TSCONFIG_EXTENDS_ARRAY: &str = "tsconfigExtendsArrayUnsupported";
const SOURCE_TYPE_FACT_CLOSURE_WATCHED_FILES: &str = "watchedFilesUnobserved";
const SOURCE_TYPE_FACT_CLOSURE_WORKSPACE_FOLDER: &str = "workspaceFolderUnknown";

#[derive(Debug, Clone)]
struct SourceTypeFactCacheContextV0 {
    freshness: SourceTypeFactSidecarFreshnessV0,
    closure_incomplete_reasons: BTreeSet<&'static str>,
}

// Parser-built documents pair every lexical site with an initial tier attempt.
// Sidecar reconstruction also accepts a skipped fact without that record; both
// paths report the same canonical not-attempted outcome.
pub(crate) fn initial_source_type_fact_tier_attempts(
    lexical_attempts: &[SourceTypeFactLexicalAttempt],
) -> Vec<LspSourceTypeFactTierAttemptV0> {
    lexical_attempts
        .iter()
        .map(|attempt| match attempt.lexical_disposition {
            SourceTypeFactLexicalDisposition::Resolved => LspSourceTypeFactTierAttemptV0 {
                expression_id: attempt.expression_id.clone(),
                tier: SOURCE_TYPE_FACT_TIER_LEXICAL,
                outcome: SOURCE_TYPE_FACT_OUTCOME_RESOLVED,
                reason: Some(SOURCE_TYPE_FACT_REASON_FINITE_EXACT_DOMAIN),
            },
            SourceTypeFactLexicalDisposition::TypeProviderCandidate => {
                LspSourceTypeFactTierAttemptV0 {
                    expression_id: attempt.expression_id.clone(),
                    tier: SOURCE_TYPE_FACT_TIER_TSGO,
                    outcome: SOURCE_TYPE_FACT_OUTCOME_NOT_ATTEMPTED,
                    reason: Some(SOURCE_TYPE_FACT_REASON_PROVIDER_NOT_REQUESTED),
                }
            }
            SourceTypeFactLexicalDisposition::Unresolved => LspSourceTypeFactTierAttemptV0 {
                expression_id: attempt.expression_id.clone(),
                tier: SOURCE_TYPE_FACT_TIER_TSGO,
                outcome: SOURCE_TYPE_FACT_OUTCOME_NOT_ATTEMPTED,
                reason: Some(attempt.shape_class.unsupported_reason()),
            },
            _ => LspSourceTypeFactTierAttemptV0 {
                expression_id: attempt.expression_id.clone(),
                tier: SOURCE_TYPE_FACT_TIER_TSGO,
                outcome: SOURCE_TYPE_FACT_OUTCOME_NOT_ATTEMPTED,
                reason: Some(SOURCE_TYPE_FACT_TARGET_SKIPPED_UNKNOWN_SHAPE),
            },
        })
        .collect()
}

const SOURCE_TYPE_FACT_TARGET_SKIPPED_UNKNOWN_SHAPE: &str = "unsupportedExpressionShape";

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
    let span_type_fact_targets = span_source_type_fact_targets(&document);
    if type_fact_targets.is_empty() && span_type_fact_targets.is_empty() {
        return;
    }
    let Some((request, span_request)) = tsgo_type_fact_requests_for_document(
        &document,
        type_fact_targets.as_slice(),
        span_type_fact_targets.as_slice(),
    ) else {
        let mut all_targets = type_fact_targets.clone();
        all_targets.extend(span_type_fact_targets);
        replace_tsgo_provider_unavailable_for_document(
            state,
            uri,
            all_targets.as_slice(),
            TSGO_PROVIDER_PROJECT_MISS,
        );
        return;
    };
    let cache_context = source_type_fact_cache_context(state, &document, &request);
    record_source_type_fact_closure_incomplete(state, &cache_context.closure_incomplete_reasons);
    let cache_key = source_type_fact_cache_key(
        state,
        &document,
        &request,
        type_fact_targets.as_slice(),
        &cache_context.freshness,
    );
    let cached_entries = cached_source_type_fact_entries(
        state,
        &document,
        cache_key.as_deref(),
        &cache_context.freshness,
    );
    if span_request.targets.is_empty()
        && let Some(entries) = cached_entries.as_ref()
    {
        apply_source_type_fact_results_to_document(state, uri, entries.as_slice());
        return;
    }

    let Some(tsgo_command) = tsgo_process_command_for_workspace(request.workspace_root.as_str())
    else {
        apply_tsgo_provider_unavailable_with_cached_legacy(
            state,
            uri,
            type_fact_targets.as_slice(),
            span_type_fact_targets.as_slice(),
            cached_entries.as_deref(),
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
        apply_tsgo_provider_unavailable_with_cached_legacy(
            state,
            uri,
            type_fact_targets.as_slice(),
            span_type_fact_targets.as_slice(),
            cached_entries.as_deref(),
            TSGO_PROVIDER_PROCESS_UNAVAILABLE,
        );
        return;
    }

    let pool = std::mem::take(&mut state.tsgo_workspace_process_pool);
    let mut provider = TsgoJsonRpcTypeFactProviderV0::new(pool);
    let result = if span_request.targets.is_empty() {
        provider
            .collect_type_facts(&request)
            .map(|entries| (entries, Vec::new()))
    } else {
        provider
            .collect_type_facts_with_span_targets(&request, &span_request)
            .map(|result| (result.type_fact_entries, result.span_type_fact_entries))
    }
    .ok();
    state.tsgo_workspace_process_pool = provider.into_transport();
    let Some((entries, span_entries)) = result else {
        apply_tsgo_provider_unavailable_with_cached_legacy(
            state,
            uri,
            type_fact_targets.as_slice(),
            span_type_fact_targets.as_slice(),
            cached_entries.as_deref(),
            TSGO_PROVIDER_REQUEST_FAILED,
        );
        return;
    };
    if let Some(cache_key) = cache_key {
        cache_source_type_fact_results(
            state,
            &document,
            cache_key.as_str(),
            entries.as_slice(),
            &cache_context,
        );
    }
    apply_source_type_fact_results_to_document_with_span(
        state,
        uri,
        entries.as_slice(),
        span_entries.as_slice(),
        span_type_fact_targets.as_slice(),
    );
}

fn cached_source_type_fact_entries(
    state: &mut LspShellState,
    document: &LspTextDocumentState,
    cache_key: Option<&str>,
    freshness: &SourceTypeFactSidecarFreshnessV0,
) -> Option<Vec<TsgoTypeFactResultEntryV0>> {
    let cache_key = cache_key?;
    state.source_type_fact_cache_next_use = state.source_type_fact_cache_next_use.saturating_add(1);
    let last_used = state.source_type_fact_cache_next_use;
    if let Some(entry) = state.source_type_fact_cache.get_mut(cache_key) {
        entry.last_used = last_used;
        state.source_type_fact_cache_telemetry.hit_count = state
            .source_type_fact_cache_telemetry
            .hit_count
            .saturating_add(1);
        return Some(entry.entries.clone());
    }
    match load_source_type_fact_sidecar_with_freshness(
        state,
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        cache_key,
        freshness,
    ) {
        SourceTypeFactSidecarLoadV0::Hit(entries) => {
            state.source_type_fact_cache_telemetry.hit_count = state
                .source_type_fact_cache_telemetry
                .hit_count
                .saturating_add(1);
            state.source_type_fact_cache_telemetry.sidecar_hit_count = state
                .source_type_fact_cache_telemetry
                .sidecar_hit_count
                .saturating_add(1);
            state.source_type_fact_cache.insert(
                cache_key.to_string(),
                LspSourceTypeFactCacheEntryV0 {
                    entries: entries.clone(),
                    last_used,
                },
            );
            trim_source_type_fact_cache(state);
            Some(entries)
        }
        SourceTypeFactSidecarLoadV0::Refused(reason) => {
            increment_reason_counter(
                &mut state
                    .source_type_fact_cache_telemetry
                    .sidecar_refused_by_reason,
                reason.as_str(),
            );
            state.source_type_fact_cache_telemetry.miss_count = state
                .source_type_fact_cache_telemetry
                .miss_count
                .saturating_add(1);
            None
        }
        SourceTypeFactSidecarLoadV0::Miss => {
            state.source_type_fact_cache_telemetry.miss_count = state
                .source_type_fact_cache_telemetry
                .miss_count
                .saturating_add(1);
            None
        }
    }
}

fn apply_tsgo_provider_unavailable_with_cached_legacy(
    state: &mut LspShellState,
    uri: &str,
    legacy_targets: &[SourceTypeFactTarget],
    span_targets: &[SourceTypeFactTarget],
    cached_entries: Option<&[TsgoTypeFactResultEntryV0]>,
    reason: &'static str,
) {
    let Some(cached_entries) = cached_entries.filter(|_| !span_targets.is_empty()) else {
        let mut all_targets = legacy_targets.to_vec();
        all_targets.extend_from_slice(span_targets);
        replace_tsgo_provider_unavailable_for_document(state, uri, all_targets.as_slice(), reason);
        return;
    };

    apply_source_type_fact_results_to_document_with_span(
        state,
        uri,
        cached_entries,
        &[],
        span_targets,
    );
    let span_expression_ids = span_targets
        .iter()
        .map(|target| target.expression_id.as_str())
        .collect::<BTreeSet<_>>();
    let Some(document) = state.document_mut(uri) else {
        return;
    };
    for attempt in &mut document.source_type_fact_tier_attempts {
        if span_expression_ids.contains(attempt.expression_id.as_str()) {
            attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE;
            attempt.reason = Some(reason);
        }
    }
    document
        .source_syntax_index
        .type_fact_provider_unavailable
        .retain(|fact| {
            fact.provider_id != TSGO_PROVIDER_ID
                || !span_expression_ids.contains(fact.expression_id.as_str())
        });
    document
        .source_syntax_index
        .type_fact_provider_unavailable
        .extend(
            span_targets
                .iter()
                .map(|target| SourceTypeFactProviderUnavailableFact {
                    byte_span: target.byte_span,
                    expression_id: target.expression_id.clone(),
                    target_style_uri: target.target_style_uri.clone(),
                    provider_id: TSGO_PROVIDER_ID,
                    reason,
                }),
        );
}

fn source_type_fact_cache_key(
    state: &LspShellState,
    document: &LspTextDocumentState,
    request: &TsgoTypeFactRequestV0,
    type_fact_targets: &[SourceTypeFactTarget],
    freshness: &SourceTypeFactSidecarFreshnessV0,
) -> Option<String> {
    let key = json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.source-type-fact-cache-key",
        "documentUri": document.uri,
        "documentHash": document_text_hash(document),
        "workspaceRoot": request.workspace_root,
        "configPath": request.config_path,
        "environmentFingerprint": freshness.environment_fingerprint,
        "tsgoBinaryFingerprint": freshness.tsgo_binary_fingerprint,
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

fn source_type_fact_cache_context(
    state: &LspShellState,
    document: &LspTextDocumentState,
    request: &TsgoTypeFactRequestV0,
) -> SourceTypeFactCacheContextV0 {
    let (environment_fingerprint, mut reasons) = source_type_fact_environment_fingerprint(
        request.workspace_root.as_str(),
        request.config_path.as_str(),
    );
    let tsgo_binary_fingerprint = resolve_tsgo_binary_path()
        .as_deref()
        .map(source_type_fact_tsgo_binary_fingerprint)
        .unwrap_or_else(|| "unavailable:tsgo-binary".to_string());
    let mut project_by_file = request
        .targets
        .iter()
        .map(|target| {
            (
                target.file_path.clone(),
                Value::String(request.config_path.clone()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for file_path in request
        .targets
        .iter()
        .map(|target| target.file_path.as_str())
    {
        project_by_file
            .entry(file_path.to_string())
            .or_insert(Value::Null);
    }
    let collection_provenance = json!({
        "snapshotHandle": Value::Null,
        "projectByFile": project_by_file,
        "disclosure": "provider-owned snapshot handle is released after collection and is not a validity input",
    });

    if state.source_type_fact_workspace_index_incomplete {
        reasons.insert(SOURCE_TYPE_FACT_CLOSURE_INDEX_BUDGET);
    }
    if !state.source_type_fact_watched_files_observed {
        reasons.insert(SOURCE_TYPE_FACT_CLOSURE_WATCHED_FILES);
    }
    if document.workspace_folder_uri.is_none()
        || state.documents.values().any(|candidate| {
            !crate::protocol::is_style_document_uri(candidate.uri.as_str())
                && candidate.workspace_folder_uri.is_none()
        })
    {
        reasons.insert(SOURCE_TYPE_FACT_CLOSURE_WORKSPACE_FOLDER);
    }
    if !source_type_fact_module_specifiers_are_indexed(
        state,
        document.workspace_folder_uri.as_deref(),
    ) {
        reasons.insert(SOURCE_TYPE_FACT_CLOSURE_MODULE_SPECIFIER);
    }

    SourceTypeFactCacheContextV0 {
        freshness: SourceTypeFactSidecarFreshnessV0 {
            environment_fingerprint,
            tsgo_binary_fingerprint,
            collection_provenance,
        },
        closure_incomplete_reasons: reasons,
    }
}

fn source_type_fact_environment_fingerprint(
    workspace_root: &str,
    config_path: &str,
) -> (String, BTreeSet<&'static str>) {
    let (config_chain, closure_incomplete_reasons) = source_type_fact_tsconfig_chain(config_path);
    let environment = json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.source-type-fact-environment",
        "workspaceRoot": workspace_root,
        "configPath": config_path,
        "configChain": config_chain,
    });
    let fingerprint = serde_json::to_vec(&environment)
        .ok()
        .map(|bytes| {
            compute_omena_sif_leaf_hash_v1(bytes.as_slice())
                .as_str()
                .to_string()
        })
        .unwrap_or_else(|| "unavailable:environment".to_string());
    (fingerprint, closure_incomplete_reasons)
}

fn source_type_fact_tsconfig_chain(config_path: &str) -> (Vec<Value>, BTreeSet<&'static str>) {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    let mut reasons = BTreeSet::new();
    collect_tsconfig_chain(Path::new(config_path), &mut seen, &mut rows, &mut reasons);
    (rows, reasons)
}

fn collect_tsconfig_chain(
    config_path: &Path,
    seen: &mut BTreeSet<PathBuf>,
    rows: &mut Vec<Value>,
    reasons: &mut BTreeSet<&'static str>,
) {
    let normalized = fs::canonicalize(config_path).unwrap_or_else(|_| config_path.to_path_buf());
    if !seen.insert(normalized.clone()) {
        return;
    }
    let Ok(bytes) = fs::read(config_path) else {
        rows.push(json!({
            "path": normalized.to_string_lossy(),
            "digest": format!("unreadable:{}", normalized.to_string_lossy()),
        }));
        return;
    };
    rows.push(json!({
        "path": normalized.to_string_lossy(),
        "digest": compute_omena_sif_leaf_hash_v1(bytes.as_slice()).as_str(),
    }));
    let Ok(config) = serde_json::from_slice::<Value>(bytes.as_slice()) else {
        reasons.insert(SOURCE_TYPE_FACT_CLOSURE_TSCONFIG_PARSE_FAILED);
        return;
    };
    let Some(extends_value) = config.get("extends") else {
        return;
    };
    let Some(extends) = extends_value.as_str() else {
        if extends_value.is_array() {
            reasons.insert(SOURCE_TYPE_FACT_CLOSURE_TSCONFIG_EXTENDS_ARRAY);
        }
        return;
    };
    if !extends.starts_with('.') {
        reasons.insert(SOURCE_TYPE_FACT_CLOSURE_PACKAGE_EXTENDS);
        return;
    }
    let Some(config_dir) = config_path.parent() else {
        return;
    };
    let raw_path = config_dir.join(extends);
    let next = if raw_path.extension().is_some() {
        Some(raw_path)
    } else {
        [
            raw_path.with_extension("json"),
            raw_path.join("tsconfig.json"),
        ]
        .into_iter()
        .find(|candidate| candidate.exists())
    };
    if let Some(next) = next {
        collect_tsconfig_chain(next.as_path(), seen, rows, reasons);
    }
}

fn source_type_fact_tsgo_binary_fingerprint(path: &Path) -> String {
    let metadata = fs::metadata(path).ok();
    let bytes = fs::read(path).unwrap_or_default();
    let modified = metadata
        .as_ref()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok());
    let fingerprint = json!({
        "path": path.to_string_lossy(),
        "byteLength": metadata.as_ref().map(std::fs::Metadata::len),
        "modifiedSeconds": modified.map(|duration| duration.as_secs()),
        "modifiedNanos": modified.map(|duration| duration.subsec_nanos()),
        "contentDigest": compute_omena_sif_leaf_hash_v1(bytes.as_slice()).as_str(),
    });
    serde_json::to_vec(&fingerprint)
        .ok()
        .map(|bytes| {
            compute_omena_sif_leaf_hash_v1(bytes.as_slice())
                .as_str()
                .to_string()
        })
        .unwrap_or_else(|| format!("unavailable:{}", path.to_string_lossy()))
}

fn source_type_fact_module_specifiers_are_indexed(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> bool {
    let indexed_paths = state
        .documents
        .values()
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .filter_map(|document| file_uri_to_path(document.uri.as_str()))
        .map(|path| fs::canonicalize(path.as_path()).unwrap_or(path))
        .collect::<BTreeSet<_>>();
    state
        .documents
        .values()
        .filter(|document| !crate::protocol::is_style_document_uri(document.uri.as_str()))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .all(|document| {
            if !document.source_module_specifier_index_complete {
                return false;
            }
            let Some(document_path) = file_uri_to_path(document.uri.as_str()) else {
                return false;
            };
            document.source_module_specifiers.iter().all(|specifier| {
                source_type_fact_module_specifier_is_indexed(
                    document_path.as_path(),
                    specifier.as_str(),
                    &indexed_paths,
                )
            })
        })
}

fn source_type_fact_module_specifier_is_indexed(
    document_path: &Path,
    specifier: &str,
    indexed_paths: &BTreeSet<PathBuf>,
) -> bool {
    if !specifier.starts_with('.') {
        return false;
    }
    let Some(document_dir) = document_path.parent() else {
        return false;
    };
    let raw_path = document_dir.join(specifier);
    let mut candidates = vec![raw_path.clone()];
    if raw_path.extension().is_none() {
        for extension in ["ts", "tsx", "js", "jsx", "css", "scss", "sass", "less"] {
            candidates.push(raw_path.with_extension(extension));
            candidates.push(raw_path.join(format!("index.{extension}")));
        }
    }
    candidates.into_iter().any(|candidate| {
        let normalized = fs::canonicalize(candidate.as_path()).unwrap_or(candidate);
        indexed_paths.contains(&normalized)
    })
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

fn cache_source_type_fact_results(
    state: &mut LspShellState,
    document: &LspTextDocumentState,
    cache_key: &str,
    entries: &[TsgoTypeFactResultEntryV0],
    context: &SourceTypeFactCacheContextV0,
) {
    if context.closure_incomplete_reasons.is_empty() {
        let _ = store_source_type_fact_sidecar_with_freshness(
            state,
            document.workspace_folder_uri.as_deref(),
            document.uri.as_str(),
            cache_key,
            entries,
            &context.freshness,
        );
    }
    state.source_type_fact_cache_next_use = state.source_type_fact_cache_next_use.saturating_add(1);
    state.source_type_fact_cache.insert(
        cache_key.to_string(),
        LspSourceTypeFactCacheEntryV0 {
            entries: entries.to_vec(),
            last_used: state.source_type_fact_cache_next_use,
        },
    );
    trim_source_type_fact_cache(state);
}

fn increment_reason_counter(counters: &mut BTreeMap<String, u64>, reason: &str) {
    let count = counters.entry(reason.to_string()).or_default();
    *count = count.saturating_add(1);
}

fn record_source_type_fact_closure_incomplete(
    state: &mut LspShellState,
    reasons: &BTreeSet<&'static str>,
) {
    for reason in reasons {
        increment_reason_counter(
            &mut state
                .source_type_fact_cache_telemetry
                .closure_incomplete_by_reason,
            reason,
        );
    }
}

fn trim_source_type_fact_cache(state: &mut LspShellState) {
    while state.source_type_fact_cache.len() > SOURCE_TYPE_FACT_CACHE_MAX_ENTRIES {
        let Some(key) = state
            .source_type_fact_cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        state.source_type_fact_cache.remove(key.as_str());
    }
}

fn span_source_type_fact_targets(document: &LspTextDocumentState) -> Vec<SourceTypeFactTarget> {
    document
        .source_type_fact_lexical_attempts
        .iter()
        .filter(|attempt| {
            attempt.lexical_disposition == SourceTypeFactLexicalDisposition::Unresolved
                && span_type_fact_shape_is_supported(attempt.shape_class)
        })
        .filter_map(|attempt| {
            let (prefix, suffix) =
                span_type_fact_template_affixes(document.text.as_str(), attempt.byte_span)?;
            Some(SourceTypeFactTarget {
                byte_span: attempt.byte_span,
                expression_id: attempt.expression_id.clone(),
                target_style_uri: attempt.target_style_uri.clone(),
                prefix,
                suffix,
            })
        })
        .collect()
}

fn span_type_fact_shape_is_supported(shape: SourceTypeFactExpressionShape) -> bool {
    matches!(
        shape,
        SourceTypeFactExpressionShape::Call
            | SourceTypeFactExpressionShape::Arithmetic
            | SourceTypeFactExpressionShape::LogicalOperator
            | SourceTypeFactExpressionShape::ComputedNonLiteral
            | SourceTypeFactExpressionShape::NestedTemplate
    )
}

fn span_type_fact_template_affixes(
    source: &str,
    expression_span: ParserByteSpanV0,
) -> Option<(String, String)> {
    let before_expression = source.get(..expression_span.start)?;
    let Some(interpolation_start) = before_expression.rfind("${") else {
        return Some((String::new(), String::new()));
    };
    let before_in_interpolation = source.get(interpolation_start + 2..expression_span.start)?;
    if !before_in_interpolation.chars().all(char::is_whitespace) {
        // A completed earlier template is unrelated to this expression. An
        // active wrapper is ambiguous, so retain the lexical prefix instead.
        return if before_in_interpolation.contains(['}', '`']) {
            Some((String::new(), String::new()))
        } else {
            None
        };
    }

    let after_expression = source.get(expression_span.end..)?;
    let relative_interpolation_end = after_expression.find('}')?;
    let interpolation_end = expression_span.end + relative_interpolation_end;
    if !source
        .get(expression_span.end..interpolation_end)?
        .chars()
        .all(char::is_whitespace)
    {
        return None;
    }

    let prefix_start = source
        .get(..interpolation_start)?
        .char_indices()
        .rev()
        .take_while(|(_, character)| is_ascii_word_continue(*character))
        .last()
        .map(|(index, _)| index)
        .unwrap_or(interpolation_start);
    let suffix_start = interpolation_end + 1;
    let suffix_end = source
        .get(suffix_start..)?
        .char_indices()
        .take_while(|(_, character)| is_ascii_word_continue(*character))
        .last()
        .map(|(index, character)| suffix_start + index + character.len_utf8())
        .unwrap_or(suffix_start);
    Some((
        source.get(prefix_start..interpolation_start)?.to_string(),
        source.get(suffix_start..suffix_end)?.to_string(),
    ))
}

fn tsgo_type_fact_requests_for_document(
    document: &LspTextDocumentState,
    type_fact_targets: &[SourceTypeFactTarget],
    span_type_fact_targets: &[SourceTypeFactTarget],
) -> Option<(TsgoTypeFactRequestV0, TsgoSpanTypeFactRequestV0)> {
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
    let span_targets = span_type_fact_targets
        .iter()
        .filter_map(|target| {
            let start_position =
                utf16_position_for_byte_offset(document.text.as_str(), target.byte_span.start)?;
            let end_position =
                utf16_position_for_byte_offset(document.text.as_str(), target.byte_span.end)?;
            Some(TsgoSpanTypeFactTargetV0::new(
                file_path.clone(),
                target.expression_id.clone(),
                start_position,
                end_position,
            ))
        })
        .collect::<Vec<_>>();
    let workspace_root = workspace_root.to_string_lossy().to_string();
    let config_path = config_path.to_string_lossy().to_string();
    Some((
        TsgoTypeFactRequestV0 {
            workspace_root: workspace_root.clone(),
            config_path: config_path.clone(),
            targets,
        },
        TsgoSpanTypeFactRequestV0::new(workspace_root, config_path, span_targets),
    ))
}

#[cfg(test)]
fn tsgo_type_fact_request_for_document(
    document: &LspTextDocumentState,
    type_fact_targets: &[SourceTypeFactTarget],
) -> Option<TsgoTypeFactRequestV0> {
    let (request, _) = tsgo_type_fact_requests_for_document(document, type_fact_targets, &[])?;
    if request.targets.is_empty() {
        return None;
    }
    Some(request)
}

pub(crate) fn apply_source_type_fact_results_to_document(
    state: &mut LspShellState,
    uri: &str,
    entries: &[TsgoTypeFactResultEntryV0],
) {
    apply_source_type_fact_results_to_document_with_span(state, uri, entries, &[], &[]);
}

fn apply_source_type_fact_results_to_document_with_span(
    state: &mut LspShellState,
    uri: &str,
    entries: &[TsgoTypeFactResultEntryV0],
    span_entries: &[TsgoSpanTypeFactResultEntryV0],
    span_targets: &[SourceTypeFactTarget],
) {
    let Some(document) = state.document(uri).cloned() else {
        return;
    };
    let legacy_targets = document.source_syntax_index.type_fact_targets.clone();
    let mut targets = legacy_targets.clone();
    targets.extend_from_slice(span_targets);
    let span_targets_by_id = span_targets
        .iter()
        .map(|target| (target.expression_id.as_str(), target))
        .collect::<BTreeMap<_, _>>();
    let mut projection_entries = entries.to_vec();
    projection_entries.extend(
        span_entries
            .iter()
            .filter(|entry| {
                span_targets_by_id
                    .get(entry.expression_id.as_str())
                    .is_some_and(|target| {
                        span_type_fact_entry_admissibility(
                            entry,
                            target.prefix.as_str(),
                            target.suffix.as_str(),
                        )
                        .is_ok()
                    })
            })
            .map(|entry| TsgoTypeFactResultEntryV0 {
                file_path: entry.file_path.clone(),
                expression_id: entry.expression_id.clone(),
                resolved_type: entry.resolved_type.clone(),
            }),
    );
    let mut references = document.source_syntax_index.selector_references.clone();
    restore_source_type_fact_prefix_references(
        &mut references,
        document
            .source_type_fact_retired_prefix_references
            .as_slice(),
    );
    remove_source_type_fact_selector_references(
        &mut references,
        document.source_type_fact_selector_references.as_slice(),
    );
    let unavailable_facts =
        tsgo_provider_unavailable_facts_for_type_targets(legacy_targets.as_slice(), entries);
    ensure_referenced_style_documents_loaded_for_type_facts(state, targets.as_slice());
    let projections = project_source_type_fact_targets_with_query(
        state,
        &document,
        targets.as_slice(),
        projection_entries.as_slice(),
    );
    let complete_projection_ids = complete_tsgo_projection_expression_ids(
        targets.as_slice(),
        projection_entries.as_slice(),
        projections.as_slice(),
    );
    let tier_attempts = source_type_fact_tier_attempts_with_span_results(
        document.source_type_fact_lexical_attempts.as_slice(),
        entries,
        span_entries,
        span_targets,
        &complete_projection_ids,
    );
    let mut next_type_fact_references = Vec::new();
    for (target, selector_name) in projections {
        let reference_span = type_fact_template_spans(document.text.as_str(), &target)
            .map(|spans| spans.selector_span)
            .unwrap_or(target.byte_span);
        let reference = source_selector_reference(
            reference_span,
            Some(selector_name),
            SourceSelectorReferenceMatchKind::Exact,
            target.target_style_uri.as_deref(),
            SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection,
        );
        references.push(reference.clone());
        next_type_fact_references.push(reference);
    }
    let mut next_retired_prefix_references = Vec::new();
    for target in targets
        .iter()
        .filter(|target| complete_projection_ids.contains(target.expression_id.as_str()))
    {
        let Some(spans) = type_fact_template_spans(document.text.as_str(), target) else {
            continue;
        };
        next_retired_prefix_references.extend(take_type_fact_prefix_references(
            &mut references,
            spans.prefix_span,
            target.prefix.as_str(),
            target.target_style_uri.as_deref(),
        ));
    }
    canonicalize_omena_query_source_selector_references(&mut references);
    let Some(document) = state.document_mut(uri) else {
        return;
    };
    document.source_syntax_index.selector_references = references;
    document.source_type_fact_selector_references = next_type_fact_references;
    document.source_type_fact_retired_prefix_references = next_retired_prefix_references;
    document.source_type_fact_tier_attempts = tier_attempts;
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

fn span_type_fact_entry_admissibility(
    entry: &TsgoSpanTypeFactResultEntryV0,
    prefix: &str,
    suffix: &str,
) -> Result<(), &'static str> {
    if entry.outcome != SOURCE_TYPE_FACT_OUTCOME_RESOLVED {
        return Err(SOURCE_TYPE_FACT_REASON_OUTCOME_NOT_RESOLVED);
    }
    if !entry.span_exact {
        return Err(SOURCE_TYPE_FACT_REASON_SPAN_NOT_EXACT);
    }
    if entry.non_nullish_member_count == 0 {
        return Err(SOURCE_TYPE_FACT_REASON_EMPTY_EXACT_DOMAIN);
    }
    if entry.non_nullish_member_count != entry.resolved_member_count {
        return Err(SOURCE_TYPE_FACT_REASON_MEMBER_COUNT_MISMATCH);
    }
    if entry.resolved_type.kind != "union" {
        return Err(SOURCE_TYPE_FACT_REASON_NON_UNION_RESOLVED_TYPE);
    }
    if entry.resolved_type.values.is_empty() {
        return Err(SOURCE_TYPE_FACT_REASON_EMPTY_RESOLVED_VALUES);
    }
    if !entry
        .resolved_type
        .values
        .iter()
        .all(|value| value.chars().all(is_css_name_continue))
    {
        return Err(SOURCE_TYPE_FACT_REASON_INVALID_CSS_IDENTIFIER_CHARACTER);
    }
    if !entry
        .resolved_type
        .values
        .iter()
        .all(|value| is_safe_css_identifier(format!("{prefix}{value}{suffix}").as_str()))
    {
        return Err(SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER);
    }
    Ok(())
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
    document.source_type_fact_tier_attempts = source_type_fact_tier_attempts_with_unavailable(
        document.source_type_fact_lexical_attempts.as_slice(),
        reason,
    );
    let retired_prefix_references =
        std::mem::take(&mut document.source_type_fact_retired_prefix_references);
    restore_source_type_fact_prefix_references(
        &mut document.source_syntax_index.selector_references,
        retired_prefix_references.as_slice(),
    );
    remove_source_type_fact_selector_references(
        &mut document.source_syntax_index.selector_references,
        document.source_type_fact_selector_references.as_slice(),
    );
    document.source_type_fact_selector_references.clear();
    canonicalize_omena_query_source_selector_references(
        &mut document.source_syntax_index.selector_references,
    );
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

fn source_type_fact_tier_attempts_with_unavailable(
    lexical_attempts: &[SourceTypeFactLexicalAttempt],
    reason: &'static str,
) -> Vec<LspSourceTypeFactTierAttemptV0> {
    let mut attempts = initial_source_type_fact_tier_attempts(lexical_attempts);
    for (lexical, attempt) in lexical_attempts.iter().zip(attempts.iter_mut()) {
        if lexical.lexical_disposition == SourceTypeFactLexicalDisposition::TypeProviderCandidate
            || (lexical.lexical_disposition == SourceTypeFactLexicalDisposition::Unresolved
                && span_type_fact_shape_is_supported(lexical.shape_class))
        {
            attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE;
            attempt.reason = Some(reason);
        }
    }
    attempts
}

fn source_type_fact_tier_attempts_with_results(
    lexical_attempts: &[SourceTypeFactLexicalAttempt],
    entries: &[TsgoTypeFactResultEntryV0],
    complete_projection_ids: &BTreeSet<String>,
) -> Vec<LspSourceTypeFactTierAttemptV0> {
    let entries_by_id = entries
        .iter()
        .map(|entry| (entry.expression_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut attempts = initial_source_type_fact_tier_attempts(lexical_attempts);
    for (lexical, attempt) in lexical_attempts.iter().zip(attempts.iter_mut()) {
        if !matches!(
            lexical.lexical_disposition,
            SourceTypeFactLexicalDisposition::TypeProviderCandidate
        ) {
            continue;
        }
        if complete_projection_ids.contains(lexical.expression_id.as_str()) {
            attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_RESOLVED;
            attempt.reason = None;
            continue;
        }
        match entries_by_id.get(lexical.expression_id.as_str()) {
            None => {
                attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE;
                attempt.reason = Some(TSGO_PROVIDER_MISSING_RESULT);
            }
            Some(entry) if entry.resolved_type.kind != "union" => {
                attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE;
                attempt.reason = Some(TSGO_PROVIDER_UNRESOLVABLE);
            }
            Some(_) => {
                attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_UNRESOLVED;
                attempt.reason = Some(SOURCE_TYPE_FACT_REASON_NON_EXACT_DOMAIN);
            }
        }
    }
    attempts
}

fn source_type_fact_tier_attempts_with_span_results(
    lexical_attempts: &[SourceTypeFactLexicalAttempt],
    entries: &[TsgoTypeFactResultEntryV0],
    span_entries: &[TsgoSpanTypeFactResultEntryV0],
    span_targets: &[SourceTypeFactTarget],
    complete_projection_ids: &BTreeSet<String>,
) -> Vec<LspSourceTypeFactTierAttemptV0> {
    let mut attempts = source_type_fact_tier_attempts_with_results(
        lexical_attempts,
        entries,
        complete_projection_ids,
    );
    let span_entries_by_id = span_entries
        .iter()
        .map(|entry| (entry.expression_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let span_targets_by_id = span_targets
        .iter()
        .map(|target| (target.expression_id.as_str(), target))
        .collect::<BTreeMap<_, _>>();
    for (lexical, attempt) in lexical_attempts.iter().zip(attempts.iter_mut()) {
        if lexical.lexical_disposition != SourceTypeFactLexicalDisposition::Unresolved
            || !span_type_fact_shape_is_supported(lexical.shape_class)
        {
            continue;
        }
        if complete_projection_ids.contains(lexical.expression_id.as_str()) {
            attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_RESOLVED;
            attempt.reason = None;
            continue;
        }
        match span_entries_by_id.get(lexical.expression_id.as_str()) {
            None => {
                attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE;
                attempt.reason = Some(TSGO_PROVIDER_MISSING_RESULT);
            }
            Some(entry) if entry.outcome == SOURCE_TYPE_FACT_OUTCOME_REFUSED => {
                attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_REFUSED;
                attempt.reason = Some(entry.reason);
            }
            Some(entry) => {
                let admissibility = span_targets_by_id
                    .get(lexical.expression_id.as_str())
                    .map_or(Err(SOURCE_TYPE_FACT_REASON_NON_EXACT_DOMAIN), |target| {
                        span_type_fact_entry_admissibility(
                            entry,
                            target.prefix.as_str(),
                            target.suffix.as_str(),
                        )
                    });
                if let Err(reason) = admissibility {
                    attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_REFUSED;
                    attempt.reason = Some(reason);
                } else {
                    attempt.outcome = SOURCE_TYPE_FACT_OUTCOME_UNRESOLVED;
                    attempt.reason = Some(SOURCE_TYPE_FACT_REASON_NON_EXACT_DOMAIN);
                }
            }
        }
    }
    attempts
}

fn remove_source_type_fact_selector_references(
    references: &mut Vec<SourceSelectorReferenceFact>,
    type_fact_references: &[SourceSelectorReferenceFact],
) {
    references.retain(|reference| !type_fact_references.contains(reference));
}

fn restore_source_type_fact_prefix_references(
    references: &mut Vec<SourceSelectorReferenceFact>,
    retired_prefix_references: &[SourceSelectorReferenceFact],
) {
    for reference in retired_prefix_references {
        if !references.contains(reference) {
            references.push(reference.clone());
        }
    }
}

fn take_type_fact_prefix_references(
    references: &mut Vec<SourceSelectorReferenceFact>,
    prefix_span: ParserByteSpanV0,
    prefix: &str,
    target_style_uri: Option<&str>,
) -> Vec<SourceSelectorReferenceFact> {
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut retired = Vec::new();
    references.retain(|reference| {
        let matches = reference.match_kind == SourceSelectorReferenceMatchKind::Prefix
            && reference.surface == SourceSelectorReferenceSurface::OmenaQuerySourceSyntaxIndex
            && reference.byte_span == prefix_span
            && reference.selector_name.as_deref() == Some(prefix)
            && reference.target_style_uri.as_deref() == target_style_uri;
        if matches {
            retired.push(reference.clone());
        }
        !matches
    });
    retired
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TypeFactTemplateSpans {
    selector_span: ParserByteSpanV0,
    prefix_span: ParserByteSpanV0,
}

fn type_fact_template_spans(
    source: &str,
    target: &SourceTypeFactTarget,
) -> Option<TypeFactTemplateSpans> {
    let before_expression = source.get(..target.byte_span.start)?;
    let interpolation_start = before_expression.rfind("${")?;
    if !source
        .get(interpolation_start + 2..target.byte_span.start)?
        .chars()
        .all(char::is_whitespace)
    {
        return None;
    }
    let prefix_start = interpolation_start.checked_sub(target.prefix.len())?;
    if source.get(prefix_start..interpolation_start)? != target.prefix {
        return None;
    }
    let after_expression = source.get(target.byte_span.end..)?;
    let relative_interpolation_end = after_expression.find('}')?;
    let interpolation_end = target.byte_span.end + relative_interpolation_end;
    if !source
        .get(target.byte_span.end..interpolation_end)?
        .chars()
        .all(char::is_whitespace)
    {
        return None;
    }
    let suffix_start = interpolation_end + 1;
    let suffix_end = suffix_start.checked_add(target.suffix.len())?;
    if source.get(suffix_start..suffix_end)? != target.suffix {
        return None;
    }
    Some(TypeFactTemplateSpans {
        selector_span: ParserByteSpanV0 {
            start: prefix_start,
            end: suffix_end,
        },
        prefix_span: ParserByteSpanV0 {
            start: prefix_start,
            end: interpolation_start,
        },
    })
}

fn complete_tsgo_projection_expression_ids(
    targets: &[SourceTypeFactTarget],
    entries: &[TsgoTypeFactResultEntryV0],
    projections: &[(SourceTypeFactTarget, String)],
) -> BTreeSet<String> {
    let entries_by_id = entries
        .iter()
        .map(|entry| (entry.expression_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut projected_names_by_id = BTreeMap::<&str, Vec<&str>>::new();
    for (target, selector_name) in projections {
        projected_names_by_id
            .entry(target.expression_id.as_str())
            .or_default()
            .push(selector_name.as_str());
    }

    targets
        .iter()
        .filter_map(|target| {
            let entry = entries_by_id.get(target.expression_id.as_str())?;
            if entry.resolved_type.kind != "union" || entry.resolved_type.values.is_empty() {
                return None;
            }
            if entry
                .resolved_type
                .values
                .iter()
                .any(|value| !value.chars().all(is_css_name_continue))
            {
                return None;
            }
            let mut expected = entry
                .resolved_type
                .values
                .iter()
                .map(|value| format!("{}{}{}", target.prefix, value, target.suffix))
                .collect::<Vec<_>>();
            if expected.is_empty()
                || expected
                    .iter()
                    .any(|selector_name| !is_safe_css_identifier(selector_name))
            {
                return None;
            }
            expected.sort();
            expected.dedup();
            let mut projected = projected_names_by_id
                .get(target.expression_id.as_str())
                .cloned()
                .unwrap_or_default();
            projected.sort();
            projected.dedup();
            (projected.len() == expected.len()
                && projected
                    .iter()
                    .zip(expected.iter())
                    .all(|(projected, expected)| *projected == expected))
            .then(|| target.expression_id.clone())
        })
        .collect()
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
    let line_index = omena_syntax::OmenaLineIndexV0::new(document.text.as_str());
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
                "range": crate::protocol::parser_range_for_byte_span_with_line_index(
                    document.text.as_str(),
                    &line_index,
                    target.byte_span,
                ),
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
        .filter(|value| value.chars().all(is_css_name_continue))
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
    use omena_query::OmenaQueryStyleResolutionInputsV0;
    use omena_tsgo_client::TsgoResolvedTypeV0;
    use serde_json::json;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn source_type_fact_outcome_values_match_authored_contract() -> TestResult {
        let contract: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/source-type-fact-outcome-values.json"
        ))?;
        assert_eq!(contract["schemaVersion"], "0");
        assert_eq!(contract["product"], "omena.lsp.source-type-fact-outcomes");
        assert_eq!(
            contract["notAttempted"].as_str(),
            Some(SOURCE_TYPE_FACT_OUTCOME_NOT_ATTEMPTED)
        );
        Ok(())
    }

    #[test]
    fn source_type_fact_sidecar_refuses_changed_tsconfig_parent_as_environment() -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-source-type-fact-parent-freshness-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&workspace_root);
        fs::create_dir_all(workspace_root.join("src"))?;
        let parent_config = workspace_root.join("tsconfig.base.json");
        let config = workspace_root.join("tsconfig.json");
        fs::write(&parent_config, r#"{"compilerOptions":{"strict":true}}"#)?;
        fs::write(
            &config,
            r#"{"extends":"./tsconfig.base.json","compilerOptions":{}}"#,
        )?;
        let document_path = workspace_root.join("src/App.tsx");
        fs::write(&document_path, "export const value = 1;")?;
        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        let document_uri = path_to_file_uri(document_path.as_path());
        let mut state = initialized_source_type_fact_test_state(workspace_uri.as_str());
        let document = crate::lsp_text_document_state(
            document_uri.clone(),
            Some(workspace_uri.clone()),
            "typescriptreact".to_string(),
            1,
            "export const value = 1;".to_string(),
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let (environment_before, closure_incomplete_reasons) =
            source_type_fact_environment_fingerprint(
                workspace_root.to_string_lossy().as_ref(),
                config.to_string_lossy().as_ref(),
            );
        assert!(!closure_incomplete_reasons.contains(SOURCE_TYPE_FACT_CLOSURE_PACKAGE_EXTENDS));
        let freshness_before = test_sidecar_freshness(environment_before, "binary:stable");
        let entries = vec![test_type_fact_entry(document_path.as_path())];
        assert!(store_source_type_fact_sidecar_with_freshness(
            &state,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "parent-key",
            entries.as_slice(),
            &freshness_before,
        ));

        fs::write(&parent_config, r#"{"compilerOptions":{"strict":false}}"#)?;
        assert_eq!(
            load_source_type_fact_sidecar_with_freshness(
                &state,
                Some(workspace_uri.as_str()),
                document_uri.as_str(),
                "parent-key",
                &freshness_before,
            ),
            SourceTypeFactSidecarLoadV0::Hit(entries.clone()),
            "a payload-derived digest cannot detect a dependency change"
        );
        let (environment_after, _) = source_type_fact_environment_fingerprint(
            workspace_root.to_string_lossy().as_ref(),
            config.to_string_lossy().as_ref(),
        );
        let freshness_after = test_sidecar_freshness(environment_after, "binary:stable");
        assert!(
            cached_source_type_fact_entries(
                &mut state,
                &document,
                Some("parent-key"),
                &freshness_after,
            )
            .is_none()
        );
        assert_eq!(
            state
                .source_type_fact_cache_telemetry
                .sidecar_refused_by_reason
                .get("environment"),
            Some(&1)
        );

        fs::write(&parent_config, r#"{"compilerOptions":{"strict":true}}"#)?;
        let (environment_restored, _) = source_type_fact_environment_fingerprint(
            workspace_root.to_string_lossy().as_ref(),
            config.to_string_lossy().as_ref(),
        );
        let restored = test_sidecar_freshness(environment_restored, "binary:stable");
        assert_eq!(
            load_source_type_fact_sidecar_with_freshness(
                &state,
                Some(workspace_uri.as_str()),
                document_uri.as_str(),
                "parent-key",
                &restored,
            ),
            SourceTypeFactSidecarLoadV0::Hit(entries),
            "an unchanged environment must serve the cold-equivalent payload"
        );
        fs::remove_dir_all(&workspace_root)?;
        Ok(())
    }

    #[test]
    fn source_type_fact_sidecar_refuses_changed_tsgo_binary_as_binary() -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-source-type-fact-binary-freshness-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&workspace_root);
        fs::create_dir_all(&workspace_root)?;
        let document_path = workspace_root.join("App.tsx");
        let binary_path = workspace_root.join("tsgo");
        fs::write(&document_path, "export const value = 1;")?;
        fs::write(&binary_path, "binary-v1")?;
        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        let document_uri = path_to_file_uri(document_path.as_path());
        let mut state = initialized_source_type_fact_test_state(workspace_uri.as_str());
        let document = crate::lsp_text_document_state(
            document_uri.clone(),
            Some(workspace_uri.clone()),
            "typescriptreact".to_string(),
            1,
            "export const value = 1;".to_string(),
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let freshness_before = test_sidecar_freshness(
            "environment:stable",
            source_type_fact_tsgo_binary_fingerprint(binary_path.as_path()),
        );
        assert!(store_source_type_fact_sidecar_with_freshness(
            &state,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "binary-key",
            &[test_type_fact_entry(document_path.as_path())],
            &freshness_before,
        ));

        fs::write(&binary_path, "binary-v2")?;
        let freshness_after = test_sidecar_freshness(
            "environment:stable",
            source_type_fact_tsgo_binary_fingerprint(binary_path.as_path()),
        );
        assert!(
            cached_source_type_fact_entries(
                &mut state,
                &document,
                Some("binary-key"),
                &freshness_after,
            )
            .is_none()
        );
        assert_eq!(
            state
                .source_type_fact_cache_telemetry
                .sidecar_refused_by_reason
                .get("binary"),
            Some(&1)
        );
        fs::remove_dir_all(&workspace_root)?;
        Ok(())
    }

    #[test]
    fn incomplete_source_type_fact_closure_skips_sidecar_but_keeps_memory_answer() -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-source-type-fact-fail-soft-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&workspace_root);
        fs::create_dir_all(&workspace_root)?;
        let document_path = workspace_root.join("App.tsx");
        fs::write(&document_path, "export const value = 1;")?;
        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        let document_uri = path_to_file_uri(document_path.as_path());
        let mut state = initialized_source_type_fact_test_state(workspace_uri.as_str());
        let document = crate::lsp_text_document_state(
            document_uri.clone(),
            Some(workspace_uri.clone()),
            "typescriptreact".to_string(),
            1,
            "export const value = 1;".to_string(),
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let freshness = test_sidecar_freshness("environment:stable", "binary:stable");
        let context = SourceTypeFactCacheContextV0 {
            freshness: freshness.clone(),
            closure_incomplete_reasons: BTreeSet::from([SOURCE_TYPE_FACT_CLOSURE_INDEX_BUDGET]),
        };
        let entries = vec![test_type_fact_entry(document_path.as_path())];
        record_source_type_fact_closure_incomplete(&mut state, &context.closure_incomplete_reasons);
        cache_source_type_fact_results(
            &mut state,
            &document,
            "fail-soft-key",
            entries.as_slice(),
            &context,
        );
        let path = crate::source_type_fact_cache::source_type_fact_sidecar_file_path_for_test(
            &state,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
        )
        .ok_or_else(|| std::io::Error::other("sidecar path should resolve"))?;
        assert!(
            !path.exists(),
            "incomplete closure must not write a sidecar"
        );
        assert_eq!(
            cached_source_type_fact_entries(
                &mut state,
                &document,
                Some("fail-soft-key"),
                &freshness,
            ),
            Some(entries)
        );
        assert_eq!(
            state
                .source_type_fact_cache_telemetry
                .closure_incomplete_by_reason
                .get(SOURCE_TYPE_FACT_CLOSURE_INDEX_BUDGET),
            Some(&1)
        );
        fs::remove_dir_all(&workspace_root)?;
        Ok(())
    }

    #[test]
    fn jsonc_tsconfig_extends_incompleteness_skips_sidecar_but_keeps_memory_answer() -> TestResult {
        assert_unresolved_tsconfig_extends_fails_soft(
            "jsonc-comment",
            "{\n  // TypeScript accepts comments in tsconfig files.\n  \"extends\": \"./tsconfig.base.json\"\n}\n",
            SOURCE_TYPE_FACT_CLOSURE_TSCONFIG_PARSE_FAILED,
        )
    }

    #[test]
    fn array_tsconfig_extends_incompleteness_skips_sidecar_but_keeps_memory_answer() -> TestResult {
        assert_unresolved_tsconfig_extends_fails_soft(
            "array-extends",
            "{\n  \"extends\": [\"./tsconfig.base.json\"]\n}\n",
            SOURCE_TYPE_FACT_CLOSURE_TSCONFIG_EXTENDS_ARRAY,
        )
    }

    #[test]
    fn source_type_fact_sidecar_refuses_schema_zero_shard() -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-source-type-fact-schema-zero-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&workspace_root);
        fs::create_dir_all(&workspace_root)?;
        let document_path = workspace_root.join("App.tsx");
        fs::write(&document_path, "export const value = 1;")?;
        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        let document_uri = path_to_file_uri(document_path.as_path());
        let mut state = initialized_source_type_fact_test_state(workspace_uri.as_str());
        let document = crate::lsp_text_document_state(
            document_uri.clone(),
            Some(workspace_uri.clone()),
            "typescriptreact".to_string(),
            1,
            "export const value = 1;".to_string(),
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let freshness = test_sidecar_freshness("environment:stable", "binary:stable");
        let entries = vec![test_type_fact_entry(document_path.as_path())];
        assert!(store_source_type_fact_sidecar_with_freshness(
            &state,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "schema-key",
            entries.as_slice(),
            &freshness,
        ));
        let path = crate::source_type_fact_cache::source_type_fact_sidecar_file_path_for_test(
            &state,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
        )
        .ok_or_else(|| std::io::Error::other("sidecar path should resolve"))?;
        let mut shard = serde_json::from_slice::<Value>(fs::read(path.as_path())?.as_slice())?;
        assert_eq!(
            shard["schemaVersion"], "1",
            "new source type fact sidecars must use schema version 1"
        );
        shard["schemaVersion"] = Value::String("0".to_string());
        fs::write(path.as_path(), serde_json::to_vec(&shard)?)?;

        assert!(
            cached_source_type_fact_entries(&mut state, &document, Some("schema-key"), &freshness,)
                .is_none()
        );
        assert_eq!(
            state
                .source_type_fact_cache_telemetry
                .sidecar_refused_by_reason
                .get("schema"),
            Some(&1)
        );
        eprintln!("schemaVersion=1 oldShardVersion=0 refusalReason=schema");
        fs::remove_dir_all(&workspace_root)?;
        Ok(())
    }

    #[test]
    fn source_type_fact_cache_evicts_least_recently_used_entry() {
        let mut state = LspShellState::default();
        for index in 0..=SOURCE_TYPE_FACT_CACHE_MAX_ENTRIES {
            state.source_type_fact_cache.insert(
                format!("key-{index:03}"),
                LspSourceTypeFactCacheEntryV0 {
                    entries: Vec::new(),
                    last_used: index as u64,
                },
            );
        }
        if let Some(entry) = state.source_type_fact_cache.get_mut("key-000") {
            entry.last_used = u64::MAX;
        }
        trim_source_type_fact_cache(&mut state);

        assert!(state.source_type_fact_cache.contains_key("key-000"));
        assert!(!state.source_type_fact_cache.contains_key("key-001"));
        assert_eq!(
            state.source_type_fact_cache.len(),
            SOURCE_TYPE_FACT_CACHE_MAX_ENTRIES
        );
    }

    fn assert_unresolved_tsconfig_extends_fails_soft(
        fixture_id: &str,
        config_source: &str,
        expected_reason: &'static str,
    ) -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-source-type-fact-{fixture_id}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&workspace_root);
        fs::create_dir_all(workspace_root.join("src"))?;
        let parent_config = workspace_root.join("tsconfig.base.json");
        let config = workspace_root.join("tsconfig.json");
        fs::write(&parent_config, r#"{"compilerOptions":{"strict":true}}"#)?;
        fs::write(&config, config_source)?;
        let document_path = workspace_root.join("src/App.tsx");
        fs::write(&document_path, "export const value = 1;")?;
        let workspace_uri = path_to_file_uri(workspace_root.as_path());
        let document_uri = path_to_file_uri(document_path.as_path());
        let mut state = initialized_source_type_fact_test_state(workspace_uri.as_str());
        state.source_type_fact_watched_files_observed = true;
        let document = crate::lsp_text_document_state(
            document_uri.clone(),
            Some(workspace_uri.clone()),
            "typescriptreact".to_string(),
            1,
            "export const value = 1;".to_string(),
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let request = TsgoTypeFactRequestV0 {
            workspace_root: workspace_root.to_string_lossy().to_string(),
            config_path: config.to_string_lossy().to_string(),
            targets: Vec::new(),
        };
        let context_before = source_type_fact_cache_context(&state, &document, &request);
        record_source_type_fact_closure_incomplete(
            &mut state,
            &context_before.closure_incomplete_reasons,
        );
        let entries = vec![test_type_fact_entry(document_path.as_path())];
        cache_source_type_fact_results(
            &mut state,
            &document,
            fixture_id,
            entries.as_slice(),
            &context_before,
        );
        let path = crate::source_type_fact_cache::source_type_fact_sidecar_file_path_for_test(
            &state,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
        )
        .ok_or_else(|| std::io::Error::other("sidecar path should resolve"))?;

        fs::write(&parent_config, r#"{"compilerOptions":{"strict":false}}"#)?;
        let context_after = source_type_fact_cache_context(&state, &document, &request);
        assert_eq!(
            context_after.freshness.environment_fingerprint,
            context_before.freshness.environment_fingerprint,
            "an unresolved extends edge cannot fingerprint the edited parent"
        );
        let memory_answer = cached_source_type_fact_entries(
            &mut state,
            &document,
            Some(fixture_id),
            &context_after.freshness,
        );
        let recorded_reason = state
            .source_type_fact_cache_telemetry
            .closure_incomplete_by_reason
            .get(expected_reason)
            .copied();
        assert!(
            !path.exists()
                && context_before
                    .closure_incomplete_reasons
                    .contains(expected_reason)
                && memory_answer == Some(entries)
                && recorded_reason == Some(1),
            "unresolved tsconfig extends must fail soft: sidecar_exists={}, reasons={:?}, memory_answer={memory_answer:?}, recorded_reason={recorded_reason:?}",
            path.exists(),
            context_before.closure_incomplete_reasons,
        );
        eprintln!(
            "closureReason={expected_reason} sidecarExists={} memoryAnswer={} parentEditFingerprintChanged={}",
            path.exists(),
            memory_answer.is_some(),
            context_after.freshness.environment_fingerprint
                != context_before.freshness.environment_fingerprint,
        );
        fs::remove_dir_all(&workspace_root)?;
        Ok(())
    }

    fn initialized_source_type_fact_test_state(workspace_uri: &str) -> LspShellState {
        let mut state = LspShellState::default();
        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "workspaceFolders": [{
                        "uri": workspace_uri,
                        "name": "source-type-fact-freshness",
                    }],
                },
            }),
        );
        state
    }

    fn test_sidecar_freshness(
        environment_fingerprint: impl Into<String>,
        tsgo_binary_fingerprint: impl Into<String>,
    ) -> SourceTypeFactSidecarFreshnessV0 {
        SourceTypeFactSidecarFreshnessV0 {
            environment_fingerprint: environment_fingerprint.into(),
            tsgo_binary_fingerprint: tsgo_binary_fingerprint.into(),
            collection_provenance: json!({
                "snapshotHandle": Value::Null,
                "projectByFile": {},
                "disclosure": "test",
            }),
        }
    }

    fn test_type_fact_entry(file_path: &Path) -> TsgoTypeFactResultEntryV0 {
        TsgoTypeFactResultEntryV0 {
            file_path: file_path.to_string_lossy().to_string(),
            expression_id: "test-expression".to_string(),
            resolved_type: TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["small".to_string()],
            },
        }
    }

    #[test]
    fn css_identifier_safety_matches_shared_cases() -> TestResult {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../omena-css-identifier-safety-cases.json"
        ))?;
        let safe = cases["safe"]
            .as_array()
            .ok_or_else(|| std::io::Error::other("safe identifier cases must be an array"))?;
        let unsafe_cases = cases["unsafe"]
            .as_array()
            .ok_or_else(|| std::io::Error::other("unsafe identifier cases must be an array"))?;
        assert!(!safe.is_empty(), "safe identifier cases must not be empty");
        assert!(
            !unsafe_cases.is_empty(),
            "unsafe identifier cases must not be empty"
        );
        for value in safe {
            let value = value
                .as_str()
                .ok_or_else(|| std::io::Error::other("safe identifier case must be a string"))?;
            assert!(is_safe_css_identifier(value), "expected safe: {value:?}");
        }
        for value in unsafe_cases {
            let value = value
                .as_str()
                .ok_or_else(|| std::io::Error::other("unsafe identifier case must be a string"))?;
            assert!(!is_safe_css_identifier(value), "expected unsafe: {value:?}");
        }
        Ok(())
    }

    #[test]
    fn exact_span_admissibility_reports_each_failed_contract() {
        let base = TsgoSpanTypeFactResultEntryV0::resolved(
            "/workspace/App.ts".to_string(),
            "expression-1".to_string(),
            "exactFiniteDomain",
            1,
            TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["primary".to_string()],
            },
        );
        assert_eq!(span_type_fact_entry_admissibility(&base, "", ""), Ok(()));

        let mut outcome = base.clone();
        outcome.outcome = "unresolved";
        let mut span = base.clone();
        span.span_exact = false;
        let mut empty_domain = base.clone();
        empty_domain.non_nullish_member_count = 0;
        empty_domain.resolved_member_count = 0;
        let mut count_mismatch = base.clone();
        count_mismatch.resolved_member_count = 0;
        let mut non_union = base.clone();
        non_union.resolved_type.kind = "unresolvable";
        let mut empty_values = base.clone();
        empty_values.resolved_type.values.clear();
        let mut invalid_character = base.clone();
        invalid_character.resolved_type.values = vec!["not valid".to_string()];
        let mut unsafe_start = base;
        unsafe_start.resolved_type.values = vec!["9valid".to_string()];

        for (entry, expected_reason) in [
            (outcome, SOURCE_TYPE_FACT_REASON_OUTCOME_NOT_RESOLVED),
            (span, SOURCE_TYPE_FACT_REASON_SPAN_NOT_EXACT),
            (empty_domain, SOURCE_TYPE_FACT_REASON_EMPTY_EXACT_DOMAIN),
            (
                count_mismatch,
                SOURCE_TYPE_FACT_REASON_MEMBER_COUNT_MISMATCH,
            ),
            (non_union, SOURCE_TYPE_FACT_REASON_NON_UNION_RESOLVED_TYPE),
            (empty_values, SOURCE_TYPE_FACT_REASON_EMPTY_RESOLVED_VALUES),
            (
                invalid_character,
                SOURCE_TYPE_FACT_REASON_INVALID_CSS_IDENTIFIER_CHARACTER,
            ),
            (unsafe_start, SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER),
        ] {
            assert_eq!(
                span_type_fact_entry_admissibility(&entry, "", ""),
                Err(expected_reason)
            );
        }
    }

    #[test]
    fn span_type_fact_admission_uses_the_composed_selector_name() {
        let cases = [
            ("btn-", "", vec!["1".to_string(), "2".to_string()], Ok(())),
            (
                "",
                "",
                vec!["1".to_string(), "2".to_string()],
                Err(SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER),
            ),
            (
                "",
                "-tail",
                vec!["9x".to_string()],
                Err(SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER),
            ),
            (
                "btn-",
                "",
                vec!["not valid".to_string()],
                Err(SOURCE_TYPE_FACT_REASON_INVALID_CSS_IDENTIFIER_CHARACTER),
            ),
        ];

        for (prefix, suffix, values, expected) in cases {
            let entry = TsgoSpanTypeFactResultEntryV0::resolved(
                "/workspace/App.ts".to_string(),
                format!("expression-{prefix}-{suffix}"),
                "exactFiniteDomain",
                values.len(),
                TsgoResolvedTypeV0 {
                    kind: "union",
                    values,
                },
            );
            assert_eq!(
                span_type_fact_entry_admissibility(&entry, prefix, suffix),
                expected,
                "admission should classify the composed selector {prefix}<member>{suffix}"
            );
        }
    }

    #[test]
    fn exact_span_type_facts_project_only_complete_css_identifier_domains() -> TestResult {
        let workspace_root = std::env::temp_dir().join(format!(
            "omena-lsp-exact-span-type-facts-{}",
            std::process::id()
        ));
        let src_dir = workspace_root.join("src");
        let source_path = src_dir.join("App.tsx");
        let style_path = src_dir.join("App.module.scss");
        let _ = std::fs::remove_dir_all(&workspace_root);
        std::fs::create_dir_all(&src_dir)?;
        std::fs::write(workspace_root.join("tsconfig.json"), "{}")?;
        let source_text = r#"import bind from "classnames/bind";
	import styles from "./App.module.scss";
	const cx = bind.bind(styles);
	declare function pickTone(): "primary" | "secondary";
	declare function pickSize(): 1 | 2;
	export const App = () => <>
	  <div className={cx(`theme-${pickTone()}-active`)} />
	  <div className={cx(`btn-${pickSize()}`)} />
	</>;"#;
        let style_text = ".theme-primary-active { color: red; }\n.theme-secondary-active { color: blue; }\n.btn-1 { display: block; }\n.btn-2 { display: grid; }";
        std::fs::write(&source_path, source_text)?;
        std::fs::write(&style_path, style_text)?;
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
                    "workspaceFolders": [{
                        "uri": workspace_uri,
                        "name": "exact-span-type-facts",
                    }],
                },
            }),
        );
        for (uri, language_id, text) in [
            (style_uri.as_str(), "scss", style_text),
            (source_uri.as_str(), "typescriptreact", source_text),
        ] {
            handle_lsp_message(
                &mut state,
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/didOpen",
                    "params": {
                        "textDocument": {
                            "uri": uri,
                            "languageId": language_id,
                            "version": 1,
                            "text": text,
                        },
                    },
                }),
            );
        }

        let (target, numeric_target, file_path, baseline_references) = {
            let document = state
                .document(source_uri.as_str())
                .ok_or_else(|| std::io::Error::other("source document should be open"))?;
            let span_targets = span_source_type_fact_targets(document);
            let target = span_targets
                .iter()
                .find(|target| {
                    source_text.get(target.byte_span.start..target.byte_span.end)
                        == Some("pickTone()")
                })
                .cloned()
                .ok_or_else(|| std::io::Error::other("call expression should be harvested"))?;
            let numeric_target = span_targets
                .iter()
                .find(|target| {
                    source_text.get(target.byte_span.start..target.byte_span.end)
                        == Some("pickSize()")
                })
                .cloned()
                .ok_or_else(|| {
                    std::io::Error::other("numeric call expression should be harvested")
                })?;
            assert_eq!(target.prefix, "theme-");
            assert_eq!(target.suffix, "-active");
            assert_eq!(numeric_target.prefix, "btn-");
            assert_eq!(numeric_target.suffix, "");
            let unavailable_attempts = source_type_fact_tier_attempts_with_unavailable(
                document.source_type_fact_lexical_attempts.as_slice(),
                TSGO_PROVIDER_PROCESS_UNAVAILABLE,
            );
            assert!(unavailable_attempts.iter().any(|attempt| {
                attempt.expression_id == target.expression_id
                    && attempt.outcome == SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE
                    && attempt.reason == Some(TSGO_PROVIDER_PROCESS_UNAVAILABLE)
            }));
            let (_, span_request) =
                tsgo_type_fact_requests_for_document(document, &[], std::slice::from_ref(&target))
                    .ok_or_else(|| std::io::Error::other("span request should build"))?;
            let request_target = span_request
                .targets
                .first()
                .ok_or_else(|| std::io::Error::other("span request should contain one target"))?;
            assert_eq!(
                request_target.start_position,
                utf16_position_for_byte_offset(source_text, target.byte_span.start)
                    .ok_or_else(|| std::io::Error::other("start position should convert"))?
            );
            assert_eq!(
                request_target.end_position,
                utf16_position_for_byte_offset(source_text, target.byte_span.end)
                    .ok_or_else(|| std::io::Error::other("end position should convert"))?
            );
            (
                target,
                numeric_target,
                request_target.file_path.clone(),
                document
                    .source_syntax_index
                    .selector_references
                    .iter()
                    .filter(|reference| {
                        reference.surface
                            != SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection
                    })
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        };

        let resolved = TsgoSpanTypeFactResultEntryV0::resolved(
            file_path.clone(),
            target.expression_id.clone(),
            "exactFiniteDomain",
            2,
            TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["primary".to_string(), "secondary".to_string()],
            },
        );
        apply_source_type_fact_results_to_document_with_span(
            &mut state,
            source_uri.as_str(),
            &[],
            std::slice::from_ref(&resolved),
            std::slice::from_ref(&target),
        );
        let document = state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?;
        let projected = document
            .source_syntax_index
            .selector_references
            .iter()
            .filter(|reference| {
                reference.surface == SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection
            })
            .filter_map(|reference| reference.selector_name.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            projected,
            BTreeSet::from([
                "theme-primary-active".to_string(),
                "theme-secondary-active".to_string()
            ])
        );
        assert!(
            !document
                .source_syntax_index
                .selector_references
                .iter()
                .any(|reference| {
                    reference.match_kind == SourceSelectorReferenceMatchKind::Prefix
                        && reference.selector_name.as_deref() == Some("theme-")
                })
        );
        assert!(
            document
                .source_type_fact_tier_attempts
                .iter()
                .any(|attempt| {
                    attempt.expression_id == target.expression_id
                        && attempt.outcome == SOURCE_TYPE_FACT_OUTCOME_RESOLVED
                })
        );

        let numeric = TsgoSpanTypeFactResultEntryV0::resolved(
            file_path.clone(),
            numeric_target.expression_id.clone(),
            "exactFiniteDomain",
            2,
            TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["1".to_string(), "2".to_string()],
            },
        );
        apply_source_type_fact_results_to_document_with_span(
            &mut state,
            source_uri.as_str(),
            &[],
            std::slice::from_ref(&numeric),
            std::slice::from_ref(&numeric_target),
        );
        let document = state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?;
        let numeric_projected = document
            .source_syntax_index
            .selector_references
            .iter()
            .filter(|reference| {
                reference.surface == SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection
            })
            .filter_map(|reference| reference.selector_name.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            numeric_projected,
            BTreeSet::from(["btn-1".to_string(), "btn-2".to_string()])
        );
        assert!(
            document
                .source_type_fact_tier_attempts
                .iter()
                .any(|attempt| {
                    attempt.expression_id == numeric_target.expression_id
                        && attempt.outcome == SOURCE_TYPE_FACT_OUTCOME_RESOLVED
                        && attempt.reason.is_none()
                })
        );

        let refused = TsgoSpanTypeFactResultEntryV0::refused(
            file_path.clone(),
            target.expression_id.clone(),
            "nodeSpanMismatch",
            false,
            0,
            0,
        );
        apply_source_type_fact_results_to_document_with_span(
            &mut state,
            source_uri.as_str(),
            &[],
            std::slice::from_ref(&refused),
            std::slice::from_ref(&target),
        );
        let document = state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?;
        assert_eq!(
            document
                .source_syntax_index
                .selector_references
                .iter()
                .filter(|reference| {
                    reference.surface != SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection
                })
                .cloned()
                .collect::<Vec<_>>(),
            baseline_references
        );
        assert!(
            document
                .source_type_fact_tier_attempts
                .iter()
                .any(|attempt| {
                    attempt.expression_id == target.expression_id
                        && attempt.outcome == SOURCE_TYPE_FACT_OUTCOME_REFUSED
                        && attempt.reason == Some("nodeSpanMismatch")
                })
        );

        let unsafe_value = TsgoSpanTypeFactResultEntryV0::resolved(
            file_path,
            target.expression_id.clone(),
            "exactFiniteDomain",
            1,
            TsgoResolvedTypeV0 {
                kind: "union",
                values: vec!["9valid".to_string()],
            },
        );
        let unsafe_target = SourceTypeFactTarget {
            prefix: String::new(),
            suffix: String::new(),
            ..target.clone()
        };
        apply_source_type_fact_results_to_document_with_span(
            &mut state,
            source_uri.as_str(),
            &[],
            std::slice::from_ref(&unsafe_value),
            std::slice::from_ref(&unsafe_target),
        );
        let document = state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?;
        assert!(
            document
                .source_type_fact_tier_attempts
                .iter()
                .any(|attempt| {
                    attempt.expression_id == target.expression_id
                        && attempt.outcome == SOURCE_TYPE_FACT_OUTCOME_REFUSED
                        && attempt.reason == Some(SOURCE_TYPE_FACT_REASON_UNSAFE_CSS_IDENTIFIER)
                })
        );

        let _ = std::fs::remove_dir_all(&workspace_root);
        Ok(())
    }

    #[test]
    fn span_type_fact_template_affixes_refuse_ambiguous_wrappers() -> Result<(), String> {
        let source = "const value = `theme-${(pickTone())}-active`;";
        let start = source
            .find("pickTone()")
            .ok_or_else(|| "fixture should contain call".to_string())?;
        let end = start + "pickTone()".len();

        assert_eq!(
            span_type_fact_template_affixes(source, ParserByteSpanV0 { start, end }),
            None
        );
        Ok(())
    }

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
            ".small { color: red; }\n.medium { color: blue; }\n.tone-light { opacity: 0.5; }\n.tone-dark { opacity: 1; }",
        )?;
        let source_text = r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
interface BadgeProps { size: "small" | "medium"; }
declare function pickTone(): "light" | "dark";
export function Badge({ size }: BadgeProps) {
  return <span className={cx(size, `tone-${pickTone()}`)} />;
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
                        "text": ".small { color: red; }\n.medium { color: blue; }\n.tone-light { opacity: 0.5; }\n.tone-dark { opacity: 1; }",
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

        let (cache_key, entry, freshness) = {
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
            let context = source_type_fact_cache_context(&state, document, &request);
            let cache_key = source_type_fact_cache_key(
                &state,
                document,
                &request,
                type_fact_targets.as_slice(),
                &context.freshness,
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
                context.freshness,
            )
        };
        crate::source_type_fact_cache::store_source_type_fact_sidecar_with_freshness(
            &state,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            cache_key.as_str(),
            &[entry],
            &freshness,
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

        let document = state
            .document(source_uri.as_str())
            .ok_or_else(|| std::io::Error::other("source document should remain open"))?;
        let selector_names = document
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
            document
                .source_syntax_index
                .selector_references
                .iter()
                .any(|reference| {
                    reference.match_kind == SourceSelectorReferenceMatchKind::Prefix
                        && reference.selector_name.as_deref() == Some("tone-")
                }),
            "an unavailable span query must preserve its lexical prefix reference"
        );
        assert!(
            document
                .source_type_fact_tier_attempts
                .iter()
                .any(|attempt| {
                    attempt.outcome == SOURCE_TYPE_FACT_OUTCOME_UNAVAILABLE
                        && attempt.reason == Some(TSGO_PROVIDER_NO_TRANSPORT)
                })
        );
        assert_eq!(
            document
                .source_syntax_index
                .type_fact_provider_unavailable
                .iter()
                .filter(|fact| fact.provider_id == TSGO_PROVIDER_ID)
                .count(),
            1,
            "cached legacy facts should remain resolved while the span target reports unavailable"
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

        let (cache_key, resolved_entry, unresolved_entry, freshness) = {
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
            let context = source_type_fact_cache_context(&state, document, &request);
            let cache_key = source_type_fact_cache_key(
                &state,
                document,
                &request,
                type_fact_targets.as_slice(),
                &context.freshness,
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
                context.freshness,
            )
        };
        crate::source_type_fact_cache::store_source_type_fact_sidecar_with_freshness(
            &state,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            cache_key.as_str(),
            &[resolved_entry],
            &freshness,
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

        crate::source_type_fact_cache::store_source_type_fact_sidecar_with_freshness(
            &state,
            Some(workspace_uri.as_str()),
            source_uri.as_str(),
            cache_key.as_str(),
            std::slice::from_ref(&unresolved_entry),
            &freshness,
        );
        state.source_type_fact_cache.insert(
            cache_key.clone(),
            LspSourceTypeFactCacheEntryV0 {
                entries: vec![unresolved_entry],
                last_used: 1,
            },
        );

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
