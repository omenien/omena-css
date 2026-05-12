# Changelog

## [Unreleased]

### Changed

- **Parser-owned facts CLIs** - `omena-parser-style-facts` and `omena-parser-lex` now live in `omena-parser` as parser-owned binaries, moving the parser public lane off the query/runner-owned command surface.
- **Parser scaffold gate ownership** - `check:rust-parser-scaffold` now validates `omena-parser` directly instead of the legacy `engine-style-parser` crate, making the parser lane start from the green-field parser track.
- **Parser-owned selected-query facts output** - the packaged `engine-shadow-runner` compatibility commands now route parser facts and lexer summaries through `omena-parser` directly and advertise the parser-owned `omena-parser.style-facts` product.
- **Bridge source-backed design tokens** - `omena-bridge` now exposes design-token workspace declarations directly from style source through the `omena-parser` semantic boundary, reducing the remaining legacy `Stylesheet` parser dependency surface.
- **Query design-token bridge consumption** - `omena-query` now delegates workspace design-token declaration collection to `omena-bridge` instead of rebuilding parser facts locally in the semantic graph batch path.
- **Bridge parser dependency boundary** - `omena-bridge` no longer depends directly on `engine-style-parser`; legacy graph signatures consume parser compatibility types re-exported by `omena-semantic` while bridge tests exercise the source-backed path.
- **Semantic parser dependency boundary** - `omena-semantic` now owns its parser/semantic contract DTOs and builds the semantic boundary from `omena-parser` source-backed facts, removing its direct `engine-style-parser` dependency while preserving CSS custom-property context ranking and Sass selector-resolution semantics.
- **Parser CSS Modules value references** - `omena-parser` now promotes CSS Modules `@value` references found inside declaration values into parser-owned style facts, reducing the remaining gap between the parser-owned fact surface and the legacy parser public-product lane.
- **Parser parity-lite summary** - `check:rust-parser-parity-lite` now reads its actual CSS-family structural summary from `omena-parser-summary` instead of `engine-style-parser-summary`, reducing the parser public-product lane's remaining legacy producer surface.
- **Parser CSS Modules intermediate producer** - `check:rust-parser-css-modules-intermediate` now reads its actual index-bridge summary from `omena-parser-css-modules-intermediate` instead of the legacy `engine-style-parser` producer, covering CSS Modules values, custom properties, Sass symbols, wrappers, keyframes, composes, and nested BEM parity through the parser-owned path.
- **Parser canonical producer wrappers** - `check:rust-parser-canonical-candidate`, `check:rust-parser-evaluator-candidates`, and `check:rust-parser-canonical-producer` now run `omena-parser-*` actual producers instead of the legacy `engine-style-parser` binaries.
- **Checker tier catalog** - `omena-checker` now classifies the rule registry into M/S/T tiers with boundary summary counts, and the transitional checker CLI rule help prints S/T tier metadata for TypeScript-owned runtime diagnostics.
- **CME explain CLI surface** - added `pnpm cme explain ...` as the product-facing command for expression value-domain/provenance explanations, with `pnpm cme explain expression ...` kept as the explicit subcommand form.
- **Query-owned transform context producer** - `omena-query` now derives transform execution context from workspace style sources, covering reachable name seeds, direct import inlines, CSS Modules class rewrites, and local `composes` export expansion before transform plan/execute consume the context.
- **Transform incremental execution** - `omena-transform-passes` now exposes a persistent `omena-incremental`/Salsa-backed transform execution path with source, context, plan, pass, and execution dependency nodes, so clean transform runs can reuse the previous execution summary instead of re-running the pass pipeline.
- **Transform source-map provenance** - transform provenance nodes now expose diff-derived mutation spans, and the print boundary emits source-map segments from those spans while identity print output keeps line-level mappings.
- **Parser semantic name consumption** - `omena-parser` now projects parser style facts into typed `omena-interner` name kinds and validates them through the Salsa interners, moving parser/interner integration out of the next-surface list.
- **Semantic SoA name tables** - `omena-semantic` now exposes selector, custom-property, and Sass semantic name tables backed by typed `omena-interner` Salsa interners, promoting the `semanticSoaTables` and `semanticSoaNameTables` surfaces out of the next queue.
- **Resolver tsconfig path mapping** - `omena-resolver` now resolves style modules through explicit tsconfig-style path aliases, including Sass partial candidates, moving `tsconfigPathMapping` out of the resolver decoupling queue.
- **Resolver specifier runtime** - `omena-resolver` now exposes a batch specifier-resolution runtime and runner command over relative, package, tsconfig-path, external, and unresolved style module sources.
- **Query transform module split** - `omena-query` now keeps transform planning/execution/context façade code in a dedicated `style::transform` module while preserving the public query API.
- **Query evaluation runtime** - `omena-query` now exposes a runtime-backed evaluation summary and runner command that ties the selected-query adapter, resolver runtime index, expression-domain Salsa runtime, and parser-owned style-document summaries into one decoupled surface.
- **Sass module graph closure** - `omena-parser` now surfaces `@forward show/hide` visibility filters on Sass module edge facts, and `omena-query` consumes those facts to report Sass module graph closure, cycles, and filter metadata without query-local source rescans.
- **Parser CST equivalence** - `omena-parser` now exposes a runtime CST equivalence summary proving that parser CST nodes/tokens consume the shared `omena-syntax` `SyntaxKind` contract with source-text round-trip and typed-wrapper evidence.
- **Cascade WPT-style seed corpus** - `omena-cascade` now runs a 200+ case WPT-style ordering matrix for origin/importance, layers, scope proximity, and specificity while keeping the full external WPT mirror as a separate conformance target.
- **Parser Pratt value boundary** - `omena-parser` now reports Pratt value parser core coverage separately from the still-deferred full CSS property-value grammar registry, avoiding the older over-broad `fullPrattValueParser` blocker label.
- **Parser recursive-descent boundary** - `omena-parser` now reports recursive-descent parser core coverage separately from the still-deferred complete external CSS-family spec mirror, avoiding the older over-broad `fullRecursiveDescentGrammar` blocker label.
- **Parser-owned query contracts** - `omena-query` now owns its public parser position/range/byte-span and style-language contracts through `omena-parser`, removing its direct `engine-style-parser` dependency while keeping bridge/semantic compatibility conversions at the boundary.
- **Bridge style resolution** - `omena-bridge` now resolves `.sass` style candidates through its own extension filter instead of depending on the legacy parser language enum for import-specifier resolution.
- **Reduced product overlap semantics** - `omena-abstract-value` now computes prefix/suffix reduced-product minimum lengths with overlap awareness and accounts for required character constraints, so `Pr ⊗ Su` and `Pr/Su ⊗ CI` intersections no longer exclude valid selectors or understate required selector length.
- **Query adapter status** - `omena-query.selected-query-adapter-capabilities` now reports the selected-query adapter as `runtimeBacked`, matching the packaged runner/protocol/default-candidate path instead of the older declaration-only transition status.
- **Parser product range indexing** - `omena-parser` now builds a per-source line index for public-product range conversion, restoring the Z5 parser-product cut-over gate without weakening its ratio threshold.

## [5.0.0] - 2026-05-06

### Changed

- **Rust stable core closure** - declared the v5 Rust core milestone after the 4.15-4.19 feature chapters landed the query-owned LSP boundary, refactor automation, M-tier rule surfaces, Z5 performance baseline, full reduced-product/provenance work, and hover provenance UI.
- **Rust LSP migration status** - promoted the `omena-lsp-server` boundary from `thinClient` migration status to `rustStable` while keeping the thin-client endpoint as the packaged VSIX and multi-editor runtime model.
- **Release positioning** - updated the README release frame from the old 4.1.x swap-candidate wording to the v5 compiler-grade CSS Modules semantic engine posture.

## [4.19.0] - 2026-05-05

### Added

- **Abstract-value provenance trees** - `omena-abstract-value`, Rust selected-query producers, and Engine V2 expression semantics now carry a structured value-domain provenance tree for reduced class values.
- **Hover provenance surface** - source hovers now render value-domain provenance and retained constraint steps alongside the existing selector/value-domain explanation.

### Changed

- **Abstract-value module split** - `omena-abstract-value` is split into focused domain, algebra, facts, flow, provenance, reduced-product, selector-projection, and type modules instead of concentrating the public surface in one large file.
- **TS7 shadow provenance parity** - TS7 shadow expected payloads now include the Rust provenance-tree contract for expression-domain and expression-semantics evaluator/canonical-producer lanes.

## [4.18.0] - 2026-05-05

### Added

- **Z5 performance baseline** - added the `omena-benchmarks` Rust crate with Criterion micro-benchmarks for parser, semantic, and abstract-value workloads.
- **LSP macro-benchmark gate** - added a Z5 readiness bundle that compiles the benchmark harness and runs the Rust LSP runtime-loop macro benchmark under bounded request-path latency thresholds.
- **Public performance documentation** - documented the benchmark corpus, commands, comparison policy, and a reproducible local baseline snapshot in `docs/performance.md`.

## [4.17.0] - 2026-05-05

### Added

- **CSS Modules `@value` completion** - style completions now suggest local and imported `@value` tokens in declaration values and local `@value` declaration values, while preserving Sass function completions in the same contexts.
- **M-tier ESLint rules** - `eslint-plugin-css-module-explainer` now exposes `no-impossible-selector` and `no-imprecise-value` as precise split rules over the existing dynamic class diagnostics, with `configs.mTier` enabling the non-duplicating split.
- **Stylelint token recovery rules** - `stylelint-plugin-css-module-explainer` now exposes `missing-custom-property` and `missing-sass-symbol`, completing the current style-side recovery/token rule surface.

### Changed

- **Stylelint checker reuse** - stylelint rules now reuse a cached style checker report per workspace/config instead of spawning the checker once per rule, keeping the expanded rule library practical for smoke and editor-adjacent use.

## [4.16.0] - 2026-05-05

### Added

- **Refactor automation chapter** - selector, design-token, and composed-class refactors now cover the main Z2 workflow slice: rename across source/style boundaries, extract design tokens from literals, and inline composed utility classes.
- **Design-token extraction actions** - selected style literals now offer both CSS custom property extraction and CSS Modules `@value` extraction.
- **Composed-class inline actions** - `composes` tokens can be inlined from same-file, cross-file, and transitive dependency declarations while unresolved/global/cyclic cases stay blocked.

### Changed

- **CSS Modules selector rename through `composes`** - direct cross-file `composes` tokens are now rewritten during selector rename, while transitive composed dependencies remain blocked instead of applying unsafe edits.
- **CSS Modules `@value` rename through imports** - source `@value` declarations now rename through aliased and unaliased imports; unaliased importer references are updated to preserve binding correctness.

### Fixed

- **CSS value rename coverage** - local CSS Modules `@value` declarations and imported local aliases now participate in style rename planning.

## [4.15.0] - 2026-05-05

### Added

- **Omena checker split package** - `omena-checker` is now mirrored to `omenien/omena-checker`, published as `omena-checker@0.1.0`, and covered by a git consumer plus split publish-readiness checks alongside the other Omena Rust crates.

### Changed

- **Design-token package manifest routing** - Rust selected-query style semantic graph batches now accept package manifest metadata and use `sass` / `scss` / `style` / `exports` entries when building package import reachability for cross-file design-token ranking.
- **Design-token package source expansion** - selected-query graph batches now include package style files discovered through package manifests, so Rust ranking can see the actual package token declarations instead of only the manifest metadata.
- **Source import boundary split** - `omena-bridge` now owns the source import declaration producer used by the Rust LSP CSS Modules binding path, reducing `omena-lsp-server`'s direct source-parsing responsibility.
- **Source syntax boundary split** - `omena-bridge` now owns the Rust LSP source syntax index producer for CSS Modules imports, selector references, class utilities, and source type-fact targets; `omena-lsp-server` now consumes the bridge output instead of carrying the source scanner locally.
- **Checker registry boundary split** - `omena-checker` now owns the Rust-side checker rule descriptor and code-bundle registry boundary, establishing the rule catalog transition point before diagnostic execution moves out of the TypeScript checker runtime.
- **OXC-backed source import producer** - `omena-bridge` now derives source import declarations from the OXC TypeScript/TSX AST instead of an import-token scanner, starting the parser-backed source producer migration called out by the Rust LSP boundary review.
- **Bridge-owned style import resolution** - Rust LSP source and Sass module paths now delegate relative and tsconfig/jsconfig style specifier resolution to `omena-bridge`, reducing path-alias and style-candidate expansion logic inside `omena-lsp-server`.
- **OXC-backed source syntax facts** - `omena-bridge` now derives JSX `className` literals/expressions, CSS Module `styles.foo` / `styles["foo"]` source property references, and `classnames/bind` bindings/calls from the OXC TSX AST instead of source-token scanners, continuing the `omena-lsp-server` layer split from the §182 boundary review.
- **Query-owned style facts** - `omena-query` now owns style document summaries, style hover candidates, custom-property reference ranges, Sass module sources, Sass symbol facts, and Sass partial-evaluator selector candidates; `omena-lsp-server` now maps query output into LSP responses instead of parsing those facts locally.
- **Rust LSP query boundary declaration** - the Rust LSP boundary contract now declares `omena-query/styleHoverCandidates` as the style definition owner for source-provider requests, matching the implementation instead of pointing directly at parser facts.
- **Shared design-token ranking read model** - style hover, definition, and references now consume the same host-side Rust design-token ranking query instead of each resolving selected-query graph winners independently.
- **Design-token declaration candidates** - Rust selected-query style semantic graphs now expose custom-property declaration candidates, and style completions consume that shared read model for external/package CSS variable suggestions.
- **Query-owned hover render parts** - Rust LSP style hover rendering now consumes `omena-query` snippet/value/signature render parts instead of extracting selector and Sass snippets inside the LSP server.
- **Query-owned diagnostic actions** - Rust LSP missing custom-property and source selector diagnostics now consume `omena-query` diagnostic/action payloads instead of planning `createCustomProperty` / `createSelector` quickfixes inside the LSP server.
- **Query-owned source candidate resolution** - Rust LSP source-provider matching now delegates matched/unresolved selector candidate resolution, source-definition filtering, and prefix selector-name expansion to `omena-query`, keeping selector-prefix and target-style matching semantics out of the protocol layer.
- **Query-owned selector rename edits** - Rust LSP selector rename now delegates definition/reference edit planning and dotted-name replacement normalization to `omena-query`, leaving workspace edit JSON grouping in the protocol layer.
- **Query-owned Sass symbol matching** - Rust LSP Sass variable/mixin/function matching now consumes `omena-query` symbol-kind classification and declaration filtering instead of duplicating those semantics in the protocol layer.
- **Query-owned Sass module source selection** - Rust LSP Sass symbol resolution now delegates `@use` namespace/wildcard selection, `@forward` selection, and Sass built-in module filtering to `omena-query` before resolving style URIs through the bridge.

### Fixed

- **Rust-ranked token diagnostics** - style diagnostics now suppress missing custom-property warnings when the Rust design-token ranking path has already resolved the `var(--token)` reference to a winner declaration.
- **Rust-ranked token references** - style references now use Rust design-token winner ranges for external/package CSS custom property declarations before falling back to the TypeScript resolver path.

## [4.14.0] - 2026-05-05

### Added

- **BinderPluginV0 boundary** - `omena-bridge` now exposes a built-in `BinderPluginV0` boundary for source-side class-name tracking, with the current CSS Modules + `classnames/bind` / `classnames` / `clsx` behavior declared as the default built-in plugin.

### Changed

- **Production source analysis routing** - Node runtime and checker analysis now route CSS Modules binding, class utility detection, and class-reference extraction through the default binder plugin instead of wiring the cx/style scanners as separate production dependencies.

## [4.13.0] - 2026-05-05

### Added

- **tsgo-backed typed `cx()` projection** - Rust LSP source requests now consume `omena-tsgo-client` type facts for finite literal unions and template holes, projecting typed `classnames/bind` arguments such as `size` and ``font-size-${fontSize}`` onto canonical selector definitions.
- **Sass partial-evaluation selector catalog** - Rust LSP style indexing now surfaces map + `$prefix` Sass include outputs such as `color-green` / `color-blue` as generated selector candidates, allowing source prefix lookups like ``color-${color}`` to resolve instead of returning silently.

### Fixed

- **Sass semantic hover rendering** - Sass variable and mixin hovers now render resolved declaration values and callable bodies from the target definition instead of showing only the include/reference placeholder line.

## [4.12.0] - 2026-05-05

### Added

- **Value-domain-aware source completion** - source completions now use class-value domain matching to narrow CSS Module selector suggestions for property access, bracket access, string-token, and object-key prefixes.
- **Checker rule registry** - the checker exposes a rule descriptor registry plus `--list-rules`, making current diagnostics discoverable by code, category, default severity, fixability, and preset.
- **LSP runtime latency baseline** - Rust and selected-query LSP runtime loops now report per-request p50/p95/max latency for hover, definition, references, and completion alongside the existing event-loop probe budget.

### Fixed

- **Rust LSP UTF-8 recovery safety** - JS recovery scanners now advance through UTF-8 character boundaries, preventing the char-boundary panic class triggered by multibyte strings and escaped characters.
- **Next.js route-group CodeLens ownership** - workspace-folder compatibility now normalizes percent-encoded file URIs before comparison, preserving references and CodeLens inside App Router `(group)` directories.
- **Resolved target hover rendering** - source hovers and Sass symbol hovers can render unopened resolved style targets from disk instead of falling back to placeholder output when the target file is not open.

## [4.11.0] - 2026-05-01

### Changed

- **CodeLens warm-path optimization** - style CodeLens now resolves Rust style semantic graph references once per style file and reuses selector summaries across lenses, preventing per-selector graph request fan-out on larger modules.
- **CodeLens refresh coalescing** - LSP CodeLens refresh requests are now debounced and safe against disposed JSON-RPC connections, reducing refresh storms after semantic-reference updates.

### Added

- **Cross-file Sass symbol rename** - renaming a Sass module member reference now updates the resolved declaration plus incoming `@use`/wildcard member sites recorded in the style dependency graph.

## [4.10.0] - 2026-05-01

### Added

- **CFG and k-CFA abstract-value surfaces** - `omena-abstract-value` now exposes control-flow-graph pruning and k-limited call-site batch analyses for class value flow facts, giving downstream tooling explicit entry points for flow-sensitive selector semantics beyond the original one-CFA check.
- **Selected-query CFG analysis lane** - `engine-input-producers`, `omena-query`, `engine-shadow-runner`, and the Node selected-query backend now expose an expression-domain control-flow analysis command so CFG facts can travel through the same production query boundary as the incremental flow runtime.

### Changed

- **Abstract-value release gate** - the focused Rust check now covers the broader CFA family instead of only the original one-CFA surface.

## [4.9.0] - 2026-05-01

### Changed

- **Salsa incremental database** - `omena-incremental` now exposes a persistent Salsa-backed database with tracked node snapshot queries, field-granular reuse, and plan/snapshot progression owned by the database instead of external manual snapshot plumbing.
- **Query runtime reuse** - `omena-query` now exposes an expression-domain incremental flow runtime that keeps per-graph Salsa databases alive across engine-shadow-runner daemon requests and reuses clean abstract-value flow analyses.

### Fixed

- **Selected-query incremental boundary** - expression-domain flow graph construction is now shareable between producer summaries and query runtime reuse, so the selected-query layer can consume the same graph facts without rebuilding a parallel analysis path.

## [4.8.0] - 2026-05-01

### Changed

- **tsgo IPC recovery** - `omena-tsgo-client` now retries recoverable JSON-RPC type-fact batches by restarting the workspace process and replaying the batch with a fresh snapshot, covering I/O failures, missing responses, and unexpected response ids without re-entering the old TypeScript fallback path.

## [4.7.0] - 2026-05-01

### Changed

- **Rust LSP source/style path coverage** - expanded Rust LSP request handling across tsconfig/jsconfig style path aliases, dynamic `classnames/bind` values, Sass wildcard imports, Sass namespace `@use`, and forwarded Sass module definitions.
- **Parser Sass module facts** - `engine-style-parser` now tracks module-qualified Sass symbol references as external selector facts without folding them into same-file resolution.

### Fixed

- **Dynamic source selector references** - `classnames/bind` references now resolve exact local constants, object properties, object keys, logical/conditional expressions, and template/concat prefixes without falling back to cross-module candidates.
- **Sass symbol navigation paths** - Sass variable, mixin, and function references now resolve through wildcard imports, namespaced module uses, tsconfig path aliases, partials, index files, and direct forward chains.

## [4.6.0] - 2026-04-30

### Changed

- **Workspace resolver cleanup** - removed the legacy synchronous `WorkspaceTypeResolver` implementation and `createDefaultProgram` helper from `engine-core-ts`; host/runtime code now accepts only explicit `TypeResolver` injection and defaults to the tsgo-backed path.
- **Rust LSP workspace runtime registry** - moved workspace-folder ownership and indexed-style-document lifecycle policy behind an `omena-lsp-server` registry contract, with longest-root ownership and open-document preservation declared in the boundary gate.
- **Rust LSP diagnostics scheduler** - extracted Rust-owned diagnostics notification planning from the LSP message loop into a scheduler boundary covering document changes, watched style changes, configuration reloads, and initialized workspace indexing.
- **tsgo JSON-RPC type-fact provider** - added a Rust provider orchestration layer that executes initialize, snapshot, project mapping, type lookup, union expansion, and release over the managed `omena-tsgo-client` transport.
- **Rust LSP query reuse boundary** - extracted document-owned reusable indexes for style summaries, hover candidates, source syntax, and source selector candidates so provider requests consume refreshed document state instead of rescanning raw text.
- **Thin VS Code client host** - moved Rust LSP server option and document-selector construction into the thin-client runtime contract so extension activation only resolves settings, creates watchers, translates VS Code-only command arguments, and starts the client.
- **Rust LSP multi-editor distribution** - promoted Neovim/Zed/VS Code support from docs-only guidance into the `omena-lsp-server` boundary contract, with standalone Rust server endpoints and no Node LSP primary endpoint in the multi-editor path.
- **Shadow CI tsgo lane** - aligned checker release-gate shadow with the tsgo-only rust gate evidence variants and made the ESLint plugin smoke gate build its required server dist artifacts on fresh CI checkouts.
- **Extension host smoke determinism** - pinned the VS Code test host to the extension `engines.vscode` baseline by default and raised the `@vscode/test-electron` request timeout so CI does not fail while resolving the latest VS Code version.

## [4.5.0] - 2026-04-30

### Changed

- **Parser-owned selector definition facts** - `engine-style-parser` now exposes range-aware selector definition facts for LSP consumers, including CSS Module exportable class segments and resolved nested BEM suffix selectors.
- **Rust LSP thin-client closure** - the Rust LSP boundary now reports `thinClient`, enforces the no-Node-workspace-resolver product path, and declares target-aware source-candidate dedupe plus parser selector definition fact consumption as part of the request path policy.
- **Type-fact product modes** - VS Code settings now expose only tsgo-backed type-fact modes; the old current-TypeScript resolver path is no longer available as a product fallback.

### Fixed

- **Rust LSP source hover parity** - source hovers now render product-facing selector markdown from the target style definition and rule snippet instead of exposing internal Rust opened-document index wording.
- **Rust LSP target-aware definitions** - `classnames/bind` source references now prefer imported CSS Module targets over generic source candidates, preventing cross-module go-to-definition results and doubled style code-lens reference counts.
- **Rust LSP nested BEM parity** - nested SCSS `&--...` and `&__...` selectors are now consumed from parser-owned selector definition facts for definition, diagnostics, references, and code lens in the Rust LSP path.

## [4.4.0] - 2026-04-30

### Changed

- **Rust LSP source syntax index** - source-side CSS Module imports, `className` string literals, `styles.*` / `styles["..."]` references, and `classnames/bind` utility calls now share a document-level Rust source syntax index built on open/change instead of each provider re-scanning raw text on request.
- **Source provider boundary** - the Rust LSP boundary now declares `omena-lsp-server/sourceSyntaxIndex` as the source candidate owner and guards the request path with `buildSourceSyntaxIndexOnDocumentChange`.

### Fixed

- **ASI import crash** - TS/TSX files with semicolon-free import declarations no longer panic the Rust LSP server while resolving hover, definition, references, completion, or diagnostics for CSS Module references.

## [4.3.0] - 2026-04-30

### Added

- **Abstract-value 1-CFA MVD** - added call-site-discriminated 1-CFA batch analysis with per-call-site flow results, callee exit summaries, and testable derivation steps.
- **Published 1-CFA split surface** - published and verified `omena-abstract-value@0.1.6`, and updated the external git-consumer fixture to exercise the new call-site flow API.

### Changed

- **Rust release gates** - `rust/omena-abstract-value/one-cfa` now participates in the abstract-value split boundary, Rust lane bundle, and Rust release bundle.
- **Release verification order** - `pnpm release:verify` now builds the extension before Rust gate evidence so LSP smoke checks always have `dist/server/server.js` available.
- **Publish workflow release target** - GitHub Release creation now uses the checked-out commit SHA as `target_commitish`, avoiding tag-name target failures during publish.

## [4.2.0] - 2026-04-30

### Added

- **Rust LSP thin-client GA endpoint** - packaged extension builds now launch the Rust `omena-lsp-server` directly from `dist/bin/<platform>-<arch>/omena-lsp-server`, with the Node LSP server kept as a development-only fallback outside the packaged VSIX path.
- **Published split LSP runtime** - published and verified `omena-lsp-server@0.1.3` with `thinClient` migration status, alongside the existing `omena-tsgo-client`, `omena-resolver`, `omena-query`, `omena-bridge`, `omena-semantic`, `omena-incremental`, and parser/input split crates.
- **Incremental abstract-value substrate** - added incremental batch flow analysis on top of `omena-incremental`, keeping the 1-CFA abstract-value work ready for the next 4.3 line.

### Changed

- **TS7 source path boundary** - the default `tsgo` backend no longer implicitly creates a synchronous `WorkspaceTypeResolver` fallback for source request paths; direct misses now stay fast and unresolvable unless an explicit fallback is injected.
- **Release gates** - phase-3 source readiness, phase-4 thin-client readiness, resolver split readiness, and split publish readiness now cover the v4.2.0 cut line.

### Fixed

- **VSIX thin-client packaging** - packaged selected-query/default checks now verify bundled `omena-lsp-server` targets across the native release matrix.

## [4.1.27] — 2026-04-30

### Fixed

- **Rust LSP `classnames/bind` definition parity** — the Rust LSP source scanner now indexes static `cx("...")` references created by `classnames/bind` utility bindings, restoring go-to-definition for the extension-host smoke path without falling back to the Node server.

## [4.1.26] — 2026-04-30

### Added

- **Rust LSP thin-client release lane** — added the `omena-tsgo-client` phase-3 boundary, the `omena-lsp-server` phase-4 thin-client endpoint contract, and split/multi-editor gates for the Rust LSP runtime path.
- **Incremental flow substrate** — added `omena-incremental` as the incremental computation layer for abstract-value flow analysis, wired `omena-abstract-value` incremental flow planning on top of it, and propagated the split publish readiness line through `omena-incremental@0.1.0` and `omena-abstract-value@0.1.3`.

### Changed

- **Native release matrix** — CI and publish workflows now build and package `omena-lsp-server` across the same native platform matrix as `engine-shadow-runner` and `tsgo`, and the VSIX gate verifies the packaged Rust LSP server target coverage.

## [4.1.25] — 2026-04-30

### Fixed

- **Large-workspace LSP request blocking** — LSP requests no longer re-enter the synchronous TypeScript workspace resolver when the tsgo-backed source path cannot answer immediately. Unavailable source facts now fail fast as unresolved results instead of allowing hover, definition, references, diagnostics, or completion to trigger workspace-sized blocking work on the Node request path.
- **Rust LSP cancellation and workspace bounds** — the Rust LSP lane now handles already-cancelled requests before provider work starts and keeps workspace style indexing bounded, reducing the conditions that pinned duplicate Node servers on large Next.js workspaces.

### Changed

- **Rust LSP migration gates** — the phase-2 swap-readiness gate now covers the source-side canonical/evaluator paths from input facts, and the migration boundary explicitly tracks the next phase-4 thin-client endpoint before the `omena-lsp-server` split.

## [4.1.21] — 2026-04-29

### Fixed

- **Forwarded package CSS custom properties** — CSS custom property completions, hovers, and diagnostics now follow local Sass utility modules that `@forward` package style entries, so `@use "utils" as *` can resolve `var(--...)` tokens from package `style` CSS files.

### Changed

- **Rust package style graph publishing** — published `omena-engine-style-parser@0.1.5` and `omena-query@0.1.11` with plain style package entry parsing and node_modules package graph edge resolution for Rust semantic-graph consumers.

## [4.1.20] — 2026-04-29

### Changed

- **Bundled tsgo probe runtime** — the TS 7 type-fact backend now resolves an extension-owned `tsgo` binary from `dist/bin/<platform>-<arch>` by default, packages the native compiler and lib files alongside the runner matrix, and exposes `cssModuleExplainer.typeFactBackend` for bundled tsgo, workspace tsgo, or current TypeScript.

## [4.1.19] — 2026-04-29

### Changed

- **Rust-backed design-token hover targets** — CSS custom property hovers now reuse runtime semantic graph cache entries and can materialize Rust design-token winner files on demand, so imported token declarations remain hoverable even when the legacy TypeScript resolver has not indexed the winner yet.

## [4.1.18] — 2026-04-29

### Changed

- **Daemon-backed style hover graph data** — style-file hovers now route CSS custom property ranking and selector reference/identity metadata through the async Rust selected-query runner when available, aligning hover with the packaged daemon-backed definition path while preserving existing TypeScript fallbacks.

## [4.1.17] — 2026-04-29

### Changed

- **External design-token definition runtime** — style `var(--...)` go-to-definition now routes Rust semantic-graph winner lookups through the async selected-query runner, so packaged LSP requests use the daemon-backed path while preserving the existing TypeScript fallback when no Rust winner is available.

## [4.1.16] — 2026-04-29

### Added

- **Import-aware design-token ranking** — Rust selected-query semantic graphs now filter cross-file CSS custom property candidates through Sass `@use`, `@forward`, and legacy `@import` reachability instead of treating every workspace style module as an equal candidate.
- **External design-token winner ranges** — external CSS custom property winners now carry declaration ranges through the Rust semantic graph into the host read model, preparing imported token winners for precise navigation surfaces.

### Changed

- **Design-token hover wording** — cross-file custom property ranking notes now distinguish import-graph candidates from broader workspace candidates.

## [4.1.15] — 2026-04-29

### Added

- **Design-token cascade ranking** — Rust semantic graph output now exposes source-order ranked CSS custom property references, including the winning declaration and shadowed same-file declarations.
- **CSS custom property hover ranking** — `var(--...)` hovers can now surface cascade ranking context from the Rust selected-query semantic graph, showing when a source-order winner shadows earlier same-file declarations.

### Changed

- **Split crate design-token contracts** — published and pinned the ranked-reference contract through `omena-semantic@0.1.8`, `omena-bridge@0.1.7`, and `omena-query@0.1.9`.

## [4.1.14] — 2026-04-29

### Fixed

- **CSS custom property selector-context diagnostics** — unmatched workspace theme declarations no longer suppress missing `var(--...)` diagnostics or definition lookups for unrelated selector contexts.

## [4.1.13] — 2026-04-29

### Fixed

- **CSS custom property completion source ranking** — `var(--...)` completions now prefer local declarations, then explicitly imported style-token modules, then workspace-indexed fallbacks when duplicate token names exist.
- **CSS custom property completion context filtering** — media/theme-specific custom property declarations no longer override root completions when the current selector or wrapper context does not match.

## [4.1.12] — 2026-04-29

### Fixed

- **SCSS path-alias token imports** — workspace `tsconfig` path aliases used from Sass `@use` and legacy `@import` now stay indexed and resolve forwarded package tokens through diagnostics and definition lookups.
- **Nested pseudo-rule mixin diagnostics** — Sass `@include` references inside nested selectors like `&::before` and `&::after` are now attributed to the parent class, so missing mixin diagnostics are reported consistently across argument shapes.
- **Windows protocol CI portability** — package-token hover assertions now normalize path separators, matching the existing cross-platform hover behavior.

## [4.1.11] — 2026-04-29

### Fixed

- **Package token runtime resolution** — LSP dependency lookup now reads imported package token assets with plain `.css`, `.scss`, and `.less` extensions, so design-token packages that expose `variables.css` or forwarded Sass partials resolve through hover, definition, completion, and diagnostics instead of being limited to `.module.*` files.

## [4.1.10] — 2026-04-29

### Added

- **Checker evidence text output** — checker text and compact output now print source analysis evidence, including value shape and value-domain derivation labels for dynamic class misses.

### Changed

- **Source-missing derivation parity** — the default TypeScript checker path now emits the same `omena-abstract-value.reduced-class-value-derivation` evidence contract as the Rust-backed source-missing path.
- **Rust source-missing consistency** — source-missing consumer checks now compare derivation evidence so shadow gates catch evidence drift, not only finding/count drift.

## [4.1.9] — 2026-04-29

### Added

- **Value derivation evidence** — Source-missing diagnostics and checker JSON reports now preserve reduced value-domain derivation evidence, including human-readable derivation labels.

### Changed

- **Rust diagnostics performance** — Parallelized Rust-backed source diagnostics, style usage, and reference lens lookups.
- **Lint hygiene** — Core checks now run with zero lint warnings.

## [4.1.8] — 2026-04-29

### Added

- **Source-missing Flow evidence** — Rust source-missing checker producer output now includes expression-domain flow evidence, including graph/node counts and convergence status.

## [4.1.7] — 2026-04-29

### Added

- **Checker Rust flow consumer** — checker CLI now supports `--rust-flow-analysis-consumer`, adding Rust expression-domain flow graph summaries to JSON output and a compact text summary.
- **Query-boundary flow routing** — `omena-query` now owns the `input-expression-domain-flow-analysis` runner boundary and is published as `omena-query@0.1.4`.

## [4.1.6] — 2026-04-29

### Added

- **Abstract-value flow analysis** — `omena-abstract-value` now exposes a V0 1-CFA class-value flow analysis core with assign/refine/join transfers and branch-merge joins.
- **Expression-domain flow runner** — `engine-input-producers` and the packaged shadow runner now expose `input-expression-domain-flow-analysis`, with split crate releases for `omena-abstract-value@0.1.2` and `omena-engine-input-producers@0.1.3`.

## [4.1.5] — 2026-04-29

### Fixed

- **Theme-token context ranking** — CSS custom property definition and completion now match `:root[data-theme="..."]` declarations to `[data-theme="..."]` usage contexts instead of falling back to unrelated root tokens.
- **Deep forwarded token coverage** — package-root Sass entries that forward internal token modules are now covered across hover, definition, completion, and checker diagnostics.

## [4.1.4] — 2026-04-29

### Fixed

- **Design-token package exports** — package-backed Sass and CSS resolution now handles `exports` wildcard patterns, fallback arrays, and extensionless style targets that need Sass candidate expansion.
- **Forwarded package token coverage** — local utility modules that `@forward` package subpaths with `as ds_*` are now locked across hover, definition, completion, and checker diagnostics.

## [4.1.3] — 2026-04-29

### Added

- **CSS custom property product path** — custom property tokens now have stronger hover, definition, references, completion, diagnostics, and quick-fix coverage, including edit-time completions and wrapper-aware completion ranking.
- **Design-token package coverage** — Sass package-root `@forward` flows and package-backed CSS custom properties are now covered across runtime queries and checker diagnostics.

### Fixed

- **Package subpath exports** — style resolution now handles package.json `exports` subpaths such as `@design/tokens/colors`, including Sass and CSS style entry conditions.

## [4.1.2] — 2026-04-29

### Fixed

- **Async selected-query runner fallback** — non-daemon Rust selected-query calls now use an asynchronous one-shot runner path instead of wrapping the synchronous spawn path in a resolved promise, keeping LSP event-loop behavior safe even when the daemon is disabled.
- **Runtime regression coverage** — selected-query backend tests now verify the non-daemon async path returns before the delayed runner exits, guarding against accidental reintroduction of sync process spawning on async product paths.

## [4.1.1] — 2026-04-28

### Fixed

- **LSP Rust backend runtime loop** — packaged VSIX runtime now runs the Rust selected-query backend through a long-lived `engine-shadow-runner` daemon by default, avoiding per-query process spawning on hot hover, reference, lens, and diagnostics paths.
- **Daemon release guard** — `pnpm check:rust-lsp-runtime-loop` now simulates a 50-selector LSP workload, verifies event-loop probe latency, and enforces a single daemon spawn so packaged Rust backend regressions are caught before release.

## [4.1.0] — 2026-04-26

### Added

- **Phase 2 swap readiness gate** — `pnpm check:rust-phase-2-swap-readiness` now batches provider host-routing boundary enforcement, the Rust selected-query default-candidate lane, and checker release-gate shadow enforcement into one release-candidate cut-line check.
- **Rust-backed style semantic graph consumers** — style hover, references, reference lenses, definition, rename, diagnostics, completions, and style module usage now share host-side semantic graph read models and cache reuse paths instead of each provider rebuilding the same graph-shaped facts.
- **Sass module semantics** — SCSS support now resolves `@use`, `@forward`, wildcard module members, module-qualified variables/mixins/functions, Sass symbol hover/definition/rename/completion, missing-symbol diagnostics, and scoped Sass variables; Less variable scope handling is covered as well.
- **Omena semantic boundary** — the Rust workspace now includes the `omena-semantic` boundary crate, source-evidence/observation CLIs, remote git-consumer checks, and publish-readiness validation for the external `omenien/omena-semantic` split.

### Changed

- **Provider host routing is release-candidate guarded** — LSP provider surfaces are now statically checked by `pnpm check:provider-host-routing-boundary`, preventing direct provider imports of core query, semantic graph, indexing, and TypeScript resolver internals.
- **Check orchestration is the canonical gate surface** — `cme-check` inventory/doctor/plan/surface reporting now owns the release gate map, workflows route through canonical gate IDs, and legacy check aliases were trimmed.
- **Vitest CME fixtures now cover provider scenarios** — protocol and provider tests have been migrated onto marker-based fixture helpers for cursor, target, range, and LSP-position setup, reducing duplicated position math in the test suite.

## [4.0.0] — 2026-04-24

### Changed

- **Packaged Rust runner matrix** — CI and publish now build Linux, macOS, and Windows `engine-shadow-runner` artifacts, merge them into `dist/bin/`, and verify the generated VSIX preserves the required runner matrix before publishing.
- **TS 7 backend default** — `CME_TYPE_FACT_BACKEND` now defaults to `tsgo`; use `CME_TYPE_FACT_BACKEND=typescript-current` for explicit current-TypeScript comparison runs.
- **TS 7 release gate** — `pnpm release:verify` now includes `pnpm check:tsgo-release-bundle`, and Phase C readiness covers long-lived sessions, multi-root churn, watched-file invalidation, and source/style staleness under `tsgo`.
- **Node-backed TS scripts** — package scripts now run TypeScript entrypoints through `node --import tsx`, avoiding the `tsx` CLI IPC path while keeping the same script bodies.

### Removed

- **Legacy path alias fallback** — `cssModules.pathAlias` is no longer read. Use the native `cssModuleExplainer.pathAlias` setting for extension-specific CSS Module import aliases.
- **TS 7 preview compatibility names** — `tsgo-preview` script/env aliases and `CME_TSGO_PREVIEW_CHECKERS` are no longer accepted; use `tsgo` and `CME_TSGO_CHECKERS`.

## [3.15.0] — 2026-04-24

### Added

- **Packaged Rust selected-query default** — packaged VSIX runtime now selects the unified `rust-selected-query` backend by default when the bundled `engine-shadow-runner` is present, while source checkouts keep the unset default on `typescript-current`.
- **TS 7 beta Phase B coverage** — the tsgo lane now covers protocol, editing, server build, and workspace build checks with `@typescript/native-preview@beta`, fixed checker/builder settings, and `tsgo` as the canonical backend alias.

### Changed

- **Provider reads now go through host query boundaries** — LSP providers consume `engine-host-node` query helpers instead of importing core query or rewrite internals directly, and architecture tests now guard that boundary.
- **TS 7 beta wording now uses probe semantics** — `tsgo-preview` remains a deprecated compatibility alias, but the implementation and release-facing scripts now describe the backend as a host-side tsgo probe with current TypeScript resolver fallback.
- **`3.15.0` is framed as the packaged Rust default and host-boundary milestone** — the release narrative records the selected-query packaged default, provider host-boundary enforcement, and TS 7 beta operational expansion without declaring `CME_TYPE_FACT_BACKEND=tsgo` as the global default yet.

## [3.14.0] — 2026-04-22

### Added

- **TS 7 beta Phase A readiness gate** — the repo now exposes `pnpm check:ts7-phase-a-readiness`, which batches backend typecheck smoke, type-fact backend parity, and the `tsgo-preview` slice of `rust-gate-evidence` into one repeatable pre-adoption check.

### Changed

- **Checker entrance is now enforced in the release-facing Rust gate** — `style-recovery` and `source-missing` remain the current bounded checker lanes, but they are no longer release-gate shadow only. `pnpm check:rust-release-bundle` now includes `pnpm check:rust-checker-entrance`, and the checker producer metadata now marks both lanes as `releaseGateStage=enforced`.
- **`3.14.0` is now framed as a checker release-gate Rust milestone** — the release narrative moves from checker release-gate readiness to actual release-bundle enforcement, while also leaving the next `TS 7 beta Phase A` work staged as an explicit follow-up gate instead of an informal note.

## [3.13.0] — 2026-04-21

### Added

- **Expanded lint-consumer surface** — the ESLint consumer now exposes focused source-side rules for `missing-static-class`, `missing-template-prefix`, `missing-resolved-class-values`, and `missing-resolved-class-domain`, while the Stylelint consumer now covers `composes`, `@value`, and `@keyframes` resolution failures in addition to `unused-selector`.
- **Plugin consumer example workspace** — the repo now includes `examples/plugin-consumers`, a clean dual-consumer setup for ESLint and Stylelint, plus `pnpm check:plugin-consumer-example` to verify that the example wiring stays valid.

### Changed

- **`3.13.0` is now framed as a lint-consumer plugin milestone** — the release narrative now includes user-facing plugin work rather than only internal Rust lane promotion. ESLint and Stylelint consumer surfaces are broad enough to batch as one plugin-facing release unit.
- **Plugin-facing release verification is now explicit** — `pnpm check:plugin-consumers` and `pnpm check:plugin-consumer-example` now sit in the release verification path so plugin regressions are exercised before packaging.

## [3.12.0] — 2026-04-21

### Added

- **Parser consumer-boundary check** — the Rust parser track now exposes `pnpm check:rust-parser-consumer-boundary`, which consumes the parser canonical-producer output into a bounded downstream-style summary and compares it directly against the current TS style HIR.

### Changed

- **`3.12.0` is now framed as a parser consumed-boundary Rust milestone** — `engine-style-parser` no longer stops at parser-internal boundary checks. The parser public-product gate now covers both the parser lane itself and one bounded downstream consumer over the canonical producer output.
- **Parser public-product gate now means lane plus consumer** — `pnpm check:rust-parser-public-product` now runs `pnpm check:rust-parser-lane` and `pnpm check:rust-parser-consumer-boundary`, and the broader Rust lane bundle inherits that stronger parser boundary.

## [3.11.0] — 2026-04-21

### Added

- **Parser canonical-candidate lane** — the Rust parser track now exposes a versioned parser canonical-candidate bundle and parser canonical-producer signal on top of the existing parity-lite and CSS Modules intermediate artifacts.
- **Dedicated parser canonical checks** — `pnpm check:rust-parser-canonical-candidate` and `pnpm check:rust-parser-canonical-producer` now validate the parser lane's promoted artifact boundary directly.

### Changed

- **`3.11.0` is now framed as a parser canonical-candidate Rust milestone** — the parser track no longer stops at a public-product gate plus bounded intermediate producer. `engine-style-parser` now carries an explicit canonical-candidate / canonical-producer ladder within the parser/public-product lane.
- **Parser lane bundle now includes promoted parser boundary checks** — `pnpm check:rust-parser-lane`, `pnpm check:rust-parser-public-product`, `pnpm check:rust-lane-bundle`, and release-facing Rust validation now run the parser canonical-candidate and canonical-producer steps as part of the parser lane.

## [3.10.0] — 2026-04-21

### Added

- **Parser public-product gate** — the Rust parser track now has a canonical `pnpm check:rust-parser-public-product` command that packages scaffold tests, bounded parser parity-lite, and CSS Modules intermediate producer validation behind one parser/public-product boundary.
- **Expanded parser CSS Modules intermediate facts** — the Rust parser intermediate now carries wrapper-aware `keyframes`, `value`, and `composes` source/target facts, including imported value-ref sources, `@value` dependency ownership, wrapper-scoped local/imported value refs, wrapper-scoped `composes` kind splits, and wrapper-scoped imported `composes` source paths.

### Changed

- **`3.10.0` is now framed as a parser public-product Rust milestone** — the release narrative no longer treats the parser track as a watch-only sidecar. `engine-style-parser` now has its own canonical public-product gate while still participating in the broader Rust lane bundle.
- **Release-facing Rust validation now names the parser lane explicitly** — `pnpm check:rust-lane-bundle`, `pnpm check:rust-release-bundle`, and `RELEASING.md` now refer to `pnpm check:rust-parser-public-product` as the canonical parser/public-product gate.

## [3.9.0] — 2026-04-20

### Added

- **Consolidated semantic Rust lane** — the Rust shadow path now exposes top-level semantic canonical-candidate bundles, evaluator candidates, and canonical-producer signals that unify the existing `source-side` lane with `expression-domain`.
- **Expanded semantic compare commands** — `pnpm check:rust-semantic-canonical-candidate`, `pnpm check:rust-semantic-evaluator-candidates`, and `pnpm check:rust-semantic-canonical-producer` now validate that top-level semantic lane directly against the TypeScript parity oracle.

### Changed

- **`3.9.0` is now framed as a consolidated semantic Rust milestone** — the release narrative moves one level above the source-side lane. `expression-semantics`, `source-resolution`, and `expression-domain` now participate in a single semantic lane, while `selector-usage` remains shadow-only.
- **Expression-domain evaluator coverage is now explicit and bounded** — `expression-domain` keeps its input-driven canonical artifacts, but evaluator-candidate checks are now called out as type-fact-backed coverage rather than a full-corpus semantic guarantee.

## [3.8.0] — 2026-04-20

### Added

- **Consolidated source-side Rust lane** — the Rust shadow path now exposes top-level source-side canonical-candidate bundles, evaluator candidates, and canonical-producer signals in addition to the existing family-level `expression-semantics` and `source-resolution` artifacts.
- **Expanded source-side compare commands** — `pnpm check:rust-source-side-canonical-candidate`, `pnpm check:rust-source-side-evaluator-candidates`, and `pnpm check:rust-source-side-canonical-producer` now validate the consolidated source-side lane directly against the TypeScript parity oracle.

### Changed

- **`3.8.0` is now framed as a consolidated source-side Rust milestone** — the source-side lane is no longer expressed only as per-family signals. It now has explicit top-level canonical and evaluator artifact layers, while `expression-domain` remains input-only and `selector-usage` remains shadow-only.
- **Rust producer-crate coverage is clean again under the new lane boundary** — `engine-input-producers` tests were realigned to the current certainty and selector-fragment mappings so the source-side lane lands on a green producer-crate baseline.

## [3.7.0] — 2026-04-20

### Added

- **Source-side canonical producer signals on the Rust shadow path** — `expression-semantics` and `source-resolution` now expose Rust-side evaluator-candidate bundles, canonical-candidate bundles, canonical-producer signals, and a consolidated source-side canonical-producer compare command for direct regression checks against the TypeScript oracle.

### Changed

- **`3.7.0` is now framed explicitly as a source-side-first Rust milestone** — the release narrative no longer waits for uniform all-family coverage. Source-side query families carry canonical-producer signals, while `expression-domain` and `selector-usage` remain shadow-validation families.
- **Release verification now includes Rust workspace and gate-evidence validation** — `pnpm release:verify` now runs the Rust workspace gate and the canonical `typescript-current` rust gate-evidence pass alongside the existing TypeScript build/test checks.

## [3.6.0] — 2026-04-19

### Added

- **Rust shadow validation now checks query-plan construction** — the internal Rust shadow runner now compares canonical V2 type-fact input, input-derived query plans, and parity summaries against the TypeScript engine to guard the Phase IV transition with explicit compare commands.
- **Rust workspace Stage 0 baseline** — the repository now includes a pinned Rust toolchain, shared workspace lint/format settings, and a `pnpm check:rust-workspace` gate so shadow-engine work lands behind stable repo-local validation from the start.

### Changed

- **V2 is now the canonical contract** — live parity helpers, type-fact assembly, and query/output assembly now run through the V2 path first, while V1 remains only as a historical compatibility view derived from V2.
- **V1 parity and host builders are now explicitly historical** — V1 parity scripts, host builders, and type-fact tables now live under historical naming and namespaces, making the active contract surface unambiguous in both code and release operations.
- **Release narrative now matches the actual migration state** — this release closes the V1 sunset waves and records the current Rust shadow validation baseline without overstating Rust as an independent semantic producer yet.

## [3.5.0] — 2026-04-18

### Added

- **V2 constrained contract exposure** — `TypeFactTableV2`, `EngineOutputV2.queryResults`, and `pnpm explain:expression --json` now expose constrained string facts across three rollout bundles: Bundle 1 (`suffix`, `prefixSuffix`), Bundle 2 (`charInclusion`), and Bundle 3 (`composite`).
- **V2 parity corpus and explain surface** — dedicated V2 smoke/golden fixtures now cover prefix-suffix, character-inclusion, and composite flow cases, and `explain:expression` prints the same structured V2 analysis metadata in both text and JSON modes.

### Changed

- **Pattern B contract rollout is now proven in production code** — V2 keeps a stable top-level schema (`exact | finiteSet | constrained | unknown | top`) while growing only sub-discriminators (`constraintKind`), which allowed Bundle 1/2/3 to land without reshaping the top-level contract.
- **V2 type facts now normalize directly** — the V2 path no longer routes through V1 upcasts. `TypeFactTableV2` is normalized directly while the frozen V1 path continues to run independently in parallel.
- **Reduced-product analysis is externally visible in V2** — the internal `suffix`, `prefixSuffix`, `charInclusion`, and `composite` domains now drive user-visible V2 metadata instead of remaining internal-only analysis details.

## [3.4.0] — 2026-04-17

### Added

- **Checker as a second consumer** — the workspace batch checker now has stable JSON report metadata, named bundles, presets, compact changed-file output, semantic smoke coverage, and contract parity smoke/golden fixtures.
- **Token-semantic first pass surfaces** — `@keyframes` and `@value` now participate in hover, definition, references, diagnostics, and recovery flows, including imported `@value` bindings between style modules.
- **Contract freeze baseline** — `EngineInputV1`, `EngineOutputV1`, `TypeFactTableV1`, `CheckerReportV1`, and selected query parity are now assembled and snapshot-tested for future Rust parity work. Richer string-fact domains remain a future `V2` concern rather than expanding the frozen `V1` shape.
- **Release batch corpus** — stable release verification now uses a curated clean batch-check corpus instead of a repo-wide `ci` pass, so intentional negative recovery fixtures do not block shipping.

### Changed

- **Internal server structure is now split by role** — the monolithic server tree is divided into `engine-core-ts`, `engine-host-node`, `lsp-server`, and `checker-cli` subprojects, matching the runtime, core, transport, and batch-consumer boundaries.
- **Release operations include contract validation** — release verification now includes contract parity smoke/golden checks and the release batch corpus in addition to the semantic smoke pass.
- **Current roadmap ranges are consolidated in one release** — this release rolls up the feature-closure, internal split, and contract-freeze work that landed across the 3.3/3.4/3.5/3.6 roadmap waves.

### Fixed

- **Prefix concatenation precision** — left-hand prefixes are now preserved under concatenation with exact values, finite sets, and other prefixes, preventing avoidable drops to `top` in common BEM-style patterns.

## [3.2.0] — 2026-04-15

### Added

- **Architecture hardening runtime split** — workspace execution is now explicitly divided into settings, analysis, and style runtimes, with a transport-agnostic runtime sink for logging, diagnostics clearing, and CodeLens refresh requests.
- **Incremental reference storage** — selector references, module usages, and dependency reverse lookups now update contribution-by-contribution instead of rebuilding whole derived maps on every record or forget.
- **Package-ready entry boundaries** — core query, rewrite, semantic, and runtime entrypoints are now explicit, with architecture tests enforcing dependency direction for future extraction into standalone engine packages.

### Changed

- **Runtime invalidation is now explicit** — watched-file classification, dependency snapshots, and invalidation planning are separated into dedicated runtime contracts instead of being assembled ad hoc inside handler wiring.
- **Semantic storage is now collector/store based** — reference contribution collection, reference storage, and dependency storage now have distinct responsibilities, which makes the runtime easier to reason about and cheaper to update incrementally.
- **Style rewrite policy is derived, not embedded** — rename and rewrite planning now consume a style rewrite policy summary instead of directly interpreting raw nested/BEM policy fields.
- **Provider boundaries are stricter** — providers read query/rewrite façades instead of deep semantic, binder, or runtime internals, and architecture invariant tests lock that boundary in place.
- **Examples QA matrix expanded again** — the sandbox now includes dedicated diagnostics-recovery, bracket-access, and `.module.less` coverage so the remaining runtime surfaces can be checked without ad hoc setup.

### Fixed

- **Local packaging from development checkouts** — `.worktrees/` and `.pnpm-store/` are now excluded from the VSIX, preventing `vsce package` failures and accidental bundling of local development artifacts.
## [3.1.1] — 2026-04-14

### Added

- **Multi-root workspace routing** — workspace folders now carry resource-scoped settings and path alias resolution independently, so mixed repos can use different CSS Modules conventions without restarting the server.
- **`composes` dependency graph** — cross-file and same-file `composes` edges now participate in selector usage, Find References, hover, definition, rename safety, and CodeLens.
- **Style-side inspect surface** — selector hover now reports usage and dependency context, `composes` tokens support hover/definition/references, and CodeLens titles distinguish composed and dynamic references.
- **Source and style dependency invalidation** — watched file changes now recompute only affected open documents for source imports, style dependencies, and settings-driven reanalysis.

### Changed

- **Stable promotion version** — the `3.1.0` version number was already consumed by the Marketplace pre-release channel, so the first stable cut of this feature line ships as `3.1.1`.
- **Compatibility path alias guidance** — the native `cssModuleExplainer.pathAlias` key is now the preferred setting; falling back to `cssModules.pathAlias` logs a deprecation notice per workspace root.
- **Examples sandbox expanded** — the manual QA matrix now includes dedicated `composes` coverage alongside the multi-root, shadowing, non-finite dynamic, and nested style fact scenarios.

### Fixed

- **Nested and composed style diagnostics** — unresolved `composes` modules/selectors now surface SCSS diagnostics, and missing composed modules offer the same create-file quick fix flow as missing source-side module imports.

## 3.0.0

### Major Changes

- Replace the old heuristic runtime with the 3.0 semantic pipeline: document facts, scoped binding, abstract class-value analysis, provider-facing read models, and generic rewrite planning now form the production path.
- Make source-side binding scope-aware across `cx`, `styles`, imports, locals, and shadowing instead of relying on line-range and document-order heuristics.
- Unify dynamic class reasoning under a shared abstract-value domain so flow, unions, template prefixes, and non-finite cases follow one contract.
- Move provider behavior onto explicit read models and rewrite policies, reducing provider-local semantic glue and removing the old semantic-graph-first runtime path.
- Expand the examples sandbox into a 3.0 manual QA matrix covering nested style facts, shadowing, and non-finite dynamic resolution.

### Patch Changes

- Fix nested `&.class` compound selector registration so classes introduced inside nested compounds resolve to the selector that actually introduced them without overwriting parent facts.

## 2.1.0

### Minor Changes

- [#7](https://github.com/yongsk0066/css-module-explainer/pull/7) [`0d9462a`](https://github.com/yongsk0066/css-module-explainer/commit/0d9462a76337d0c9a6fa5234b4b06f3ef84657c8) Thanks [@yongsk0066](https://github.com/yongsk0066)! - Add support for resolving CSS Module imports through `tsconfig.json` and `jsconfig.json`
  `compilerOptions.paths`, including wildcard aliases.

  This release also refreshes the examples sandbox with a dedicated tsconfig-path scenario
  so alias-import regressions are exercised outside the test harness.

## 2.0.1

### Patch Changes

- [#2](https://github.com/yongsk0066/css-module-explainer/pull/2) [`d8ee9bd`](https://github.com/yongsk0066/css-module-explainer/commit/d8ee9bda19f0107a8f4aebe4139a6f0eba182452) Thanks [@yongsk0066](https://github.com/yongsk0066)! - Refresh SCSS reference count code lenses when semantic reference data changes so reference counts stay in sync after source analysis.

All notable changes to this project will be documented in this
file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] — 2026-04-13

### Added

- **Semantic runtime across the feature set** — source and style analysis now run through HIR documents, a semantic graph, shared queries, and flow-aware resolution. Hover, definition, references, rename, diagnostics, code actions, and unused-selector checks all resolve through the same runtime path.
- **Missing-selector creation quick fix** — unresolved class diagnostics can now add the missing selector directly to the target CSS Module.
- **Missing-module file quick fix** — unresolved CSS Module import diagnostics can now create the missing module file from the code action menu.
- **Dynamic hover explanations** — hover now explains when a class reference was resolved through local flow, type-union fallback, or template-prefix expansion, including candidate lists for non-exact matches.
- **Explicit rename block reasons** — rename now returns concrete failure reasons for dynamic expressions, alias-only views, unsafe nested selectors, and non-direct reference cases.

### Changed

- **Legacy compatibility layers removed from runtime** — the extension no longer routes live behavior through legacy class-ref or class-map compatibility shells. The runtime is now semantic-first end to end.
- **Examples sandbox aligned with the workspace toolchain** — `examples/` now installs through the root workspace, uses a current React Vite plugin path, and ships with editor settings that match the baseline QA mode.
- **README rewritten** — the project overview, configuration, architecture, and development sections now reflect the current runtime and release shape.
- **Release workflows now sync server version before build** — CI and publish workflows run `scripts/release.sh` before building so `serverInfo.version` matches the packaged extension version.

## [1.8.0] — 2026-04-12

### Added

- **Bracket-access style references** — `styles['foo-bar']` element-access expressions are now recognized alongside `styles.fooBar` dot-access. Hover, definition, diagnostics, references, and rename all work through bracket syntax.
- **Dotted property chain resolution** — `cx(sizes.large)` where `sizes` is a `const` object with string-literal properties now resolves to the property's value. Works for local objects, named imports, default imports, namespace imports, and renamed imports.
- **Import-aware type resolution** — the TypeResolver now follows import bindings (`import { sizes } from './theme'`) through `checker.getAliasedSymbol`, enabling cross-file variable/template expansion in the reverse index.

### Fixed

- **Source-file save staleness** — saving a `.ts`/`.tsx`/`.js`/`.jsx` file now invalidates the TypeResolver's cached `ts.Program` and drops stale analysis-cache entries for all open source documents, so reverse-index expansions rebuild with fresh type data. Previously, type changes were invisible until server restart.
- **Reverse-index cascade on source change** — after a source-file watcher event, the analysis cache for open TSX/TS documents is invalidated so `onAnalyze` re-fires and the reverse index rebuilds. Without this, Find References, CodeLens, unused-selector diagnostics, and rename readiness stayed frozen against old type data.
- **Import shadowing regression** — `findIdentifierSymbol` now uses a local-first / import-fallback 2-pass strategy. A local parameter `sizes` correctly shadows an import with the same name, matching TypeScript's scoping rules in the common case.

### Changed

- **Watcher glob expanded** — file watchers now cover `.d.ts`, `tsconfig*.json`, and `jsconfig*.json` in addition to source files, so declaration and config changes also trigger TypeResolver invalidation.
- **SCSS parser split** — `scss-parser.ts` (was 436 lines) split into `scss-parser.ts` (pipeline) + `scss-selector-utils.ts` (pure utilities).
- **BEM suffix extraction** — `classifyBemSuffixSite` 6-parameter data clump collapsed into `BemParentContext` interface; BEM logic extracted to `core/scss/bem-suffix.ts`.
- **Rename module split** — `rename.ts` (372 lines) split into `rename/index.ts` + `rename/build-edit.ts`.
- **AliasResolverHolder extraction** — shared-closure pattern extracted from inline composition-root code to a standalone class in `core/cx/alias-resolver.ts`.
- **Lint cleanup** — all 9 lint warnings resolved (0 warnings, 0 errors).

## [1.7.0] — 2026-04-12

### Fixed

- **Find References + CodeLens under `classnameTransform`** — under `camelCase` or `camelCaseOnly` modes, Find References from a SCSS selector returned empty results and CodeLens showed `0 references` or rendered duplicate lenses. The reverse-index query now routes through the canonical SCSS selector name regardless of which alias view the cursor sits on, and CodeLens deduplicates entries by canonical name so each logical class renders exactly one lens.
- **SCSS diagnostics refresh on `classnameTransform` change** — switching the transform mode in settings left open `.module.scss` files with stale unused-selector diagnostics until the user edited the file. The reload handler now routes each open document to the right scheduler method by language, so SCSS unused-selector checks recompute immediately on a mode change.
- **Reverse-index memory leak on document close** — closing a TSX file did not drop its contribution from the workspace reverse index. On a long session the index grew unbounded, and a SCSS unused-selector check run after close still treated the closed file's references as live. The `onDidClose` listener now calls `reverseIndex.forget(uri)`.
- **Unicode class-name identifiers** — selectors like `.한글`, `.日本語`, or `.español-btn` were silently dropped from the class map because the extraction regex was ASCII-only. Widened to Unicode property classes (`\p{L}`, `\p{N}`, `\p{M}`) so every script CSS Modules accepts survives, including NFD-decomposed combining marks.

### Changed

- **`canonicalNameOf` helper** — the `info.originalName ?? info.name` pattern (5 call sites across references, reference-lens, scss-diagnostics, rename, and reverse-index) is extracted to a single `canonicalNameOf(info)` function in `classname-transform.ts`.
- **Exhaustive ClassRef dispatch** — three switch statements over the `ClassRef` discriminated union (`hover-renderer`, `diagnostics`, `reverse-index`) now carry `never` sentinel defaults so a future union extension fails at compile time instead of silently falling through.
- **Configuration table rewritten** — the README settings section is rebuilt from `package.json contributes.configuration` as the source of truth. Six fictional settings removed; ten real settings documented.
- **CHANGELOG backfill** — 1.2.0 and 1.3.0 entries added from git history; 1.1.0 jargon rewritten.
- **Benchmark wired through real parsers** — `providers.bench.ts` now measures the actual `scanCxImports` + `parseClassRefs` AST walkers instead of hardcoded stubs, and delegates ProviderDeps construction to `makeBaseDeps` so the shape stays current.

### Removed

- **Dead `ProviderDeps.aliasResolver` field** — no provider consumed it. The alias resolver the analysis cache depends on is wired separately through `DocumentAnalysisCacheDeps`.
- **Section-divider comments** — `// ───` horizontal-rule comments removed from 7 files.

## [1.6.0] — 2026-04-11

### Added

- **Missing CSS Module diagnostic** — `import styles from './typo.module.scss'` now emits a `missing-module` warning when the target file does not exist on disk. Fires for any file with a CSS Module import, including pure `styles.x` access without `classnames/bind`. Configurable via `cssModuleExplainer.diagnostics.missingModule` (default `true`).
- **Path alias compat — `cssModules.pathAlias`** — the `cssModules.pathAlias` config is read as-is, so `import styles from '@styles/button.module.scss'` resolves when the workspace has `"cssModules.pathAlias": { "@styles": "src/styles" }` in its settings. Existing workspace settings continue to work without migration. Alias matching uses longest-prefix order, so `{ "@": "src", "@styles": "src/styles" }` correctly routes `@styles/button` to `src/styles/button` regardless of config key order. `${workspaceFolder}` substitution is supported. Wildcards and tsconfig.json `compilerOptions.paths` auto-detection are not yet supported — tracked for a future release.
- **`classnameTransform` setting** — expose SCSS classes under five modes matching css-loader's `localsConvention`: `asIs` (default, unchanged behavior), `camelCase` (original + camelCase alias), `camelCaseOnly` (camelCase only), `dashes` (original + dashes-to-camel alias), `dashesOnly` (dashes-to-camel only). With `camelCase` active, both `styles['btn-primary']` and `styles.btnPrimary` resolve against a single `.btn-primary` selector. Alias entries participate in BEM-safe rename — renaming `btnPrimary` rewrites the original `.btn-primary` token in SCSS and every call site in TSX, with each site getting the form that matches how it accesses the class. `camelCaseOnly` and `dashesOnly` reject alias rename because the reverse transform from camelCase back to the original SCSS separator is lossy; use `camelCase` / `dashes` for editor-driven rename workflows. Configurable via `cssModuleExplainer.scss.classnameTransform` (default `"asIs"`).

## [1.5.1] — 2026-04-11

### Added

- **`&`-nested BEM rename** — `.button { &--primary {} }` and `.button { &__icon {} }` can now be renamed directly from the SCSS selector. Only the `--primary` / `__icon` suffix slice is rewritten in the SCSS file; every `cx('button--primary')` reference in the workspace updates in lockstep. Compound nested forms (`&.active`), pseudo (`&:hover`), grouped parents, non-bare parents (`.card:hover { &--x }`), grouped-nested children (`&--a, &--b`), and multi-`&` tokens remain safely rejected.

### Fixed

- **Latent corruption in `&`-nested range fallback** — previously, `SelectorInfo.range` for `&`-nested entries was a synthesized column that could span past the nested token into whitespace. Earlier releases defensively rejected rename on those entries; 1.5.1 computes the correct raw-token range using postcss absolute source offsets, eliminating the fallback path entirely.

## [1.5.0] — 2026-04-11

### Fixed

- **Rename no longer corrupts template literals** — `cx(\`btn-${weight}\`)` style calls were silently rewritten when a referenced class was renamed, destroying the template source. The reverse index now distinguishes direct and synthesized entries; rename filters out synthesized ones while Find References keeps them. Both the TSX-side and SCSS-side prepareRename now reject classes with template/variable references uniformly.
- **Incremental file updates no longer dropped after initial indexing** — `IndexerWorker.pushFile()` was inert once the initial workspace walk finished, so file-watcher events silently fell on the floor. A new `PushSignal` async-iterable replaces the old signal so `drain()` parks on incoming push events via `for await` without exiting.
- **SCSS diagnostics now reflect unsaved edits** — `classMapForPath` consults the in-memory `TextDocuments` buffer before falling back to disk, so unused-selector and unknown-class diagnostics respond immediately to unsaved SCSS changes.
- **`&`-nested SCSS rename rejected defensively** — the parser now flips `SelectorInfo.isNested = true` when the raw source contained `&`, and `rename` returns `null` for those entries instead of rewriting the synthesized fallback range.
- **Reverse-index staleness on SCSS file changes** — when a SCSS module gained or lost a class, cached TSX analysis entries were not invalidated, leaving template/variable expansions stale until the user touched the TSX buffer. `onDidChangeWatchedFiles` now invalidates every TSX entry that referenced the changed SCSS path before rescheduling diagnostics.
- **Invalid user config values no longer leak through untyped** — the settings loader validates inputs via explicit type guards and falls back to defaults for wrong types, unknown severities, `NaN`, `Infinity`, etc.

### Changed

- **Unified `ClassRef` domain model** — legacy `CxCallInfo` and `StylePropertyRef` types collapsed into a single `ClassRef` discriminated union (`static | template | variable`, tagged with `origin: "cxCall" | "styleAccess"`). Every provider now dispatches through a single `withClassRefAtCursor` front stage; the parallel `withCxCallAtCursor` / `withStyleRefAtCursor` dispatch pattern is gone.
- **Error boundary at every LSP handler** — new `wrapHandler(name, impl, fallback)` helper wraps each provider export with a try/catch + `logError` + safe default. The nine hand-rolled try/catch blocks in individual providers are deleted.
- **Single-source DI** — `HandlerContext.getBundle()` and the `CompositionBundle` interface deleted; the four style-index / indexer capabilities (`invalidateStyle`, `pushStyleFile`, `indexerReady`, `stopIndexer`) are now flat fields on `ProviderDeps`.
- **Completion pipeline collapsed** — the two parallel pipelines added in v1.4.0 for `classnames/bind` vs `clsx / classnames` are unified into a single `findCompletionContext` helper that iterates once over bindings and style imports. `isInsideCxCall` renamed to `isInsideCall`. `detectClassUtilImports` moved to the binding-detector layer and exposed via `AnalysisEntry.classUtilNames`.
- **Binding detector single-walk** — `collectImports` now makes exactly one pass over `sourceFile.statements` instead of two.
- **Dead code removed** — `FileTask.kind`, `IndexerWorkerDeps.onTsxFile`, `buildStyleImportRegex`, and every `@deprecated` legacy type marker deleted.
- **Type assertions minimized** — zero `as` casts in the server tree outside the documented `getRuntimeSyntax` helper (the single `unknown → postcss.Syntax` narrowing) and the `as const` / `as readonly` widenings. `parseSettings` uses type guards. `scss-parser.ts` relies on postcss's discriminated union directly. `CreateServerOptions` is a discriminated union of `"auto" | "streams"` transports. `ShowReferencesArgs` is a shared tuple type; the client middleware narrows via a single `isShowReferencesArgs` guard instead of three `as` casts.
- **Incremental release tooling** — new `scripts/release.sh` syncs `SERVER_VERSION` with `package.json` before the build.

## [1.4.0] — 2026-04-11

### Added

- **clsx / classnames support** — Autocomplete, hover, and go-to-definition for `clsx(styles.btn)` and `classNames(styles.btn)` patterns, alongside the existing `classnames/bind` support.
- **Unused selector detection** — CSS class selectors in `.module.scss` files with zero references are flagged with `DiagnosticTag.Unnecessary` (faded text). Template and variable call sites suppress false positives; `composes` references are honored.
- **Rename Symbol** — Bidirectional rename between `.module.scss` selectors and `cx('className')` / `styles.className` references across the workspace. `&`-nested SCSS selectors are rejected in this release.
- **Example scenario 10-clsx** — New sandbox scenario demonstrating `clsx(styles.btn, isActive && styles.active)`.

### Fixed

- **styles.x now works in files without classnames/bind** — Extracted style-import scanning from the cx binding detector so `styles.className` hover and go-to-definition work in any file with a `.module.*` import, regardless of whether `classnames/bind` is used.

### Changed

- **CallSite type narrowed** — Internal `CallSite.binding: CxBinding` and `CxCallBase.binding: CxBinding` replaced with `scssModulePath: string`. Eliminates synthetic binding objects and narrows the dependency graph.
- **Diagnostics scheduler extracted** — Debounce and index-readiness gating moved out of `handler-registration.ts` into a dedicated module.
- **Test fixtures consolidated** — `test/_fixtures/test-helpers.ts` exposes shared `makeBaseDeps`, `info`, `infoAtLine`, and `siteAt` helpers. All provider test files migrated.

### Configuration

- Added `cssModuleExplainer.diagnostics.unusedSelector` (default: `true`).
- Added `cssModuleExplainer.features.rename` (default: `true`).

## [1.3.0] — 2026-04-10

### Fixed

- **Multi-line `cx()` calls now register** — class literals in
  `cx()` calls spanning more than one line are captured by the
  AST walker, so every line in a multi-line argument list
  participates in hover, completion, references, and diagnostics.
- **Reference CodeLens on class selectors** — the "N references"
  CodeLens above every `.module.scss` class selector is now
  wired through `textDocument/codeLens` and opens VS Code's
  built-in references panel on click.
- **Empty reference lenses suppressed** — classes with zero
  references no longer emit a `"0 references"` lens; the lens
  is omitted entirely so the editor gutter stays clean.

## [1.2.0] — 2026-04-10

### Added

- **LESS support** — `.module.less` files parse through
  postcss-less; every provider that works on `.module.scss`
  works on `.module.less`.
- **Namespace imports** — `import * as styles from './x.module.scss'`
  is recognised alongside the default-import form.
- **String-aware completion gating** — completion no longer
  triggers inside string literals that happen to be passed to a
  `cx()` call, avoiding spurious popups inside quoted content.
- **Direct `styles.x` access** — hover, definition, and
  completion work on `styles.className` property access in any
  file, independent of whether `classnames/bind` is imported.
- **Template reverse-index expansion** — template-literal and
  variable-kind call sites (e.g. `` cx(`btn-${weight}`) ``) are
  expanded against the class map at index time, so Find
  References on a selector surfaces every dynamically-referenced
  site.
- **`cx(props.variant)` property access** — bare property-
  access identifiers passed to `cx()` resolve against the same
  TypeScript string-literal union machinery used for plain
  variables.
- **`composes:` declarations** — SCSS classes that compose from
  a sibling class (same-file or `from '.otherFile.module.scss'`)
  are treated as used by the unused-selector check, preventing
  false-positive hints.
- **Grouped selector support** — `a, b {}` rules now contribute
  both `a` and `b` to the class map with their own source
  ranges, so hover and go-to-definition pick the right selector.
- **Settings schema** — first `contributes.configuration` entry
  in `package.json` exposes user-facing settings through the VS
  Code settings UI. Per-feature toggles and diagnostic
  configuration land in this release.

### Fixed

- **Levenshtein suggestion bounded** — the "did you mean?" hint
  in diagnostics uses a bounded-edit Levenshtein with early
  termination so very long class names do not slow the check.

### Changed

- **Module resolution switched to bundler** — server and
  shared packages compile under `"moduleResolution": "Bundler"`,
  removing the `.js` extension suffixes from internal imports.
- **Node engine bumped to 24** — `engines.node` set to `>=24`;
  `engines.vscode` pinned to `^1.115.0`.
- **Shared LruMap utility** — `StyleIndexCache`,
  `SourceFileCache`, and `DocumentAnalysisCache` delegate
  eviction to a shared `LruMap<K, V>` helper, removing three
  identical inline implementations.
- **Shared `FakeTypeResolver` fixture** — fourteen inline copies
  of a fake `TypeResolver` across provider tests collapsed into
  a single `test/_fixtures/fake-type-resolver.ts`.
- **SCSS index module split** — `scss-index.ts` separated from
  the parser file so `StyleIndexCache` and `parseStyleModule`
  live in distinct modules.
- **Composition root split** — the startup factory split out
  settings, scheduler, indexer, and type-resolver factories so
  the root stays a thin DI wire-up.
- **Release workflow** — CI publishes to the VS Code marketplace
  on tagged releases.
- **`examples/scenarios/*`** — all nine scenario sub-packages
  fully implemented with dedicated README walkthroughs.

## [1.1.0] — 2026-04-10

### Changed

- **LRU cache refactor** — `StyleIndexCache`, `SourceFileCache`,
  and `DocumentAnalysisCache` now delegate eviction logic to a
  shared `LruMap<K, V>` utility, removing three identical
  `private put()` methods.
- **hover.ts cleanup** — duplicated synthetic binding object
  extracted to a local variable; `kind: "static" as const`
  added for type safety.
- **CI pipeline** — `dist/` build artifact is uploaded in the
  `check` job and downloaded in `package`, eliminating a
  redundant `pnpm build`. Added `concurrency` (cancel
  in-progress), top-level `permissions: { contents: read }`,
  and `if-no-files-found: error` on both artifact uploads.

### Fixed

- **Internal planning references removed** — stale describe-block
  names, comments, and doc strings that referenced internal
  project-phase shorthand were rewritten in neutral language so
  external readers of the test suite are not confronted with
  workflow jargon.

## [1.0.0] — 2026-04-10

First marketplace-ready release. Everything below was built
from scratch in a single sprint; there is no prior
production history to migrate from.

### Added

**Providers** (all dispatched through a single front-stage
`withCxCallAtCursor` + a per-(uri, version) analysis cache so
hot paths stay under 1 ms):

- **Definition provider** (`textDocument/definition`) —
  `LocationLink[]` with origin, target, and target selection
  ranges so VS Code's peek view highlights correctly.
- **Hover provider** (`textDocument/hover`) — markdown card
  with workspace-relative source location, formatted SCSS
  rule, and a multi-match layout capped at 10 candidates.
- **Completion provider** (`textDocument/completion`) —
  triggered on `'`, `"`, `` ` ``, `,`; emits one
  `CompletionItem` per class in the bound classMap with a
  live markdown preview in the documentation field.
- **Diagnostics provider** (`textDocument/publishDiagnostics`)
  — 200 ms debounced push model with per-call error
  isolation. Unknown static classes emit warnings with a
  "did you mean?" hint (Levenshtein ≤ 3). Template prefix
  mismatches and partial union mismatches are reported with
  distinct messages.
- **Code actions provider** (`textDocument/codeAction`) —
  `quickfix` actions consuming the diagnostic's
  `data.suggestion` payload. One-click rename.
- **References provider** (`textDocument/references`) — runs
  on class selectors inside `.module.{scss,css}` files,
  returns every `cx()` call site workspace-wide.
- **Reference code-lens** (`textDocument/codeLens`) — inline
  "N references" counter above every class selector, linked
  to VS Code's built-in references panel.

**Parsing and indexing**:

- **SCSS index** — `parseStyleModule` + `StyleIndexCache`
  cover edge cases: `:global`/`:local` selectors, `&`
  ampersand nesting, group selectors, cascade last-wins
  duplicate handling, `@keyframes` / `@font-face` exclusion,
  `@media`/`@at-root` unwrapping.
- **`cx` binding detector** — AST-based two-pass scanner
  over the TypeScript `ts.SourceFile`; recognizes free
  identifier names, aliased imports
  (`import cn from 'classnames/bind'`), multi-binding per
  file, function-scoped bindings with tracked scope ranges.
- **`cx` call parser** — seven-branch AST dispatch (string
  literal, object literal, `&&` / `?:` conditionals,
  template literal, identifier, array literal, spread).
  Multi-line is free (AST is line-agnostic).
- **Call resolver** — pure dispatch by call kind: static →
  `classMap.get`; template → prefix filter; variable →
  `TypeResolver.resolve` + union member filter.
- **TypeScript 2-tier strategy** — in-flight
  `SourceFileCache` for live editor text (ms-scale parses)
  plus a workspace `TypeResolver` that lazily builds
  `ts.Program` instances keyed on the tsconfig root for
  string-literal union resolution.
- **Document analysis cache** — single-parse hub keyed on
  `(uri, TextDocument.version)` with a content-hash
  fallback for the "version bumped but content is
  identical" edge case. `onAnalyze` hook fires the reverse
  index write exactly once per (uri, version) — never on
  provider hot paths.
- **Workspace reverse index** — `(scssPath, className) →
CallSite[]` forward map plus a `uri → keys` back
  pointer for O(|callSites|) `forget(uri)` on document
  close. Static call kinds only; template/variable are
  explicitly skipped for this release.
- **Indexer worker** — cancellable background walker
  built on `fast-glob` streams and a `for-await` +
  sync-drain pattern. Yields to the event loop via
  `node:timers/promises.setImmediate` between files so
  LSP requests always preempt.
- **File watcher** — dynamic `DidChangeWatchedFiles`
  registration gated on client capability. Invalidates
  `StyleIndexCache` + re-queues the changed file through
  the indexer + reschedules diagnostics for every open
  document.

**Composition root**:

- Single `createServer({ reader, writer, overrides })`
  factory. Production entrypoint passes `process.stdin`
  / `process.stdout`; Tier 2 tests pass paired
  `PassThrough` streams wrapped in
  `StreamMessageReader`/`Writer` to bypass
  vscode-languageserver's auto-`process.exit` handlers.

**Tooling**:

- **pnpm** workspace (`shared`/`server`/`client` +
  private `examples/`).
- **TypeScript 6.0.2** with `NodeNext` module resolution
  and strict mode across the board.
- **Rolldown 1.0.0-rc.15** bundler producing CJS output
  for the VS Code extension host.
- **Vitest 4.1** with two test tiers (`unit/`,
  `protocol/`) and a `bench/` perf suite.
- **oxlint 1.59** + **oxfmt 0.44** replacing ESLint and
  Prettier. Zero `eslint-disable` comments in the source
  tree.

**Quality gates**:

- 253 Tier 1 + Tier 2 tests, 0 lint warnings, build clean.
- Cold hover ~0.029 ms, cold definition ~0.028 ms, cold
  completion ~0.013 ms.

### Known limitations

- Template-literal and variable-kind call sites are NOT
  in the reverse index for 1.0. Find References works on
  static calls only; a follow-up will resolve template
  prefixes and union members to individual class names
  before recording.
- Diagnostics do NOT emit a warning for a missing SCSS
  file (e.g. after a delete). They silently skip the
  document until the file reappears.
- `isInsideCxCall` (used by completion gating) is a naive
  paren-depth walker — it does not understand string or
  comment context. Edge cases like `cx(')')` return
  slightly wrong answers.
- Tier 3 E2E (real VS Code via `@vscode/test-electron`)
  is deferred; the release relies on manual dogfooding
  via `examples/`.

## [0.0.1] — 2026-04-09

Repository scaffolding. Not published.
