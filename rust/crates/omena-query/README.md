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
- selected-query style semantic graph adapter wrappers. These preserve the
  `omena-semantic.style-semantic-graph` products while delegating graph assembly
  to `omena-bridge`, including the `omena-semantic.css-modules-semantics`
  per-file seed surface.
- `omena-query.css-modules-cross-file-resolution` — batch-level CSS Modules
  relation resolver for `composes`, `@value`, and ICSS import/export sources.
  This resolves import sources, same-edge name matches, transitive closure, and
  cycle detection for the current parser fact surface.

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
