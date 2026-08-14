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
    let effects = complete_tide_workspace_republish(&mut state, generation, Vec::new());
    // The completion's source refresh is shaped by the SAME cone: App.tsx
    // depends on the seed's reverse closure and re-enters the per-file
    // arm; a source outside the cone must not.
    let touches = |uri: &str| {
        effects
            .deferred_diagnostics
            .iter()
            .any(|dispatch| dispatch.uri == uri)
            || effects.outputs.iter().any(|output| {
                output
                    .value
                    .pointer("/params/uri")
                    .and_then(serde_json::Value::as_str)
                    == Some(uri)
            })
    };
    assert!(
        touches("file:///workspace/src/App.tsx"),
        "a cone completion refreshes the cone's dependent sources"
    );
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
    state.tide_reopen_republish_window(crate::tide::TideDisownCauseV0::all(
        crate::tide::TideInputKindV0::DocumentSet,
    ));
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

#[test]
fn disown_collision_census_attributes_in_cone_and_out_of_cone_drivers()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = std::env::temp_dir().join(format!(
        "omena-tide-disown-census-{}-{}",
        std::process::id(),
        crate::current_time_millis()
    ));
    let src_dir = workspace_path.join("src");
    std::fs::create_dir_all(src_dir.as_path())?;
    let alpha_path = src_dir.join("Alpha.module.scss");
    let bystander_path = src_dir.join("Bystander.module.scss");
    let external_path = src_dir.join("_External.scss");
    let app_path = src_dir.join("App.tsx");
    std::fs::write(alpha_path.as_path(), ".alpha { color: red; }\n")?;
    std::fs::write(bystander_path.as_path(), ".bystander { color: green; }\n")?;
    std::fs::write(external_path.as_path(), "$brand: blue;\n")?;
    std::fs::write(
        app_path.as_path(),
        "import styles from './Alpha.module.scss';\nconst view = styles.al;\n",
    )?;

    let workspace_uri = crate::protocol::path_to_file_uri(workspace_path.as_path());
    let alpha_uri = crate::protocol::path_to_file_uri(alpha_path.as_path());
    let bystander_uri = crate::protocol::path_to_file_uri(bystander_path.as_path());
    let external_uri = crate::protocol::path_to_file_uri(external_path.as_path());
    let app_uri = crate::protocol::path_to_file_uri(app_path.as_path());
    let mut state = LspShellState::default();
    enable_deferred_external_sif_refresh(&mut state);
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "workspaceFolders": [{
                "uri": workspace_uri,
                "name": "workspace",
            }] },
        }),
    );
    open_document(
        &mut state,
        alpha_uri.as_str(),
        "scss",
        ".alpha { color: red; }",
    );
    open_document(
        &mut state,
        bystander_uri.as_str(),
        "scss",
        ".bystander { color: green; }",
    );
    open_document(
        &mut state,
        app_uri.as_str(),
        "typescriptreact",
        "import styles from './Alpha.module.scss';\nconst view = styles.al;",
    );
    settle_sif_lane(&mut state)?;
    if let Some(startup) = prepare_tide_workspace_republish_job(&mut state, true) {
        let generation = startup.generation;
        collect_tide_workspace_republish_streaming(startup, &|_| true);
        let _ = complete_tide_workspace_republish(&mut state, generation, Vec::new());
    }

    // Materialize the committed reverse-dependency scope before the cone
    // flush. Without it the conservative fallback is All, and Bystander is
    // correctly not classifiable as out-of-cone.
    let _ = crate::resolve_style_diagnostics_for_uri(&state, alpha_uri.as_str());
    state.tide_republish_lane.deposit(
        TideRepublishDemandV0::cone([alpha_uri.clone()]),
        state.tide_tick,
    );
    let first = prepare_tide_workspace_republish_job(&mut state, true)
        .ok_or("the alpha cone must enter flight")?;
    assert!(
        !first
            .target_uris_for_test()
            .iter()
            .any(|uri| crate::protocol::file_uri_equivalent(uri, bystander_uri.as_str())),
        "the seeded edit must be outside the frozen alpha cone"
    );

    // A real unrelated document edit changes the deferred external-SIF
    // document set. Removing the URI-set cause at that production advance
    // site makes the out-of-cone assertion below fail.
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": { "uri": bystander_uri, "version": 2 },
                "contentChanges": [{
                    "text": format!(
                        "@use \"{}\" as external;\n.bystander {{ color: external.$brand; }}",
                        external_uri
                    ),
                }],
            },
        }),
    );

    let hover = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "textDocument/hover",
            "params": {
                "textDocument": { "uri": alpha_uri },
                "position": { "line": 0, "character": 2 },
            },
        }),
    );
    assert!(
        hover
            .as_ref()
            .and_then(|value| value.pointer("/result/contents"))
            .is_some(),
        "the scripted hover must execute against the measured workspace"
    );
    let completion = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/completion",
            "params": {
                "textDocument": { "uri": app_uri },
                "position": { "line": 1, "character": 22 },
            },
        }),
    );
    assert!(
        completion
            .as_ref()
            .and_then(|value| value.pointer("/result/items"))
            .and_then(serde_json::Value::as_array)
            .is_some(),
        "the scripted completion must execute against the measured workspace"
    );

    settle_sif_lane(&mut state)?;
    let second = prepare_tide_workspace_republish_job(&mut state, true)
        .ok_or("carried demand must enter a second flight")?;
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "workspace/didChangeConfiguration",
            "params": { "settings": { "omena": { "diagnostics": {
                "severity": "error",
                "deepAnalysis": true,
            } } } },
        }),
    );

    let snapshot = state.snapshot();
    let total: u64 = snapshot.tide_disowns_total.values().sum();
    let out_of_cone: u64 = snapshot.tide_disowns_out_of_cone.values().sum();
    assert_eq!(
        snapshot.tide_disowns_total.get("documentSet"),
        Some(&1),
        "the unrelated document-set edit must disown one tide"
    );
    assert_eq!(
        snapshot.tide_disowns_out_of_cone.get("documentSet"),
        Some(&1),
        "the deliberately seeded unrelated edit must be classified out-of-cone"
    );
    assert_eq!(
        snapshot.tide_disowns_total.get("diagnosticSettings"),
        Some(&1),
        "the global diagnostic-settings driver must disown the second tide"
    );
    assert_eq!(
        snapshot.tide_disowns_out_of_cone.get("diagnosticSettings"),
        Some(&0),
        "a global setting overlaps every in-flight cone"
    );
    assert_eq!((total, out_of_cone), (2, 1));
    println!(
        "tide-disown-census total={total} out_of_cone={out_of_cone} ratio={:.2} threshold=0.20 disposition=step-1-remains",
        out_of_cone as f64 / total as f64
    );

    // The jobs are intentionally not executed: the test measures collision
    // attribution at the reopen boundary. Dropping them proves both waves
    // were genuinely in flight because their generation watches moved.
    drop(first);
    drop(second);
    let _ = std::fs::remove_dir_all(workspace_path);
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

#[test]
fn completion_refreshes_open_source_documents_against_the_settled_corpus()
-> Result<(), &'static str> {
    let mut state = republish_fixture_state();
    handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///workspace/src/App.tsx",
                    "languageId": "typescriptreact",
                    "version": 1,
                    "text": "import styles from \"./Alpha.module.scss\";\nconst view = <div className={styles.alpha} />;",
                },
            },
        }),
    );
    settle_sif_lane(&mut state)?;
    state
        .tide_republish_lane
        .deposit(TideRepublishDemandV0::All, 0);
    let job = prepare_tide_workspace_republish_job(&mut state, true).ok_or("gate must open")?;
    let generation = job.generation;
    collect_tide_workspace_republish_streaming(job, &|_| true);
    let effects = complete_tide_workspace_republish(&mut state, generation, Vec::new());
    let refreshes_source = effects
        .deferred_diagnostics
        .iter()
        .any(|dispatch| dispatch.uri == "file:///workspace/src/App.tsx")
        || effects.outputs.iter().any(|output| {
            output
                .value
                .pointer("/params/uri")
                .and_then(serde_json::Value::as_str)
                == Some("file:///workspace/src/App.tsx")
        });
    assert!(
        refreshes_source,
        "a current-generation completion must re-enter open SOURCE documents through the per-file arm — their diagnostics were computed against a pre-settle corpus"
    );
    Ok(())
}
