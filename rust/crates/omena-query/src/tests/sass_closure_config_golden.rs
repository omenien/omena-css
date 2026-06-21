//! 0c characterization golden for the SLICE-2 config-state worklist (replaces the RawAllPaths
//! super-poly closure enumeration). The oracle corpus had NO config-bearing diamonds. Each test
//! pins the CURRENT (RawAllPaths) behavior so the worklist swap is a reviewed re-baseline:
//! - the DIAGNOSTIC bytes (sassModuleInstanceIdentity incl "in N hop(s)", sassModuleConfigurationConflict)
//!   are the byte-identity oracle and MUST stay identical after the worklist;
//! - graph_closure_edge_count + the multiplicity/depth structure are the RE-BASELINE surface that the
//!   worklist shrinks (path-duplicate + non-min-depth edges collapse to one representative).

use super::support::sample_input;
use crate::{
    OmenaQueryStyleSourceInputV0, summarize_omena_query_style_diagnostics_for_workspace_file,
    summarize_omena_query_style_semantic_graph_batch_from_sources,
};

fn sources(entries: &[(&str, &str)]) -> Vec<OmenaQueryStyleSourceInputV0> {
    entries
        .iter()
        .map(|(path, source)| OmenaQueryStyleSourceInputV0 {
            style_path: path.to_string(),
            style_source: source.to_string(),
        })
        .collect()
}

// EQUAL-config convergence (a reaches d via b AND via c, both `with red`) + DIFFERENT-config (via e
// `with green`). The two red paths produce TWO closure edges with an identical observable tuple,
// differing only in `.path` -> the worklist collapses them to ONE (count drops). The green path
// drives a ConfigurationConflict. The two distinct identity_keys each emit ONE identity diagnostic.
const DIAMOND: [(&str, &str); 5] = [
    (
        "/tmp/d.scss",
        "$brand: blue !default; .base { color: $brand; }",
    ),
    ("/tmp/b.scss", "@forward \"./d\" with ($brand: red);"),
    ("/tmp/c.scss", "@forward \"./d\" with ($brand: red);"),
    ("/tmp/e.scss", "@forward \"./d\" with ($brand: green);"),
    (
        "/tmp/a.scss",
        "@use \"./b\" as b; @use \"./c\" as c; @use \"./e\" as e;",
    ),
];

#[test]
fn golden_equal_and_different_config_convergence_diamond() -> Result<(), &'static str> {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(DIAMOND, &input);
    let resolution = &batch.sass_module_resolution;

    // RE-BASELINE (worklist shrinks this): the two `with red` a->d edges share one observable tuple
    // and differ only in .path; today they are stored as 2, the worklist will store 1.
    let a_to_d_red = resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| {
            edge.from_style_path == "/tmp/a.scss"
                && edge.target_style_path == "/tmp/d.scss"
                && edge.configuration_signature.contains("brand=3:red")
        })
        .count();
    assert_eq!(
        a_to_d_red, 2,
        "RawAllPaths stores both red convergence paths"
    );
    assert_eq!(
        resolution.graph_closure_edge_count, 9,
        "re-baseline target: drops to 8 when the red path-duplicate collapses"
    );

    // INVARIANT (must stay byte-identical after the worklist): the diagnostic bytes.
    let diagnostics = summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/a.scss",
        sources(&DIAMOND).as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;
    let identity = diagnostics
        .diagnostics
        .iter()
        .filter(|d| d.code == "sassModuleInstanceIdentity")
        .collect::<Vec<_>>();
    assert_eq!(
        identity.len(),
        2,
        "one identity per distinct config: {identity:?}"
    );
    assert!(identity.iter().all(|d| d.message.contains("in 2 hop(s)")));
    assert!(
        identity.iter().any(|d| d.message.contains("brand=3:red"))
            && identity.iter().any(|d| d.message.contains("brand=5:green"))
    );
    let conflict = diagnostics
        .diagnostics
        .iter()
        .filter(|d| d.code == "sassModuleConfigurationConflict")
        .collect::<Vec<_>>();
    assert_eq!(conflict.len(), 1, "{conflict:?}");
    assert!(
        conflict[0].message.contains("2 different configurations")
            && conflict[0].message.contains("brand=3:red")
            && conflict[0].message.contains("brand=5:green")
    );
    Ok(())
}

// MIN-DEPTH-wins: a reaches the SAME configured d-red at two GRAPH depths (via x = depth 2, via
// y->x = depth 3) -> ONE config-state at two depths. The dedup key omits depth, so the diagnostic
// embeds the MIN depth ("in 2 hop(s)"). The worklist records the min depth and emits ONE edge -> the
// depth-3 edge collapses (count drops), the diagnostic stays "in 2 hop(s)".
const MIN_DEPTH: [(&str, &str); 4] = [
    (
        "/tmp/md_d.scss",
        "$brand: blue !default; .x { color: $brand; }",
    ),
    ("/tmp/md_x.scss", "@forward \"./md_d\" with ($brand: red);"),
    ("/tmp/md_y.scss", "@forward \"./md_x\";"),
    (
        "/tmp/md_a.scss",
        "@use \"./md_x\" as x; @use \"./md_y\" as y;",
    ),
];

#[test]
fn golden_min_depth_wins_over_deeper_same_config_path() -> Result<(), &'static str> {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(MIN_DEPTH, &input);
    let resolution = &batch.sass_module_resolution;

    // RE-BASELINE: a reaches the SAME configured d-red at two graph depths (via x = depth 2, via
    // y->x = depth 3); the worklist keeps the min-depth representative only.
    let a_to_d_depths = resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| {
            edge.from_style_path == "/tmp/md_a.scss"
                && edge.target_style_path == "/tmp/md_d.scss"
                && edge.configuration_signature.contains("brand=3:red")
        })
        .map(|edge| edge.depth)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(a_to_d_depths, [2, 3].into_iter().collect());

    // INVARIANT: the identity diagnostic embeds the MIN depth ("in 2 hop(s)").
    let diagnostics = summarize_omena_query_style_diagnostics_for_workspace_file(
        "/tmp/md_a.scss",
        sources(&MIN_DEPTH).as_slice(),
        &[],
        &[],
        None,
    )
    .ok_or("workspace diagnostics")?;
    let identity = diagnostics
        .diagnostics
        .iter()
        .find(|d| {
            d.code == "sassModuleInstanceIdentity"
                && d.message.contains("/tmp/md_d.scss")
                && d.message.contains("hop(s)")
        })
        .ok_or("graph-reach identity diagnostic")?;
    assert!(
        identity.message.contains("in 2 hop(s)"),
        "min depth wins: {}",
        identity.message
    );
    Ok(())
}

// rule_ordinal / field FAN-OUT: two `@forward` of the same target that DIFFER (plain vs prefixed)
// are two distinct closure edges that must NOT collapse — the sentinel for a worklist that wrongly
// dedups on config-state alone (they share an empty config but differ on forward_prefix).
const FAN_OUT: [(&str, &str); 2] = [
    (
        "/tmp/fo_d.scss",
        "$brand: blue !default; .x { color: $brand; }",
    ),
    (
        "/tmp/fo_a.scss",
        "@forward \"./fo_d\"; @forward \"./fo_d\" as foo-*;",
    ),
];

#[test]
fn golden_distinct_forward_prefix_fan_out_does_not_collapse() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(FAN_OUT, &input);
    let resolution = &batch.sass_module_resolution;

    let fo_a_to_d = resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| {
            edge.from_style_path == "/tmp/fo_a.scss" && edge.target_style_path == "/tmp/fo_d.scss"
        })
        .map(|edge| edge.forward_prefix.clone())
        .collect::<std::collections::BTreeSet<_>>();
    // INVARIANT: both the plain (None) and prefixed (foo-) forward edges survive — the worklist
    // must NOT collapse same-config different-field edges.
    assert_eq!(
        fo_a_to_d,
        [None, Some("foo-".to_string())].into_iter().collect()
    );
    assert_eq!(resolution.graph_closure_edge_count, 2);
}
