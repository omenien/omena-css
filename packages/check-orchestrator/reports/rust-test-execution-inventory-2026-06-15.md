# Rust Test Execution Inventory - 2026-06-15

## Methodology

- Base commit: `f46b348e`.
- Workspace crate count: 47 crates under `rust/crates`.
- Executable test count command:
  `cargo test --manifest-path rust/Cargo.toml --workspace --all-features -- --list --format terse`.
- Static attribute count: Rust files under each crate's `src`, `tests`, and `benches` directories,
  counting `#[test]` and `#[tokio::test]`.
- PR/push execution classification: `.github/workflows/ci.yml` `pnpm omena-check` targets were
  expanded through the check-orchestrator plan, then `cargo test -p <crate>` commands were extracted
  from reachable script commands.
- Classification policy: `empty` = 0 listed tests, `trivial` = 1-3 listed tests, `meaningful` = 4+
  listed tests.

## Summary

- Total crates: 47.
- Listed executable tests: 1681.
- Static test attributes: 1681.
- Crates with PR/push Rust test execution: 10.
- Crates without PR/push Rust test execution: 37.
- Meaningful suites without PR/push Rust test execution: 29.
- Empty crates: 3.
- Trivial suites: 6.
- Meaningful suites: 38.

The previous audit anchors are partly stale at current HEAD: `omena-bundler` still has 10 listed
tests, but `omena-cli` now has 74 listed tests, not 69.

## PR/Push-Tested Crates

`omena-bridge`, `omena-cascade`, `omena-diff-test`, `omena-lsp-server`, `omena-query`,
`omena-query-checker-orchestrator`, `omena-query-core`, `omena-resolver`, `omena-semantic`,
`omena-tsgo-client`.

## Inventory

| Crate                              | Listed tests | Static attrs | src attrs | integration attrs | doctests | Classification | PR/push test execution |
| ---------------------------------- | -----------: | -----------: | --------: | ----------------: | -------: | -------------- | ---------------------- |
| `engine-shadow-runner`             |            0 |            0 |         0 |                 0 |        0 | empty          | no                     |
| `engine-style-parser`              |           37 |           37 |        37 |                 0 |        0 | meaningful     | no                     |
| `omena-abstract-value`             |           73 |           73 |        73 |                 0 |        0 | meaningful     | no                     |
| `omena-benchmarks`                 |            6 |            6 |         6 |                 0 |        0 | meaningful     | no                     |
| `omena-bridge`                     |           72 |           72 |        72 |                 0 |        0 | meaningful     | yes                    |
| `omena-bundler`                    |           10 |           10 |        10 |                 0 |        0 | meaningful     | no                     |
| `omena-cascade`                    |           53 |           53 |        53 |                 0 |        0 | meaningful     | yes                    |
| `omena-categorical`                |           20 |           20 |        20 |                 0 |        0 | meaningful     | no                     |
| `omena-checker`                    |           35 |           34 |        34 |                 0 |        1 | meaningful     | no                     |
| `omena-cli`                        |           74 |           74 |        74 |                 0 |        0 | meaningful     | no                     |
| `omena-diff-test`                  |           20 |           22 |        20 |                 2 |        0 | meaningful     | yes                    |
| `omena-engine-input-producers`     |           43 |           43 |        43 |                 0 |        0 | meaningful     | no                     |
| `omena-ensemble`                   |            8 |            8 |         8 |                 0 |        0 | meaningful     | no                     |
| `omena-incremental`                |           16 |           16 |        16 |                 0 |        0 | meaningful     | no                     |
| `omena-interner`                   |            6 |            6 |         6 |                 0 |        0 | meaningful     | no                     |
| `omena-lawvere`                    |            6 |            6 |         6 |                 0 |        0 | meaningful     | no                     |
| `omena-lsp-server`                 |          167 |          167 |       167 |                 0 |        0 | meaningful     | yes                    |
| `omena-meta-macros`                |           10 |            8 |         7 |                 1 |        2 | meaningful     | no                     |
| `omena-napi`                       |           30 |           30 |        30 |                 0 |        0 | meaningful     | no                     |
| `omena-parser`                     |          160 |          160 |       160 |                 0 |        0 | meaningful     | no                     |
| `omena-query`                      |          326 |          326 |       326 |                 0 |        0 | meaningful     | yes                    |
| `omena-query-checker-orchestrator` |           13 |           13 |        13 |                 0 |        0 | meaningful     | yes                    |
| `omena-query-core`                 |            1 |            1 |         1 |                 0 |        0 | trivial        | yes                    |
| `omena-query-transform-runner`     |            1 |            1 |         1 |                 0 |        0 | trivial        | no                     |
| `omena-refinement`                 |            8 |            8 |         8 |                 0 |        0 | meaningful     | no                     |
| `omena-refinement-trait`           |            1 |            1 |         1 |                 0 |        0 | trivial        | no                     |
| `omena-resolver`                   |           65 |           65 |        65 |                 0 |        0 | meaningful     | yes                    |
| `omena-rg-flow`                    |            9 |            9 |         9 |                 0 |        0 | meaningful     | no                     |
| `omena-semantic`                   |           26 |           26 |        26 |                 0 |        0 | meaningful     | yes                    |
| `omena-sif`                        |           31 |           31 |        31 |                 0 |        0 | meaningful     | no                     |
| `omena-smt`                        |           10 |           10 |        10 |                 0 |        0 | meaningful     | no                     |
| `omena-spec-audit`                 |            3 |            3 |         3 |                 0 |        0 | trivial        | no                     |
| `omena-streaming-ifds`             |           10 |           10 |        10 |                 0 |        0 | meaningful     | no                     |
| `omena-syntax`                     |            7 |            7 |         7 |                 0 |        0 | meaningful     | no                     |
| `omena-testkit`                    |           32 |           32 |        32 |                 0 |        0 | meaningful     | no                     |
| `omena-transform-bundle`           |            0 |            0 |         0 |                 0 |        0 | empty          | no                     |
| `omena-transform-cst`              |            3 |            3 |         3 |                 0 |        0 | trivial        | no                     |
| `omena-transform-egg`              |           17 |           17 |        17 |                 0 |        0 | meaningful     | no                     |
| `omena-transform-passes`           |          185 |          185 |       185 |                 0 |        0 | meaningful     | no                     |
| `omena-transform-print`            |           13 |           13 |        13 |                 0 |        0 | meaningful     | no                     |
| `omena-transform-target`           |           13 |           13 |        13 |                 0 |        0 | meaningful     | no                     |
| `omena-tsgo-client`                |           25 |           25 |        25 |                 0 |        0 | meaningful     | yes                    |
| `omena-umbrella`                   |            0 |            0 |         0 |                 0 |        0 | empty          | no                     |
| `omena-variational`                |            6 |            6 |         6 |                 0 |        0 | meaningful     | no                     |
| `omena-wasm`                       |           21 |           22 |        22 |                 0 |        0 | meaningful     | no                     |
| `omena-zk-audit`                   |            6 |            6 |         6 |                 0 |        0 | meaningful     | no                     |
| `omena-zk-circuit`                 |            3 |            3 |         3 |                 0 |        0 | trivial        | no                     |
