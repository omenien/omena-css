# Rust Test Execution Timing Probe - 2026-06-15

## Methodology

- Base commit for this timing slice: `aa59a579`.
- Repository: `omenien/omena-css`, default branch `master`.
- CI source: `gh run list --workflow CI --branch master --limit 60`, with `gh run view --json jobs`
  for job durations and targeted `gh run view --job <id> --log` cache-line checks.
- Local host: macOS on `aarch64-apple-darwin`.
- Rust toolchain: `cargo 1.96.0 (30a34c682 2026-05-25)`.
- Nextest: `cargo-nextest 0.9.137`.

## CI Rust-Tier Timing

The current `rust-workspace` PR/push job is still compile-and-lint only. It runs:

`pnpm omena-check run rust/workspace`

which resolves to:

`cargo fmt --manifest-path rust/Cargo.toml --all --check && cargo check --manifest-path rust/Cargo.toml && cargo clippy --manifest-path rust/Cargo.toml --all-targets --all-features -- -D warnings`

Recent observed `rust-workspace` durations:

| Source                             |                                                                            Run | Cache state                                        | Result                                              |                                  Duration |
| ---------------------------------- | -----------------------------------------------------------------------------: | -------------------------------------------------- | --------------------------------------------------- | ----------------------------------------: |
| Latest successful master push      | [`27503876492`](https://github.com/omenien/omena-css/actions/runs/27503876492) | restore-key hit, full match false                  | success                                             |                                      134s |
| Warm full-match sample             | [`27500863062`](https://github.com/omenien/omena-css/actions/runs/27500863062) | full cache hit                                     | success                                             |                                       78s |
| Last 60 completed master `CI` runs |                                            `27413881619` through `27503876492` | all checked logs were cache hit or restore-key hit | mixed run conclusions, `rust-workspace` job present | min 58s / median 72s / avg 73s / max 134s |

Targeted cache-log scan across the latest 60 completed master `CI` runs did not find a true
`Cache not found` / cold-cache `rust-workspace` run. The available CI data therefore proves only
warm-cache and restore-key-hit timing. It does **not** prove cold-cache budget fit for adding a
workspace test-execution tier.

## Local Workspace Test Timing

| Command                                                                                       | Result      |                                    Tests | Wall time |
| --------------------------------------------------------------------------------------------- | ----------- | ---------------------------------------: | --------: |
| `cargo test --manifest-path rust/Cargo.toml --workspace --no-fail-fast`                       | success     | cargo runner output, default feature set |    46.90s |
| `cargo nextest run --manifest-path rust/Cargo.toml --workspace --no-fail-fast`                | success     |                              1665 passed |     9.96s |
| `cargo test --manifest-path rust/Cargo.toml --workspace --all-features`                       | failed fast |               stopped in `omena-cascade` |     1.36s |
| `cargo nextest run --manifest-path rust/Cargo.toml --workspace --all-features --no-fail-fast` | failed      |          1680 run, 1677 passed, 3 failed |    12.04s |

The default-feature workspace suite is currently runnable locally, and `nextest` is materially faster
than the serial cargo test runner on this host. The `--all-features` workspace suite is not currently
green, so an all-features PR test-execution tier would require test fixes or a deliberately narrower
policy before wiring.

## All-Features Failures Observed

`cargo nextest run --manifest-path rust/Cargo.toml --workspace --all-features --no-fail-fast`
reported these failures:

- `omena-cascade grn::tests::grn_explicit_attractor_basin_proof_covers_all_n_le_16`
- `omena-cli::bin/omena-cli tests::compress_command_enforces_budget_bits`
- `omena-transform-passes tests::nesting_layers::layer_flatten_obligation_acceptance_tracks_smt_sat_result`

These failures are policy-relevant: `check:rust-workspace` compiles all test code with
`clippy --all-targets --all-features`, but executing the full all-features workspace test suite is
not currently a green gate.

## Budget Implication

The measured local default-feature workspace test run is small enough that a PR-reachable test tier
is plausible if it uses `nextest` and stays scoped to the meaningful suites identified in
`rust-test-execution-inventory-2026-06-15.md`.

However, this probe does **not** close the cold-cache budget question. Recent CI history only provides
warm-cache or restore-key-hit evidence. Before wiring a broad workspace test tier, one of these must
be true:

- capture a real cold-cache CI run without changing product policy;
- run a controlled temporary cold-key CI experiment and record it separately;
- choose a narrower PR tier whose budget is justified by measured warm CI plus local nextest timing,
  while keeping cold-cache risk explicit.

## Decision Input

This artifact supports the Slice 3 user decision, but does not make that decision.

Reasonable options now:

- Keep PR/push as compile-and-lint only and document tests as nightly/manual.
- Add a PR-reachable default-feature `nextest` tier for the meaningful product/runtime suites.
- First fix the three all-features failures, then reassess an all-features test-execution tier.

No CI wiring should be added until the coverage policy is chosen explicitly.
