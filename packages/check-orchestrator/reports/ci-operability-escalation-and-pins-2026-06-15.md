# CI Operability Escalation And Tool Pins

## Summary

This report records the non-topology CI operability decisions behind the scheduled-tier escalation
and OXC toolchain pin checks.

## Scheduled-Tier Escalation

Scheduled workflows now have a durable failure escalation path:

- `.github/workflows/nightly-soak.yml`
- `.github/workflows/omena-css-drift.yml`
- `.github/workflows/security-audit.yml`

Each grants `issues: write` and calls `./.github/actions/escalate-ci-failure` from a
`failure()`-guarded step. The action deduplicates open issues by exact title and comments on the
existing issue when a persistent failure repeats.

Cost tier: low. This is an additive notification path and does not alter the CI topology.

## Drift Verdict

`omena-css-drift.yml` keeps job-level `continue-on-error: true` because the workflow is an advisory
drift report for rustdoc coverage and H1 readiness. The failure is still escalated to a tracking
issue, so advisory status no longer means silent rot.

Cost tier: low. Existing advisory behavior is preserved while visibility increases.

## Tool Pin Coherence

The JavaScript OXC toolchain surface is exact-pinned:

- root `package.json` devDependency `oxlint`
- root `package.json` devDependency `oxfmt`
- `packages/oxlint-plugin/package.json` peerDependency `oxlint`

`check:tool-pin-coherence` and `omena-check doctor` now reject non-exact pins and cross-manifest
`oxlint` skew. The risk model is lock-refresh and Dependabot events: CI installs with
`pnpm install --frozen-lockfile`, so identical commits do not drift during ordinary CI runs.

Cost tier: low. This is a static manifest check.

## Workspace Lints Correction

`rust/Cargo.toml` workspace lint tables are active. All 47 crate manifests inherit them using:

```toml
[lints]
workspace = true
```

The stale audit premise used the dotted `lints.workspace = true` spelling and therefore counted the
wrong shape. No Rust lint-policy change is required.

Cost tier: none. This is a recorded correction only.

## Dependabot Churn Policy

Exact pins can increase dependency-update PR volume. The npm Dependabot surface keeps the existing
weekly cadence and cooldown policy, and groups the JavaScript OXC toolchain (`oxlint`, `oxfmt`,
`@oxc/*`, `oxc-*`) into a single update lane. This records the churn mitigation without changing
the update cadence.

Cost tier: low. Grouping reduces skew and PR noise without changing the schedule.

## Nightly Red Evidence Boundary

The chronic nightly red window remains workflow-comment testimony from `nightly-soak.yml`, not a
run-history-derived fact in this report.
