#![cfg(all(feature = "parallel-style-diagnostics", feature = "test-support"))]

use super::*;

use crate::parallel_style_wave::resolved_parallel_style_wave_targets;

static RESOLVER_IDENTITY_COUNTER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn resolver_identity_index_reuses_filesystem_generation_across_content_edits() -> TestResult {
    let _counter_guard = resolver_identity_counter_test_guard()?;
    let baseline = read_resolver_identity_index_baseline()?;
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
    let first_build_count = omena_query::omena_resolver_style_identity_index_build_count_for_test();

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
    assert_eq!(
        omena_query::omena_resolver_style_identity_index_build_count_for_test(),
        first_build_count.saturating_add(baseline.content_edit_index_build_delta),
        "content edits must not reconstruct the filesystem identity index"
    );

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
    assert_eq!(
        omena_query::omena_resolver_style_identity_index_build_count_for_test(),
        first_build_count.saturating_add(baseline.filesystem_event_index_build_delta),
        "filesystem events should reconstruct the identity index exactly once"
    );
    Ok(())
}

#[test]
fn resolver_identity_index_construction_work_scales_with_style_path_count() -> TestResult {
    let _counter_guard = resolver_identity_counter_test_guard()?;
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
}

struct ResolverIdentityIndexBaseline {
    small_style_path_count: usize,
    large_style_path_count: usize,
    index_build_count_per_generation: usize,
    content_edit_index_build_delta: usize,
    filesystem_event_index_build_delta: usize,
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
        content_edit_index_build_delta: baseline_usize(&baseline, "contentEditIndexBuildDelta")?,
        filesystem_event_index_build_delta: baseline_usize(
            &baseline,
            "filesystemEventIndexBuildDelta",
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

fn resolver_identity_counter_test_guard()
-> Result<std::sync::MutexGuard<'static, ()>, Box<dyn std::error::Error>> {
    RESOLVER_IDENTITY_COUNTER_TEST_LOCK
        .lock()
        .map_err(|_| std::io::Error::other("resolver identity counter test lock poisoned").into())
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
