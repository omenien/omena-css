# `omena-query`

Internal Rust crate for the Omena query boundary.

This crate owns the consumer-facing query surface that groups producer query
fragments with the abstract-value projection contract.
Source-resolution wrappers now route through `omena-resolver` so resolver
ownership can move independently while query output contracts stay stable.

Current public products:

- `omena-query.boundary` — summary of the query boundary and delegated
  producer fragment surfaces.
- `omena-query.fragment-bundle` — grouped expression semantics, source
  resolution, and selector usage query fragments.
- `omena-query.selected-query-adapter-capabilities` — runtime-backed backend
  capability matrix and engine-shadow-runner command contract for the current
  selected-query adapter path, including the expression semantics payload
  contracts exposed to downstream query consumers.
- selected-query query fragment wrappers for expression semantics, source
  resolution, and selector usage runner commands.
- selected-query canonical producer wrappers for source resolution,
  expression semantics, and selector usage runner commands. These keep the
  existing JSON output contracts stable while moving ownership into
  `omena-query`.
- selected-query source-resolution runtime index wrapper for the
  `input-omena-resolver-source-resolution-runtime` runner command. This exposes
  the resolver-owned expression-to-selector runtime product through the selected
  query boundary.
- selected-query expression-domain flow analysis wrapper for the
  `input-expression-domain-flow-analysis` runner command. This exposes the
  `omena-abstract-value` flow product through the query boundary while keeping
  the lower-level product name stable.
- selected-query expression-domain control-flow analysis wrapper for the
  `input-expression-domain-control-flow-analysis` runner command. This exposes
  CFG-aware abstract-value analysis through the selected-query boundary.
- selected-query expression-domain incremental flow runtime for the
  `input-expression-domain-incremental-flow-analysis` runner command. This
  keeps per-graph `omena-incremental` Salsa databases alive across daemon
  requests so repeated query analysis can reuse clean graph results.
- `omena-query.evaluation-runtime` — runtime-backed selected-query execution
  summary that ties the adapter capability matrix, resolver runtime index,
  expression-domain Salsa runtime, and parser-owned style-document summary
  source into one decoupled runner command.
- selected-query style semantic graph adapter wrappers. These preserve the
  `omena-semantic.style-semantic-graph` products while delegating graph assembly
  to `omena-bridge`, including the `omena-semantic.css-modules-semantics`
  per-file seed surface.
- `omena-query.style-context-index` — consumer read model for the
  semantic-owned `@layer`, `@container`, and `@scope` context index.
- `omena-query.diagnostics-for-file` — file-scoped diagnostics read model for
  style diagnostics and cross-language source missing-selector diagnostics.
- `omena-query.completion-at` — position-scoped completion read model for
  style-side token completions and bridge-aware source selector completions.
- `omena-query.refs-for-class` — workspace-scoped selector reference read model
  for CSS Module definitions and source references.
- `omena-query.rename-plan` — workspace edit read model for selector rename
  plans across CSS Module definitions and source references.
- `omena-query.css-modules-cross-file-resolution` — batch-level CSS Modules
  relation resolver for `composes`, `@value`, and ICSS import/export sources.
  This resolves import sources, same-edge name matches, transitive closure, and
  cycle detection for the current parser fact surface.
- `omena-query.sass-module-cross-file-resolution` — batch-level Sass module
  resolver for parser-owned `@use`, `@forward`, and `@import` facts. This now
  resolves module graph closure, cycle detection, and `@forward show/hide`
  visibility filters without re-scanning style sources in the query layer.
- `omena-query.transform-plan` — post-v5 omena-css transform facade that combines
  bundle planning, target lowering, optional egg rewrite planning, CSS printing,
  and the current transform execution runtime summary.
- `omena-query.consumer-check-style-source` and
  `omena-query.consumer-build-style-source` — stable consumer facades used by
  `omena-cli`, `omena-wasm`, and `omena-napi` so those crates do not depend on
  parser or transform crates directly.

Primary check:

```sh
pnpm cme-check run rust/omena-query/boundary
```

Boundary ownership check:

```sh
pnpm cme-check run rust/omena-query/runner-boundary
```

Split boundary check:

```sh
pnpm cme-check bundle rust/omena-query/split-boundary
```
