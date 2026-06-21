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
use omena_sif::{
    OmenaSifExportsV1, OmenaSifGeneratorV1, OmenaSifSourceSyntaxV1, OmenaSifSourceV1, OmenaSifV1,
    OmenaSifVariableExportV1,
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
            style_source: "@use \"@workspace/tokens\" as tokens;\n@use \"./mid\";\n\
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
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/ExternalPackage.module.scss".to_string(),
            style_source: "@use \"@app/theme/index\" as ds;\n\
                           .external { color: ds.$ds_gray-700; border-radius: ds.$ds_radius-card; }\n"
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/node_modules/@app/theme/index.scss".to_string(),
            style_source: "@forward \"@design/tokens/colors\";\n@forward \"./radius\";\n"
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/node_modules/@app/theme/_radius.scss".to_string(),
            style_source: "$ds_radius-card: 12px;\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/node_modules/@design/tokens/colors.scss".to_string(),
            style_source: "$ds_gray-700: #374151;\n".to_string(),
        },
        // Cyclic / deep-chain corpus extension: the default corpus above is acyclic. Under
        // omena-diff-test (built WITHOUT hypergraph-ifds) these fixtures pin the NATIVE
        // sassUseCycle detector across the warm / salsa-memo / parallel-view arms; they do NOT
        // reach the hypergraph-ifds-gated cross-file SCC/closure primitives (those are covered by
        // the omena-streaming-ifds guard tests). A two-node `@use` ring -> sassUseCycle.
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_cycle_a.scss".to_string(),
            style_source: "@use \"./cycle_b\";\n.cycle-a { color: red; }\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_cycle_b.scss".to_string(),
            style_source: "@use \"./cycle_a\";\n.cycle-b { color: blue; }\n".to_string(),
        },
        // A deep `@forward` chain (4 hops h0->h1->h2->h3->h4) — acyclic, evaluated across the 3
        // arms for parity (no cycle diagnostic).
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_fwd_h0.scss".to_string(),
            style_source: "@forward \"./fwd_h1\";\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_fwd_h1.scss".to_string(),
            style_source: "@forward \"./fwd_h2\";\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_fwd_h2.scss".to_string(),
            style_source: "@forward \"./fwd_h3\";\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_fwd_h3.scss".to_string(),
            style_source: "@forward \"./fwd_h4\";\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_fwd_h4.scss".to_string(),
            style_source: "$deep_token: 8px;\n".to_string(),
        },
        // A chord cycle: a -> b -> c -> a with the chord a -> c (a 3-node loop with an extra
        // intra-loop edge) -> sassUseCycle on the `_chord_a` entry.
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_chord_a.scss".to_string(),
            style_source: "@use \"./chord_b\";\n@use \"./chord_c\";\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_chord_b.scss".to_string(),
            style_source: "@use \"./chord_c\";\n".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/_chord_c.scss".to_string(),
            style_source: "@use \"./chord_a\";\n".to_string(),
        },
    ];
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: vec![
            OmenaQueryStylePackageManifestV0 {
                package_json_path: "/workspace/node_modules/@app/theme/package.json".to_string(),
                package_json_source: "{\"exports\":{\"./index\":{\"sass\":\"./index.scss\"}}}"
                    .to_string(),
            },
            OmenaQueryStylePackageManifestV0 {
                package_json_path: "/workspace/node_modules/@design/tokens/package.json"
                    .to_string(),
                package_json_source: "{\"exports\":{\"./colors\":{\"sass\":\"./colors.scss\"}}}"
                    .to_string(),
            },
        ],
        tsconfig_path_mappings: vec![omena_query::OmenaQueryTsconfigPathMappingV0 {
            base_path: "/workspace".to_string(),
            pattern: "@workspace/*".to_string(),
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
/// accepts; Auto external classification matches the memo host and LSP.
pub fn evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
    target_style_path: &str,
    corpus: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
        target_style_path,
        corpus,
        source_documents,
        package_manifests,
        None,
        OmenaQueryExternalModuleModeV0::Auto,
        external_sifs,
        resolution_inputs,
    )
}

/// One lifecycle position's full input set.
struct SalsaMemoPhaseInputsV0<'inputs> {
    corpus: &'inputs [OmenaQueryStyleSourceInputV0],
    source_documents: &'inputs [OmenaQuerySourceDocumentInputV0],
    package_manifests: &'inputs [OmenaQueryStylePackageManifestV0],
    external_sifs: &'inputs [OmenaQueryExternalSifInputV0],
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
/// The lifecycle includes a non-empty `external_sifs` phase so the memo host's
/// SIF input surface is compared against from-scratch evaluation, not assumed
/// equivalent by shape alone.
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
                    inputs.external_sifs,
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
                    inputs.external_sifs,
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
    let external_sif_corpus = salsa_memo_lifecycle_external_sif_corpus(corpus);
    let external_sifs = salsa_memo_lifecycle_external_sif_inputs();

    let base = SalsaMemoPhaseInputsV0 {
        corpus,
        source_documents: &[],
        package_manifests: &[],
        external_sifs: &[],
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
        compare(
            "afterExternalSifsAppear",
            &SalsaMemoPhaseInputsV0 {
                corpus: external_sif_corpus.as_slice(),
                external_sifs: external_sifs.as_slice(),
                ..base
            },
        ),
        compare("afterExternalSifsRevert", &base),
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

fn salsa_memo_lifecycle_external_sif_corpus(
    corpus: &[OmenaQueryStyleSourceInputV0],
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut external_corpus = corpus.to_vec();
    if let Some(entry) = external_corpus.first_mut() {
        entry.style_source.push_str(
            "\n@use \"https://cdn.example/salsa-memo/tokens.scss\" as externalTokens;\n\
             .external { color: externalTokens.$brand; }\n",
        );
    }
    external_corpus
}

fn salsa_memo_lifecycle_external_sif_inputs() -> Vec<OmenaQueryExternalSifInputV0> {
    let Ok(sif) = OmenaSifV1::from_static_exports(
        "https://cdn.example/salsa-memo/tokens.scss",
        OmenaSifGeneratorV1 {
            name: "omena-diff-test-fixture".to_string(),
            version: "0.0.0".to_string(),
            toolchain_id: "omena-diff-test-fixture@0.0.0".to_string(),
        },
        OmenaSifSourceV1 {
            syntax: OmenaSifSourceSyntaxV1::Scss,
        },
        OmenaSifExportsV1 {
            variables: vec![OmenaSifVariableExportV1 {
                name: "$brand".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: Vec::new(),
            functions: Vec::new(),
            placeholders: Vec::new(),
            forwards: Vec::new(),
        },
        Vec::new(),
        b"$brand: red !default;",
    ) else {
        return Vec::new();
    };
    vec![OmenaQueryExternalSifInputV0 {
        canonical_url: sif.canonical_url.clone(),
        sif,
    }]
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
    fn default_corpus_cyclic_fixtures_witness_cross_file_cycle_diagnostics()
    -> Result<(), &'static str> {
        // SLICE-0 characterization (RED witness): the cyclic fixtures appended to the default
        // corpus must produce the cross-file `sassUseCycle` diagnostic at HEAD, pinning the
        // behavior the SLICE-A (one shared SCC owner) and SLICE-2 (config-state worklist) refactors
        // must keep byte-identical. The full-output byte-identity across arms is already covered by
        // the equivalence tests now that the corpus is cyclic; this asserts the cycle is detected.
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();

        let ring = evaluate_workspace_diagnostics_from_scratch_v0(
            "/workspace/src/_cycle_a.scss",
            &corpus,
            &resolution_inputs,
        )
        .ok_or("ring entry diagnostics")?;
        let ring_cycles = ring
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "sassUseCycle")
            .collect::<Vec<_>>();
        assert!(
            !ring_cycles.is_empty(),
            "the `@use` ring must witness a sassUseCycle diagnostic: {:?}",
            ring.diagnostics
        );
        assert!(
            ring_cycles
                .iter()
                .all(|diagnostic| diagnostic.severity == "error"),
            "sassUseCycle is error-severity: {ring_cycles:?}"
        );
        assert!(
            ring_cycles.iter().any(|diagnostic| diagnostic
                .message
                .contains("/workspace/src/_cycle_a.scss")
                && diagnostic.message.contains("/workspace/src/_cycle_b.scss")),
            "the cycle names both ring members: {ring_cycles:?}"
        );

        let chord = evaluate_workspace_diagnostics_from_scratch_v0(
            "/workspace/src/_chord_a.scss",
            &corpus,
            &resolution_inputs,
        )
        .ok_or("chord entry diagnostics")?;
        // 0a golden: the chord SCC {a,b,c} (edges a->b, a->c, b->c, c->a) has TWO elementary
        // circuits through `_chord_a` (a->b->c->a and a->c->a), so target=_chord_a keeps BOTH
        // sassUseCycle diagnostics, anchored on DIFFERENT `@use` statements. The SLICE-A producer
        // rewire (off the RawAllPaths closure scan onto SCC-gated elementary-circuit enumeration)
        // must preserve this set; a one-ring-per-SCC producer would drop the a->c diagnostic.
        let chord_cycles = chord
            .diagnostics
            .iter()
            .filter(|d| d.code == "sassUseCycle")
            .collect::<Vec<_>>();
        assert_eq!(
            chord_cycles.len(),
            2,
            "chord target keeps both elementary-circuit diagnostics: {chord_cycles:?}"
        );
        assert!(chord_cycles.iter().all(|d| d.severity == "error"));
        // the 3-cycle a->b->c->a, anchored on the `@use \"./chord_b\"` statement (line 0)
        assert!(
            chord_cycles
                .iter()
                .any(|d| d.message.contains("_chord_b.scss") && d.range.start.line == 0),
            "3-cycle diagnostic on the chord_b @use: {chord_cycles:?}"
        );
        // the 2-cycle a->c->a (via the chord edge), anchored on the `@use \"./chord_c\"` (line 1)
        assert!(
            chord_cycles
                .iter()
                .any(|d| !d.message.contains("_chord_b.scss") && d.range.start.line == 1),
            "2-cycle diagnostic on the chord_c @use: {chord_cycles:?}"
        );
        Ok(())
    }

    fn dense_k4_cycle_corpus() -> Vec<OmenaQueryStyleSourceInputV0> {
        // A complete K4 module digraph (every node `@use`s the other three) — a DENSE SCC where the
        // CanonicalFirstTarget visited-prune drops elementary circuits that the RawAllPaths producer
        // (and the SLICE-A SCC-gated enumeration) keep. Distinguishes Option D from a regression to
        // CanonicalFirstTarget-as-producer, which matches on the chord but not here.
        let nodes = ["a", "b", "c", "d"];
        nodes
            .iter()
            .map(|node| OmenaQueryStyleSourceInputV0 {
                style_path: format!("/workspace/src/_k4_{node}.scss"),
                style_source: nodes
                    .iter()
                    .filter(|other| *other != node)
                    .map(|other| format!("@use \"./k4_{other}\";\n"))
                    .collect::<String>(),
            })
            .collect()
    }

    #[test]
    fn dense_scc_witnesses_full_elementary_circuit_diagnostic_set() -> Result<(), &'static str> {
        let corpus = dense_k4_cycle_corpus();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let diagnostics = evaluate_workspace_diagnostics_from_scratch_v0(
            "/workspace/src/_k4_a.scss",
            &corpus,
            &resolution_inputs,
        )
        .ok_or("k4 entry diagnostics")?;
        let cycles = diagnostics
            .diagnostics
            .iter()
            .filter(|d| d.code == "sassUseCycle")
            .collect::<Vec<_>>();
        // 0a golden: target=_k4_a sees ALL 15 elementary circuits through it (len 2: 3, len 3: 6,
        // len 4: 6), evenly anchored across its three `@use` edges (5 per line). A producer using
        // the CanonicalFirstTarget visited-prune drops the revisit-requiring circuits and yields
        // fewer; Option D's SCC-confined enumeration must reproduce the full 15.
        assert_eq!(
            cycles.len(),
            15,
            "K4 target keeps the full elementary-circuit set: {}",
            cycles.len()
        );
        assert!(cycles.iter().all(|d| d.severity == "error"));
        for line in 0..3usize {
            assert_eq!(
                cycles.iter().filter(|d| d.range.start.line == line).count(),
                5,
                "each of the three @use edges anchors 5 circuits (line {line})"
            );
        }
        Ok(())
    }

    #[test]
    fn salsa_memo_equivalence_holds_across_the_cache_lifecycle() {
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let report =
            summarize_workspace_diagnostics_salsa_memo_equivalence_v0(&corpus, &resolution_inputs);
        assert_eq!(report.phases.len(), 14);
        assert!(
            report.all_phases_identical,
            "salsa-memoized diagnostics must be byte-identical to from-scratch in every lifecycle phase: {report:?}"
        );
    }

    #[test]
    fn salsa_memo_lifecycle_exercises_nonempty_external_sifs() {
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let report =
            summarize_workspace_diagnostics_salsa_memo_equivalence_v0(&corpus, &resolution_inputs);
        assert!(
            report
                .phases
                .iter()
                .any(|phase| phase.phase == "afterExternalSifsAppear"),
            "the salsa memo oracle must include a non-empty external_sifs phase: {report:?}"
        );
    }

    #[test]
    fn salsa_memo_lifecycle_external_sifs_phase_is_not_vacuous() -> Result<(), &'static str> {
        let (corpus, resolution_inputs) = omena_diff_cache_equivalence_default_corpus_v0();
        let external_corpus = salsa_memo_lifecycle_external_sif_corpus(&corpus);
        let external_sifs = salsa_memo_lifecycle_external_sif_inputs();
        let entry_path = external_corpus
            .first()
            .ok_or("non-empty external SIF corpus")?
            .style_path
            .clone();
        let without =
            serde_json::to_string(&evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
                entry_path.as_str(),
                &external_corpus,
                &[],
                &[],
                &[],
                &resolution_inputs,
            ))
            .map_err(|_| "serialize without")?;
        let with =
            serde_json::to_string(&evaluate_workspace_diagnostics_from_scratch_with_inputs_v0(
                entry_path.as_str(),
                &external_corpus,
                &[],
                &[],
                external_sifs.as_slice(),
                &resolution_inputs,
            ))
            .map_err(|_| "serialize with")?;
        assert_ne!(
            without, with,
            "the external_sifs lifecycle phase must change from-scratch diagnostics"
        );
        Ok(())
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
