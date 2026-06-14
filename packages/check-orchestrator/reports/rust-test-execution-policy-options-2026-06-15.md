# Rust Test Execution Policy Options - 2026-06-15

## Status

This report is decision input only. It does not choose a policy and does not wire a new CI gate.

Already committed evidence:

- `rust-test-execution-inventory-2026-06-15.md`: 47 crates, 1681 listed tests, 29 meaningful suites without PR/push test execution.
- `rust-test-execution-timing-2026-06-15.md`: current `rust-workspace` CI timing, local cargo/nextest timing, and all-features failures.

## Current Constraint

The PR/push `rust-workspace` tier compiles and lints all Rust test code, but it does not execute a
test runner. The gap is execution coverage, not compilation coverage.

`--all-features` workspace execution is not currently green. A PR test tier that executes
`--all-features` must first fix or explicitly exclude the current failures:

- `omena-cascade grn::tests::grn_explicit_attractor_basin_proof_covers_all_n_le_16`
- `omena-cli::bin/omena-cli tests::compress_command_enforces_budget_bits`
- `omena-transform-passes tests::nesting_layers::layer_flatten_obligation_acceptance_tracks_smt_sat_result`

## Candidate A: Keep PR/Push Compile+Lint Only

Policy:

- Keep `rust-workspace` as `fmt + check + clippy --all-targets --all-features`.
- Treat Rust test execution as nightly/manual/release-lane coverage.

Implementation shape:

- No workflow change.
- Add an explicit policy note to the check documentation if this is the chosen contract.

Tradeoff:

- Lowest PR cost.
- Leaves 29 meaningful Rust suites without PR/push execution.

## Candidate B: Add a Default-Feature Product Test Tier

Policy:

- Add a PR/push reachable test-execution gate for the highest-value product/runtime crates.
- Keep it default-feature only until the all-features failures are fixed.

Recommended first package set:

- `omena-transform-passes` - 185 listed tests.
- `omena-parser` - 160 listed tests.
- `omena-cli` - 74 listed tests.
- `omena-interner` - 6 listed tests.
- `omena-bundler` - 10 listed tests.
- `omena-napi` - 30 listed tests.
- `omena-wasm` - 21 listed tests.
- `omena-sif` - 31 listed tests.

Measured local command:

```sh
cargo test --manifest-path rust/Cargo.toml \
  -p omena-transform-passes \
  -p omena-parser \
  -p omena-cli \
  -p omena-interner \
  -p omena-bundler \
  -p omena-napi \
  -p omena-wasm \
  -p omena-sif \
  --no-fail-fast
```

Result: success, 19.49s wall-clock on local macOS.

Measured local nextest command:

```sh
cargo nextest run --manifest-path rust/Cargo.toml \
  -p omena-transform-passes \
  -p omena-parser \
  -p omena-cli \
  -p omena-interner \
  -p omena-bundler \
  -p omena-napi \
  -p omena-wasm \
  -p omena-sif \
  --no-fail-fast
```

Result: success, 512 tests passed, 15.58s wall-clock on local macOS.

Implementation shape:

- Add a package target such as `check:rust-product-test-execution`.
- Add a declared gate such as `rust/product-test-execution` with `ciTier: "rust-workspace"` and `ciGroup: "rust-workspace"`.
- Add `pnpm omena-check run rust/product-test-execution` to the existing `rust-workspace` CI job.
- Regenerate `packages/check-orchestrator/CHECKS.md`.
- Verify `pnpm omena-check doctor`.
- Verify `pnpm --silent omena-check list --json` shows the new gate with non-null `ciTier`.

Cargo runner note:

- `cargo test -p ...` avoids adding a new CI installer dependency.
- `nextest` is faster locally but requires a pinned CI installation path before it can be used safely in GitHub Actions.

## Candidate C: Fix All-Features Failures Before Adding Execution Coverage

Policy:

- Do not add a new PR/push execution tier until `cargo nextest run --workspace --all-features --no-fail-fast` is green.

Implementation shape:

- Fix the three all-features failures first.
- Re-run the timing probe after the suite is green.
- Reconsider whether the PR tier should execute all meaningful crates or the full all-features workspace suite.

Tradeoff:

- Strongest semantic coverage if completed.
- Delays closing the current PR/push execution hole.

## Decision Needed

The next implementation step requires choosing one policy:

- `A`: Keep PR/push compile+lint only.
- `B`: Add the default-feature product test tier now.
- `C`: Fix all-features failures first, then choose the execution tier.

No CI wiring should be added until this decision is made explicitly.
