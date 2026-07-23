# Contributing

## Prerequisites

- Rust `1.94` (workspace MSRV, edition 2024 — pinned in `rust/Cargo.toml`).
- Node `>=22` with pnpm `10` (pinned via `packageManager` in `package.json`);
  run `pnpm install` once at the repo root.
- Checks run through the `pnpm omena-check` orchestrator: every gate has a
  stable id, `pnpm omena-check list` enumerates them, and the generated
  [check inventory](packages/check-orchestrator/CHECKS.md) maps ids to scripts.
  Task recipes below name the smallest relevant target.

## Commit Messages

Use plain imperative commit subjects:

```text
Add parser differential coverage
Tighten transform workspace packaging
Fix source-map segment ordering
```

Keep commit messages understandable without private planning documents. Do not
use internal planning labels, phase names, issue-triage shorthand, or private
catalog identifiers in public history.

## Verification

Run the smallest relevant check for the files you changed, then run the broader
gate before release-oriented changes. Prefer existing `pnpm omena-check` targets
when a target exists for the changed subsystem.

## Task Recipes

These recipes identify the authority, generated artifacts, and smallest useful
checks for common extension work. Do not add a second registry beside the named
authority.

### Add A Product Verb

1. Add the Clap variant in `rust/crates/omena-cli/src/commands.rs`, the matching
   `ProductVerb` variant and spelling, and one direct dispatch handler.
2. Update `rust/crates/omena-cli/verb-census.json` and classify the verb in
   `config-schema-census.json`. Add it to only the persona presets that can run
   the complete command path.
3. Add parser, dispatch, output, and error-path tests. A reserved stub is not a
   shipped product verb.
4. Regenerate the public command reference and inspect the resulting diff.

```bash
pnpm omena-check run rust/omena-cli-verb-census
cargo test --manifest-path rust/Cargo.toml -p omena-cli
pnpm update:docs-reference-surface
pnpm omena-check run docs/reference-surface
```

### Add A Style Intelligence Provider

1. Implement `StyleIntelligenceProvider` in
   `rust/crates/omena-bridge/src/style_intelligence.rs` and register its metadata
   in `BUILT_IN_STYLE_INTELLIGENCE_PROVIDERS`.
2. Feed the provider from parser/source facts. Do not execute user config or add
   a provider-local parser, filesystem walk, or semantic fallback.
3. Declare the real `FactPrecision`, update the production precision census when
   a new source site is introduced, and preserve typed unresolved outcomes.
4. Test class-universe projection, completion, hover, graph binding, and a
   fail-closed unsupported case through the shared snapshot.

```bash
cargo test --manifest-path rust/Cargo.toml -p omena-bridge style_intelligence
pnpm omena-check run rust/omena-fact-precision-census
pnpm omena-check run rust/omena-bridge/boundary
```

### Add An `omena.toml` Key

1. Add the typed field in `rust/crates/omena-cli/src/config/schema.rs`. Public
   TOML keys use the serde camel-case spelling.
2. Wire resolution, overrides, environment policy, and the owning product verb.
   Unknown or unsupported values must fail with a typed user-action error.
3. Update `config-schema-census.json` when table ownership changes and add loader,
   override, and consumer tests.
4. Regenerate the configuration reference; every public TOML fence is executed
   by the docs gate.

```bash
pnpm omena-check run rust/omena-config-schema-census
cargo test --manifest-path rust/Cargo.toml -p omena-cli config
pnpm update:docs-reference-surface
pnpm omena-check run docs/reference-surface
```

## Proposing A New Crate Boundary

Omena product capabilities begin behind the shared query and product egress. A
capability becomes a separate crate only when measured ownership pressure
justifies the additional public API, dependency, build, and release boundary.
Every boundary review provides all four measurements below; a decision without
executable measurements is incomplete.

| Criterion             | Required measurement                                                                                                                                                               | Required response                                                                                                            |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| API surface stability | Count commits touching the candidate boundary from a named base revision and identify the public-surface snapshot or manifest that detects drift.                                  | Explain how accepted API churn will be reviewed and which gate fails on unreviewed drift.                                    |
| Dependency direction  | Derive direct workspace dependencies and consumers from `cargo metadata` or an equivalent source graph. Report cycles and dependencies that point back into a higher-level facade. | Explain why the boundary improves directionality or name the condition that will cause it to be folded.                      |
| Build cost            | Record a reproducible warm command sample and a successful CI job envelope. Warm timings are diagnostic, not portable performance claims.                                          | Explain how the boundary contains rebuild cost or define the threshold that triggers a new review.                           |
| Consumer count        | Derive consumers from Cargo metadata or source imports and record the complete set.                                                                                                | Explain why the number and diversity of consumers warrants the boundary or define the migration needed before consolidation. |

Each review ends in exactly one state: `promote` (introduce the boundary in a
separately reviewed change), `retain` (the measurements justify the current
boundary), or `revisit` (keep the current topology until every named re-review
condition is measurable and satisfied). An unfavorable measurement cannot be
omitted, and `revisit` requires concrete conditions. The review records a
topology decision only; it never performs the topology change itself.

The machine-readable authority is
[`rust/product-surface-boundary-reviews.json`](rust/product-surface-boundary-reviews.json).
Run `pnpm omena-check run rust/product-surface-boundary-reviews` to recompute
deterministic measurements; pass `--measure` to its underlying script when
refreshing local timing samples.

## Generated Documentation

Run `pnpm update:docs-reference-surface` after changing CLI commands, personas,
configuration, SDK workflow models, or LSP capabilities. Commit generated files
with the authority change and verify check inventory remains closed:

```bash
pnpm omena-check run tooling/orchestrator-inventory
pnpm omena-check run docs/reference-surface
```

## Broader Validation

Use the generated [check inventory](packages/check-orchestrator/CHECKS.md) to
select a focused target. Before release-facing work, run the repository and
package gates relevant to the change; maintainers follow the
[release runbook](RELEASING.md).

```bash
pnpm check
pnpm test
pnpm build
```
