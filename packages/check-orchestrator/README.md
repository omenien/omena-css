# Check Orchestrator

Internal gate inventory and runner for Omena CSS Modules.

This package maintains the typed gate inventory for Omena CSS Modules. It still
mirrors root `package.json` scripts, and it can also load declared gates whose
commands, dependencies, CI tier, and compatibility aliases are modeled directly
in the orchestrator. CI and release verification can route through the
manifest-backed CLI while legacy script names stay valid:

```sh
pnpm omena-check list
pnpm omena-check run core/check
pnpm omena-check bundle rust/release/bundle
pnpm omena-check bundle tsgo/release/bundle
pnpm omena-check bundle release/release/verify
pnpm omena-check plan release/release/verify
pnpm omena-check shards rust/closure-fast --json
pnpm omena-check doctor
pnpm omena-check surface
pnpm omena-check inventory --check
pnpm omena-check affected --base=origin/master
pnpm omena-check affected --evidence --check --base=origin/master
pnpm omena-check probe
```

Root scripts remain the compatibility surface for package-derived gates, but
migrated gates should use declared manifest metadata as their source of truth.
Aggregate root scripts and workflows should depend on canonical `omena-check`
gate IDs instead of chaining legacy `check:*` script names directly. The
orchestrator layer provides stable gate IDs, grouping, bundle introspection,
argument forwarding, execution plans, and doctor checks so workflows do not need
to duplicate every script name.

## Execution model

The manifest assigns one executor to every gate:

- `dependencies` expands scripts made entirely of canonical
  `omena-check run|bundle` calls into the real gate graph. Shared descendants are
  executed once per successful run.
- `direct` runs shell-free Node, Cargo, Git, and Rustup commands without another
  `pnpm run` process. `&&`-only command sequences retain ordering and stop at the
  first failure. Package-derived direct commands retain their npm lifecycle
  environment.
- `package-script` is the compatibility fallback for commands that use shell
  features, inline environment assignments, or mixed executors.

Declared `timeoutMinutes` values are enforced across the complete gate, including
dependency and direct-command sequences. A timeout terminates the process group,
not only the immediate wrapper process.

`--summary` retains at most 1 MiB of output per gate by default and spills the
complete stream to `.omena-ci/check-output/` when that limit is exceeded. Override
the retained budget with `OMENA_CHECK_SUMMARY_MAX_OUTPUT_BYTES`. Summary schema
version 2 records output bytes, truncation, spill paths, timeout state, and timing.
CI uploads both the JSON summaries and overflow logs, including hidden paths.

Closure-fast shards keep expensive query API and query core work on separate
GitHub runners. CI obtains its matrix from `omena-check shards`, so shard names
have one manifest authority rather than a duplicated YAML list. The recorded
per-gate durations are the input for future duration-based rebalancing; cache
hits are intentionally not enabled until gates declare complete file,
environment, and toolchain inputs.

`doctor` also rejects GitHub workflow calls that bypass `omena-check` for
manifest-covered package scripts, non-canonical or unknown `omena-check` targets,
and `bundle` calls pointed at non-bundle gates. It warns on alias chains so
public check names stay flat. Declared gates with a CI tier are also checked
against workflow reachability so a gate cannot claim `closure-fast` or `verify`
coverage while being absent from that workflow tier.

`surface` prints the current gate count, alias-chain count, and largest bundles
by unique leaf dependencies. Use it before broad gate rewrites to identify the
smallest stable surface needed by the next migration.

`CHECKS.md` is generated from the manifest. Update it with
`pnpm omena-check inventory --write` after adding, renaming, or regrouping check
scripts.

## Focused CI feedback

`affected` classifies committed and working-tree changes and recommends the
smallest registered probe profiles that cover the edited product area. Unknown
paths and workflow topology changes fail closed by requiring the complete CI
graph. The final merge-boundary run remains authoritative even when every
focused probe passes.

Run a profile locally when the host supports it:

```sh
pnpm omena-check affected --base=origin/master
pnpm omena-check probe rust-cli
```

When evidence needs a Linux, Windows, or macOS GitHub runner, dispatch the same
profile from the committed `HEAD` through the dedicated scratch ref:

```sh
pnpm ci:probe -- linux-benchmark
pnpm ci:probe -- cross-platform-cli --watch
```

The remote helper does not include uncommitted files and does not wait unless
`--watch` is supplied. It updates `codex/ci-probe`, which does not trigger the
full push workflow, and dispatches the allowlisted `CI Probe` workflow. Group
coherent local commits and run the complete CI graph once at the final boundary
instead of using full `master` pushes as an interactive debugger.

## Evidence affected preview

`affected --evidence` extends the same path plan with the generated scanner
surface and evidence-writer registries. Check mode lists affected scanners,
artifact writers in dependency order, excluded scanners, hand-authored or orphan
artifacts marked `NOT-REFRESHED`, and every non-path input marked
`NOT-PREVIEWABLE`. If the ordinary path plane requires full CI, the evidence
result is always labelled `INSUFFICIENT-ALONE`; it never turns a full-CI decision
into a narrow claim.

The pre-push hook adds `--preview`. It runs only check-mode gates, cheapest
ledger-priced gates first, up to a fixed 60-second estimated budget. Unpriced or
over-budget gates are printed in the skipped list and remain for CI. A selected
gate failure blocks the push. `OMENA_EVIDENCE_SWEEP=0` skips only this preview;
`LEFTHOOK=0` remains the documented whole-hook emergency override.

Writers are opt-in and never run from pre-push:

```sh
pnpm omena-check evidence-surfaces --check
pnpm omena-check evidence-writers --check
pnpm omena-check affected --evidence --write --base=origin/master
```

Write mode invokes generated and self-writing artifacts in declared DAG order
with input digests checked again after each writer exits. Hand-authored and
orphan artifacts are printed as `NOT-REFRESHED` with their procedure or review
disposition. Calendar time, environment, git history, concurrent worktree
mutation, external checkout/network state, built binaries, and toolchain bytes
are explicitly outside diff-preview authority and are never reported green.
The token-shape measurement writer therefore requires
`OMENA_TOKEN_CORPUS_ROOT`, `OMENA_TOKEN_IDENTITY_REACT_TS_CSS`,
`OMENA_TOKEN_IDENTITY_MKN`, and `OMENA_TOKEN_IDENTITY_DOCUSAURUS`; absent inputs
produce a typed `NOT-PREVIEWABLE` refusal instead of replaying stale temporary
paths.
