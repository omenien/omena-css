# Architecture

Last code-level revisit: 2026-07-19.

omena-css has one architectural problem: CSS-family facts, source-language
bindings, diagnostics, transforms, and editor features must answer the same
semantic question consistently. A feature should not change its answer because
it entered through the CLI, LSP, NAPI, or WASM surface.

## System Map

```text
CSS / SCSS / Sass / Less text
  -> omena-parser tokens and CST
  -> omena-cst-typed projections
  -> semantic, resolver, cascade, value, and cross-file facts
  -> omena-query read and decision facade
  -> CLI | Rust LSP | NAPI | WASM | build adapters

JS / TS and component source
  -> Rust source frontend + tsgo type facts
  -> source/style identity and occurrence facts
  -> the same omena-query facade
```

The primary engine is Rust. TypeScript remains an adapter surface for the VS
Code host, protocol orchestration, compatibility checks, and generated contract
consumption. It is not a second semantic authority.

## Crate Families

`rust/omena-product-path-matrix.json` is the machine-checked role inventory. Its
current product path spans 41 crates; names are grouped here by responsibility,
not by dependency order.

Syntax, identity, and fact authority:

- `omena-syntax`, `omena-parser`, `omena-cst-typed`, `omena-semantic`
- `omena-resolver`, `omena-engine-input-producers`, `omena-bridge`, `omena-sif`
- `omena-tsgo-client`, `omena-interner`, `omena-product-hints`

Semantic policy and query composition:

- `omena-abstract-value`, `omena-value-lattice`, `omena-scss-eval`
- `omena-cascade`, `omena-cascade-proof`, `omena-refinement`, `omena-refinement-trait`
- `omena-cross-file-summary`, `omena-streaming-ifds`, `omena-incremental`
- `omena-evidence-graph`, `omena-checker`, `omena-query-core`
- `omena-query-checker-orchestrator`, `omena-query-transform-runner`, `omena-query`

Transform, build, and editor runtime:

- `omena-transform-cst`, `omena-transform-passes`, `omena-transform-target`
- `omena-transform-print`, `omena-transform-egg`, `omena-bundler`
- `omena-lsp-server`

Shipped entry points and bindings:

- `omena-cli`, `omena-zk-audit`, `omena-napi`, `omena-wasm`

Support and aggregation:

- `omena-meta-macros`, `omena-transform-bundle`, `omena-umbrella`

Check/evidence crates, the retained `engine-style-parser` oracle, and research
fixtures remain outside this product-path count. They can challenge or measure
the engine but cannot become semantic authorities by being present in the
workspace.

## Authority And Projection

The parser owns tokenization, concrete syntax, spans, dialect routing, and
parser-derived style facts. `omena-cst-typed` projects typed views from that CST;
consumers do not recover equivalent facts with request-local raw scans. The
syntax census and egress closure enforce this absence:

- `scripts/check-rust-omena-syntax-authority-raw-scan-census.ts`
- `scripts/check-rust-cst-typed-egress-closure.ts`

Facts state what is present. Semantic policy decides what those facts mean.
Binding, resolution, cascade, abstract values, diagnostics, and transform
admission therefore sit above parser facts instead of being embedded in the
lexer or duplicated in providers.

The same rule still applies to the TypeScript compatibility layer. Source
binding belongs in
`server/engine-core-ts/src/core/binder/source-binder.ts`; providers consume its
result rather than rebuilding import or selector meaning.

## One Owner Per Question

- Syntax and spans belong to `omena-parser` and `omena-cst-typed`.
- Canonical file/package identity belongs to `omena-resolver`.
- Cascade order and winner evidence belong to `omena-cascade`.
- Dynamic class/value approximation belongs to `omena-abstract-value` and
  `omena-value-lattice`.
- Cross-file reachability belongs to summary and IFDS substrates.
- User-facing read and decision models belong to `omena-query`.
- Transport shaping belongs to the consuming CLI, LSP, or binding.

The abstract class-value domain currently has ten variants: Bottom, Exact,
FiniteSet, Automaton, Prefix, Suffix, PrefixSuffix, CharInclusion, Composite,
and Top. Projection preserves precision and provenance; callers do not collapse
the domain to a boolean before policy has made its decision.

## Query And Read Boundaries

`rust/crates/omena-query/src/lib.rs` is the product facade over parser,
resolution, semantic, checker, and transform substrates. Consumers ask it for
stable summaries and typed outcomes rather than traversing every lower-level
store directly.

This boundary keeps providers thin:

- lower layers own facts and proofs,
- the query layer owns composition and fail-closed outcomes,
- product adapters own URI/range conversion, rendering, and transport envelopes.

The generated Engine V2 IDL keeps cross-language JSON shapes synchronized;
runtime dependency bags and host callbacks stay outside the wire contract.

## Product Entry Points

### Rust LSP

`rust/crates/omena-lsp-server/src/message_loop.rs` routes protocol messages while
`rust/crates/omena-lsp-server/src/state.rs` owns workspace/document state.
Request adapters consume query summaries; scheduling, cancellation, snapshots,
and response serialization remain LSP responsibilities. The server must not
reparse CSS semantics merely because a request arrived through JSON-RPC.

### Unified CLI

`rust/crates/omena-cli/src/commands.rs` defines the command grammar and
`rust/crates/omena-cli/src/dispatch.rs` routes it. Domain modules call the same
query, checker, transform, bundle, provenance, and write-safety contracts used
elsewhere. JSON envelopes are transport products, not alternate semantics.

### SDK And Bindings

`rust/omena-sdk-workflow-parity-matrix.json` covers snapshot, query,
diagnostics, build, and explain workflows across four surfaces: NAPI, WASM,
CLI, and LSP. Host capabilities differ, but shared workflows and typed error
classes must remain semantically aligned.

### TypeScript Adapters

`client/src/extension.ts` starts and supervises the Rust server, registers VS
Code UI, and forwards configuration/file events. Generated TypeScript contracts
and shadow checks protect compatibility; they do not make the Node host a
fallback semantic engine.

## Dependency Direction

Dependencies flow from product entry points toward query/policy crates and then
toward fact/identity crates. Theory and evidence crates do not acquire inward
dependencies on product facades without an explicit reviewed exception.

The direction is enforced by:

- `scripts/check-rust-product-path-matrix.ts`
- `scripts/check-rust-layer-dependency-exceptions.ts`
- `test/unit/architecture/package-boundaries.test.ts`

The TypeScript boundary test also prevents core packages from importing provider
or runtime layers. An allowlist is a reviewed debt record, not permission to add
an untracked cycle.

## Invalidation Is A Contract

Workspace state is revisioned. Open-document changes, watched files, resolution
inputs, SIF artifacts, and source type facts invalidate named inputs; snapshots
pin a read view. Incremental reuse is correct only when the result matches a
fresh computation for the same revision.

The LSP may defer optimizing work, but it publishes baseline and final results
under revision/coalescing rules. A stale worker cannot overwrite a newer edit,
and a read view cannot become a live mutable-host escape hatch. These are
correctness properties first and latency optimizations second.

Session hosting follows the same contract: `omena-query` is the single semantic
authority, and the resident `omenad` process is an optional transport that is
never a second engine and never mandatory. Session identity is the workspace
root plus the resolved-config digest, and snapshot-named requests fail closed
when the snapshot is stale.

Workspace routing, crate ownership, and wire ownership are documented in these
gate-backed maps:

- [Workspace session routing](docs/internals/workspace-session-routing.md)
- [Crate boundary review](CONTRIBUTING.md#proposing-a-new-crate-boundary)
- [Engine V2 contract IDL decisions](server/engine-core-ts/src/contracts/engine-v2-contract-idl-decisions.md)

## Invariants As Absence

Healthy layering is often demonstrated by what production code does not do:

- no second CSS tokenizer or CST authority in a consumer,
- no provider-local reconstruction of binding or cascade semantics,
- no transport-specific fork of query results,
- no facade dependency pulled into a lower fact crate without review,
- no source write that bypasses FixSafety and write evidence,
- no stale revision publishing over a current LSP result,
- no research fixture or legacy oracle silently promoted to product authority.

## Current Intentional Limits

- `engine-style-parser` remains a differential/benchmark oracle until its
  retirement evidence is complete; it is not a product parser path.
- Sass compilation is delegated to the pinned Dart Sass authority. omena-css
  owns analysis, interfaces, bounded evaluation, and compatibility evidence,
  not a claim of complete Sass compiler replacement.
- Static analysis preserves typed unknown or unresolved outcomes for dynamic
  source, external tools, and unsupported grammar instead of guessing.
- Four-surface SDK parity covers named workflows and error classes; it does not
  erase host-specific capabilities.
- Formal and research substrates strengthen evidence where wired, but their
  existence is not a theorem-complete product claim.
