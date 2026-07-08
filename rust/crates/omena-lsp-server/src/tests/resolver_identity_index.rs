#![cfg(all(feature = "parallel-style-diagnostics", feature = "test-support"))]

use super::*;

use crate::parallel_style_wave::resolved_parallel_style_wave_targets;
use omena_testkit::{InstrumentationSessionV0, with_instrumentation_session};

#[test]
fn resolver_identity_index_reuses_filesystem_generation_across_content_edits() -> TestResult {
    with_instrumentation_session(InstrumentationSessionV0::default(), || {
        let first_uri = "file:///workspace/src/Alpha.module.scss";
        let second_uri = "file:///workspace/src/Beta.module.scss";
        let mut state = LspShellState::default();

        open_style_document(&mut state, first_uri, 1, ".alpha { color: red; }");
        open_style_document(&mut state, second_uri, 1, ".beta { color: blue; }");

        omena_query::reset_omena_resolver_style_identity_cache_for_test();
        let document_uris = vec![first_uri.to_string(), second_uri.to_string()];
        assert_eq!(
            resolved_parallel_style_wave_targets(&state, document_uris.as_slice(), 2).len(),
            2
        );
        let first_index = resolver_identity_index_ptr(&state)?;

        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": {
                    "textDocument": {
                        "uri": first_uri,
                        "version": 2,
                    },
                    "contentChanges": [
                        {
                            "text": ".alpha { color: green; }",
                        },
                    ],
                },
            }),
        );
        assert_eq!(
            resolved_parallel_style_wave_targets(&state, document_uris.as_slice(), 2).len(),
            2
        );
        let after_content_edit = resolver_identity_index_ptr(&state)?;
        // The rebuild witness is Arc POINTER identity, not the global build
        // counter: the counter is process-wide and other tests' waves bump
        // it concurrently (measured ~1/3 flake under parallel execution),
        // while a rebuild of THIS state's index always allocates a new Arc.
        assert_eq!(
            after_content_edit, first_index,
            "content edits must not rebuild the filesystem identity index"
        );

        handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "workspace/didChangeWatchedFiles",
                "params": {
                    "changes": [
                        {
                            "uri": second_uri,
                            "type": 2,
                        },
                    ],
                },
            }),
        );
        assert_eq!(
            resolved_parallel_style_wave_targets(&state, document_uris.as_slice(), 2).len(),
            2
        );
        let after_filesystem_event = resolver_identity_index_ptr(&state)?;

        assert_ne!(
            after_filesystem_event, first_index,
            "filesystem events must rebuild the filesystem identity index"
        );
        // "Exactly once" per generation stays pinned by
        // `construction_work_scales_with_style_path_count`, whose counter
        // window is a single wave call; this test's window spanned whole
        // LSP message turns, which is where the raced reads landed.
        Ok(())
    })
}

#[test]
fn resolver_identity_index_construction_work_scales_with_style_path_count() -> TestResult {
    with_instrumentation_session(InstrumentationSessionV0::default(), || {
        let baseline = read_resolver_identity_index_baseline()?;
        let small =
            identity_index_construction_counts_for_style_corpus(baseline.small_style_path_count)?;
        let large =
            identity_index_construction_counts_for_style_corpus(baseline.large_style_path_count)?;

        assert_eq!(
            small.0, baseline.index_build_count_per_generation,
            "a production wave should construct one filesystem identity index"
        );
        assert_eq!(
            large.0, baseline.index_build_count_per_generation,
            "a larger production wave should still construct one filesystem identity index"
        );
        let allowed_large_work = small
            .1
            .saturating_mul(baseline.construction_work_multiplier_numerator)
            .checked_div(baseline.construction_work_multiplier_denominator)
            .ok_or_else(|| std::io::Error::other("invalid resolver identity baseline divisor"))?
            .saturating_add(baseline.construction_work_epsilon);
        assert!(
            large.1 <= allowed_large_work,
            "identity index construction work should scale with path count: small={small:?} large={large:?} allowed_large_work={allowed_large_work}"
        );
        Ok(())
    })
}

struct ResolverIdentityIndexBaseline {
    small_style_path_count: usize,
    large_style_path_count: usize,
    index_build_count_per_generation: usize,
    construction_work_multiplier_numerator: usize,
    construction_work_multiplier_denominator: usize,
    construction_work_epsilon: usize,
}

fn read_resolver_identity_index_baseline()
-> Result<ResolverIdentityIndexBaseline, Box<dyn std::error::Error>> {
    let baseline_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("baselines")
        .join("resolver-identity-index-baseline-v0.json");
    let baseline: serde_json::Value =
        serde_json::from_str(std::fs::read_to_string(baseline_path)?.as_str())?;
    Ok(ResolverIdentityIndexBaseline {
        small_style_path_count: baseline_usize(&baseline, "smallStylePathCount")?,
        large_style_path_count: baseline_usize(&baseline, "largeStylePathCount")?,
        index_build_count_per_generation: baseline_usize(
            &baseline,
            "indexBuildCountPerGeneration",
        )?,
        construction_work_multiplier_numerator: baseline_usize(
            &baseline,
            "constructionWorkMultiplierNumerator",
        )?,
        construction_work_multiplier_denominator: baseline_usize(
            &baseline,
            "constructionWorkMultiplierDenominator",
        )?,
        construction_work_epsilon: baseline_usize(&baseline, "constructionWorkEpsilon")?,
    })
}

fn baseline_usize(
    baseline: &serde_json::Value,
    key: &'static str,
) -> Result<usize, Box<dyn std::error::Error>> {
    baseline
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| std::io::Error::other(format!("missing {key}")))?
        .try_into()
        .map_err(Into::into)
}

fn open_style_document(state: &mut LspShellState, uri: &str, version: i64, text: &str) {
    handle_lsp_message(
        state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": "scss",
                    "version": version,
                    "text": text,
                },
            },
        }),
    );
}

fn identity_index_construction_counts_for_style_corpus(
    style_count: usize,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut state = LspShellState::default();
    let mut uris = Vec::new();
    for index in 0..style_count {
        let uri = format!("file:///workspace/src/Style{index}.module.scss");
        let text = format!(".style{index} {{ color: red; }}");
        open_style_document(&mut state, uri.as_str(), 1, text.as_str());
        uris.push(uri);
    }
    omena_query::reset_omena_resolver_style_identity_cache_for_test();
    assert_eq!(
        resolved_parallel_style_wave_targets(&state, uris.as_slice(), 2).len(),
        style_count
    );
    Ok((
        omena_query::omena_resolver_style_identity_index_build_count_for_test(),
        omena_query::omena_resolver_style_identity_index_build_work_count_for_test(),
    ))
}

fn resolver_identity_index_ptr(state: &LspShellState) -> Result<usize, Box<dyn std::error::Error>> {
    let memo = state.resolver_identity_index_memo_lock();
    let index = memo
        .as_ref()
        .ok_or_else(|| std::io::Error::other("missing resolver identity index memo"))?;
    Ok(std::sync::Arc::as_ptr(&index.index) as usize)
}
