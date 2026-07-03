//! M3 executor round-trip tests (rfcs#111 §8.5): prepare → collect → apply →
//! complete against a real two-document corpus, plus the disowned-tide path
//! where a window reopen drops the pending applies.

use super::handle_lsp_message;
use crate::tide::TideDemandV0;
use crate::{
    LspShellState, apply_tide_workspace_republish_item, collect_tide_workspace_republish_streaming,
    complete_tide_workspace_republish, enable_deferred_external_sif_refresh,
    prepare_tide_workspace_republish_job,
};
use serde_json::json;

fn open_style_document(state: &mut LspShellState, uri: &str, text: &str) {
    handle_lsp_message(
        state,
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

fn republish_fixture_state() -> LspShellState {
    let mut state = LspShellState::default();
    enable_deferred_external_sif_refresh(&mut state);
    open_style_document(
        &mut state,
        "file:///workspace/src/Alpha.module.scss",
        ".alpha { color: red; }",
    );
    open_style_document(
        &mut state,
        "file:///workspace/src/Beta.module.scss",
        ".beta { color: blue; }",
    );
    state
}

#[test]
fn republish_tide_round_trip_covers_the_corpus() {
    let mut state = republish_fixture_state();
    let tick = 0;
    state
        .tide_republish_lane
        .deposit(TideDemandV0::WorkspaceRepublish, tick);

    // The SIF lane holds startup demand (enable_deferred deposits), which
    // closes the republish frontier: no flush yet.
    assert!(
        prepare_tide_workspace_republish_job(&mut state, true).is_none(),
        "republish must wait for the SIF lane to settle"
    );

    // Settle the SIF lane the way the loop does: flush its demand into a job
    // and apply the (unchanged) result.
    let sif_job = crate::prepare_deferred_external_sif_refresh_job(&mut state)
        .expect("startup SIF demand must flush");
    let sif_result = crate::collect_deferred_external_sif_refresh(sif_job);
    crate::apply_deferred_external_sif_refresh_result(&mut state, sif_result);

    let job = prepare_tide_workspace_republish_job(&mut state, true)
        .expect("settled frontier + idle courtesy must flush");
    let generation = job.generation;
    assert!(
        prepare_tide_workspace_republish_job(&mut state, true).is_none(),
        "one in-flight tide per lane"
    );

    let mut chunks = Vec::new();
    collect_tide_workspace_republish_streaming(job, &mut |result| {
        chunks.push(result);
        true
    });
    assert!(
        chunks.last().is_some_and(|chunk| chunk.final_chunk),
        "the stream must terminate with a final chunk"
    );
    let mut items = Vec::new();
    let mut uncovered = Vec::new();
    for chunk in chunks {
        assert_eq!(chunk.generation, generation);
        items.extend(chunk.items);
        uncovered.extend(chunk.uncovered_uris);
    }
    assert_eq!(
        items.len() + uncovered.len(),
        2,
        "every corpus target is either covered or reported uncovered"
    );

    let mut published = 0usize;
    for item in items {
        let outputs = apply_tide_workspace_republish_item(&mut state, item);
        assert!(!outputs.is_empty(), "an applied item must publish");
        published += 1;
    }
    let effects = complete_tide_workspace_republish(&mut state, generation, uncovered.clone());
    assert!(
        published > 0 || !effects.deferred_diagnostics.is_empty() || !effects.outputs.is_empty(),
        "the tide must reach every target through the wave or the fallback arm"
    );
    assert!(
        !state.tide_republish_lane.in_flight(),
        "completion re-arms the lane"
    );
}

#[test]
fn disowned_republish_tide_drops_leftovers_and_rearms() {
    let mut state = republish_fixture_state();
    let sif_job = crate::prepare_deferred_external_sif_refresh_job(&mut state)
        .expect("startup SIF demand must flush");
    let sif_result = crate::collect_deferred_external_sif_refresh(sif_job);
    crate::apply_deferred_external_sif_refresh_result(&mut state, sif_result);

    state
        .tide_republish_lane
        .deposit(TideDemandV0::WorkspaceRepublish, 0);
    let job = prepare_tide_workspace_republish_job(&mut state, true).expect("gate open");
    let generation = job.generation;

    // The settle window reopens while the tide is in flight: the generation
    // watch moves, the wave aborts at item boundaries, and completion with
    // the stale generation must drop leftovers without touching the lane's
    // NEW generation.
    state.tide_reopen_republish_window();
    assert!(state.tide_republish_lane_generation() > generation);

    let mut chunks = Vec::new();
    collect_tide_workspace_republish_streaming(job, &mut |result| {
        chunks.push(result);
        true
    });
    assert!(
        chunks.iter().all(|chunk| chunk.items.is_empty()),
        "an aborted wave covers nothing"
    );
    assert!(chunks.last().is_some_and(|chunk| chunk.final_chunk));
    let uncovered: Vec<String> = chunks
        .into_iter()
        .flat_map(|chunk| chunk.uncovered_uris)
        .collect();
    let effects = complete_tide_workspace_republish(&mut state, generation, uncovered);
    assert!(
        effects.outputs.is_empty() && effects.deferred_diagnostics.is_empty(),
        "a disowned tide must not schedule fallback work"
    );
    assert!(!state.tide_republish_lane.in_flight());
}
