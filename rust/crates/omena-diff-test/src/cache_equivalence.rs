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
    OmenaQueryExternalModuleModeV0, OmenaQueryStyleDiagnosticsForFileV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
};
use serde::Serialize;

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
