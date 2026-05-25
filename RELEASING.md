# Releasing

This document describes the current release procedure.

It is an operations document. It should not contain rollout history or project
planning notes.

## Branches

- `master`: stable releases
- `next`: preview releases

## Version rules

Stable and preview releases both use numeric extension versions.

For the current M5 release chapter, `5.0.0` is already published on the VS Code
Marketplace and Open VSX. The next stable release candidate is `5.1.0`; do not
attempt to publish another `5.0.0` artifact.

Allowed:

- `3.2.0`
- `3.2.1`
- `3.3.0`
- `3.4.0`
- `3.5.0`
- `3.6.0`
- `3.7.0`
- `3.8.0`
- `3.9.0`
- `3.10.0`
- `3.11.0`
- `3.12.0`
- `3.13.0`
- `3.14.0`
- `3.15.0`
- `4.0.0`
- `4.1.0`
- `5.0.0`
- `5.1.0`

Do not use:

- `3.2.0-alpha.1`
- `3.2.0-beta.1`

VS Code Marketplace preview publishing requires:

1. `major.minor.patch` version strings
2. `--pre-release` packaging and publishing for preview builds

A version used for preview must not be reused later for stable.

Reference:

- https://code.visualstudio.com/api/working-with-extensions/publishing-extension#pre-release-extensions

## Channels

Stable:

- branch: `master`
- workflow input: `channel=stable`

Preview:

- branch: `next`
- workflow input: `channel=preview`

Open VSX does not document preview behavior the same way Marketplace does. Use
Marketplace as the primary preview channel. Treat Open VSX preview publishing as
optional.

Reference:

- https://github.com/eclipse-openvsx/openvsx/wiki/Publishing-Extensions

## Pre-release verification

Before a release:

```bash
pnpm install
pnpm release:verify
pnpm check:release-m5-class-value-universe-matrix
pnpm check:plugin-consumer-example
pnpm check:plugin-consumers
pnpm test:extension-host
pnpm check:rust-parser-public-product
pnpm check:rust-lane-bundle
pnpm check:rust-release-bundle
pnpm check:semantic-smoke
pnpm check:release-batch
pnpm check:contract-parity-v2-smoke
pnpm check:contract-parity-v2-golden
pnpm --dir examples exec tsc -p tsconfig.json --noEmit
pnpm --dir examples build
pnpm package
```

## Release claim discipline

Public release text should describe shipped behavior and the evidence that
proves it. Avoid internal milestone labels, planning shorthand, and P-numbering
in README, CHANGELOG, release notes, and Marketplace text.

Use these rules before publishing:

- Map every user-visible claim to a release gate, pushed commit, CI run, or
  packaged artifact check.
- Treat V0 Rust/TypeScript contracts as internal substrate unless a product gate
  exercises them through the shipped extension, CLI, or public crate surface.
- Do not describe substrate as final APIs, completed proofs, or external
  runtime support. In particular, do not claim a Datalog host, egglog binding,
  modal theorem prover, belief-propagation paper result, or safety
  margin unless a later product gate directly proves that behavior.
- For issue #61, release text may mention only the Finding-D class-value-universe
  substrate when the CSS Modules fallback, vanilla-extract recipe, and cva phase
  1 gates are green. Do not close or describe the broader #61 resolver/Sass/
  workspace/paradigm RFC as completed unless it is separately evidenced.
- Automation and testkit surfaces are release-framed only when their fixture
  grammar, schema version, known-failure policy, and failure modes are enforced
  by gates. Unstable automation remains internal.
- Cargo crate versioning stays on the gradual `0.2.x` line for this release
  chapter. Do not publish or describe a Cargo `1.0.0` API-freeze line until the
  corresponding API-freeze evidence exists.

`pnpm check:release-m5-api-freeze-audit` is the release/API-freeze wording
gate for this chapter. It verifies the M5 disposition table, the issue #61
Finding-D scope boundary, Cargo `0.2.x` version policy, the theory-claim
ladder, and the publish path's packaged Rust LSP type-fact protocol gate before
release artifacts or publish commands are treated as release evidence.

`pnpm check:release-m5-class-value-universe-matrix` is the release-facing
fixture matrix for the issue #61 Finding-D slice. It verifies CSS Modules finite
fallback, vanilla-extract recipe, and cva phase 1 class-value universes through
the binder, analysis-cache, query-summary, and selector-projection paths. It
also records the slots axis as reserved/deferred for this release chapter.

VSIX packaging must use `scripts/package-extension-vsix.ts`, not direct
repo-root `vsce package`. VSCE walks the full current working directory before
applying `.vscodeignore`, so a checkout with large ignored Rust build caches can
turn package/list into an unbounded filesystem walk. The staging packager copies
only `package.json`, README/CHANGELOG/LICENSE, `.vscodeignore`, and `dist/`
into a temporary runtime-only directory, runs VSCE there, and writes the VSIX
back to the repo root for the packaged artifact gates. The staged copy must use
the repository `.vscodeignore`; otherwise sourcemaps or dev-only Node server
build output can leak into the publish artifact.

`pnpm release:verify` does:

1. sync `SERVER_VERSION`
2. run `pnpm check:release-m5-api-freeze-audit`
3. run `pnpm check`
4. run `pnpm check:plugin-consumer-example`
5. run `pnpm check:plugin-consumers`
6. run `pnpm check:rust-release-bundle`
7. run `pnpm check:tsgo-release-bundle`
8. run `pnpm test`
9. run `pnpm package`

`pnpm check:semantic-smoke` is the canonical semantic smoke pass. It is not the
release gate yet. It gives one repeatable workspace/checker sanity check before
packaging.

`pnpm check:plugin-consumer-example` verifies the clean repo-local example
workspace under both lint consumers. It is the closest release-facing check for
copy-paste setup viability.

`pnpm check:plugin-consumers` is the current plugin-facing consumer batch gate.
It runs the ESLint and Stylelint smoke consumers together, so user-facing
lint-plugin regressions are exercised before packaging.

The smoke corpus is defined in `scripts/semantic-smoke-corpus.ts`. Treat that
file as the release-facing semantic fixture list. Update it when a new semantic
surface becomes release-relevant.

`pnpm check:contract-parity-v2-smoke` verifies that the canonical engine
contracts can still be assembled across the parity corpus.

`pnpm check:contract-parity-v2-golden` verifies the normalized
`EngineInputV2` / `EngineOutputV2` golden fixtures under
`test/_fixtures/contract-parity-v2/`.

`pnpm check:rust-parser-public-product` is the canonical parser/public-product
gate. It currently runs `pnpm check:rust-parser-lane` plus
`pnpm check:rust-parser-consumer-boundary`, so the parser canonical-candidate,
parser evaluator-candidates, parser canonical-producer, and one bounded
downstream consumer check all stay green together.

`pnpm check:rust-checker-entrance` is the current official checker-canonical
entrance gate. It currently aliases `pnpm check:rust-checker-bounded-lanes`,
which runs the bounded `style-recovery`, `source-missing`, and `style-unused`
checker lanes.
That checker entrance is now included in both the broader Rust lane and the
default stable release gate.

`pnpm check:rust-checker-style-unused-lane` is the third bounded checker lane.
It runs the `style-unused` canonical-candidate, canonical-producer, and
consumer-boundary checks over the Stylelint smoke fixture and is included in
`pnpm check:rust-checker-entrance`.

`pnpm check:rust-checker-expanded-lanes` remains as a compatibility alias for
the same release-enforced checker aggregate.

`pnpm check:rust-checker-promotion-review` is the operator check for that
promotion decision. It validates the current checker-lane gate metadata and
confirms the bounded checker lanes are now inside both `rust-lane-bundle` and
`rust-release-bundle`.

`pnpm check:rust-checker-broader-lane-readiness` locks the broader-lane
promotion criteria for those bounded checker lanes. It currently requires three
bounded lanes, a shared promotion-review command, and a broader target of
`pnpm check:rust-lane-bundle`. The current state is that all three lanes are
promoted into the broader Rust lane and enforced in the release gate.

`pnpm check:rust-checker-real-project-bounded` adds one more promotion-evidence
layer on top of the smoke fixtures. It validates the bounded checker lanes
against a small multi-file real-project-like corpus, so promotion decisions do
not rely on smoke-only evidence.

`pnpm check:rust-checker-promotion-evidence` is the aggregate operator command
for checker-lane promotion evidence. It currently runs promotion review,
broader-lane readiness, and the bounded real-project corpus check.

`pnpm check:rust-checker-release-gate-readiness` locks the release-gate
promotion criteria for those checker lanes. It currently requires broader-lane
promotion evidence, a release target of `pnpm check:rust-release-bundle`, a
shadow soak target of `pnpm check:rust-checker-release-gate-shadow`, and the
same minimum bounded-lane count of `3`. The current state is now
`includedInRustReleaseBundle=true` for all three lanes.

`pnpm check:rust-checker-release-gate-shadow` is the current post-enforcement
observation soak for checker entrance. It runs `pnpm check:rust-release-bundle`,
`pnpm check:rust-checker-entrance`, and `pnpm check:rust-checker-promotion-evidence`
together after the checker lanes have already been enforced in the release bundle.

`.github/workflows/checker-release-gate-shadow.yml` runs the same shadow soak on
every `master` push. It builds the repo artifacts and the `server/engine-core-ts`
and `server/engine-host-node` dist outputs required by
`pnpm check:rust-gate-evidence`, then executes the release bundle components,
checker entrance, and promotion evidence as separate workflow steps so failures
remain attributable after the stable release gate flip.

`pnpm check:ts7-phase-a-readiness` is the current Phase A gate for the TS 7
beta path. `pnpm check:ts7-phase-a-shadow` is the current non-release
shadow path for that same tsgo backend lane. The explicit tsgo-backed
operational commands are:

- `pnpm check:release-batch-tsgo`
- `pnpm check:real-project-corpus-tsgo`
- `pnpm check:lsp-server-smoke-tsgo`

`pnpm check:ts7-phase-a-shadow` runs those two tsgo-backed operational
commands plus the tsgo-backed LSP smoke together.

`pnpm check:ts7-phase-a-stability` is the direct stability check for the two
tsgo-specific risk points that the basic shadow path does not prove by itself:
repeated `EngineInputV2.typeFacts` ordering stability and concurrent
checker-process output stability under `CME_TYPE_FACT_BACKEND=tsgo`. The
unset `CME_TYPE_FACT_BACKEND` default is now `tsgo`, and the explicit
`CME_TYPE_FACT_BACKEND=typescript-current` fallback remains available for
comparison. It runs tsgo through the repo-pinned `@typescript/native-preview`
devDependency and repeats backend smoke under fixed `--checkers` values (`1`, `2`, `4`) via
`CME_TSGO_CHECKERS`.

`pnpm check:ts7-phase-a-tsgo-lane` is the current limited non-release
aggregate for Phase A. It runs the readiness gate, non-release shadow path,
stability check, and the repo build together.

`.github/workflows/ts7-phase-a-shadow.yml` runs the readiness gate plus that
non-release shadow path on every `master` push and on manual dispatch. It now
also builds the repo first and includes the stability check. This workflow is
observational only; it is not part of `pnpm check:rust-release-bundle`.

`pnpm check:ts7-phase-a-shadow-review` is the current operator review command
for that workflow. It reads recent `TS7 Phase A Shadow` history through `gh`
and reports whether the repo has accumulated the current minimum of `3`
successful shadow runs before any broader release-facing judgment.

`pnpm check:ts7-phase-a-decision-ready` is the current Phase A lock point. It
requires both the local limited tsgo lane and the shadow-run threshold to be
green before any broader release-facing judgment.

`pnpm check:ts7-phase-b-protocol-tsgo` is the first bounded protocol-layer
tsgo path for TS 7 beta Phase B. It runs a focused subset of protocol tests
(`lifecycle`, `hover`, `definition`, `diagnostics`, `completion`) under
`CME_TYPE_FACT_BACKEND=tsgo`.

`pnpm check:ts7-phase-b-editing-tsgo` is the second bounded protocol-layer
tsgo path for TS 7 beta Phase B. It runs the editing/reference subset
(`references`, `rename`, `code-actions`) under
`CME_TYPE_FACT_BACKEND=tsgo`.

`pnpm check:ts7-phase-b-build-tsgo` is the current bounded build-mode
tsgo path for TS 7 beta Phase B. It runs
`pnpm exec tsgo -b server/tsconfig.json --checkers 2 --builders 2`.

`pnpm check:ts7-phase-b-workspace-build-tsgo` is the current workspace-level
project-reference tsgo path for TS 7 beta Phase B. It runs
`pnpm exec tsgo -b tsconfig.json --checkers 2 --builders 2`.

`pnpm check:ts7-phase-b-readiness` is the current entry check for Phase B. It
requires `pnpm check:ts7-phase-a-decision-ready` first, then the bounded
protocol tsgo, editing tsgo, server build tsgo, and workspace build tsgo
subsets.

`pnpm check:tsgo-operational-lane` is the current bounded non-release
operational lane for the tsgo backend. It runs the local
`pnpm check:ts7-phase-a-tsgo-lane` plus the bounded Phase B protocol,
editing, server-build, workspace-build, and Phase C edge-readiness tsgo
subsets.

`pnpm check:tsgo-release-bundle` is the current release-shaped tsgo variant.
Today it aliases `pnpm check:tsgo-operational-lane` and is part of
`pnpm release:verify`.

`pnpm check:ts7-phase-c-readiness` is the first TS 7 Phase C edge-readiness
slice. It runs long-lived LSP session edits, multi-root workspace churn,
watched-file invalidation, and source/style staleness checks under
`CME_TYPE_FACT_BACKEND=tsgo`.

`pnpm check:selected-query-boundary` is the current local lock point for the
`3.9` selected-query/editor-path transition. It exercises the protocol subset
for `definition`, `hover`, `completion`, `references`, `rename`, and
`codeLens` after those providers have been routed through `engine-host-node`
helpers instead of directly owning the main core query/rewrite reads inside the
LSP layer. It is a milestone boundary, not a stable release gate.

`pnpm check:rust-selected-query-consumers` is the current local lock point for
the first live Rust selected-query consumer slice. It exercises the explicit
unit/runtime coverage for the opt-in Rust consumer paths currently wired into
source `definition`, source `hover`, source `references`, source `rename`, style `hover`
usage-summary resolution, style `references` location resolution, style
`reference-lens` title/count summary and location resolution, style module
usage / style diagnostics unused-selector resolution, style `rename`
rewrite-safety and direct edit-site resolution, `explain-expression`, source
diagnostics symbol-ref invalid-class analysis, and host-side `engine-query-v2` query-result emission for
`source-expression-resolution`,
`expression-semantics`, and `selector-usage`.
`CME_SELECTED_QUERY_BACKEND=rust-selected-query` is the unified explicit backend
for that consumer slice; the narrower `rust-source-resolution`,
`rust-expression-semantics`, and `rust-selector-usage` values remain available
for isolated debugging. In packaged VSIX runtime, an unset
`CME_SELECTED_QUERY_BACKEND` now selects `rust-selected-query` when the
packaged/prebuilt `engine-shadow-runner` is available. Source checkouts keep the
unset default on `typescript-current` so local `dist/` artifacts do not change
dev/test behavior. Use `CME_SELECTED_QUERY_BACKEND=auto` to explicitly exercise
the Rust-if-packaged-runner-available default in source checkouts.

`pnpm check:rust-selected-query-default-candidate` is the current
default-candidate evidence lane for
the selected-query backend. It first runs
`pnpm check:rust-selected-query-release-default`, which builds the packaged
runner and runs the full protocol suite with `CME_SELECTED_QUERY_BACKEND=auto`.
It then runs the explicit unit/runtime Rust
selected-query consumer slice plus the full protocol suite with the unified Rust
selected-query backend enabled and `CME_ENGINE_SHADOW_RUNNER=prebuilt`. The
warmup avoids first-use `cargo run` compilation contention inside parallel
protocol tests, and the explicit prebuilt mode makes the shadow lane exercise
the warmed runner binary instead of the cargo wrapper. Ad-hoc local runs still
use `cargo run` by default, so they do not accidentally reuse a stale
`rust/target` binary. Prebuilt mode resolves an explicit
`CME_ENGINE_SHADOW_RUNNER_PATH`, a packaged
`dist/bin/<platform>-<arch>/engine-shadow-runner`, or the warmed
`rust/target/debug` runner. It is regression evidence for keeping the packaged
runner matrix safe while `rust-selected-query` is the packaged default. GitHub
Actions runs the same lane in the
`Rust Selected Query Default Candidate` shadow workflow on `master`.

`pnpm check:rust-phase-2-swap-readiness` is the current v4.1 release-candidate
cut-line gate. It first enforces `pnpm check:provider-host-routing-boundary`,
then runs the Rust selected-query default-candidate lane and the checker
release-gate shadow soak. Treat this as candidate evidence for the Phase 2
swap; it is not an omena-semantic V1 freeze or an end-state freeze.

`pnpm build` prepares the current-platform release `engine-shadow-runner` at
`dist/bin/<platform>-<arch>/engine-shadow-runner`. `pnpm package`,
`pnpm release:verify`, `./scripts/publish-extension.sh`, and the CI package job
run `pnpm check:packaged-engine-shadow-runner-matrix` before VSIX packaging so
the release artifact contains executable runner targets. CI and the publish
workflow build Linux, macOS, and Windows runner artifacts, merge them back into
`dist/bin/`, and require all three platform families before packaging. After
VSIX packaging they also run `pnpm check:packaged-selected-query-default`, which
verifies the generated VSIX file set makes packaged runtime choose
`rust-selected-query` by default while excluding checkout-only Rust/source
markers and preserving the required runner matrix. `pnpm package` and
`./scripts/publish-extension.sh` also run
`pnpm check:packaged-omena-lsp-server-type-fact-protocol`, which verifies that
the packaged VSIX can run the Rust `omena-lsp-server` type-fact protocol path
against the packaged `tsgo` binary. The publish script publishes the already
validated VSIX with `vsce publish --packagePath` so Marketplace upload cannot
silently re-run repo-root packaging.

`pnpm check:editor-path-boundary` is the current local lock point for the
editor-path runtime transition after the selected-query cut. It runs
`pnpm check:selected-query-boundary` plus the protocol subset for
`diagnostics`, `scss-diagnostics`, `code-actions`, watched-file invalidation,
workspace-folder changes, and settings reload after those paths have been
routed through `engine-host-node` helpers or runtime aggregates, including
host-side code-action planning. It is a milestone boundary, not a stable
release gate.

`pnpm check:operational-lane` is the current limited non-release default lane.
Today it resolves to the tsgo-backed operational lane.

`.github/workflows/tsgo-operational-shadow.yml` is the observational workflow
for that lane. It builds the repo, runs the tsgo-backed release batch,
real-project corpus, LSP smoke, and the full operational lane as separate
steps, and records repeatable shadow history for the next limited non-release
default judgment.

`pnpm check:operational-shadow-review` is the current limited non-release
default review command. Today it resolves to
`pnpm check:tsgo-operational-shadow-review`, which reads recent
`TSGO Operational Shadow` history through `gh` and reports whether the repo
has accumulated the current minimum of `3` successful shadow runs before any
limited non-release default judgment.

`pnpm check:ts7-decision-ready` is the current top-level judgment gate for the
TS 7 track. It requires `Phase A decision-ready`, `Phase B readiness`, and
`Phase C readiness` before any broader release-facing judgment.

`TS 7 Phase C` is now part of the release-facing tsgo bundle. Its current scope
covers watch/incremental behavior, long-lived LSP session behavior beyond the
one-shot smoke path, and multi-root/workspace-edge cases.

`pnpm check:rust-checker-release-gate-shadow-review` is the current operator
review command for enforcement readiness. It reads recent
`Checker Release Gate Shadow` workflow history through `gh` and reports whether
the repo has accumulated the current minimum of `3` successful shadow runs that
were required before enforcement.

`pnpm check:rust-lane-bundle` is the broader Rust lane gate. It combines the
current semantic producer boundary checks, `pnpm check:rust-parser-public-product`,
and `pnpm check:rust-checker-entrance`.

`pnpm check:rust-release-bundle` is the release-facing Rust gate. It runs the
workspace hygiene pass, the current semantic producer boundary checks,
`pnpm check:rust-parser-public-product`, `pnpm check:rust-checker-entrance`,
and the current `rust-gate-evidence` measurement step.

`pnpm check:rust-split-boundaries` is the current operational check for the two
external Rust split repos. It is not part of the default stable release gate.
Run it when validating split-repo sync, remote-consumer viability, or split
boundary changes.

`V2` is the canonical live contract surface for release validation. Historical
`V1` parity commands remain available only to validate the frozen compatibility
view derived from `V2`.

Frozen V1 baseline commands remain available for historical validation only:

- `pnpm check:contract-parity-v1-smoke`
- `pnpm check:contract-parity-v1-golden`
- `pnpm update:contract-parity-v1-golden`

`pnpm check:release-batch` is the release-facing batch checker gate. It runs the
current `ci` preset against the curated clean corpus in
`scripts/release-batch-corpus.ts`. Use this instead of a repo-wide
`pnpm check:workspace -- . --preset ci` run because `examples/` intentionally
contains negative recovery fixtures that should not block a stable release.

For focused local review, prefer the changed-file presets:

```bash
pnpm check:workspace -- . --preset changed-source --changed-file src/App.tsx
pnpm check:workspace -- . --preset changed-style --changed-file src/Button.module.scss
```

Preset policy:

- `ci` => warning-only `ci-default` bundle
- `changed-source` => `source-missing` bundle with compact text output
- `changed-style` => `style-recovery` + `style-unused` bundles with compact text output

Treat `pnpm check:workspace -- . --preset ci` as the release-facing batch
checker command for full-repo operational review. It exercises the current CLI
preset policy instead of the raw default checker output, but it is not the
stable release gate while intentional negative fixtures live in the repo.

Use `pnpm check:workspace -- --list-bundles` to inspect the current named bundle map.

## Publish workflow

Publishing is done through the `Publish Extension` GitHub Actions workflow.

Inputs:

- `ref`
- `channel`
- `publish_marketplace`
- `publish_openvsx`
- `create_github_release`

The workflow:

1. checks out the requested ref
2. builds Linux, macOS, and Windows `engine-shadow-runner` artifacts
3. installs dependencies
4. runs `./scripts/publish-extension.sh`
5. packages the VSIX with the merged runner matrix
6. publishes to Marketplace and/or Open VSX
7. optionally creates a GitHub release

## Stable release procedure

1. merge the release branch into `master`
2. run `Publish Extension`
3. use:
   - `ref=master`
   - `channel=stable`
   - `publish_marketplace=true`
   - `publish_openvsx=true` or `false`
   - `create_github_release=true`

## Preview release procedure

1. merge the target preview work into `next`
2. update the preview version
3. run `Publish Extension`
4. use:
   - `ref=next`
   - `channel=preview`
   - `publish_marketplace=true`
   - `publish_openvsx=false` or `true`
   - `create_github_release=true`

## Changesets

User-facing pull requests should include a changeset.

PRs that only touch:

- docs
- tests
- CI
- `examples/`

can use `changeset:skip`.

## 4.0 compatibility removals

`cssModules.pathAlias` fallback support has been removed for the 4.0 line.
Use `cssModuleExplainer.pathAlias` instead.

Before publishing 4.0, verify:

- `server/engine-core-ts/src/settings.ts`
- `README.md`
- `package.json` configuration metadata
- changelog / release notes

## Local publish

```bash
pnpm release:publish
```

Environment variables used by the publish script:

- `RELEASE_CHANNEL=stable|preview`
- `PUBLISH_MARKETPLACE=true|false`
- `PUBLISH_OPENVSX=true|false`
- `VSCE_PAT`
- `OVSX_PAT`

The publish script also reads a repo-root `.env` file when present.
