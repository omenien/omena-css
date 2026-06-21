//! Differential guard for the config-state worklist: it must emit byte-identical DIAGNOSTICS to the
//! legacy RawAllPaths all-paths enumeration on every corpus (the worklist only shrinks the internal
//! closure-edge set by collapsing non-min-depth duplicates; observable diagnostics are invariant).
//! Each corpus runs twice through the SAME diagnostic pipeline — once with the worklist, once with
//! RawAllPaths forced via `crate::style::with_rawallpaths_closure` — and the diagnostic bytes are
//! compared. The battery covers the convergence/min-depth/fan-out shapes plus the genuinely cyclic
//! config-mutating graph (the one case where a path enumerator and a state worklist can disagree).

use crate::{
    OmenaQueryStyleSourceInputV0, summarize_omena_query_style_diagnostics_for_workspace_file,
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

/// Sorted (code, message) pairs — the full observable diagnostic surface for a workspace file.
fn diagnostic_fingerprint(entry: &str, corpus: &[(&str, &str)]) -> Vec<(String, String)> {
    let Some(summary) = summarize_omena_query_style_diagnostics_for_workspace_file(
        entry,
        sources(corpus).as_slice(),
        &[],
        &[],
        None,
    ) else {
        return Vec::new();
    };
    let mut fingerprint = summary
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code.to_string(), diagnostic.message.clone()))
        .collect::<Vec<_>>();
    fingerprint.sort();
    fingerprint
}

// EQUAL/DIFFERENT-config convergence diamond: a reaches d via b (with red) AND via c (with red) AND
// via e (with green). RawAllPaths enumerates every path; the worklist forms every (node, config)
// state. Both surface the two distinct configs at d -> identical ConfigurationConflict + identity.
const CONVERGENCE: [(&str, &str); 5] = [
    (
        "/tmp/cv_d.scss",
        "$brand: blue !default; .base { color: $brand; }",
    ),
    ("/tmp/cv_b.scss", "@forward \"./cv_d\" with ($brand: red);"),
    ("/tmp/cv_c.scss", "@forward \"./cv_d\" with ($brand: red);"),
    (
        "/tmp/cv_e.scss",
        "@forward \"./cv_d\" with ($brand: green);",
    ),
    (
        "/tmp/cv_a.scss",
        "@use \"./cv_b\" as b; @use \"./cv_c\" as c; @use \"./cv_e\" as e;",
    ),
];

// NESTED config merge: a forwards b/c with distinct $x, b/c forward d with distinct $y -> d reached
// under {x:1,y:10} and {x:2,y:20}. The config-merge candidate from the counterexample hunt.
const NESTED_MERGE: [(&str, &str); 5] = [
    (
        "/tmp/nm_d.scss",
        "$x: blue !default; $y: purple !default; .base { color: $x; border-color: $y; }",
    ),
    ("/tmp/nm_b.scss", "@forward \"./nm_d\" with ($y: 10);"),
    ("/tmp/nm_c.scss", "@forward \"./nm_d\" with ($y: 20);"),
    (
        "/tmp/nm_a.scss",
        "@forward \"./nm_b\" with ($x: 1); @forward \"./nm_c\" with ($x: 2);",
    ),
    ("/tmp/nm_app.scss", "@use \"./nm_a\" as a;"),
];

// Linear config chain (acyclic; the hunt mislabeled this "cycle"): driver @use a, b forwards a, c
// forwards b. Simple paths == all walks, so worklist and RawAllPaths trivially agree.
const CHAIN: [(&str, &str); 4] = [
    ("/tmp/ch_a.scss", "$x: 0 !default; .base { color: $x; }"),
    ("/tmp/ch_b.scss", "@forward \"./ch_a\" with ($x: 1);"),
    ("/tmp/ch_c.scss", "@forward \"./ch_b\" with ($x: 2);"),
    ("/tmp/ch_driver.scss", "@use \"./ch_c\" as c;"),
];

// GENUINE config-mutating cycle: a forwards b with $x:1, b forwards a with $x:2 -> a<->b cycle. A
// path enumerator breaks the cycle at the first revisit; a (node,config) worklist can form a config
// state by traversing the cycle. This is the only shape where the two enumeration strategies can
// observably disagree, so it is the load-bearing case for the byte-identity invariant.
const CONFIG_CYCLE: [(&str, &str); 3] = [
    (
        "/tmp/cy_a.scss",
        "$x: 0 !default; @forward \"./cy_b\" with ($x: 1); .a { color: $x; }",
    ),
    (
        "/tmp/cy_b.scss",
        "$x: 0 !default; @forward \"./cy_a\" with ($x: 2); .b { color: $x; }",
    ),
    ("/tmp/cy_driver.scss", "@use \"./cy_a\" as a;"),
];

/// (label, entry file, corpus of (path, source)).
type BatteryCase = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str)],
);

const BATTERY: &[BatteryCase] = &[
    ("convergence", "/tmp/cv_a.scss", &CONVERGENCE),
    ("nested_merge", "/tmp/nm_app.scss", &NESTED_MERGE),
    ("chain", "/tmp/ch_driver.scss", &CHAIN),
    ("config_cycle", "/tmp/cy_driver.scss", &CONFIG_CYCLE),
];

#[test]
fn worklist_emits_byte_identical_diagnostics_to_rawallpaths() {
    let mut exercised_non_empty = false;
    for (label, entry, corpus) in BATTERY {
        let worklist = diagnostic_fingerprint(entry, corpus);
        let rawallpaths =
            crate::style::with_rawallpaths_closure(|| diagnostic_fingerprint(entry, corpus));
        assert_eq!(
            worklist, rawallpaths,
            "diagnostics diverge between the worklist and RawAllPaths on corpus `{label}`"
        );
        exercised_non_empty |= !worklist.is_empty();
    }
    assert!(
        exercised_non_empty,
        "battery produced no diagnostics — the differential is not exercising the closure"
    );
}
