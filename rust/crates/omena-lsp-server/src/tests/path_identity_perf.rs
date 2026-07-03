//! Path-identity perf gates (RFC: LSP main-loop O(N^2) path-identity hot path).
//!
//! The document store derives file identity through `protocol::normalize_path` on
//! every access (key derivation) and, on a lookup miss, once per existing key in
//! a linear `file_uri_equivalent` scan; `normalize_path` performs an
//! `fs::canonicalize` syscall. During workspace indexing this is an O(N^2) syscall
//! storm that starves the main loop. The fix is canonicalize-once (memoize the
//! chokepoint). These gates assert the canonicalize-once invariant directly at
//! `normalize_path`: repeated identity of the same existing path performs zero
//! syscalls, and touching N distinct existing paths over M waves performs at most
//! N syscalls (once per distinct path), not N*M. Both are RED on the pre-fix code
//! (one syscall per call) and GREEN after the memo.

use super::TestResult;
use crate::protocol::{
    canonicalize_syscall_count, normalize_path, reset_canonicalize_syscall_count,
};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn fresh_dir(tag: &str) -> std::io::Result<PathBuf> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let dir = std::env::temp_dir().join(format!(
        "omena_pathid_{}_{}_{}",
        tag,
        std::process::id(),
        unique
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn make_scss(dir: &Path, name: &str) -> std::io::Result<PathBuf> {
    let path = dir.join(name);
    std::fs::write(&path, ".x { color: red; }\n")?;
    Ok(path)
}

#[test]
fn normalize_path_repeated_identity_is_zero_syscall() -> TestResult {
    let dir = fresh_dir("repeat")?;
    let path = make_scss(&dir, "a.module.scss")?;

    // Prime: the first canonicalize of an existing path is the unavoidable
    // one-time cost.
    let primed = normalize_path(path.clone());

    reset_canonicalize_syscall_count();
    const REPEATS: usize = 64;
    for _ in 0..REPEATS {
        let again = normalize_path(path.clone());
        assert_eq!(again, primed, "canonical form must be stable across repeats");
    }

    let syscalls = canonicalize_syscall_count();
    assert_eq!(
        syscalls, 0,
        "repeating identity of the same existing path must perform ZERO fs::canonicalize \
         syscalls (canonicalize-once memo); got {syscalls} over {REPEATS} repeats"
    );

    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn normalize_path_canonicalizes_each_distinct_path_at_most_once() -> TestResult {
    let dir = fresh_dir("scaling")?;
    const N: usize = 64;
    const PASSES: usize = 8;
    let mut paths: Vec<PathBuf> = Vec::with_capacity(N);
    for i in 0..N {
        paths.push(make_scss(&dir, &format!("m{i}.module.scss"))?);
    }

    // Mirror the document-store hot path: each of PASSES waves re-derives identity
    // for every path (the per-batch reinsert + linear scan). The storm is
    // O(N * PASSES) syscalls today; canonicalize-once makes it O(N).
    reset_canonicalize_syscall_count();
    for _ in 0..PASSES {
        for path in &paths {
            let _ = normalize_path(path.clone());
        }
    }

    let syscalls = canonicalize_syscall_count();
    assert!(
        syscalls <= N,
        "indexing {N} distinct existing paths over {PASSES} waves must perform at most \
         one fs::canonicalize per distinct path (<= {N}); got {syscalls} \
         (pre-fix this is ~{} = N*PASSES)",
        N * PASSES
    );

    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
