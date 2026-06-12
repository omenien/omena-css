//! RFC 0009 §0: the cached-vs-from-scratch diagnostic equivalence oracle.
//!
//! Every cache/incremental pillar of RFC 0009 (salsa memoization, persistent
//! caching, compute-deferral tiering, source-corpus indexing) is allowed to
//! merge only when the diagnostics it produces are provably byte-identical to
//! a from-scratch evaluation over the same corpus. This module is that gate's
//! substrate: it evaluates the full workspace diagnostics entry point — the
//! same one the shipped LSP/CLI resolve through — under two evaluator
//! disciplines and compares the serialized results byte-for-byte.
//!
//! Until the first memoizing layer lands, the warm-pass self-equivalence run
//! pins the CURRENT contract: the query layer is pure, so re-evaluating a file
//! after a full warm round over the corpus must be byte-identical to a single
//! fresh evaluation. Any hidden cross-call state that later sneaks into the
//! hot path trips this gate. When RFC 0009 Pillar B lands the salsa-backed
//! path behind a feature flag, the same harness diffs the straight-line
//! evaluator against the salsa evaluator instead.

use omena_query::{
    OmenaQueryExternalModuleModeV0, OmenaQueryExternalSifInputV0, OmenaQuerySourceDocumentInputV0,
    OmenaQueryStyleDiagnosticsForFileV0, OmenaQueryStyleMemoDatabaseV0, OmenaQueryStyleMemoHostV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, resolve_memo_workspace_style_diagnostics_from_view,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Per-file byte-equivalence outcome. The serialized payloads are embedded
/// only on mismatch so green reports stay small.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffCacheEquivalenceFileReportV0 {
    /// Corpus file whose diagnostics were compared.
    pub style_path: String,
    /// Whether both evaluators produced byte-identical serialized diagnostics.
    pub identical: bool,
    /// Baseline (from-scratch) serialization, embedded on mismatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_json: Option<String>,
    /// Candidate (cached/incremental) serialization, embedded on mismatch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_json: Option<String>,
}

/// Corpus-level equivalence report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffCacheEquivalenceReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product identity.
    pub product: &'static str,
    /// Label of the from-scratch evaluator (the oracle side).
    pub baseline_evaluator: &'static str,
    /// Label of the cached/incremental evaluator under test.
    pub candidate_evaluator: &'static str,
    /// Corpus size.
    pub file_count: usize,
    /// Files whose results were byte-identical.
    pub identical_file_count: usize,
    /// Whether the gate holds over the whole corpus.
    pub all_files_identical: bool,
    /// Per-file outcomes.
    pub files: Vec<OmenaDiffCacheEquivalenceFileReportV0>,
}

/// The straight-line, from-scratch evaluation of one corpus file through the
/// full workspace diagnostics entry point. This is the oracle side of every
/// equivalence comparison.
pub fn evaluate_workspace_diagnostics_from_scratch_v0(
    target_style_path: &str,
    corpus: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
        target_style_path,
        corpus,
        &[],
        &[],
        None,
        OmenaQueryExternalModuleModeV0::Ignored,
        &[],
        resolution_inputs,
    )
}

/// Compare two evaluators file-by-file over a corpus, byte-for-byte on the
/// serialized diagnostics. `baseline` is the from-scratch oracle; `candidate`
/// is the cached/incremental path under test.
pub fn summarize_workspace_diagnostics_equivalence_v0<Baseline, Candidate>(
    corpus: &[OmenaQueryStyleSourceInputV0],
    baseline_evaluator: &'static str,
    baseline: Baseline,
    candidate_evaluator: &'static str,
    candidate: Candidate,
) -> OmenaDiffCacheEquivalenceReportV0
where
    Baseline: Fn(&str) -> Option<OmenaQueryStyleDiagnosticsForFileV0>,
    Candidate: Fn(&str) -> Option<OmenaQueryStyleDiagnosticsForFileV0>,
{
    let files = corpus
        .iter()
        .map(|file| {
            let baseline_json = serialized_diagnostics(baseline(file.style_path.as_str()));
            let candidate_json = serialized_diagnostics(candidate(file.style_path.as_str()));
            let identical = baseline_json == candidate_json;
            OmenaDiffCacheEquivalenceFileReportV0 {
                style_path: file.style_path.clone(),
                identical,
                baseline_json: (!identical).then_some(baseline_json),
                candidate_json: (!identical).then_some(candidate_json),
            }
        })
        .collect::<Vec<_>>();
    let identical_file_count = files.iter().filter(|file| file.identical).count();
    OmenaDiffCacheEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.cache-equivalence",
        baseline_evaluator,
        candidate_evaluator,
        file_count: files.len(),
        identical_file_count,
        all_files_identical: identical_file_count == files.len(),
        files,
    }
}

/// Warm-pass self-equivalence: a single fresh evaluation per file (baseline)
/// versus a re-evaluation that runs AFTER a full warm round over the whole
/// corpus plus a first evaluation of the same file (candidate). Today both
/// sides reach the same pure functions, so this pins purity; once a memoizing
/// layer sits behind the entry point, the candidate side is exactly the
/// cache-hit position and any divergence fails the gate.
pub fn summarize_workspace_diagnostics_warm_pass_equivalence_v0(
    corpus: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaDiffCacheEquivalenceReportV0 {
    // Warm round: evaluate the whole corpus once before any comparison.
    for file in corpus {
        let _ = evaluate_workspace_diagnostics_from_scratch_v0(
            file.style_path.as_str(),
            corpus,
            resolution_inputs,
        );
    }
    summarize_workspace_diagnostics_equivalence_v0(
        corpus,
        "straightLineFromScratch",
        |target_style_path| {
            evaluate_workspace_diagnostics_from_scratch_v0(
                target_style_path,
                corpus,
                resolution_inputs,
            )
        },
        "straightLineAfterWarmRound",
        |target_style_path| {
            let _ = evaluate_workspace_diagnostics_from_scratch_v0(
                target_style_path,
                corpus,
                resolution_inputs,
            );
            evaluate_workspace_diagnostics_from_scratch_v0(
                target_style_path,
                corpus,
                resolution_inputs,
            )
        },
    )
}

/// Default oracle corpus: a small workspace that exercises the cross-file
/// machinery every RFC 0009 pillar touches — a tsconfig-aliased `@use`, a
/// transitive `@forward` chain, a cross-file `@extend`, custom properties,
/// and a genuinely-missing symbol so the compared diagnostics are non-empty.
pub fn omena_diff_cache_equivalence_default_corpus_v0() -> (
    Vec<OmenaQueryStyleSourceInputV0>,
    OmenaQueryStyleResolutionInputsV0,
) {
    let corpus = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: "@use \"@app/tokens\" as tokens;\n@use \"./mid\";\n\
                           .app { color: tokens.$brand; background: tokens.$missing; }\n\
                           .extended { @extend %base; }\n"
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/app/_tokens.scss".to_string(),
            style_source: "$brand: red;\n%base { color: blue; }\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_mid.scss".to_string(),
            style_source: "@forward \"./leaf\";\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_leaf.scss".to_string(),
            style_source: ":root { --tone: green; }\n.leaf { color: var(--tone); }\n".to_string(),
        },
    ];
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        tsconfig_path_mappings: vec![omena_query::OmenaQueryTsconfigPathMappingV0 {
            base_path: "/workspace".to_string(),
            pattern: "@app/*".to_string(),
            target_patterns: vec!["src/app/*".to_string()],
        }],
        ..Default::default()
    };
    (corpus, resolution_inputs)
}

/// One phase of the salsa-memo differential: a named corpus state compared
/// between the straight-line evaluator and the SAME long-lived memo host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffSalsaMemoEquivalencePhaseV0 {
    /// Which lifecycle position the memo host is in for this comparison.
    pub phase: &'static str,
    /// The per-file equivalence report for this corpus state.
    pub report: OmenaDiffCacheEquivalenceReportV0,
}

/// RFC 0009 Pillar B merge gate: the salsa-memoized evaluator must be
/// byte-identical to the straight-line evaluator across the cache lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffSalsaMemoEquivalenceReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product identity.
    pub product: &'static str,
    /// Lifecycle phases compared.
    pub phases: Vec<OmenaDiffSalsaMemoEquivalencePhaseV0>,
    /// Total per-file comparisons across all phases.
    pub comparison_count: usize,
    /// Whether every comparison in every phase was byte-identical.
    pub all_phases_identical: bool,
}

/// The from-scratch evaluation over the FULL input surface the memo host
/// accepts; the external mode is derived from SIF presence exactly as the
/// host and the LSP derive it.
pub fn evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
    target_style_path: &str,
    corpus: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let external_mode = if external_sifs.is_empty() {
        OmenaQueryExternalModuleModeV0::Ignored
    } else {
        OmenaQueryExternalModuleModeV0::Sif
    };
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
        target_style_path,
        corpus,
        source_documents,
        package_manifests,
        None,
        external_mode,
        external_sifs,
        resolution_inputs,
    )
}

/// One lifecycle position's full input set.
struct SalsaMemoPhaseInputsV0<'inputs> {
    corpus: &'inputs [OmenaQueryStyleSourceInputV0],
    source_documents: &'inputs [OmenaQuerySourceDocumentInputV0],
    package_manifests: &'inputs [OmenaQueryStylePackageManifestV0],
    resolution_inputs: &'inputs OmenaQueryStyleResolutionInputsV0,
}

/// Drive ONE long-lived `OmenaQueryStyleMemoHostV0` through the cache
/// lifecycle — cold start, warm revalidation (memo-hit position), a targeted
/// edit, a revert, a file removal and re-add, and change+revert pairs on the
/// resolution inputs, source documents, and package manifests — comparing
/// every state byte-for-byte against the straight-line from-scratch
/// evaluator. The change phases are what catch a stale memo: a cache that
/// survives an input change diverges there and fails the gate.
///
/// Known residual: `external_sifs` is exercised only as the constant empty
/// set here (a valid `OmenaSifV1` is too heavy to fabricate in this corpus);
/// its sync arm is shape-identical to the other plain-data fields and the
/// SIF/sigil behaviour is covered end-to-end by the omena-lsp-server test
/// suite, which runs with the memoized path as default.
pub fn summarize_workspace_diagnostics_salsa_memo_equivalence_v0(
    corpus: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaDiffSalsaMemoEquivalenceReportV0 {
    let host = RefCell::new(OmenaQueryStyleMemoHostV0::new());
    let compare = |phase: &'static str,
                   inputs: &SalsaMemoPhaseInputsV0<'_>|
     -> OmenaDiffSalsaMemoEquivalencePhaseV0 {
        let report = summarize_workspace_diagnostics_equivalence_v0(
            inputs.corpus,
            "straightLineFromScratch",
            |target_style_path| {
                evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
                    target_style_path,
                    inputs.corpus,
                    inputs.source_documents,
                    inputs.package_manifests,
                    &[],
                    inputs.resolution_inputs,
                )
            },
            "salsaMemoHost",
            |target_style_path| {
                host.borrow_mut().workspace_style_diagnostics(
                    target_style_path,
                    inputs.corpus,
                    inputs.source_documents,
                    inputs.package_manifests,
                    &[],
                    inputs.resolution_inputs,
                )
            },
        );
        OmenaDiffSalsaMemoEquivalencePhaseV0 { phase, report }
    };

    // The edit must CHANGE the from-scratch diagnostics, or the
    // afterTargetedEdit phase could pass vacuously while a stale memo serves
    // pre-edit results: a cross-file @extend of a nowhere-defined placeholder
    // is flagged by the workspace missingExtendTarget pass.
    let mut edited_corpus = corpus.to_vec();
    if let Some(last) = edited_corpus.last_mut() {
        last.style_source
            .push_str("\n.salsa-memo-edit { @extend %salsa-memo-missing-target; }\n");
    }
    let mut removed_corpus = corpus.to_vec();
    removed_corpus.pop();
    // Dropping the path mappings changes how aliased imports resolve, so a
    // sync bug on the resolution-inputs field diverges here.
    let stripped_resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    // A source document that uses only SOME selectors flips the
    // unused-selector pass for the rest, so a sync bug on the
    // source-documents field diverges here (non-vacuity pinned in tests).
    let usage_source_documents = salsa_memo_lifecycle_usage_source_document(corpus)
        .into_iter()
        .collect::<Vec<_>>();
    let probe_package_manifests = vec![OmenaQueryStylePackageManifestV0 {
        package_json_path: "/workspace/node_modules/@salsa-memo/probe/package.json".to_string(),
        package_json_source: "{\"name\":\"@salsa-memo/probe\",\"style\":\"index.scss\"}"
            .to_string(),
    }];

    let base = SalsaMemoPhaseInputsV0 {
        corpus,
        source_documents: &[],
        package_manifests: &[],
        resolution_inputs,
    };
    let phases = vec![
        compare("coldStart", &base),
        compare("warmRevalidation", &base),
        compare(
            "afterTargetedEdit",
            &SalsaMemoPhaseInputsV0 {
                corpus: edited_corpus.as_slice(),
                ..base
            },
        ),
        compare("afterRevert", &base),
        compare(
            "afterFileRemoval",
            &SalsaMemoPhaseInputsV0 {
                corpus: removed_corpus.as_slice(),
                ..base
            },
        ),
        compare("afterFileReAdd", &base),
        compare(
            "afterResolutionInputsChange",
            &SalsaMemoPhaseInputsV0 {
                resolution_inputs: &stripped_resolution_inputs,
                ..base
            },
        ),
        compare("afterResolutionInputsRevert", &base),
        compare(
            "afterSourceDocumentsAppear",
            &SalsaMemoPhaseInputsV0 {
                source_documents: usage_source_documents.as_slice(),
                ..base
            },
        ),
        compare("afterSourceDocumentsRevert", &base),
        compare(
            "afterPackageManifestsChange",
            &SalsaMemoPhaseInputsV0 {
                package_manifests: probe_package_manifests.as_slice(),
                ..base
            },
        ),
        compare("afterPackageManifestsRevert", &base),
    ];
    let comparison_count = phases.iter().map(|phase| phase.report.file_count).sum();
    let all_phases_identical = phases.iter().all(|phase| phase.report.all_files_identical);
    OmenaDiffSalsaMemoEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.salsa-memo-equivalence",
        phases,
        comparison_count,
        all_phases_identical,
    }
}

/// RFC 0009 Pillar F (rfcs#68) merge gate: N concurrent fixed-revision read
/// views over ONE `sync_workspace_for_parallel_resolve` bundle must be
/// byte-identical to the straight-line evaluator at every lifecycle position
/// — cold start, warm revalidation (the memo-hit position inside the views),
/// a targeted edit (the `set_*` happens AFTER the previous phase's views all
/// dropped, so this phase also witnesses the pending-write release), and the
/// revert. This turns the rfcs#64 spike's identical-reads result into a
/// standing gate.
///
/// Workers run on `std::thread::scope` rather than rayon: the spike proved
/// both drivers equivalent for fixed-revision reads, and scoped threads keep
/// this oracle crate free of the rayon dependency (rayon stays confined to
/// omena-lsp-server).
pub fn summarize_workspace_diagnostics_parallel_salsa_views_equivalence_v0(
    corpus: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaDiffSalsaMemoEquivalenceReportV0 {
    let host = RefCell::new(OmenaQueryStyleMemoHostV0::new());
    let compare = |phase: &'static str,
                   corpus_state: &[OmenaQueryStyleSourceInputV0]|
     -> OmenaDiffSalsaMemoEquivalencePhaseV0 {
        let candidate_jsons =
            parallel_salsa_view_diagnostics_jsons(&host, corpus_state, resolution_inputs);
        let files = corpus_state
            .iter()
            .zip(candidate_jsons)
            .map(|(file, candidate_json)| {
                let baseline_json = serialized_diagnostics(
                    evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
                        file.style_path.as_str(),
                        corpus_state,
                        &[],
                        &[],
                        &[],
                        resolution_inputs,
                    ),
                );
                let identical = baseline_json == candidate_json;
                OmenaDiffCacheEquivalenceFileReportV0 {
                    style_path: file.style_path.clone(),
                    identical,
                    baseline_json: (!identical).then_some(baseline_json),
                    candidate_json: (!identical).then_some(candidate_json),
                }
            })
            .collect::<Vec<_>>();
        let identical_file_count = files.iter().filter(|file| file.identical).count();
        OmenaDiffSalsaMemoEquivalencePhaseV0 {
            phase,
            report: OmenaDiffCacheEquivalenceReportV0 {
                schema_version: "0",
                product: "omena-diff-test.cache-equivalence",
                baseline_evaluator: "straightLineFromScratch",
                candidate_evaluator: "parallelSalsaFixedRevisionViews",
                file_count: files.len(),
                identical_file_count,
                all_files_identical: identical_file_count == files.len(),
                files,
            },
        }
    };

    // Same lifecycle edit as the serial gate: it must CHANGE the entry
    // file's from-scratch diagnostics (non-vacuity pinned there), so a stale
    // or torn parallel read diverges here instead of passing silently.
    let mut edited_corpus = corpus.to_vec();
    if let Some(last) = edited_corpus.last_mut() {
        last.style_source
            .push_str("\n.salsa-memo-edit { @extend %salsa-memo-missing-target; }\n");
    }

    let phases = vec![
        compare("coldStart", corpus),
        compare("warmRevalidation", corpus),
        compare("afterTargetedEdit", edited_corpus.as_slice()),
        compare("afterRevert", corpus),
    ];
    let comparison_count = phases.iter().map(|phase| phase.report.file_count).sum();
    let all_phases_identical = phases.iter().all(|phase| phase.report.all_files_identical);
    OmenaDiffSalsaMemoEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.parallel-salsa-views-equivalence",
        phases,
        comparison_count,
        all_phases_identical,
    }
}

/// One parallel-arm round: sync the host once (all `set_*` loop-side, before
/// the handle exists), fan one scoped thread per corpus file out over cloned
/// handles — each rebuilds its own fixed-revision view and resolves its
/// target — and collect the serialized diagnostics in corpus order. The sync
/// bundle (handle included) drops before this function returns. A corpus the
/// host refuses (duplicate paths — not reachable from the default corpus)
/// evaluates through the straight-line bypass, mirroring the host's own
/// semantics.
fn parallel_salsa_view_diagnostics_jsons(
    host: &RefCell<OmenaQueryStyleMemoHostV0>,
    corpus_state: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Vec<String> {
    let Some(sync) = host.borrow_mut().sync_workspace_for_parallel_resolve(
        corpus_state,
        &[],
        &[],
        &[],
        resolution_inputs,
    ) else {
        return corpus_state
            .iter()
            .map(|file| {
                serialized_diagnostics(evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
                    file.style_path.as_str(),
                    corpus_state,
                    &[],
                    &[],
                    &[],
                    resolution_inputs,
                ))
            })
            .collect();
    };
    let files_by_path = sync
        .files
        .iter()
        .map(|(style_path, file)| (style_path.as_str(), *file))
        .collect::<BTreeMap<_, _>>();
    let workspace = sync.workspace;
    // The barrier pins TRUE concurrency: every worker holds a live view at
    // the same instant before any query runs, so the gate cannot pass by
    // accidentally serializing the reads on a saturated runner.
    let start_barrier = std::sync::Barrier::new(corpus_state.len());
    let jsons = std::thread::scope(|scope| {
        let workers = corpus_state
            .iter()
            .map(|file| {
                let handle = sync.handle.clone();
                let target = files_by_path.get(file.style_path.as_str()).copied();
                let start_barrier = &start_barrier;
                scope.spawn(move || {
                    let db = OmenaQueryStyleMemoDatabaseV0::from_handle(handle);
                    start_barrier.wait();
                    let summary = target.and_then(|target| {
                        resolve_memo_workspace_style_diagnostics_from_view(&db, workspace, target)
                    });
                    serialized_diagnostics(summary)
                })
            })
            .collect::<Vec<_>>();
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .unwrap_or_else(|_| "parallelViewWorkerPanicked".to_string())
            })
            .collect::<Vec<_>>()
    });
    drop(sync);
    jsons
}

/// A source document importing the corpus entry file and using exactly one
/// of its selectors, so the unused-selector pass has a usage signal to flip.
fn salsa_memo_lifecycle_usage_source_document(
    corpus: &[OmenaQueryStyleSourceInputV0],
) -> Option<OmenaQuerySourceDocumentInputV0> {
    let entry = corpus.first()?;
    let file_name = entry.style_path.rsplit('/').next()?;
    Some(OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/src/SalsaMemoUsage.tsx".to_string(),
        source_source: format!(
            "import styles from \"./{file_name}\";\nexport const used = styles.app;\n"
        ),
        source_syntax_index: None,
        has_unresolved_style_import: false,
    })
}

fn serialized_diagnostics(diagnostics: Option<OmenaQueryStyleDiagnosticsForFileV0>) -> String {
    serde_json::to_string(&diagnostics)
        .unwrap_or_else(|error| format!("unserializableDiagnostics:{error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warm_pass_equivalence_holds_over_the_default_corpus() {
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let report =
            summarize_workspace_diagnostics_warm_pass_equivalence_v0(&corpus, &resolution_inputs);
        assert_eq!(report.file_count, corpus.len());
        assert!(
            report.all_files_identical,
            "warm-pass results must be byte-identical to from-scratch: {report:?}"
        );
    }

    #[test]
    fn default_corpus_produces_nonempty_diagnostics() -> Result<(), &'static str> {
        // The oracle is only meaningful when it compares non-trivial output:
        // the deliberately-missing `tokens.$missing` must produce at least one
        // diagnostic on the entry file.
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let diagnostics = evaluate_workspace_diagnostics_from_scratch_v0(
            "/workspace/src/App.module.scss",
            &corpus,
            &resolution_inputs,
        )
        .ok_or("workspace diagnostics for the corpus entry file")?;
        assert!(
            !diagnostics.diagnostics.is_empty(),
            "expected non-empty diagnostics, got {:?}",
            diagnostics.diagnostics
        );
        Ok(())
    }

    #[test]
    fn salsa_memo_equivalence_holds_across_the_cache_lifecycle() {
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let report =
            summarize_workspace_diagnostics_salsa_memo_equivalence_v0(&corpus, &resolution_inputs);
        assert_eq!(report.phases.len(), 12);
        assert!(
            report.all_phases_identical,
            "salsa-memoized diagnostics must be byte-identical to from-scratch in every lifecycle phase: {report:?}"
        );
    }

    #[test]
    fn parallel_salsa_views_equivalence_holds_across_the_lifecycle() {
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let report = summarize_workspace_diagnostics_parallel_salsa_views_equivalence_v0(
            &corpus,
            &resolution_inputs,
        );
        assert_eq!(report.phases.len(), 4);
        assert_eq!(report.comparison_count, corpus.len() * 4);
        assert!(
            report.all_phases_identical,
            "parallel fixed-revision view diagnostics must be byte-identical to from-scratch in every lifecycle phase: {report:?}"
        );
    }

    #[test]
    fn salsa_memo_lifecycle_edit_phase_is_not_vacuous() -> Result<(), &'static str> {
        // The targeted-edit phase only catches a stale memo if the edit
        // actually changes the from-scratch diagnostics of the edited file.
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let mut edited_corpus = corpus.clone();
        let last = edited_corpus.last_mut().ok_or("non-empty corpus")?;
        last.style_source
            .push_str("\n.salsa-memo-edit { @extend %salsa-memo-missing-target; }\n");
        let edited_path = last.style_path.clone();
        let before = serde_json::to_string(&evaluate_workspace_diagnostics_from_scratch_v0(
            edited_path.as_str(),
            &corpus,
            &resolution_inputs,
        ))
        .map_err(|_| "serialize before")?;
        let after = serde_json::to_string(&evaluate_workspace_diagnostics_from_scratch_v0(
            edited_path.as_str(),
            &edited_corpus,
            &resolution_inputs,
        ))
        .map_err(|_| "serialize after")?;
        assert_ne!(
            before, after,
            "the lifecycle edit must change the edited file's from-scratch diagnostics"
        );
        Ok(())
    }

    #[test]
    fn salsa_memo_lifecycle_source_documents_phase_is_not_vacuous() -> Result<(), &'static str> {
        // The source-documents phase only catches a sync bug on that field if
        // the usage document actually changes the entry file's diagnostics.
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let usage = salsa_memo_lifecycle_usage_source_document(&corpus)
            .ok_or("usage source document for the default corpus")?;
        let entry_path = corpus.first().ok_or("non-empty corpus")?.style_path.clone();
        let without =
            serde_json::to_string(&evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
                entry_path.as_str(),
                &corpus,
                &[],
                &[],
                &[],
                &resolution_inputs,
            ))
            .map_err(|_| "serialize without")?;
        let with =
            serde_json::to_string(&evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
                entry_path.as_str(),
                &corpus,
                std::slice::from_ref(&usage),
                &[],
                &[],
                &resolution_inputs,
            ))
            .map_err(|_| "serialize with")?;
        assert_ne!(
            without, with,
            "the usage source document must change the entry file's from-scratch diagnostics"
        );
        Ok(())
    }

    #[test]
    fn oracle_detects_a_diverging_candidate() -> Result<(), &'static str> {
        // Non-vacuity: a candidate that loses a file's diagnostics must fail
        // the gate — this is the failure mode a broken cache would produce.
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let report = summarize_workspace_diagnostics_equivalence_v0(
            &corpus,
            "straightLineFromScratch",
            |target_style_path| {
                evaluate_workspace_diagnostics_from_scratch_v0(
                    target_style_path,
                    &corpus,
                    &resolution_inputs,
                )
            },
            "candidateDroppingEntryFile",
            |target_style_path| {
                if target_style_path == "/workspace/src/App.module.scss" {
                    None
                } else {
                    evaluate_workspace_diagnostics_from_scratch_v0(
                        target_style_path,
                        &corpus,
                        &resolution_inputs,
                    )
                }
            },
        );
        assert!(!report.all_files_identical);
        let entry = report
            .files
            .iter()
            .find(|file| file.style_path == "/workspace/src/App.module.scss")
            .ok_or("entry file report")?;
        assert!(!entry.identical);
        assert!(entry.baseline_json.is_some() && entry.candidate_json.is_some());
        Ok(())
    }
}
