use crate::{
    DiagnosticsPipelineTierPlanV0, LspDeferredDiagnosticsDispatchV0, LspOptimizingTierFeedback,
    LspShellState, OPTIMIZING_DIAGNOSTICS_DELAY_MS, ScheduledLspOutput,
    is_resolution_config_document_uri, is_style_document_uri,
    prepare_deferred_source_diagnostics_for_uri, prepare_deferred_style_diagnostics_for_uri,
    protocol::{canonical_file_uri, file_uri_equivalent, file_uri_to_path},
    resolution_inputs_for_workspace_uri, resolve_document_diagnostics_for_uri,
    resolve_source_diagnostics_for_uri, resolve_workspace_folder_uri, workspace_folder_compatible,
};
#[cfg(feature = "salsa-style-diagnostics")]
use omena_query::{apply_reverse_dependency_delta_v0, reverse_dependency_index_from_edges_v0};
use omena_query::{
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    summarize_omena_query_analyzed_graph, summarize_omena_query_sass_module_sources,
};
use serde::Serialize;
use serde_json::{Value, json};
#[cfg(any(test, feature = "test-support"))]
use std::cell::Cell;
use std::collections::{BTreeSet, VecDeque};
use std::fs;

/// rfcs#61 FIX-1: per peer-recompute walk, the maximum number of style files whose
/// import edges are expanded while deciding whether an open style document
/// (transitively) imports the changed one. Mirrors the workspace-index budgeting
/// philosophy so a pathological import graph cannot stall the loop.
const STYLE_PEER_DISK_WALK_MAX_FILES: usize = 64;

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static SOURCE_CHANGE_REPUBLISH_FANOUT: Cell<u64> = const { Cell::new(0) };
}

#[cfg(any(test, feature = "test-support"))]
#[allow(dead_code)]
pub(crate) fn reset_source_change_republish_fanout_for_test() {
    SOURCE_CHANGE_REPUBLISH_FANOUT.with(|cell| cell.set(0));
}

#[cfg(any(test, feature = "test-support"))]
#[allow(dead_code)]
pub(crate) fn read_source_change_republish_fanout_for_test() -> u64 {
    SOURCE_CHANGE_REPUBLISH_FANOUT.with(Cell::get)
}

#[cfg(any(test, feature = "test-support"))]
fn record_source_change_republish_fanout_for_test(count: usize) {
    SOURCE_CHANGE_REPUBLISH_FANOUT.with(|cell| {
        cell.set(cell.get() + count as u64);
    });
}

#[cfg(not(any(test, feature = "test-support")))]
fn record_source_change_republish_fanout_for_test(_count: usize) {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustDiagnosticsSchedulerBoundaryV0 {
    pub product: &'static str,
    pub owner: &'static str,
    pub scheduling_model: &'static str,
    pub event_policy: Vec<&'static str>,
    pub request_path_policy: Vec<&'static str>,
}

pub fn rust_diagnostics_scheduler_contract() -> RustDiagnosticsSchedulerBoundaryV0 {
    RustDiagnosticsSchedulerBoundaryV0 {
        product: "omena-lsp-server.diagnostics-scheduler",
        owner: "omena-lsp-server/diagnosticsScheduler",
        scheduling_model: "deterministicNotificationPlanner",
        event_policy: vec![
            "publishOnOpenChangeClose",
            "refreshSourceDiagnosticsForStyleChanges",
            "refreshOpenStyleImporterDiagnosticsForStyleChanges",
            "dedupeWatchedStyleDiagnostics",
            "refreshSourceDiagnosticsForResolutionConfigChanges",
            "publishIndexedSourceDiagnosticsOnlyWhenOpen",
            "refreshOpenDocumentsOnConfigurationChange",
            "refreshOpenDocumentsAfterWorkspaceIndexing",
            "publishBaselineDiagnosticsBeforeOptimizingDiagnostics",
            "deferOptimizingDiagnosticsOnRustPath",
            "deferSourceAndStylePeerFanoutOptimizingDiagnostics",
            "coalesceStaleOptimizingDiagnosticsByDocument",
            "workspaceIndexProgressTracksBackgroundJobLifecycle",
            "tierUpHotStyleDiagnosticsIntoAnalyzedGraphFeedback",
            // RFC 0009 Pillar F (rfcs#68): present only when the wave arm is
            // compiled in; the serial arm's contract is unchanged.
            #[cfg(feature = "parallel-style-diagnostics")]
            "parallelizeMemoEligibleStyleWavesOrderPreserving",
        ],
        request_path_policy: vec![
            "noNodeDiagnosticsSchedulerOnRustLspPath",
            "diagnosticsNotificationsStayRustOwned",
            "closedDocumentsPublishEmptyDiagnostics",
            "baselineTierUsesFastFactsV0ForStyleDocuments",
            "optimizingTierUsesAnalyzedGraphV0ForStyleDocuments",
            "optimizingDiagnosticsUseRustScheduledOutputDelay",
            "delayedOptimizingDiagnosticsUseLatestDocumentGeneration",
            "deferredDiagnosticsCompletionChannelDrainsOnLoop",
            "baselineDiagnosticsConsumeOptimizingTierFeedback",
            "hoverCompletionProvidersConsumeOptimizingTierFeedback",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiagnosticsScheduleEvent {
    TextDocument {
        uri: String,
        is_close: bool,
        /// `false` exactly when a didOpen delivered text byte-identical to
        /// the document the index already admitted: nothing cross-file can
        /// have changed, so the source/peer fan-outs (whose SCOPING alone
        /// costs a committed-selector build on the loop) are skipped. Every
        /// other event conservatively claims a change.
        content_changed: bool,
    },
    WatchedFiles {
        uris: Vec<String>,
    },
    ConfigurationChanged,
    Initialized,
}

pub(crate) fn diagnostics_schedule_event(
    method: Option<&str>,
    document_uri: Option<String>,
    watched_file_uris: Vec<String>,
    content_changed: bool,
) -> Option<DiagnosticsScheduleEvent> {
    match method {
        Some("textDocument/didOpen" | "textDocument/didChange" | "textDocument/didClose") => {
            document_uri.map(|uri| DiagnosticsScheduleEvent::TextDocument {
                uri,
                is_close: method == Some("textDocument/didClose"),
                content_changed,
            })
        }
        Some("workspace/didChangeWatchedFiles") => Some(DiagnosticsScheduleEvent::WatchedFiles {
            uris: watched_file_uris,
        }),
        Some("workspace/didChangeConfiguration") => {
            Some(DiagnosticsScheduleEvent::ConfigurationChanged)
        }
        Some("initialized") => Some(DiagnosticsScheduleEvent::Initialized),
        _ => None,
    }
}

pub(crate) fn run_diagnostics_schedule(
    state: &mut LspShellState,
    event: DiagnosticsScheduleEvent,
) -> Vec<ScheduledLspOutput> {
    run_diagnostics_schedule_effects_with_deferral(state, event, false).outputs
}

#[derive(Debug, Default)]
pub(crate) struct DiagnosticsScheduleEffectsV0 {
    pub(crate) outputs: Vec<ScheduledLspOutput>,
    pub(crate) deferred_diagnostics: Vec<LspDeferredDiagnosticsDispatchV0>,
}

impl DiagnosticsScheduleEffectsV0 {
    pub(crate) fn from_outputs(outputs: Vec<ScheduledLspOutput>) -> Self {
        Self {
            outputs,
            deferred_diagnostics: Vec::new(),
        }
    }

    pub(crate) fn extend(&mut self, effects: DiagnosticsScheduleEffectsV0) {
        self.outputs.extend(effects.outputs);
        self.deferred_diagnostics
            .extend(effects.deferred_diagnostics);
    }
}

pub(crate) fn run_diagnostics_schedule_effects(
    state: &mut LspShellState,
    event: DiagnosticsScheduleEvent,
) -> DiagnosticsScheduleEffectsV0 {
    run_diagnostics_schedule_effects_with_deferral(state, event, true)
}

fn run_diagnostics_schedule_effects_with_deferral(
    state: &mut LspShellState,
    event: DiagnosticsScheduleEvent,
    enable_deferred_style_diagnostics: bool,
) -> DiagnosticsScheduleEffectsV0 {
    match event {
        DiagnosticsScheduleEvent::TextDocument {
            uri,
            is_close,
            content_changed,
        } => diagnostics_for_text_document_event(
            state,
            uri.as_str(),
            is_close,
            content_changed,
            enable_deferred_style_diagnostics,
        ),
        DiagnosticsScheduleEvent::WatchedFiles { uris } => {
            diagnostics_for_watched_files(state, uris, enable_deferred_style_diagnostics)
        }
        DiagnosticsScheduleEvent::ConfigurationChanged | DiagnosticsScheduleEvent::Initialized => {
            diagnostics_for_open_documents(state, enable_deferred_style_diagnostics)
        }
    }
}

fn diagnostics_for_text_document_event(
    state: &mut LspShellState,
    uri: &str,
    is_close: bool,
    content_changed: bool,
    enable_deferred_style_diagnostics: bool,
) -> DiagnosticsScheduleEffectsV0 {
    let phase_started = std::time::Instant::now();
    let mut effects = if is_close {
        state.style_module_interface_memo.borrow_mut().remove(uri);
        DiagnosticsScheduleEffectsV0::from_outputs(vec![publish_immediate_diagnostics_output(
            uri,
            json!([]),
        )])
    } else if enable_deferred_style_diagnostics
        && let Some(effects) = deferred_diagnostics_for_uri(state, uri)
    {
        effects
    } else {
        let diagnostics = resolve_document_diagnostics_for_uri(state, uri);
        DiagnosticsScheduleEffectsV0::from_outputs(publish_tiered_diagnostics_notifications(
            state,
            uri,
            diagnostics,
        ))
    };

    crate::loop_trace!(
        "sched-own-doc uri={} took_ms={}",
        uri,
        phase_started.elapsed().as_millis()
    );
    let phase_started = std::time::Instant::now();
    // An open that delivered the exact text the index already admitted
    // cannot have changed any OTHER document's diagnostics; the fan-outs
    // below exist for content changes, and even their scoping pays a
    // committed-selector build on the loop.
    if is_style_document_uri(uri) && content_changed {
        for source_uri in source_uris_for_text_style_change_diagnostics(state, uri) {
            effects.extend(
                if enable_deferred_style_diagnostics {
                    deferred_diagnostics_for_uri(state, source_uri.as_str())
                } else {
                    None
                }
                .unwrap_or_else(|| {
                    let diagnostics =
                        resolve_source_diagnostics_for_uri(state, source_uri.as_str());
                    DiagnosticsScheduleEffectsV0::from_outputs(
                        publish_tiered_diagnostics_notifications(
                            state,
                            source_uri.as_str(),
                            diagnostics,
                        ),
                    )
                }),
            );
        }
        crate::loop_trace!(
            "sched-source-fanout uri={} took_ms={}",
            uri,
            phase_started.elapsed().as_millis()
        );
        let phase_started = std::time::Instant::now();
        for peer_uri in style_uris_for_style_peer_change_diagnostics(state, uri) {
            effects.extend(
                if enable_deferred_style_diagnostics {
                    deferred_diagnostics_for_uri(state, peer_uri.as_str())
                } else {
                    None
                }
                .unwrap_or_else(|| {
                    let diagnostics =
                        resolve_document_diagnostics_for_uri(state, peer_uri.as_str());
                    DiagnosticsScheduleEffectsV0::from_outputs(
                        publish_tiered_diagnostics_notifications(
                            state,
                            peer_uri.as_str(),
                            diagnostics,
                        ),
                    )
                }),
            );
        }
        crate::loop_trace!(
            "sched-peer-fanout uri={} took_ms={}",
            uri,
            phase_started.elapsed().as_millis()
        );
    }

    effects
}

fn diagnostics_for_watched_files(
    state: &mut LspShellState,
    uris: Vec<String>,
    enable_deferred_diagnostics: bool,
) -> DiagnosticsScheduleEffectsV0 {
    let mut style_uris_to_refresh = BTreeSet::new();
    let mut config_uris_to_refresh = BTreeSet::new();
    let mut document_uris_to_refresh = BTreeSet::new();
    for uri in uris {
        if is_style_document_uri(uri.as_str()) {
            style_uris_to_refresh.insert(uri);
        } else if is_resolution_config_document_uri(uri.as_str()) {
            config_uris_to_refresh.insert(uri);
        }
    }
    for uri in style_uris_to_refresh {
        document_uris_to_refresh.insert(uri.clone());
        for source_uri in source_uris_for_style_change_diagnostics(state, uri.as_str()) {
            document_uris_to_refresh.insert(source_uri);
        }
        for peer_uri in style_uris_for_style_peer_change_diagnostics(state, uri.as_str()) {
            document_uris_to_refresh.insert(peer_uri);
        }
    }
    for uri in config_uris_to_refresh {
        for document_uri in
            document_uris_for_resolution_config_change_diagnostics(state, uri.as_str())
        {
            document_uris_to_refresh.insert(document_uri);
        }
    }
    diagnostics_effects_for_document_uris(
        state,
        document_uris_to_refresh.into_iter().collect(),
        enable_deferred_diagnostics,
    )
}

fn diagnostics_for_open_documents(
    state: &mut LspShellState,
    enable_deferred_diagnostics: bool,
) -> DiagnosticsScheduleEffectsV0 {
    let document_uris = open_document_uris_for_diagnostics(state);
    diagnostics_effects_for_document_uris(state, document_uris, enable_deferred_diagnostics)
}

pub(crate) fn diagnostics_effects_for_document_uris(
    state: &mut LspShellState,
    document_uris: Vec<String>,
    enable_deferred_diagnostics: bool,
) -> DiagnosticsScheduleEffectsV0 {
    if !enable_deferred_diagnostics {
        return DiagnosticsScheduleEffectsV0::from_outputs(diagnostics_outputs_for_document_uris(
            state,
            document_uris,
        ));
    }

    let mut effects = DiagnosticsScheduleEffectsV0::default();
    for document_uri in document_uris {
        effects.extend(
            deferred_diagnostics_for_uri(state, document_uri.as_str()).unwrap_or_else(|| {
                let diagnostics =
                    resolve_document_diagnostics_for_uri(state, document_uri.as_str());
                DiagnosticsScheduleEffectsV0::from_outputs(
                    publish_tiered_diagnostics_notifications(
                        state,
                        document_uri.as_str(),
                        diagnostics,
                    ),
                )
            }),
        );
    }
    effects
}

/// Resolve + publish diagnostics for `document_uris` in their given
/// (canonical) order — the straight serial arm. Under
/// `parallel-style-diagnostics` the sibling arm below computes memo-eligible
/// style targets on a bounded wave first, but publishes through this very
/// loop shape in the SAME order, so the notification stream is byte-identical
/// between the arms (gated by tests and the publish-order expectations).
#[cfg(not(feature = "parallel-style-diagnostics"))]
fn diagnostics_outputs_for_document_uris(
    state: &mut LspShellState,
    document_uris: Vec<String>,
) -> Vec<ScheduledLspOutput> {
    let mut outputs = Vec::new();
    for document_uri in document_uris {
        let diagnostics = resolve_document_diagnostics_for_uri(state, document_uri.as_str());
        outputs.extend(publish_tiered_diagnostics_notifications(
            state,
            document_uri.as_str(),
            diagnostics,
        ));
    }
    outputs
}

/// RFC 0009 Pillar F (rfcs#68): the wave-assisted arm. The parallel wave
/// resolves memo-eligible style targets first (joining — and dropping every
/// salsa handle/view — before this function touches `&mut state`); the loop
/// below then walks the SAME canonical order as the serial arm, consuming
/// wave results where present (write-behind + publish loop-side) and running
/// the unchanged serial resolve for everything else, including any target
/// whose worker panicked (which then panics exactly where the serial arm
/// would).
#[cfg(feature = "parallel-style-diagnostics")]
fn diagnostics_outputs_for_document_uris(
    state: &mut LspShellState,
    document_uris: Vec<String>,
) -> Vec<ScheduledLspOutput> {
    diagnostics_outputs_for_document_uris_with_min_parallel_targets(
        state,
        document_uris,
        crate::parallel_style_wave::PARALLEL_STYLE_WAVE_MIN_PARALLEL_TARGETS,
    )
}

#[cfg(feature = "parallel-style-diagnostics")]
fn diagnostics_outputs_for_document_uris_with_min_parallel_targets(
    state: &mut LspShellState,
    document_uris: Vec<String>,
    min_parallel_targets: usize,
) -> Vec<ScheduledLspOutput> {
    let mut wave_results = crate::parallel_style_wave::resolved_parallel_style_wave_targets(
        state,
        document_uris.as_slice(),
        min_parallel_targets,
    );
    let mut outputs = Vec::new();
    for (index, document_uri) in document_uris.iter().enumerate() {
        if let Some(resolved) = wave_results.remove(&index) {
            // Mirrors the serial resolve's tail: write-behind (computed
            // values only — cache hits return before the write there) then
            // tiered publish, on the loop, in canonical order.
            if let Some(slot) = resolved.disk_cache_slot.as_ref() {
                slot.store_write_behind(state, &resolved.diagnostics);
            }
            outputs.extend(publish_tiered_diagnostics_notifications(
                state,
                document_uri.as_str(),
                resolved.diagnostics,
            ));
            continue;
        }
        let diagnostics = resolve_document_diagnostics_for_uri(state, document_uri.as_str());
        outputs.extend(publish_tiered_diagnostics_notifications(
            state,
            document_uri.as_str(),
            diagnostics,
        ));
    }
    outputs
}

pub(crate) fn open_document_uris_for_diagnostics(state: &LspShellState) -> Vec<String> {
    state
        .open_document_uris
        .iter()
        .filter_map(|file_id| {
            state
                .document_for_file_id(*file_id)
                .map(|document| document.uri.clone())
        })
        .collect()
}

fn source_uris_for_text_style_change_diagnostics(
    state: &LspShellState,
    style_uri: &str,
) -> Vec<String> {
    let source_uris = if style_module_interface_changed_for_text_event(state, style_uri) {
        let broad_source_uris = state
            .documents
            .values()
            .filter(|document| !is_style_document_uri(document.uri.as_str()))
            .filter(|document| state.has_open_document_uri(document.uri.as_str()))
            .filter(|document| {
                state.document(style_uri).is_none_or(|style_document| {
                    workspace_folder_compatible(
                        style_document.workspace_folder_uri.as_deref(),
                        document,
                    )
                })
            })
            .map(|document| document.uri.clone())
            .collect::<Vec<_>>();
        scoped_source_republish_uris_for_style_change(state, style_uri, broad_source_uris)
    } else {
        Vec::new()
    };
    record_source_change_republish_fanout_for_test(source_uris.len());
    source_uris
}

/// Compare the style document's CURRENT module-interface projection against
/// the last one this fan-out saw and remember the new one. Equal projections
/// mean an interface-preserving edit — the transaction commit would report
/// no `changed_module_interface_paths`, so no open source document's
/// diagnostics can move. One single-file parse; never a selector build.
/// First sight of a document (didOpen, post-close reopen) reads as changed.
fn style_module_interface_changed_for_text_event(state: &LspShellState, style_uri: &str) -> bool {
    let Some(document) = state.document(style_uri) else {
        return true;
    };
    let projection = omena_query::summarize_omena_query_module_interface_projection(
        style_uri,
        document.text.as_str(),
    );
    let mut memo = state.style_module_interface_memo.borrow_mut();
    match memo.get(style_uri) {
        Some(previous) if *previous == projection => false,
        _ => {
            memo.insert(style_uri.to_string(), projection);
            true
        }
    }
}

fn source_uris_for_style_change_diagnostics(state: &LspShellState, style_uri: &str) -> Vec<String> {
    let workspace_folder_uri = state
        .document(style_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, style_uri));
    let source_uris = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| state.has_open_document_uri(document.uri.as_str()))
        .filter(|document| {
            workspace_folder_uri.as_deref().is_none_or(|workspace_uri| {
                workspace_folder_compatible(Some(workspace_uri), document)
            })
        })
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    let source_uris = scoped_source_republish_uris_for_style_change(state, style_uri, source_uris);
    record_source_change_republish_fanout_for_test(source_uris.len());
    source_uris
}

/// Keep a source document iff it can DEPEND on the changed style module:
/// its own imported-style targets (per-document data, refreshed on every
/// source open/change — never stale) intersect the changed module plus its
/// reverse closure over the memo index. The memo covers the style→style
/// hops (style resolves refresh it); the source hop deliberately comes from
/// the document itself because source opens never build a selector, so the
/// memo's source edges can lag one open behind. A source with an
/// UNRESOLVED style import stays in conservatively ONLY while the memo has
/// never seen it — once a selector build placed the document's edges, the
/// memo's workspace-resolver knowledge is strictly better than the
/// per-document flag, and honoring the flag past that point would
/// republish the source on EVERY style change forever.
pub(crate) fn scoped_source_republish_uris_for_style_change(
    state: &LspShellState,
    style_uri: &str,
    broad_source_uris: Vec<String>,
) -> Vec<String> {
    let memo_filtered = with_reverse_dependency_index(state, |index| {
        let seeds = BTreeSet::from([style_uri.to_string()]);
        let mut relevant = reverse_dependency_closure_for_lsp_paths(index, &seeds);
        relevant.extend(seeds);
        let memo_knows_source = |uri: &str| {
            index.rev.values().any(|dependents| {
                dependents
                    .iter()
                    .any(|dependent| file_uri_equivalent(dependent, uri))
            })
        };
        broad_source_uris
            .iter()
            .filter(|uri| {
                if file_uri_set_contains_equivalent(&relevant, uri.as_str()) {
                    return true;
                }
                state.document(uri.as_str()).is_some_and(|document| {
                    (document.has_unresolved_style_import && !memo_knows_source(uri.as_str()))
                        || document
                            .source_syntax_index
                            .imported_style_bindings
                            .iter()
                            .any(|binding| {
                                file_uri_set_contains_equivalent(
                                    &relevant,
                                    binding.style_uri.as_str(),
                                )
                            })
                })
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    if let Some(filtered) = memo_filtered {
        return filtered;
    }
    // A changed style file we hold no document for (watched-only, never
    // opened or indexed) fails OPEN: we know nothing about its
    // dependents, matching the old selector arm which also widened when
    // the file was outside the open corpus.
    if state.document(style_uri).is_none() {
        return broad_source_uris;
    }
    // Salsa arm, KNOWN document, no memo yet: the cold window before
    // the first selector build completes. Direct import evidence is
    // sufficient here — a TRANSITIVE dependent (source → intermediate
    // style → edited) is reached by the intermediate's own fan-out when
    // the settle-gated All republish walks every style target, so
    // falling back to the broad every-open-source list would only
    // re-add the noise the old loop-side selector build existed to
    // avoid. The straight-line arm has no memo machinery and no settle
    // wave, so it keeps the broad fallback.
    #[cfg(feature = "salsa-style-diagnostics")]
    return filter_source_uris_by_direct_style_dependency(state, style_uri, broad_source_uris);
    #[cfg(not(feature = "salsa-style-diagnostics"))]
    broad_source_uris
}

/// The memo-less arm of the fan-out filter: keep a source iff its OWN
/// import bindings name the changed style module (or its imports are
/// unresolved, which makes its dependency set unknowable).
#[cfg(feature = "salsa-style-diagnostics")]
fn filter_source_uris_by_direct_style_dependency(
    state: &LspShellState,
    style_uri: &str,
    broad_source_uris: Vec<String>,
) -> Vec<String> {
    let relevant = file_uri_identity_aliases(style_uri);
    broad_source_uris
        .into_iter()
        .filter(|uri| {
            state.document(uri.as_str()).is_some_and(|document| {
                document.has_unresolved_style_import
                    || document
                        .source_syntax_index
                        .imported_style_bindings
                        .iter()
                        .any(|binding| {
                            file_uri_set_contains_equivalent(&relevant, binding.style_uri.as_str())
                        })
            })
        })
        .collect()
}

pub(crate) fn reverse_dependency_closure_for_lsp_paths(
    index: &omena_query::ReverseDependencyIndexV0,
    seeds: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut closure = BTreeSet::new();
    let mut seen_lookup_paths = BTreeSet::new();
    let mut queue = VecDeque::new();
    for seed in seeds {
        queue.extend(file_uri_identity_aliases(seed));
    }
    while let Some(path) = queue.pop_front() {
        for lookup_path in file_uri_identity_aliases(path.as_str()) {
            if !seen_lookup_paths.insert(lookup_path.clone()) {
                continue;
            }
            let Some(dependents) = index.rev.get(lookup_path.as_str()) else {
                continue;
            };
            for dependent in dependents {
                if closure.insert(dependent.clone()) {
                    queue.extend(file_uri_identity_aliases(dependent));
                }
            }
        }
    }
    closure
}

fn file_uri_identity_aliases(uri: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::from([uri.to_string()]);
    if let Some(canonical) = canonical_file_uri(uri) {
        aliases.insert(canonical);
    }
    aliases
}

pub(crate) fn file_uri_set_contains_equivalent(values: &BTreeSet<String>, uri: &str) -> bool {
    values.contains(uri) || values.iter().any(|value| file_uri_equivalent(value, uri))
}

/// Run `f` against the loop's reverse-dependency memo index, or `None`
/// while no memo exists.
///
/// MEMO-ONLY, never the selector: building the committed selector here
/// ran on the LOOP for every style edit's fan-out scoping — measured at
/// 8-13s per edit on a cold host, burying hover and goto-definition
/// behind it. The memo is refreshed by every selector build that
/// happens anyway (the serial arm on the loop, workers through the
/// completion channel via [`refresh_reverse_dependency_index_memo`]),
/// so its staleness window is one compute latency: an edge added inside
/// that window is covered by ITS OWN edit's republish, and publication
/// supersession keeps late results from clobbering fresh ones.
///
/// Borrow-scoped on purpose: the callback shape keeps every consumer from
/// cloning the whole index, which the previous accessor did on EVERY style
/// event — a per-keystroke allocation proportional to the workspace's edge
/// count. `f` must not re-enter the memo (the refresh sites all run outside
/// scoping calls).
#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) fn with_reverse_dependency_index<R>(
    state: &LspShellState,
    f: impl FnOnce(&omena_query::ReverseDependencyIndexV0) -> R,
) -> Option<R> {
    let memo_slot = state.reverse_dependency_index_memo.borrow();
    memo_slot.as_ref().map(|memo| f(&memo.index))
}

/// Off-loop selector builds feed the loop's reverse-dependency memo through
/// this: the caller passes the summary its compute already produced plus
/// the ledger epoch CAPTURED AT DISPATCH TIME (edits racing the compute
/// must re-stale the memo, so the stamp may not be the apply-time epoch).
#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) fn refresh_reverse_dependency_index_memo(
    state: &LspShellState,
    revision: u64,
    summary: &omena_query::OmenaQueryCrossFileSummaryV0,
    dispatch_ledger_epoch: u64,
) {
    if !summary.capabilities.source_selector_reference_edges_ready {
        return;
    }
    let mut memo_slot = state.reverse_dependency_index_memo.borrow_mut();
    match memo_slot.as_mut() {
        Some(memo) => {
            if memo.revision != revision || memo.summary_hash != summary.summary_hash {
                apply_reverse_dependency_delta_v0(&mut memo.index, summary.edges.as_slice());
                memo.revision = revision;
                memo.summary_hash = summary.summary_hash.clone();
            }
            memo.ledger_epoch = memo.ledger_epoch.max(dispatch_ledger_epoch);
        }
        None => {
            *memo_slot = Some(crate::state::LspReverseDependencyIndexMemo {
                revision,
                summary_hash: summary.summary_hash.clone(),
                ledger_epoch: dispatch_ledger_epoch,
                index: reverse_dependency_index_from_edges_v0(summary.edges.as_slice()),
            });
        }
    }
}

#[cfg(not(feature = "salsa-style-diagnostics"))]
pub(crate) fn with_reverse_dependency_index<R>(
    _state: &LspShellState,
    _f: impl FnOnce(&omena_query::ReverseDependencyIndexV0) -> R,
) -> Option<R> {
    None
}

/// rfcs#61 FIX-1: open style documents whose transitive `@use`/`@forward`/`@import`
/// closure reaches `changed_style_uri`. The closure is resolved over DISK with the
/// same specifier resolver navigation uses (alias-aware, existence-checked), NOT the
/// in-memory open-document graph — an intermediate partial that is not open (or fell
/// outside the indexing budget) must not break the importer chain, otherwise a stale
/// diagnostic on the importer survives edits to its dependency.
fn style_uris_for_style_peer_change_diagnostics(
    state: &LspShellState,
    changed_style_uri: &str,
) -> Vec<String> {
    state
        .open_document_uris
        .iter()
        .filter_map(|file_id| {
            state
                .document_for_file_id(*file_id)
                .map(|document| document.uri.clone())
        })
        .filter(|uri| is_style_document_uri(uri.as_str()))
        .filter(|uri| !file_uri_equivalent(uri.as_str(), changed_style_uri))
        .filter(|uri| style_disk_import_closure_reaches(state, uri.as_str(), changed_style_uri))
        .collect()
}

/// Walks `from_uri`'s Sass import closure breadth-first, reading not-open
/// intermediates from disk, and reports whether `needle_uri` is reachable.
fn style_disk_import_closure_reaches(
    state: &LspShellState,
    from_uri: &str,
    needle_uri: &str,
) -> bool {
    let workspace_folder_uri = state
        .document(from_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, from_uri));
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    let mut visited = BTreeSet::from([from_uri.to_string()]);
    let mut queue = VecDeque::from([from_uri.to_string()]);
    let mut remaining_files = STYLE_PEER_DISK_WALK_MAX_FILES;

    while let Some(uri) = queue.pop_front() {
        if remaining_files == 0 {
            return false;
        }
        remaining_files -= 1;
        let text = match state.document(uri.as_str()) {
            Some(document) => document.text.clone(),
            None => {
                let Some(path) = file_uri_to_path(uri.as_str()) else {
                    continue;
                };
                let Ok(text) = fs::read_to_string(path) else {
                    continue;
                };
                text
            }
        };
        let Some(sources) = summarize_omena_query_sass_module_sources(uri.as_str(), text.as_str())
        else {
            continue;
        };
        let specifiers = sources
            .module_use_edges
            .iter()
            .map(|edge| edge.source.clone())
            .chain(sources.module_forward_sources.iter().cloned());
        for specifier in specifiers {
            let Some(resolved) = resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
                uri.as_str(),
                workspace_folder_uri.as_deref(),
                specifier.as_str(),
                &resolution_inputs,
            ) else {
                continue;
            };
            if file_uri_equivalent(resolved.as_str(), needle_uri) {
                return true;
            }
            if visited.insert(resolved.clone()) {
                queue.push_back(resolved);
            }
        }
    }
    false
}

fn document_uris_for_resolution_config_change_diagnostics(
    state: &LspShellState,
    config_uri: &str,
) -> Vec<String> {
    let workspace_folder_uri = resolve_workspace_folder_uri(state, config_uri);
    state
        .documents
        .values()
        .filter(|document| {
            workspace_folder_uri.as_deref().is_none_or(|workspace_uri| {
                workspace_folder_compatible(Some(workspace_uri), document)
            })
        })
        .filter(|document| {
            is_style_document_uri(document.uri.as_str())
                || state.has_open_document_uri(document.uri.as_str())
        })
        .map(|document| document.uri.clone())
        .collect()
}

fn publish_diagnostics_notification(uri: &str, diagnostics: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics,
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticsPipelineTier {
    Baseline,
    Optimizing,
}

pub(crate) fn publish_tiered_diagnostics_notifications(
    state: &mut LspShellState,
    uri: &str,
    diagnostics: Value,
) -> Vec<ScheduledLspOutput> {
    let Some(diagnostics) = diagnostics.as_array() else {
        return vec![publish_immediate_diagnostics_output(uri, diagnostics)];
    };
    record_diagnostics_schedule(state, uri);
    prewarm_optimizing_tier_feedback_for_hot_style_document(state, uri);
    let tier_plan = diagnostics_pipeline_tier_plan_for_uri(state, uri);
    let baseline_diagnostics = baseline_diagnostics_for_slice(diagnostics, tier_plan);
    let full_diagnostics = full_diagnostics_for_slice(diagnostics, tier_plan);

    let mut outputs = if baseline_diagnostics.is_empty() && full_diagnostics != baseline_diagnostics
    {
        Vec::new()
    } else {
        vec![publish_immediate_diagnostics_output(
            uri,
            json!(baseline_diagnostics),
        )]
    };
    if full_diagnostics != baseline_diagnostics {
        outputs.push(ScheduledLspOutput::delayed_coalesced(
            publish_diagnostics_notification(uri, json!(full_diagnostics)),
            OPTIMIZING_DIAGNOSTICS_DELAY_MS,
            diagnostics_coalesce_key(uri),
        ));
    }
    outputs
}

fn deferred_diagnostics_for_uri(
    state: &mut LspShellState,
    uri: &str,
) -> Option<DiagnosticsScheduleEffectsV0> {
    if is_style_document_uri(uri) {
        return deferred_style_diagnostics_for_text_document_event(state, uri);
    }

    let probe_tier_plan = diagnostics_pipeline_tier_plan_for_uri(state, uri);
    let (baseline_diagnostics, mut dispatch) =
        prepare_deferred_source_diagnostics_for_uri(state, uri, probe_tier_plan)?;
    record_diagnostics_schedule(state, uri);
    let tier_plan = diagnostics_pipeline_tier_plan_for_uri(state, uri);
    dispatch.coalesce_key = diagnostics_coalesce_key(uri);
    dispatch.tier_plan = tier_plan;
    let baseline_diagnostics = baseline_diagnostics_for_plan(&baseline_diagnostics, tier_plan);
    let outputs = if baseline_diagnostics.is_empty() {
        Vec::new()
    } else {
        vec![publish_immediate_diagnostics_output(
            uri,
            json!(baseline_diagnostics),
        )]
    };
    Some(DiagnosticsScheduleEffectsV0 {
        outputs,
        deferred_diagnostics: vec![dispatch],
    })
}

fn deferred_style_diagnostics_for_text_document_event(
    state: &mut LspShellState,
    uri: &str,
) -> Option<DiagnosticsScheduleEffectsV0> {
    if !is_style_document_uri(uri) {
        return None;
    }
    let probe_tier_plan = diagnostics_pipeline_tier_plan_for_uri(state, uri);
    let (baseline_diagnostics, mut dispatch) =
        prepare_deferred_style_diagnostics_for_uri(state, uri, probe_tier_plan)?;
    record_diagnostics_schedule(state, uri);
    prewarm_optimizing_tier_feedback_for_hot_style_document(state, uri);
    let tier_plan = diagnostics_pipeline_tier_plan_for_uri(state, uri);
    dispatch.coalesce_key = diagnostics_coalesce_key(uri);
    dispatch.tier_plan = tier_plan;
    Some(DiagnosticsScheduleEffectsV0 {
        outputs: vec![publish_immediate_diagnostics_output(
            uri,
            json!(baseline_diagnostics_for_plan(
                &baseline_diagnostics,
                tier_plan
            )),
        )],
        deferred_diagnostics: vec![dispatch],
    })
}

fn publish_immediate_diagnostics_output(uri: &str, diagnostics: Value) -> ScheduledLspOutput {
    ScheduledLspOutput::immediate_coalesced(
        publish_diagnostics_notification(uri, diagnostics),
        diagnostics_coalesce_key(uri),
    )
}

fn diagnostics_coalesce_key(uri: &str) -> String {
    format!("textDocument/publishDiagnostics:{uri}")
}

fn record_diagnostics_schedule(state: &mut LspShellState, uri: &str) {
    if let Some(document) = state.document_mut(uri) {
        document.diagnostics_schedule_count = document.diagnostics_schedule_count.saturating_add(1);
    }
}

fn prewarm_optimizing_tier_feedback_for_hot_style_document(state: &mut LspShellState, uri: &str) {
    if !is_style_document_uri(uri) {
        return;
    }
    let Some(document) = state.document_mut(uri) else {
        return;
    };
    if document.diagnostics_schedule_count < 2 {
        return;
    }
    if document
        .optimizing_tier_feedback
        .as_ref()
        .is_some_and(|feedback| feedback.document_version == document.version)
    {
        return;
    }
    let analyzed_graph =
        summarize_omena_query_analyzed_graph(document.uri.as_str(), document.text.as_str());
    document.optimizing_tier_feedback = Some(LspOptimizingTierFeedback {
        schema_version: "0",
        product: "omena-lsp-server.optimizing-tier-feedback",
        document_version: document.version,
        policy: "hotStyleDiagnosticsPrewarm",
        consumer: "diagnosticsPipelineTierPlanAndProviderRequests",
        analyzed_graph,
    });
}

fn diagnostics_pipeline_tier_plan_for_uri(
    state: &LspShellState,
    uri: &str,
) -> DiagnosticsPipelineTierPlanV0 {
    if is_style_document_uri(uri)
        && let Some(document) = state.document(uri)
    {
        // The version-keyed prewarm feedback already embeds the fast facts, so a cache
        // hit costs zero style-fact collections and a miss runs the analysis exactly
        // once for both tier evidences. The hit path may only consume input-independent
        // fields (the tier labels are constants); consuming data fields (node_count,
        // selector_count, ...) here would make correctness depend on the feedback
        // invalidation set staying exhaustive — re-audit it first.
        let cached_feedback = document
            .optimizing_tier_feedback
            .as_ref()
            .filter(|feedback| feedback.document_version == document.version);
        let (baseline_evidence, optimizing_evidence) = match cached_feedback {
            Some(feedback) => (
                feedback.analyzed_graph.fast_facts.tier,
                feedback.analyzed_graph.tier,
            ),
            None => {
                let analyzed_graph = summarize_omena_query_analyzed_graph(
                    document.uri.as_str(),
                    document.text.as_str(),
                );
                (analyzed_graph.fast_facts.tier, analyzed_graph.tier)
            }
        };
        return DiagnosticsPipelineTierPlanV0 {
            baseline_evidence,
            optimizing_evidence,
            baseline_feedback_evidence: cached_feedback.map(|_| "analyzedGraphV0HotStylePrewarm"),
        };
    }

    DiagnosticsPipelineTierPlanV0 {
        baseline_evidence: "sourceSyntaxIndexV0",
        optimizing_evidence: "workspaceSourceDiagnosticsV0",
        baseline_feedback_evidence: None,
    }
}

fn diagnostic_pipeline_tier(diagnostic: &Value) -> DiagnosticsPipelineTier {
    match diagnostic
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "missingStaticClass"
        | "missingTemplatePrefix"
        | "missingResolvedClassValues"
        | "missingResolvedClassDomain"
        | "missingSelector"
        | "missingModule"
        | "missingCustomProperty" => DiagnosticsPipelineTier::Baseline,
        _ => DiagnosticsPipelineTier::Optimizing,
    }
}

fn baseline_diagnostics_for_plan(
    diagnostics: &Value,
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Vec<Value> {
    diagnostics
        .as_array()
        .map(|diagnostics| baseline_diagnostics_for_slice(diagnostics, tier_plan))
        .unwrap_or_default()
}

fn baseline_diagnostics_for_slice(
    diagnostics: &[Value],
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Vec<Value> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic_pipeline_tier(diagnostic) == DiagnosticsPipelineTier::Baseline
        })
        .map(|diagnostic| {
            annotate_diagnostic_pipeline_tier(
                diagnostic.clone(),
                DiagnosticsPipelineTier::Baseline,
                tier_plan.baseline_evidence,
                tier_plan.baseline_feedback_evidence,
            )
        })
        .collect()
}

#[cfg(feature = "salsa-style-diagnostics")]
fn full_diagnostics_for_plan(
    diagnostics: &Value,
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Vec<Value> {
    diagnostics
        .as_array()
        .map(|diagnostics| full_diagnostics_for_slice(diagnostics, tier_plan))
        .unwrap_or_default()
}

fn full_diagnostics_for_slice(
    diagnostics: &[Value],
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Vec<Value> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let tier = diagnostic_pipeline_tier(diagnostic);
            let evidence = match tier {
                DiagnosticsPipelineTier::Baseline => tier_plan.baseline_evidence,
                DiagnosticsPipelineTier::Optimizing => tier_plan.optimizing_evidence,
            };
            annotate_diagnostic_pipeline_tier(
                diagnostic.clone(),
                tier,
                evidence,
                tier_plan.baseline_feedback_evidence,
            )
        })
        .collect()
}

#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) fn deferred_full_diagnostics_notification(
    uri: &str,
    diagnostics: Value,
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Value {
    publish_diagnostics_notification(
        uri,
        json!(full_diagnostics_for_plan(&diagnostics, tier_plan)),
    )
}

fn annotate_diagnostic_pipeline_tier(
    mut diagnostic: Value,
    tier: DiagnosticsPipelineTier,
    tier_evidence: &'static str,
    baseline_feedback_evidence: Option<&'static str>,
) -> Value {
    let Some(diagnostic) = diagnostic.as_object_mut() else {
        return diagnostic;
    };
    let data = diagnostic
        .entry("data")
        .or_insert_with(|| json!({}))
        .as_object_mut();
    if let Some(data) = data {
        data.insert(
            "pipelineTier".to_string(),
            json!(match tier {
                DiagnosticsPipelineTier::Baseline => "baseline",
                DiagnosticsPipelineTier::Optimizing => "optimizing",
            }),
        );
        data.insert("pipelineTierEvidence".to_string(), json!(tier_evidence));
        if tier == DiagnosticsPipelineTier::Baseline
            && let Some(feedback_evidence) = baseline_feedback_evidence
        {
            data.insert("pipelineTierFeedback".to_string(), json!(feedback_evidence));
        }
    }
    Value::Object(diagnostic.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_lsp_events_to_diagnostics_schedule_events() {
        assert_eq!(
            diagnostics_schedule_event(
                Some("textDocument/didChange"),
                Some("file:///repo/src/App.tsx".to_string()),
                Vec::new(),
                true,
            ),
            Some(DiagnosticsScheduleEvent::TextDocument {
                uri: "file:///repo/src/App.tsx".to_string(),
                is_close: false,
                content_changed: true,
            }),
        );
        assert_eq!(
            diagnostics_schedule_event(
                Some("textDocument/didClose"),
                Some("file:///repo/src/App.tsx".to_string()),
                Vec::new(),
                true,
            ),
            Some(DiagnosticsScheduleEvent::TextDocument {
                uri: "file:///repo/src/App.tsx".to_string(),
                is_close: true,
                content_changed: true,
            }),
        );
        assert_eq!(
            diagnostics_schedule_event(
                Some("workspace/didChangeWatchedFiles"),
                None,
                vec!["file:///repo/src/App.module.scss".to_string()],
                true,
            ),
            Some(DiagnosticsScheduleEvent::WatchedFiles {
                uris: vec!["file:///repo/src/App.module.scss".to_string()],
            }),
        );
    }

    #[test]
    fn publishes_baseline_before_optimizing_diagnostics() {
        let mut state = LspShellState::default();
        let outputs = crate::handle_lsp_message_scheduled_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///workspace-a/src/App.module.scss",
                        "languageId": "scss",
                        "version": 1,
                        "text": ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }",
                    },
                },
            }),
        );

        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].delay_millis, None);
        assert_eq!(
            outputs[1].delay_millis,
            Some(OPTIMIZING_DIAGNOSTICS_DELAY_MS)
        );
        assert_eq!(
            outputs[0].value.pointer("/params/diagnostics/0/code"),
            Some(&json!("missingCustomProperty")),
        );
        assert_eq!(
            outputs[0]
                .value
                .pointer("/params/diagnostics/0/data/pipelineTier"),
            Some(&json!("baseline")),
        );
        assert_eq!(
            outputs[0]
                .value
                .pointer("/params/diagnostics/0/data/pipelineTierEvidence"),
            Some(&json!("fastFactsV0")),
        );
        assert!(
            outputs[0]
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|diagnostics| diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.pointer("/code")
                        != Some(&json!("unreachableDeclaration"))))
        );

        assert!(
            outputs[1]
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|full_diagnostics| full_diagnostics.iter().any(
                    |diagnostic| diagnostic.pointer("/code")
                        == Some(&json!("missingCustomProperty"))
                        && diagnostic.pointer("/data/pipelineTier") == Some(&json!("baseline"))
                ))
        );
        assert!(
            outputs[1]
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|full_diagnostics| full_diagnostics.iter().any(
                    |diagnostic| diagnostic.pointer("/code")
                        == Some(&json!("unreachableDeclaration"))
                        && diagnostic.pointer("/data/pipelineTier") == Some(&json!("optimizing"))
                        && diagnostic.pointer("/data/pipelineTierEvidence")
                            == Some(&json!("analyzedGraphV0"))
                ))
        );
    }

    #[test]
    fn hot_style_diagnostics_prewarm_optimizing_feedback_for_baseline() -> Result<(), &'static str>
    {
        let uri = "file:///workspace-a/src/App.module.scss";
        let mut state = LspShellState::default();
        let first_outputs = crate::handle_lsp_message_scheduled_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }",
                    },
                },
            }),
        );
        assert_eq!(first_outputs.len(), 2);
        assert!(
            first_outputs[0]
                .value
                .pointer("/params/diagnostics/0/data/pipelineTierFeedback")
                .is_none()
        );

        let second_outputs = crate::handle_lsp_message_scheduled_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {
                        "uri": uri,
                        "version": 2,
                    },
                    "contentChanges": [{
                        "text": ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }\n.card { color: green; }",
                    }],
                },
            }),
        );
        assert_eq!(second_outputs.len(), 2);
        assert_eq!(
            diagnostic_data_value(
                second_outputs[0].value.pointer("/params/diagnostics"),
                "missingCustomProperty",
                "pipelineTierFeedback",
            ),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );
        assert_eq!(
            diagnostic_data_value(
                second_outputs[1].value.pointer("/params/diagnostics"),
                "missingCustomProperty",
                "pipelineTierFeedback",
            ),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );

        let document = state.document(uri).ok_or("style document stays open")?;
        assert_eq!(document.diagnostics_schedule_count, 2);
        let feedback = document
            .optimizing_tier_feedback
            .as_ref()
            .ok_or("hot style diagnostics should prewarm optimizing feedback")?;
        assert_eq!(
            feedback.product,
            "omena-lsp-server.optimizing-tier-feedback"
        );
        assert_eq!(feedback.document_version, 2);
        assert_eq!(feedback.policy, "hotStyleDiagnosticsPrewarm");
        assert_eq!(
            feedback.consumer,
            "diagnosticsPipelineTierPlanAndProviderRequests"
        );
        assert_eq!(feedback.analyzed_graph.tier, "analyzedGraphV0");
        assert_eq!(feedback.analyzed_graph.fast_facts.selector_count, 2);

        let hover_response = crate::handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "textDocument/hover",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": {
                        "line": 2,
                        "character": 2,
                    },
                },
            }),
        );
        assert_eq!(
            hover_response
                .as_ref()
                .and_then(|value| value.pointer("/result/data/providerTierFeedback/provider")),
            Some(&json!("textDocument/hover")),
        );
        assert_eq!(
            hover_response
                .as_ref()
                .and_then(|value| value.pointer("/result/data/providerTierFeedback/feedback")),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );

        let completion_response = crate::handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "textDocument/completion",
                "params": {
                    "textDocument": {
                        "uri": uri,
                    },
                    "position": {
                        "line": 2,
                        "character": 0,
                    },
                },
            }),
        );
        let completion_items = completion_response
            .as_ref()
            .and_then(|value| value.pointer("/result/items"))
            .and_then(Value::as_array)
            .ok_or("completion items should be present")?;
        let card_completion = completion_items
            .iter()
            .find(|item| item.pointer("/label") == Some(&json!(".card")))
            .ok_or("card selector completion should be present")?;
        assert_eq!(
            card_completion.pointer("/data/providerTierFeedback/provider"),
            Some(&json!("textDocument/completion")),
        );
        assert_eq!(
            card_completion.pointer("/data/providerTierFeedback/feedback"),
            Some(&json!("analyzedGraphV0HotStylePrewarm")),
        );
        Ok(())
    }

    fn diagnostic_data_value<'a>(
        diagnostics: Option<&'a Value>,
        code: &str,
        key: &str,
    ) -> Option<&'a Value> {
        diagnostics
            .and_then(Value::as_array)?
            .iter()
            .find(|diagnostic| diagnostic.pointer("/code") == Some(&json!(code)))?
            .pointer(format!("/data/{key}").as_str())
    }

    /// RFC 0009 Pillar F (rfcs#68) fixtures: one workspace, three open style
    /// documents (each with a real diagnostic so the parity compare is
    /// non-trivial) plus one source consumer.
    #[cfg(feature = "parallel-style-diagnostics")]
    fn parallel_wave_fixture_state() -> LspShellState {
        let mut state = LspShellState::default();
        let _ = crate::handle_lsp_message_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "workspaceFolders": [
                        { "uri": "file:///workspace-parallel-wave", "name": "parallel-wave" },
                    ],
                },
            }),
        );
        let documents = [
            (
                "file:///workspace-parallel-wave/src/Alpha.module.scss",
                "scss",
                ":root { --brand: red; }\n.alpha { width: var(--missing-alpha); }",
            ),
            (
                "file:///workspace-parallel-wave/src/Beta.module.scss",
                "scss",
                ".beta { color: var(--missing-beta); }",
            ),
            (
                "file:///workspace-parallel-wave/src/Gamma.module.scss",
                "scss",
                ".gamma { color: red; color: blue; }",
            ),
            (
                "file:///workspace-parallel-wave/src/App.tsx",
                "typescriptreact",
                "import styles from \"./Alpha.module.scss\";\nconst view = <div className={styles.alpha} />;",
            ),
        ];
        for (uri, language_id, text) in documents {
            let _ = crate::handle_lsp_message_outputs(
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
        state
    }

    /// The wave arm must publish the SAME notifications, with the same
    /// bytes, in the same order, as a forced-serial run over the same URI
    /// list (the group-size knob at `usize::MAX` disables the wave).
    #[cfg(feature = "parallel-style-diagnostics")]
    #[test]
    fn parallel_wave_outputs_match_forced_serial_in_order_and_bytes() -> Result<(), &'static str> {
        let mut parallel_state = parallel_wave_fixture_state();
        let mut serial_state = parallel_wave_fixture_state();
        let document_uris = open_document_uris_for_diagnostics(&parallel_state);
        assert_eq!(document_uris.len(), 4);

        // Non-vacuity: the wave must actually take at least two style
        // targets — otherwise this test compares serial to serial.
        let wave_results = crate::parallel_style_wave::resolved_parallel_style_wave_targets(
            &parallel_state,
            document_uris.as_slice(),
            crate::parallel_style_wave::PARALLEL_STYLE_WAVE_MIN_PARALLEL_TARGETS,
        );
        assert!(
            wave_results.len() >= 2,
            "expected at least two wave-resolved style targets, got {}",
            wave_results.len(),
        );
        drop(wave_results);

        let parallel_outputs =
            diagnostics_outputs_for_document_uris(&mut parallel_state, document_uris.clone());
        let serial_outputs = diagnostics_outputs_for_document_uris_with_min_parallel_targets(
            &mut serial_state,
            document_uris,
            usize::MAX,
        );
        assert_eq!(
            parallel_outputs, serial_outputs,
            "parallel wave outputs must be byte-identical to the serial arm, in the same order",
        );
        let nonempty_publish = parallel_outputs.iter().any(|output| {
            output
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|diagnostics| !diagnostics.is_empty())
        });
        assert!(
            nonempty_publish,
            "the parity compare must cover non-empty diagnostics",
        );
        Ok(())
    }

    /// RFC 0009 Pillar F (rfcs#68) end-to-end: multiple open style importers,
    /// ONE watched change to their shared on-disk dependency — the
    /// wave-assisted watched-files schedule must publish the SAME
    /// notifications in the SAME order as a forced-serial run over the same
    /// merged refresh set.
    #[cfg(feature = "parallel-style-diagnostics")]
    #[test]
    fn watched_change_wave_matches_forced_serial_for_open_importers() -> Result<(), String> {
        let workspace_path = std::env::temp_dir().join(format!(
            "omena-lsp-parallel-wave-watched-{}-{}",
            std::process::id(),
            crate::current_time_millis(),
        ));
        let alpha_path = workspace_path.join("src/Alpha.module.scss");
        let beta_path = workspace_path.join("src/Beta.module.scss");
        let leaf_path = workspace_path.join("src/partials/_leaf.scss");
        let alpha_text = "@use \"./partials/leaf\";\n.alpha { width: var(--missing-alpha); }\n";
        let beta_text = "@use \"./partials/leaf\";\n.beta { color: var(--missing-beta); }\n";
        fs::create_dir_all(workspace_path.join("src/partials"))
            .map_err(|error| error.to_string())?;
        fs::write(alpha_path.as_path(), alpha_text).map_err(|error| error.to_string())?;
        fs::write(beta_path.as_path(), beta_text).map_err(|error| error.to_string())?;
        fs::write(leaf_path.as_path(), "$tone: red;\n").map_err(|error| error.to_string())?;
        let workspace_uri = crate::protocol::path_to_file_uri(workspace_path.as_path());
        let alpha_uri = crate::protocol::path_to_file_uri(alpha_path.as_path());
        let beta_uri = crate::protocol::path_to_file_uri(beta_path.as_path());
        let leaf_uri = crate::protocol::path_to_file_uri(leaf_path.as_path());

        let built_state = || {
            let mut state = LspShellState::default();
            let _ = crate::handle_lsp_message_outputs(
                &mut state,
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "workspaceFolders": [
                            { "uri": workspace_uri, "name": "parallel-wave-watched" },
                        ],
                    },
                }),
            );
            for (uri, text) in [
                (alpha_uri.as_str(), alpha_text),
                (beta_uri.as_str(), beta_text),
            ] {
                let _ = crate::handle_lsp_message_outputs(
                    &mut state,
                    json!({
                        "jsonrpc": "2.0",
                        "method": "textDocument/didOpen",
                        "params": {
                            "textDocument": {
                                "uri": uri,
                                "languageId": "scss",
                                "version": 1,
                                "text": text,
                            },
                        },
                    }),
                );
            }
            state
        };
        let mut parallel_state = built_state();
        let mut serial_state = built_state();
        let probe_state = built_state();

        let parallel_outputs = run_diagnostics_schedule(
            &mut parallel_state,
            DiagnosticsScheduleEvent::WatchedFiles {
                uris: vec![leaf_uri.clone()],
            },
        );
        // One immediate publish per merged-set member, in the canonical
        // (BTreeSet) drain order — recover that order from the stream.
        let published_uris = parallel_outputs
            .iter()
            .filter(|output| output.delay_millis.is_none())
            .filter_map(|output| {
                output
                    .value
                    .pointer("/params/uri")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect::<Vec<_>>();
        assert!(
            published_uris.contains(&alpha_uri) && published_uris.contains(&beta_uri),
            "both open importers must be refreshed by the watched change: {published_uris:?}",
        );
        let mut sorted_uris = published_uris.clone();
        sorted_uris.sort();
        assert_eq!(
            published_uris, sorted_uris,
            "the watched merged set must drain in canonical order",
        );

        // Non-vacuity: the wave path actually takes both importers.
        let wave_results = crate::parallel_style_wave::resolved_parallel_style_wave_targets(
            &probe_state,
            published_uris.as_slice(),
            crate::parallel_style_wave::PARALLEL_STYLE_WAVE_MIN_PARALLEL_TARGETS,
        );
        assert!(
            wave_results.len() >= 2,
            "expected both open importers wave-resolved, got {}",
            wave_results.len(),
        );
        drop(wave_results);

        let serial_outputs = diagnostics_outputs_for_document_uris_with_min_parallel_targets(
            &mut serial_state,
            published_uris,
            usize::MAX,
        );
        assert_eq!(
            parallel_outputs, serial_outputs,
            "the watched-change wave must publish byte-identically to the serial arm",
        );

        let _ = fs::remove_dir_all(workspace_path.as_path());
        Ok(())
    }

    #[cfg(feature = "salsa-style-diagnostics")]
    #[test]
    fn style_change_deferred_fanout_counts_all_open_style_importers() -> Result<(), String> {
        let workspace_path = std::env::temp_dir().join(format!(
            "omena-lsp-deferred-style-fanout-{}-{}",
            std::process::id(),
            crate::current_time_millis(),
        ));
        let src_path = workspace_path.join("src");
        let shared_path = src_path.join("_shared.scss");
        fs::create_dir_all(src_path.as_path()).map_err(|error| error.to_string())?;
        fs::write(shared_path.as_path(), "$tone: red;\n").map_err(|error| error.to_string())?;
        let importer_specs = [
            (
                src_path.join("Alpha.module.scss"),
                "@use \"./shared\";\n.alpha { width: var(--missing-alpha); color: red; color: blue; }\n",
            ),
            (
                src_path.join("Beta.module.scss"),
                "@use \"./shared\";\n.beta { width: var(--missing-beta); color: red; color: blue; }\n",
            ),
            (
                src_path.join("Gamma.module.scss"),
                "@use \"./shared\";\n.gamma { width: var(--missing-gamma); color: red; color: blue; }\n",
            ),
        ];
        for (path, text) in importer_specs.iter() {
            fs::write(path.as_path(), text).map_err(|error| error.to_string())?;
        }
        let workspace_uri = crate::protocol::path_to_file_uri(workspace_path.as_path());
        let shared_uri = crate::protocol::path_to_file_uri(shared_path.as_path());
        let importer_uris = importer_specs
            .iter()
            .map(|(path, _)| crate::protocol::path_to_file_uri(path.as_path()))
            .collect::<BTreeSet<_>>();

        let mut state = LspShellState::default();
        let _ = crate::handle_lsp_message_scheduled_outputs_or_dispatch(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "workspaceFolders": [
                        { "uri": workspace_uri, "name": "deferred-style-fanout" },
                    ],
                },
            }),
        );
        let _ = crate::handle_lsp_message_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": shared_uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": "$tone: red;\n",
                    },
                },
            }),
        );
        for (path, text) in importer_specs.iter() {
            let uri = crate::protocol::path_to_file_uri(path.as_path());
            let _ = crate::handle_lsp_message_scheduled_outputs_or_dispatch(
                &mut state,
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/didOpen",
                    "params": {
                        "textDocument": {
                            "uri": uri,
                            "languageId": "scss",
                            "version": 1,
                            "text": text,
                        },
                    },
                }),
            );
        }

        let effects = run_diagnostics_schedule_effects(
            &mut state,
            DiagnosticsScheduleEvent::TextDocument {
                uri: shared_uri,
                is_close: false,
                content_changed: true,
            },
        );
        let importer_outputs = effects
            .outputs
            .iter()
            .filter(|output| {
                output
                    .value
                    .pointer("/params/uri")
                    .and_then(Value::as_str)
                    .is_some_and(|uri| importer_uris.contains(uri))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            importer_outputs.len(),
            importer_uris.len(),
            "each open style importer must receive exactly one immediate baseline publish"
        );
        assert!(
            importer_outputs
                .iter()
                .all(|output| output.delay_millis.is_none()),
            "style fan-out optimizing diagnostics must be deferred off the loop"
        );
        assert!(
            importer_outputs.iter().all(|output| output
                .value
                .pointer("/params/diagnostics")
                .and_then(Value::as_array)
                .is_some_and(|diagnostics| diagnostics.iter().all(|diagnostic| {
                    diagnostic.pointer("/data/pipelineTier") == Some(&json!("baseline"))
                }))),
            "loop-side fan-out publishes must only carry baseline diagnostics"
        );

        let importer_dispatches = effects
            .deferred_diagnostics
            .iter()
            .filter(|dispatch| importer_uris.contains(dispatch.uri.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            importer_dispatches.len(),
            importer_uris.len(),
            "each open style importer must enqueue one deferred optimizing dispatch"
        );
        assert!(
            importer_dispatches.iter().all(|dispatch| matches!(
                dispatch.render_inputs,
                crate::lsp_output::DeferredDiagnosticsRenderInputsV0::StyleSnapshot(_)
            )),
            "style fan-out dispatches must carry a snapshot for worker-side style input collection"
        );

        let _ = fs::remove_dir_all(workspace_path.as_path());
        Ok(())
    }

    /// End-to-end through the scheduler event: a configuration change over
    /// multiple open style documents publishes immediate non-empty baseline
    /// notifications in canonical (BTreeSet) URI order. Optimizing-only
    /// documents skip the empty baseline clear and publish on the full tier.
    #[cfg(feature = "parallel-style-diagnostics")]
    #[test]
    fn configuration_change_wave_publishes_in_canonical_order() {
        let mut state = parallel_wave_fixture_state();
        let outputs = crate::handle_lsp_message_scheduled_outputs(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "workspace/didChangeConfiguration",
                "params": { "settings": {} },
            }),
        );
        let immediate_publish_uris = outputs
            .iter()
            .filter(|output| output.delay_millis.is_none())
            .filter_map(|output| output.value.pointer("/params/uri").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert_eq!(
            immediate_publish_uris,
            vec!["file:///workspace-parallel-wave/src/Alpha.module.scss"],
        );
        let delayed_publish_uris = outputs
            .iter()
            .filter(|output| output.delay_millis.is_some())
            .filter_map(|output| output.value.pointer("/params/uri").and_then(Value::as_str))
            .collect::<Vec<_>>();
        assert!(
            delayed_publish_uris.contains(&"file:///workspace-parallel-wave/src/App.tsx"),
            "optimizing-only source diagnostics should publish on the full tier: {delayed_publish_uris:?}",
        );
    }
}
