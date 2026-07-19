# Omena CSS Modules

Semantic language features for CSS Modules in VS Code.

The extension is built for projects that use CSS Modules through
`classnames/bind`, `classnames`, `clsx`, or direct `styles.*` access.
Definition, hover, references, rename, diagnostics, and code actions all
resolve through the same semantic pipeline.

```tsx
import classNames from "classnames/bind";
import styles from "./Button.module.scss";

const cx = classNames.bind(styles);

export function Button({ active, size }: { active: boolean; size: "sm" | "lg" }) {
  return <button className={cx("button", { active }, size)}>Save</button>;
}
```

## Features

- Source-side language features
  - Go to Definition and Hover from `cx(...)`, `styles.foo`, and `styles["foo-bar"]`
  - Completion inside `cx(`, `classnames(`, and `clsx(` calls
  - Diagnostics for unknown classes and missing module imports
  - Quick fixes to replace a misspelled class, add a missing selector, or create a missing module file
- Style-side language features
  - Hover, Find References, and CodeLens from `.module.css`, `.module.scss`, and `.module.less`
  - Rename across style files and source call sites
  - Unused selector diagnostics
  - `composes` token definition/references/hover inside style modules
  - Diagnostics and quick fixes for unresolved composed modules
  - Same-file `@keyframes` hover/definition/references plus missing-target recovery
  - Local and imported `@value` definition/references/diagnostics plus missing-target recovery
- Resolution behavior
  - `classnames/bind` bindings, multiple bindings per file, and function-local bindings
  - `classnames` / `clsx` calls that use `styles.foo` or `styles["foo-bar"]`
  - Template literals and symbol references with local flow analysis and TypeScript union fallback
  - `css-loader`-compatible class name transform modes
  - Multi-root workspaces with resource-scoped transform and path-alias settings
- The `Omena CSS Modules` status bar item shows the language server lifecycle and opens the leveled
  `Omena CSS Modules` Output panel for filtered runtime logs.

## Supported patterns

- `cx("button")`
- `cx("button", { active, disabled })`
- ``cx(`button-${variant}`)``
- `cx(size)` where `size` resolves from local control flow or string-literal unions
- `classnames(styles.button, flag && styles.active)`
- `clsx(styles.button, { [styles.active]: flag })`
- `styles.button`
- `styles["button-primary"]`

CSS, SCSS, and Less modules are supported. The extension activates for
TypeScript, TSX, JavaScript, JSX, CSS, SCSS, and Less files.

## Install

Install from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=omena.omena-css)
or build from source.

```bash
pnpm install
pnpm package
code --install-extension omena-css-*.vsix
```

## Performance

The public benchmark surface is documented in [docs/performance.md](docs/performance.md).
External Sass and `omena.lock` compatibility is in [docs/sass-compat.md](docs/sass-compat.md);
plan-first codemods are in [docs/migrate-verb.md](docs/migrate-verb.md).
It covers Rust parser/semantic/abstract-value micro-benchmarks and an LSP
macro-benchmark for hover, definition, completion, references, and event-loop
latency.

## Current Release Frame

The current release ships the Rust stable core for the compiler-grade CSS Modules
semantic engine. The packaged extension defaults to the Rust `omena-lsp-server`,
the Rust selected-query runtime, and the bundled `tsgo` type-fact path; the VS
Code host remains a thin client responsible for process orchestration, UI
commands, and file watching.

Release notes should describe shipped editor/runtime behavior and the gates
that prove it. Lower-level V0 contract substrates are kept internal unless a
release gate maps them to user-visible behavior; they are not claims of final
APIs, completed proofs, or external runtime support by themselves.

## Configuration

All settings live under the `omena.*` namespace.

### CLI persona presets

The Rust CLI can load a built-in starting point through the same `omena.toml`
configuration resolver used for local files, environment interpolation, and
overrides. Select a preset with `extends`; settings in the local file override
the preset defaults.

```toml
extends = "omena:workspace-maintenance"

[lint]
profile = "strict"
```

Available preset ids are `workspace-maintenance`, `design-system-governance`,
`build-integration`, `migration-safety`, `assurance-gates`, and
`semantic-research`. Presets contain only directly wired CLI verbs. Their
manifest also records capabilities that remain unavailable instead of silently
implying support.

### Core settings

| Setting                            | Default     | Description                                                             |
| ---------------------------------- | ----------- | ----------------------------------------------------------------------- |
| `omena.features.definition`        | `true`      | Enable Go to Definition.                                                |
| `omena.features.hover`             | `true`      | Enable Hover.                                                           |
| `omena.features.completion`        | `true`      | Enable Completion.                                                      |
| `omena.features.references`        | `true`      | Enable Find References and CodeLens.                                    |
| `omena.features.rename`            | `true`      | Enable Rename.                                                          |
| `omena.diagnostics.severity`       | `"warning"` | Severity for unknown-class diagnostics.                                 |
| `omena.diagnostics.unusedSelector` | `true`      | Show unused selector hints in style modules.                            |
| `omena.diagnostics.missingModule`  | `true`      | Warn when a CSS Module import cannot be resolved.                       |
| `omena.hover.maxCandidates`        | `10`        | Maximum dynamic candidates shown in hover.                              |
| `omena.typeFactBackend`            | `"tsgo"`    | Type-fact backend: bundled tsgo, workspace tsgo, or current TypeScript. |
| `omena.scss.classnameTransform`    | `"asIs"`    | Mirror of `css-loader` `modules.localsConvention`.                      |

### Class name transform

For a selector `.btn-primary`:

| Mode            | Exposed keys                | Notes                           |
| --------------- | --------------------------- | ------------------------------- |
| `asIs`          | `btn-primary`               | Original selector only.         |
| `camelCase`     | `btn-primary`, `btnPrimary` | Both forms resolve.             |
| `camelCaseOnly` | `btnPrimary`                | Alias only. Rename is rejected. |
| `dashes`        | `btn-primary`, `btnPrimary` | Dashes become word boundaries.  |
| `dashesOnly`    | `btnPrimary`                | Alias only. Rename is rejected. |

Alias-only modes keep navigation and references, but rename is blocked
because the reverse mapping back to the original selector is lossy.

### Path alias resolution

The extension resolves non-relative CSS Module imports from:

- `compilerOptions.paths` in the workspace `tsconfig.json` or `jsconfig.json`
- native `omena.pathAlias`

This allows imports such as:

```ts
import styles from "@/components/Button.module.scss";
import theme from "@styles/theme.module.scss";
```

Configure extension-specific aliases with `omena.pathAlias`:

```jsonc
"omena.pathAlias": {
  "@styles": "src/styles"
}
```

## Architecture

The runtime follows one semantic pipeline:

```text
source/style text
  -> HIR documents
  -> source binding
  -> abstract class-value analysis
  -> read models
  -> LSP providers
```

Current structure:

```text
server/
├── engine-core-ts/      # semantic core, contracts, checker core, abstract value
├── engine-host-node/    # workspace/runtime hosting, batch checker host, parity assembly
├── lsp-server/          # generic LSP transport, providers, handler wiring
└── checker-cli/         # batch checker CLI surface
```

At a high level:

- HIR preserves document facts from source and style files.
- Binder resolves source-side names such as `cx`, `styles`, imports, locals, and shadowing.
- The abstract-value layer models dynamic class expressions such as flow branches, unions, and template prefixes.
- Semantic storage keeps workspace references, dependency lookups, and style-to-style relationships such as `composes`.
- `engine-host-node` query helpers turn low-level semantic state into stable summaries that providers consume.
- Providers adapt those summaries to LSP features such as hover, definition, references, diagnostics, and rename.

The important constraint is that providers do not recompute semantic meaning on
their own. They read the shared pipeline through the host query boundary rather
than importing core query or rewrite internals directly.

Contract status:

- `V2` is the canonical live contract surface.
- `V1` remains available only as a historical compatibility view derived from `V2`.

For a fuller design explanation, see [ARCHITECTURE.md](./ARCHITECTURE.md).

## Development

Requirements:

- Node.js `>= 22`
- `pnpm@10`

Common commands:

```bash
pnpm install
pnpm check
pnpm check:semantic-smoke
pnpm check:lsp-server-smoke
pnpm check:rust-gate-evidence
pnpm check:rust-checker-style-recovery-canonical-candidate
pnpm check:plugin-consumer-example
pnpm check:plugin-consumers
pnpm check:eslint-plugin-smoke
pnpm check:stylelint-plugin-smoke
pnpm check:release-batch
pnpm check:contract-parity-v2-smoke
pnpm check:contract-parity-v2-golden
pnpm test
pnpm test:bench
pnpm build
pnpm package
```

Batch checker:

```bash
pnpm check:semantic-smoke
pnpm check:release-batch
pnpm check:workspace -- . --preset changed-style --changed-file src/Button.module.scss
pnpm check:workspace -- . --preset changed-source --changed-file src/App.tsx
pnpm check:workspace -- . --preset ci
pnpm check:workspace -- --list-bundles
pnpm check:workspace -- . --include-bundle source-missing --summary
pnpm check:workspace -- . --preset changed-style --changed-file src/Button.module.scss --compact
pnpm check:workspace -- . --format json --fail-on none
pnpm explain:expression -- src/App.tsx:12:24
pnpm explain:expression -- src/App.tsx:12:24 --json
```

Current checker policy:

- `@keyframes` validation is same-file only in the current first pass
- `@value` validation covers local declarations and imported bindings between style modules
- named bundles group common finding families such as `ci-default`, `source-missing`, `style-recovery`, and `style-unused`
- presets also apply default bundle policy unless explicit include flags are provided
  - `ci` => `ci-default`
  - `changed-style` => `style-recovery`, `style-unused`
  - `changed-source` => `source-missing`
- `changed-style` and `changed-source` presets use compact text output by default
- `pnpm check:semantic-smoke` is the canonical repo-local smoke command
- `pnpm check:lsp-server-smoke` spawns the built `lsp-server` over stdio and verifies hover/definition through a generic protocol client
- `pnpm check:selected-query-boundary` is the current selected-query compatibility lock point
  - it runs the editor-facing protocol subset for `definition`, `hover`, `completion`, `references`, `rename`, and `codeLens`
  - these providers now route their main selected-query and source/style rewrite reads through `engine-host-node` helpers instead of directly owning the core query/rewrite calls in the LSP layer
- `pnpm check:rust-selected-query-consumers` is the current local lock point for the first live Rust selected-query consumer slice
  - it runs the explicit unit/runtime coverage for the current opt-in Rust consumers: source `definition`, source `hover`, source `references`, source `rename`, style `hover` usage-summary resolution, style `references` location resolution, style `reference-lens` title/count summary and location resolution, style module usage / style diagnostics unused-selector resolution, style `rename` rewrite-safety and direct edit-site resolution, `explain-expression`, source diagnostics symbol-ref invalid-class analysis, and host-side `engine-query-v2` query-result emission for `source-expression-resolution`, `expression-semantics`, and `selector-usage`
  - `OMENA_SELECTED_QUERY_BACKEND=rust-selected-query` is the unified explicit backend for that slice; the narrower `rust-source-resolution`, `rust-expression-semantics`, and `rust-selector-usage` values remain available for isolated debugging
  - in packaged VSIX runtime, an unset `OMENA_SELECTED_QUERY_BACKEND` now selects `rust-selected-query` when the packaged/prebuilt `engine-shadow-runner` is available; source checkouts keep the unset default on `typescript-current` so local `dist/` artifacts do not change dev/test behavior
  - `OMENA_SELECTED_QUERY_BACKEND=auto` explicitly exercises the same Rust-if-packaged-runner-available selection in source checkouts
- `pnpm check:rust-selected-query-default-candidate` is the current default-candidate evidence lane for `OMENA_SELECTED_QUERY_BACKEND=rust-selected-query`
  - it first runs `pnpm check:rust-selected-query-release-default`, which builds the packaged runner and runs the full protocol suite with `OMENA_SELECTED_QUERY_BACKEND=auto`
  - it then warms `engine-shadow-runner`, runs the live Rust selected-query unit consumer slice, and repeats the full protocol suite with the unified Rust selected-query backend explicitly enabled and `OMENA_ENGINE_SHADOW_RUNNER=prebuilt`
  - prebuilt runner mode is explicit so ad-hoc local runs still use `cargo run` and do not accidentally reuse a stale `rust/target` binary
  - prebuilt mode can resolve an explicit `OMENA_ENGINE_SHADOW_RUNNER_PATH`, a packaged `dist/bin/<platform>-<arch>/engine-shadow-runner`, or the warmed `rust/target/debug` runner
  - it is regression evidence for keeping the packaged runner matrix safe while `rust-selected-query` is the packaged default
  - GitHub Actions runs the same lane in the `Rust Selected Query Default Candidate` shadow workflow on `master`
- `pnpm check:rust-phase-2-swap-readiness` remains the historical Phase 2 swap evidence gate
  - it runs `pnpm check:provider-host-routing-boundary`, `pnpm check:rust-omena-lsp-server-lane`, `pnpm check:rust-selected-query-default-candidate`, and `pnpm check:rust-checker-release-gate-shadow`
  - the v5 release gate is `pnpm release:verify`, which includes the broader Rust release bundle, tsgo release bundle, plugin consumers, tests, and VSIX packaging checks
- `pnpm build` now prepares the current-platform release `engine-shadow-runner` and default-candidate `omena-lsp-server` binaries at `dist/bin/<platform>-<arch>/`
- `pnpm check:packaged-engine-shadow-runner-matrix` verifies packaged runner targets before VSIX packaging; CI and publish require Linux, macOS, and Windows runner artifacts
- `pnpm check:packaged-selected-query-default` verifies the generated VSIX file set still makes packaged runtime choose `rust-selected-query` and `omena-lsp-server` by default, excludes checkout-only Rust/source markers, and preserves the required runner matrix
- `pnpm check:editor-path-boundary` is the current editor-path runtime lock point
  - it runs `pnpm check:selected-query-boundary` plus the protocol subset for `diagnostics`, `scss-diagnostics`, `code-actions`, watched-file invalidation, workspace-folder changes, and settings reload
  - those paths now route diagnostics/checker entry, code-action planning, session bootstrap, workspace-folder mutation, watched-file invalidation, and settings-reload orchestration through `engine-host-node` helpers or aggregates instead of owning the runtime policy directly in the LSP layer
- `pnpm check:provider-host-routing-boundary` statically prevents LSP providers from importing core query, semantic graph, indexing, or TypeScript resolver internals directly
- `pnpm check:plugin-consumer-example` runs the clean repo-local lint-consumer example workspace under both ESLint and Stylelint
- `pnpm check:plugin-consumers` runs the current ESLint and Stylelint consumer smokes together
- `pnpm check:eslint-plugin-smoke` runs the ESLint consumer against JSX fixtures and asserts that source-side semantic findings are reported as ESLint diagnostics
  - current config split: aggregate `recommended`, granular `focused`, optional `dynamicMoat`
- `pnpm check:stylelint-plugin-smoke` runs the first Stylelint consumer against a CSS Modules fixture and asserts that unused selectors are reported as Stylelint diagnostics
- `pnpm check:rust-gate-evidence` records wall-clock timings for the current second/third-consumer proof set so Rust gate discussions use measured repo-local workloads instead of guesses
  - current baseline variant: `typescript-current`
  - future variants can be selected with `--variant <label>`
  - repeated runs can be summarized with `--repeat <n>`
  - current comparison slot: `tsgo` on `check:backend-typecheck-smoke`
- `pnpm check:backend-typecheck-smoke` runs a small multi-case corpus (`template-literals`, `path-alias`, `flow-relations`) for the selected typecheck backend
  - the unset `OMENA_TYPE_FACT_BACKEND` default is now `tsgo`, which activates a host-side tsgo probe before delegating symbol resolution to the current TS resolver
  - packaged extension runtime uses the bundled `dist/bin/<platform>-<arch>/tsgo` probe; `omena.typeFactBackend=tsgo-workspace` preserves the old workspace `pnpm exec tsgo` mode for power users
  - `omena.typeFactBackend=typescript-current` or `OMENA_TYPE_FACT_BACKEND=typescript-current` remains available as the explicit current-TypeScript fallback
- `OMENA_TYPE_FACT_BACKEND=tsgo pnpm check:release-batch` and `pnpm check:real-project-corpus` now exercise the checker path through the same host-side tsgo probe
- `pnpm check:type-fact-backend-parity` compares canonical `EngineInputV2.typeFacts` across `typescript-current` and `tsgo` on the backend smoke corpus
- `pnpm check:ts7-phase-a-readiness` is the current aggregate Phase A gate for the TS 7 beta path
  - it runs backend typecheck smoke, type-fact backend parity, and `rust-gate-evidence -- --variant tsgo --repeat 1 --json`
- `pnpm check:release-batch-tsgo` is the current tsgo-backed variant of the release-batch operational path
- `pnpm check:real-project-corpus-tsgo` is the current tsgo-backed variant of the real-project operational path
- `pnpm check:lsp-server-smoke-tsgo` is the current tsgo-backed variant of the stdio LSP smoke path
- `pnpm check:ts7-phase-a-shadow` is the current non-release shadow path for TS 7 beta Phase A
  - it runs the tsgo-backed release-batch, real-project-corpus, and LSP smoke commands together
- `pnpm check:ts7-phase-a-stability` directly checks the two tsgo-specific risk points that the basic shadow path does not prove by itself
  - repeated `EngineInputV2.typeFacts` snapshots from `tsgo` must stay byte-stable across runs
  - concurrent `release-batch` and `real-project-corpus` invocations under `OMENA_TYPE_FACT_BACKEND=tsgo` must keep identical outputs across rounds
  - tsgo probe calls run through the repo-pinned `@typescript/native-preview` binary in source checkouts and the packaged `dist/bin/<platform>-<arch>/tsgo` binary in VSIX runtime; the stability matrix now also repeats backend smoke under fixed `--checkers` values (`1`, `2`, `4`) via `OMENA_TSGO_CHECKERS`
- `pnpm check:ts7-phase-a-tsgo-lane` is the current limited non-release aggregate for TS 7 beta Phase A
  - it builds the repo, then runs readiness, shadow, and stability together
- `pnpm check:ts7-phase-a-shadow-review` reviews recent `TS7 Phase A Shadow` workflow history through `gh`
  - today it reports whether the repo has accumulated the current minimum of `3` successful shadow runs before any broader release-facing judgment
- `pnpm check:ts7-phase-a-decision-ready` is the current lock point for Phase A
  - it requires both the local tsgo lane and the successful-shadow-run threshold
- `pnpm check:ts7-phase-b-protocol-tsgo` is the first bounded protocol-layer tsgo path for TS 7 beta Phase B
  - it runs `lifecycle`, `hover`, `definition`, `diagnostics`, and `completion` protocol tests under `OMENA_TYPE_FACT_BACKEND=tsgo`
- `pnpm check:ts7-phase-b-editing-tsgo` is the second bounded protocol-layer tsgo path for TS 7 beta Phase B
  - it runs `references`, `rename`, and `code-actions` protocol tests under `OMENA_TYPE_FACT_BACKEND=tsgo`
- `pnpm check:ts7-phase-b-build-tsgo` is the current bounded build-mode tsgo path for TS 7 beta Phase B
  - it runs `pnpm exec tsgo -b server/tsconfig.json --checkers 2 --builders 2`
- `pnpm check:ts7-phase-b-workspace-build-tsgo` is the current workspace-level project-reference tsgo path for TS 7 beta Phase B
  - it runs `pnpm exec tsgo -b tsconfig.json --checkers 2 --builders 2`
- `pnpm check:ts7-phase-b-readiness` is the current entry check for Phase B
  - it requires `pnpm check:ts7-phase-a-decision-ready` first, then the bounded protocol, editing, server-build, and workspace-build tsgo subsets
- `pnpm check:tsgo-operational-lane` is the current bounded non-release operational lane for the tsgo backend
  - it runs the local `ts7-phase-a-tsgo-lane` plus the bounded Phase B protocol, editing, server-build, workspace-build, and Phase C edge-readiness tsgo subsets
- `pnpm check:tsgo-release-bundle` is the release-shaped tsgo variant and is part of `pnpm release:verify`
- `pnpm check:ts7-phase-c-readiness` is the first TS 7 Phase C edge-readiness slice
  - it runs long-lived LSP session edits, multi-root workspace churn, watched-file invalidation, and source/style staleness checks under `OMENA_TYPE_FACT_BACKEND=tsgo`
- `pnpm check:operational-lane` is the current limited non-release default lane
  - today it resolves to the tsgo-backed operational lane
- `pnpm check:operational-shadow-review` is the current limited non-release default review command
  - today it resolves to `pnpm check:tsgo-operational-shadow-review`
- `pnpm check:tsgo-operational-shadow-review` reviews recent `TSGO Operational Shadow` workflow history through `gh`
  - today it reports whether the repo has accumulated the current minimum of `3` successful shadow runs before any limited non-release default judgment
- `pnpm check:ts7-decision-ready` is the current top-level judgment gate for the TS 7 track
  - it requires `Phase A decision-ready`, `Phase B readiness`, and `Phase C readiness`
- `.github/workflows/tsgo-operational-shadow.yml` is the observational workflow for that bounded operational lane
  - it builds the repo, runs the tsgo-backed release batch, real-project corpus, LSP smoke, and full operational lane, and records repeatable shadow history for the next default-candidate judgment
- `.github/workflows/ts7-phase-a-shadow.yml` builds the repo, then runs the Phase A readiness gate, non-release shadow path, and stability check on every `master` push and on manual dispatch
- `pnpm check:rust-parser-scaffold` exercises the current product parser scaffold crate, `rust/crates/omena-parser`
- `pnpm check:rust-parser-git-consumer` verifies that the split parser repo can be consumed as a remote git dependency by the repo-stored standalone fixture at `rust/external-consumers/engine-style-parser-git-consumer`
- `pnpm check:rust-parser-split-boundary` verifies the full parser split boundary: parser public-product validation inside the monorepo plus remote git-consumer validation against `omena-engine-style-parser`
- `pnpm check:rust-input-producers-git-consumer` verifies that the split input-producers repo can be consumed as a remote git dependency by the repo-stored standalone fixture at `rust/external-consumers/engine-input-producers-git-consumer`
- `pnpm check:rust-input-producers-split-boundary` verifies the full input-producers split boundary: monorepo producer-boundary validation plus remote git-consumer validation against `omena-engine-input-producers`
- `pnpm check:rust-split-boundaries` runs the current Rust split-boundary checks together
- `pnpm check:rust-split-publish-readiness` verifies the published split crate set against crates.io metadata, registry dependency contracts, and `cargo publish --dry-run`
- `pnpm check:rust-split-consumer-pins` verifies that the repo-stored split consumer fixtures are pinned to the current `main` commit of each split repo
- `pnpm update:rust-split-consumer-pins` refreshes those fixture refs plus lockfiles when the split repos advance
- The current external Rust split boundaries are:
  - `omenien/omena-engine-input-producers`
  - `omenien/omena-engine-style-parser`
  - `omenien/omena-semantic`
  - `omenien/omena-abstract-value`
  - `omenien/omena-resolver`
  - `omenien/omena-bridge`
  - `omenien/omena-query`
  - keep them pinned behind repeatable remote-consumer checks before any new rename or public packaging move
- `pnpm check:rust-parser-parity-lite` compares the `omena-parser` scaffold against the current TS style parser on a bounded shared fixture set
- `pnpm check:rust-parser-css-modules-intermediate` compares the Rust parser CSS Modules intermediate facts against the current TS style HIR on a bounded shared fixture set
- `pnpm check:rust-parser-index-producer` remains as a compatibility alias for the same intermediate producer check
- `pnpm check:rust-parser-canonical-candidate` validates the versioned parser canonical-candidate bundle over the current parity-lite + CSS Modules intermediate artifacts
- `pnpm check:rust-parser-evaluator-candidates` validates the selector-level parser evaluator-candidate artifact against the current TS style HIR
- `pnpm check:rust-parser-canonical-producer` validates the parser canonical-producer signal over that canonical-candidate bundle and current gate placement
- `pnpm check:rust-parser-lane` runs the current parser lane bundle: scaffold tests, parity-lite, CSS Modules intermediate producer, parser canonical-candidate, parser evaluator-candidates, and parser canonical-producer
- `pnpm check:rust-parser-consumer-boundary` consumes the parser canonical-producer output into a bounded downstream-style summary and compares that against the current TS style HIR
- `pnpm check:rust-parser-public-product` is the canonical parser/public-product gate and currently runs the parser lane bundle plus that consumer-boundary check
- `pnpm check:rust-checker-style-recovery-canonical-candidate` is the current bounded checker-canonical entrance check; it compares the `style-recovery` checker subset against a versioned Rust shadow canonical-candidate bundle
- `pnpm check:rust-checker-style-recovery-canonical-producer` validates the matching checker canonical-producer signal for that same bounded `style-recovery` subset
- `pnpm check:rust-checker-style-recovery-consumer-boundary` validates the opt-in `checker-cli` Rust consumer path for that same bounded subset
- `pnpm check:rust-checker-style-recovery-lane` runs the full bounded checker entrance lane: canonical-candidate, canonical-producer, and opt-in consumer-boundary consistency
- `pnpm check:rust-checker-source-missing-canonical-candidate` is the matching bounded entrance check for the `source-missing` checker subset
- `pnpm check:rust-checker-source-missing-canonical-producer` validates the matching checker canonical-producer signal for that bounded `source-missing` subset
- `pnpm check:rust-checker-source-missing-consumer-boundary` validates the opt-in `checker-cli` Rust consumer path for that bounded source subset
- `pnpm check:rust-checker-source-missing-lane` runs the full bounded source-side checker lane: canonical-candidate, canonical-producer, and opt-in consumer-boundary consistency
- `pnpm check:rust-checker-style-unused-canonical-candidate` is the matching bounded entrance check for the `style-unused` subset
- `pnpm check:rust-checker-style-unused-canonical-producer` validates the matching checker canonical-producer signal for that bounded `style-unused` subset
- `pnpm check:rust-checker-style-unused-consumer-boundary` validates the opt-in `checker-cli` Rust consumer path for that bounded unused-selector subset
- `pnpm check:rust-checker-style-unused-lane` runs the full bounded unused-selector checker lane: canonical-candidate, canonical-producer, and opt-in consumer-boundary consistency
- `pnpm check:rust-checker-bounded-lanes` is the current aggregate entry for release-enforced checker-canonical lanes; today it runs `style-recovery`, `source-missing`, and `style-unused`
- `pnpm check:rust-checker-entrance` is the official checker-canonical entrance gate; it currently aliases `pnpm check:rust-checker-bounded-lanes`
- `pnpm check:rust-checker-promotion-review` validates the current promotion stance for bounded checker lanes; today it confirms all three checker lanes are inside the broader Rust lane and the release gate
- `pnpm check:rust-checker-broader-lane-readiness` locks the current broader-lane promotion criteria for those bounded lanes; today it requires three bounded lanes, a promotion-review command, and a broader target of `pnpm check:rust-lane-bundle`, with all three lanes promoted into the broader Rust lane and the release gate
- `pnpm check:rust-checker-real-project-bounded` validates all three bounded checker lanes against a small multi-file real-project-like corpus instead of smoke-only fixtures
- `pnpm check:rust-checker-promotion-evidence` runs the full bounded checker promotion evidence set: promotion review, broader-lane readiness, and real-project bounded corpus validation
- `pnpm check:rust-checker-release-gate-readiness` locks the release-gate promotion criteria for the current checker lanes; today it requires the broader-lane promotion evidence to exist, a release target of `pnpm check:rust-release-bundle`, a shadow soak target of `pnpm check:rust-checker-release-gate-shadow`, and a minimum bounded-lane count of `3`, with all three lanes now enforced in the release gate
- `pnpm check:rust-checker-release-gate-shadow` is now a post-enforcement observation soak for checker entrance; it runs the release-facing Rust bundle, the checker entrance gate, and the checker promotion evidence together
- `pnpm check:rust-checker-release-gate-shadow-review` reviews the current GitHub Actions shadow history for that workflow; today it reports whether the repo has accumulated the `3` successful shadow runs that were required before the enforcement flip
- `.github/workflows/checker-release-gate-shadow.yml` now runs that same post-enforcement observation soak on every `master` push, builds the repo artifacts plus `server/engine-core-ts` and `server/engine-host-node` dist needed by `rust-gate-evidence`, and executes the release bundle components, checker entrance, and promotion evidence as separate steps so failures are attributable from the workflow UI
- Rust shadow validation now covers:
  - input summaries: `pnpm check:rust-type-fact-compare`, `pnpm check:rust-query-plan-compare`, `pnpm check:rust-expression-domain-compare`
  - input-only candidates: `pnpm check:rust-expression-domain-candidates`
  - expression-domain evaluator candidates: `pnpm check:rust-expression-domain-evaluator-candidates`
  - query skeletons: `pnpm check:rust-*-query-fragments`
  - match fragments: `pnpm check:rust-expression-semantics-match-fragments`, `pnpm check:rust-source-resolution-match-fragments`
  - output-like candidates: `pnpm check:rust-expression-semantics-candidates`, `pnpm check:rust-source-resolution-candidates`
  - evaluator candidates: `pnpm check:rust-expression-semantics-evaluator-candidates`, `pnpm check:rust-source-resolution-evaluator-candidates`
  - canonical-candidate bundles: `pnpm check:rust-expression-domain-canonical-candidate`, `pnpm check:rust-expression-semantics-canonical-candidate`, `pnpm check:rust-source-resolution-canonical-candidate`
  - canonical-producer signals: `pnpm check:rust-expression-domain-canonical-producer`, `pnpm check:rust-expression-semantics-canonical-producer`, `pnpm check:rust-source-resolution-canonical-producer`
  - consolidated source-side lane: `pnpm check:rust-source-side-canonical-candidate`, `pnpm check:rust-source-side-evaluator-candidates`, `pnpm check:rust-source-side-canonical-producer`
  - consolidated semantic lane: `pnpm check:rust-semantic-canonical-candidate`, `pnpm check:rust-semantic-evaluator-candidates`, `pnpm check:rust-semantic-canonical-producer`
  - aggregated producer-boundary checks: `pnpm check:rust-source-side-lane`, `pnpm check:rust-semantic-lane`, `pnpm check:rust-producer-boundary`
  - parser canonical boundary: `pnpm check:rust-parser-canonical-candidate`, `pnpm check:rust-parser-evaluator-candidates`, `pnpm check:rust-parser-canonical-producer`
  - parser/public-product gate: `pnpm check:rust-parser-consumer-boundary`, `pnpm check:rust-parser-public-product`
  - bounded checker entrance: `pnpm check:rust-checker-style-recovery-canonical-candidate`
  - bounded checker producer signal: `pnpm check:rust-checker-style-recovery-canonical-producer`
  - bounded checker consumer path: `pnpm check:rust-checker-style-recovery-consumer-boundary`
  - bounded checker lane: `pnpm check:rust-checker-style-recovery-lane`
  - bounded source-side checker entrance: `pnpm check:rust-checker-source-missing-canonical-candidate`
  - bounded source-side checker producer signal: `pnpm check:rust-checker-source-missing-canonical-producer`
  - bounded source-side checker consumer path: `pnpm check:rust-checker-source-missing-consumer-boundary`
  - bounded source-side checker lane: `pnpm check:rust-checker-source-missing-lane`
  - bounded style-unused checker entrance: `pnpm check:rust-checker-style-unused-canonical-candidate`
  - bounded style-unused checker producer signal: `pnpm check:rust-checker-style-unused-canonical-producer`
  - bounded style-unused checker consumer path: `pnpm check:rust-checker-style-unused-consumer-boundary`
  - bounded style-unused checker lane: `pnpm check:rust-checker-style-unused-lane`
  - bounded checker lane aggregate: `pnpm check:rust-checker-bounded-lanes`
  - official checker entrance gate: `pnpm check:rust-checker-entrance`
  - checker lane promotion review: `pnpm check:rust-checker-promotion-review`
  - broader checker lane readiness: `pnpm check:rust-checker-broader-lane-readiness`
  - real-project bounded checker evidence: `pnpm check:rust-checker-real-project-bounded`
  - checker promotion evidence bundle: `pnpm check:rust-checker-promotion-evidence`
  - checker release-gate readiness: `pnpm check:rust-checker-release-gate-readiness`
  - checker release-gate shadow soak: `pnpm check:rust-checker-release-gate-shadow`
  - checker release-gate shadow review: `pnpm check:rust-checker-release-gate-shadow-review`
  - broader Rust lane bundle: `pnpm check:rust-lane-bundle`
  - release-facing Rust bundle: `pnpm check:rust-release-bundle`
  - full snapshot parity: `pnpm check:rust-shadow-compare`
- Current release framing is the Rust stable core on top of the Rust-backed semantic baseline:
  - `expression-semantics` and `source-resolution` still carry family-level canonical-producer signals and a shared top-level source-side lane
  - `expression-domain` carries input-only canonical artifacts plus type-fact-backed evaluator-candidate coverage on the Rust shadow path
  - a top-level `semantic` lane now consolidates `source-side + expression-domain` into one canonical-candidate / evaluator-candidate / canonical-producer path
  - `omena-parser` now owns the canonical parser/public-product gate, parser canonical-candidate bundle, parser evaluator-candidates, parser canonical-producer signal, bounded CSS Modules intermediate producer surface, and downstream consumer-boundary check over that producer output; legacy `engine-style-parser` is confined to oracle, benchmark, and split-compatibility paths
  - the ESLint and Stylelint plugin consumers now form a first plugin-facing batch with focused rule surfaces, aggregate configs, a clean example workspace, and release-facing consumer gates
  - the bounded checker entrance (`style-recovery` + `source-missing` + `style-unused`) is now enforced in `pnpm check:rust-release-bundle`
  - packaged VSIX runtime now defaults to `rust-selected-query` through a long-lived `engine-shadow-runner` daemon when the bundled runner is present, while source checkouts keep the unset default on `typescript-current`
  - non-daemon async selected-query calls also use a true asynchronous one-shot runner path, so disabling the daemon does not reintroduce event-loop blocking
  - LSP providers now consume `engine-host-node` query helpers instead of importing `core/query` internals directly, and `pnpm check:provider-host-routing-boundary` guards that provider boundary
  - `omena.lspServerRuntime=auto` now selects the bundled or built Rust `omena-lsp-server` and fails fast when no binary is available; the legacy Node LSP server is only selected through explicit `omena.lspServerRuntime=node`. The Rust path carries provider parity coverage for style providers, source selector providers, source/style diagnostics, quick fixes, watched-file style index updates, and imported-module source reference scoping
  - style providers now consume Rust-backed semantic graph read models for selector identity, references, diagnostics, completions, rename safety, hover metadata, and style-module usage through host-side caches
  - `pnpm check:rust-phase-2-swap-readiness` remains the historical Phase 2 swap batch; `pnpm release:verify` is the current v5 release gate
  - the Omena Rust split crates are published and externally consumable through crates.io; v5 treats them as the stable internal product baseline, not a separate semver-1.0 crate freeze
  - `OMENA_TYPE_FACT_BACKEND` now defaults to `tsgo`, packaged runtime carries its own tsgo binary, and `typescript-current` is retained as an explicit comparison fallback
  - `tsgo` is wired into release-batch, real-project corpus, LSP smoke, protocol/editing, server build, workspace build, and Phase C edge-readiness checks
  - the current release-shaped TS 7 aggregate is `pnpm check:tsgo-release-bundle`, and `pnpm release:verify` includes it
  - the current limited non-release default lane is `pnpm check:operational-lane`
  - the current limited non-release default review command is `pnpm check:operational-shadow-review`
  - the current top-level TS 7 judgment command is `pnpm check:ts7-decision-ready`
  - `TS 7 Phase C` now covers watch / incremental behavior, long-lived LSP session behavior beyond one-shot smoke, and multi-root / workspace-edge cases
  - `selector-usage` remains a shadow validation family, not a release-gating canonical candidate
  - current `EngineInputV2` does not preserve enough reference-level evidence to reproduce `selector-usage` semantics as an input-only canonical producer
  - the current internal Rust producer boundary is [`rust/crates/omena-engine-input-producers`](./rust/crates/omena-engine-input-producers/README.md)
  - the current internal Rust parser/public-product scaffold starts in [`rust/crates/omena-parser`](./rust/crates/omena-parser/README.md)
- `pnpm check:real-project-corpus` runs a clean multi-file corpus that mimics common product patterns (`variants`, `@value` + `@keyframes`, `composes`, `.module.less`)
- semantic smoke cases are versioned in `scripts/semantic-smoke-corpus.ts` and should be updated when new semantic surfaces become release-relevant
- `pnpm check:release-batch` is the canonical release-facing batch checker pass
- the release batch corpus is versioned in `scripts/release-batch-corpus.ts`; it stays intentionally clean even if `examples/` contains negative recovery fixtures
- `pnpm check:contract-parity-v2-smoke` and `pnpm check:contract-parity-v2-golden` are the canonical parity gates
- frozen V1 baseline commands remain available as historical references:
  - `pnpm check:contract-parity-v1-smoke`
  - `pnpm check:contract-parity-v1-golden`
  - `pnpm update:contract-parity-v1-golden`
- V2 exposes constrained bundle metadata today for:
  - Bundle 1: `suffix`, `prefixSuffix`
  - Bundle 2: `charInclusion`
  - Bundle 3: `composite`
  - `TypeFactTableV2`
  - `EngineOutputV2.queryResults`
  - `pnpm explain:expression --json`
- explicit CLI flags override preset defaults

Test layout:

| Tier      | Location          | Purpose                                              |
| --------- | ----------------- | ---------------------------------------------------- |
| Unit      | `test/unit/`      | Pure logic and provider tests with mock dependencies |
| Protocol  | `test/protocol/`  | Full LSP roundtrips through the in-process harness   |
| Benchmark | `test/benchmark/` | Provider microbenchmarks                             |

### ESLint plugin

The first ESLint consumer lives at `packages/eslint-plugin`.

Current scope:

- source-side rules only
- `missing-module`
- `missing-static-class`
- `missing-template-prefix`
- `missing-resolved-class-values`
- `missing-resolved-class-domain`
- `invalid-class-reference`
- `no-unknown-dynamic-class`
- aggregate `source-check`

Flat config example:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("@omena/eslint-plugin");

export default [...omena.configs.recommended];
```

Focused variant:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("@omena/eslint-plugin");

export default [...omena.configs.focused];
```

Optional dynamic moat:

```js
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const omena = require("@omena/eslint-plugin");

export default [...omena.configs.dynamicMoat];
```

Repo-local clean example:

- [examples/plugin-consumers](./examples/plugin-consumers/README.md)

Supported rule options today:

- `workspaceRoot`
- `classnameTransform`
- `pathAlias`
- `includeMissingModule` (`source-check` / `missing-module`)

### Stylelint plugin

The first Stylelint consumer lives at `packages/stylelint-plugin`.

Current scope:

- style-side rules only
  - `unused-selector`
  - `missing-composed-module`
  - `missing-composed-selector`
  - `missing-value-module`
  - `missing-imported-value`
  - `missing-keyframes`

See [packages/stylelint-plugin/README.md](./packages/stylelint-plugin/README.md) for usage.
See [examples/plugin-consumers](./examples/plugin-consumers/README.md) for a clean repo-local consumer workspace.

### External clients

Minimal setup docs for the generic `lsp-server`:

- [docs/clients/neovim.md](./docs/clients/neovim.md)
- [docs/clients/zed.md](./docs/clients/zed.md)

## Examples

`examples/` contains scenario-based manual QA fixtures for the extension
development host. See [examples/README.md](./examples/README.md).

## Contributing

Before opening a change:

- run `pnpm check`
- run `pnpm test`
- verify the affected scenario in `examples/` when the change touches editor behavior

Keep commits scoped to a single concern when possible.

## License

MIT. See [LICENSE](./LICENSE).
