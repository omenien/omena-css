//! Tide executor round-trip tests: prepare → collect → apply → complete
//! against a real two-document corpus, plus the disowned-tide path where a
//! window reopen drops the pending applies, plus demand-lattice targeting
//! (a cone flush covers the seeds' reverse-dependency closure, not the
//! corpus).

use super::handle_lsp_message;
use crate::tide::TideRepublishDemandV0;
use crate::{
    LspShellState, apply_tide_workspace_republish_item, collect_tide_workspace_republish_streaming,
    complete_tide_workspace_republish, enable_deferred_external_sif_refresh,
    prepare_tide_workspace_republish_job,
};
use serde_json::json;

fn open_document(state: &mut LspShellState, uri: &str, language_id: &str, text: &str) {
    handle_lsp_message(
        state,
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

fn republish_fixture_state() -> LspShellState {
    let mut state = LspShellState::default();
    enable_deferred_external_sif_refresh(&mut state);
    open_document(
        &mut state,
        "file:///workspace/src/Alpha.module.scss",
        "scss",
        ".alpha { color: red; }",
    );
    open_document(
        &mut state,
        "file:///workspace/src/Beta.module.scss",
        "scss",
        ".beta { color: blue; }",
    );
    state
}

fn settle_sif_lane(state: &mut LspShellState) -> Result<(), &'static str> {
    let sif_job = crate::prepare_deferred_external_sif_refresh_job(state)
        .ok_or("startup SIF demand must flush")?;
    let sif_result = crate::collect_deferred_external_sif_refresh(sif_job);
    crate::apply_deferred_external_sif_refresh_result(state, sif_result);
    Ok(())
}

#[test]
fn republish_tide_round_trip_covers_the_corpus() -> Result<(), &'static str> {
    let mut state = republish_fixture_state();
    let tick = 0;
    state
        .tide_republish_lane
        .deposit(TideRepublishDemandV0::All, tick);

    // The SIF lane holds startup demand (enable_deferred deposits), which
    // closes the republish frontier: no flush yet.
    assert!(
        prepare_tide_workspace_republish_job(&mut state, true).is_none(),
        "republish must wait for the SIF lane to settle"
    );

    settle_sif_lane(&mut state)?;

    let job = prepare_tide_workspace_republish_job(&mut state, true)
        .ok_or("settled frontier + idle courtesy must flush")?;
    let generation = job.generation;
    assert!(
        prepare_tide_workspace_republish_job(&mut state, true).is_none(),
        "one in-flight tide per lane"
    );

    let chunks = std::sync::Mutex::new(Vec::new());
    collect_tide_workspace_republish_streaming(job, &|result| {
        let Ok(mut chunks) = chunks.lock() else {
            return false;
        };
        chunks.push(result);
        true
    });
    let chunks = chunks
        .into_inner()
        .map_err(|_| "streaming chunks mutex should not be poisoned")?;
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
    Ok(())
}

#[test]
fn cone_flush_targets_only_the_seed_closure() -> Result<(), &'static str> {
    let mut state = LspShellState::default();
    enable_deferred_external_sif_refresh(&mut state);
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {"uri": "file:///workspace", "name": "workspace"},
                ],
            },
        }),
    );
    // Importer.module.scss uses Tokens.module.scss; Bystander is unrelated.
    open_document(
        &mut state,
        "file:///workspace/src/Tokens.module.scss",
        "scss",
        "$brand: red;\n.token { color: $brand; }",
    );
    open_document(
        &mut state,
        "file:///workspace/src/Importer.module.scss",
        "scss",
        "@use \"./Tokens.module.scss\" as tokens;\n.importer { color: red; }",
    );
    open_document(
        &mut state,
        "file:///workspace/src/Bystander.module.scss",
        "scss",
        ".bystander { color: green; }",
    );
    open_document(
        &mut state,
        "file:///workspace/src/App.tsx",
        "typescriptreact",
        "import styles from './Importer.module.scss';\nexport const a = styles.importer;",
    );
    settle_sif_lane(&mut state)?;
    // Drain the startup republish (the SIF apply deposits it).
    if let Some(job) = prepare_tide_workspace_republish_job(&mut state, true) {
        let generation = job.generation;
        collect_tide_workspace_republish_streaming(job, &|_| true);
        let _ = complete_tide_workspace_republish(&mut state, generation, Vec::new());
    }
    // A selector build feeds the reverse-dependency memo as its byproduct
    // (serial arm here; worker completions in production). Cone deposits
    // presuppose that: the SIF-delta seeding widens to All when the memo is
    // stale or absent, so a Cone demand only ever reaches the lane with a
    // fresh memo behind it.
    let _ = crate::resolve_style_diagnostics_for_uri(
        &state,
        "file:///workspace/src/Tokens.module.scss",
    );

    state.tide_republish_lane.deposit(
        TideRepublishDemandV0::cone([String::from("file:///workspace/src/Tokens.module.scss")]),
        1,
    );
    let job = prepare_tide_workspace_republish_job(&mut state, true).ok_or("cone must flush")?;
    let uris = job.target_uris_for_test();
    assert!(
        uris.iter().any(|uri| uri.ends_with("Tokens.module.scss")),
        "the seed itself is a target: {uris:?}"
    );
    assert!(
        !uris
            .iter()
            .any(|uri| uri.ends_with("Bystander.module.scss")),
        "a file outside the seed's reverse closure must NOT be a target: {uris:?}"
    );
    let generation = job.generation;
    collect_tide_workspace_republish_streaming(job, &|_| true);
    let _ = complete_tide_workspace_republish(&mut state, generation, Vec::new());
    Ok(())
}

#[test]
fn disowned_republish_tide_drops_leftovers_and_rearms() -> Result<(), &'static str> {
    let mut state = republish_fixture_state();
    settle_sif_lane(&mut state)?;

    state
        .tide_republish_lane
        .deposit(TideRepublishDemandV0::All, 0);
    let job = prepare_tide_workspace_republish_job(&mut state, true).ok_or("gate must open")?;
    let generation = job.generation;

    // The settle window reopens while the tide is in flight: the generation
    // watch moves, the wave aborts at item boundaries, and completion with
    // the stale generation must drop leftovers — the disowned demand is
    // owed again in the NEW window (per-epoch carry-over).
    state.tide_reopen_republish_window();
    assert!(state.tide_republish_lane_generation() > generation);
    assert!(
        state.tide_republish_lane.has_demand(),
        "the disowned tide's coverage carries over into the reopened window"
    );

    let chunks = std::sync::Mutex::new(Vec::new());
    collect_tide_workspace_republish_streaming(job, &|result| {
        let Ok(mut chunks) = chunks.lock() else {
            return false;
        };
        chunks.push(result);
        true
    });
    let chunks = chunks
        .into_inner()
        .map_err(|_| "streaming chunks mutex should not be poisoned")?;
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
    Ok(())
}

#[cfg(feature = "salsa-style-diagnostics")]
mod sif_delta_seeding {
    use crate::LspShellState;
    use crate::external_sif_loader::republish_demand_for_external_sif_delta;
    use crate::state::LspReverseDependencyIndexMemo;
    use crate::tide::TideRepublishDemandV0;
    use omena_query::{OmenaQueryExternalSifInputV0, ReverseDependencyIndexV0};
    use std::collections::{BTreeMap, BTreeSet};

    fn external_sif(url: &str, content: &[u8]) -> Option<OmenaQueryExternalSifInputV0> {
        let sif = omena_sif::OmenaSifV1::from_static_exports(
            url,
            omena_sif::OmenaSifGeneratorV1 {
                name: "fixture".to_string(),
                version: "0.1.0".to_string(),
                toolchain_id: "fixture@0.1.0".to_string(),
            },
            omena_sif::OmenaSifSourceV1 {
                syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
            },
            omena_sif::OmenaSifExportsV1 {
                variables: Vec::new(),
                mixins: Vec::new(),
                functions: Vec::new(),
                placeholders: Vec::new(),
                forwards: Vec::new(),
            },
            Vec::new(),
            content,
        )
        .ok()?;
        Some(OmenaQueryExternalSifInputV0 {
            canonical_url: url.to_string(),
            sif,
        })
    }

    fn state_with_reverse_index(edges: &[(&str, &str)]) -> LspShellState {
        let mut rev: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (target, dependent) in edges {
            rev.entry(target.to_string())
                .or_default()
                .insert(dependent.to_string());
        }
        let state = LspShellState::default();
        *state.reverse_dependency_index_memo.borrow_mut() = Some(LspReverseDependencyIndexMemo {
            revision: 1,
            summary_hash: "fixture".to_string(),
            ledger_epoch: 0,
            index: ReverseDependencyIndexV0 {
                rev,
                edges_by_from: BTreeMap::new(),
            },
        });
        state
    }

    #[test]
    fn changed_sif_with_attributed_importers_seeds_a_cone() -> Result<(), &'static str> {
        let url = "https://cdn.example/tokens.scss";
        let importer = "file:///workspace/src/User.module.scss";
        let mut state = state_with_reverse_index(&[(url, importer)]);
        state.resolution.external_sifs = vec![external_sif(url, b"$brand: red;").ok_or("old sif")?];
        let next = vec![external_sif(url, b"$brand: blue;").ok_or("new sif")?];
        assert_eq!(
            republish_demand_for_external_sif_delta(&state, next.as_slice()),
            TideRepublishDemandV0::cone([importer.to_string()]),
        );
        Ok(())
    }

    #[test]
    fn unattributed_url_and_missing_index_widen_to_all() -> Result<(), &'static str> {
        let url = "https://cdn.example/tokens.scss";
        let mut state = state_with_reverse_index(&[("https://other.example/x.scss", "file:///a")]);
        state.resolution.external_sifs = Vec::new();
        let next = vec![external_sif(url, b"$brand: red;").ok_or("sif")?];
        assert_eq!(
            republish_demand_for_external_sif_delta(&state, next.as_slice()),
            TideRepublishDemandV0::All,
            "an unattributable changed url must widen"
        );

        let mut cold = LspShellState::default();
        cold.resolution.external_sifs = Vec::new();
        assert_eq!(
            republish_demand_for_external_sif_delta(&cold, next.as_slice()),
            TideRepublishDemandV0::All,
            "no reverse index (cold start) must widen"
        );
        Ok(())
    }

    #[test]
    fn stale_reverse_index_widens_to_all() -> Result<(), &'static str> {
        let url = "https://cdn.example/tokens.scss";
        let importer = "file:///workspace/src/User.module.scss";
        let mut state = state_with_reverse_index(&[(url, importer)]);
        state.resolution.external_sifs = vec![external_sif(url, b"$brand: red;").ok_or("old sif")?];
        // A corpus-shaping input advances past the memo's stamp: the rev-set
        // for the url is PRESENT but may be missing a just-added importer,
        // so presence alone must not narrow the demand.
        state
            .tide_ledger
            .advance(&[crate::tide::TideInputKindV0::DocumentText]);
        let next = vec![external_sif(url, b"$brand: blue;").ok_or("new sif")?];
        assert_eq!(
            republish_demand_for_external_sif_delta(&state, next.as_slice()),
            TideRepublishDemandV0::All,
            "a stale reverse index must widen, never guess"
        );
        Ok(())
    }

    #[test]
    fn unchanged_sif_set_deposits_nothing() -> Result<(), &'static str> {
        let url = "https://cdn.example/tokens.scss";
        let mut state = state_with_reverse_index(&[(url, "file:///a")]);
        let sif = external_sif(url, b"$brand: red;").ok_or("sif")?;
        state.resolution.external_sifs = vec![sif.clone()];
        assert_eq!(
            republish_demand_for_external_sif_delta(&state, std::slice::from_ref(&sif)),
            TideRepublishDemandV0::None,
        );
        Ok(())
    }
}
