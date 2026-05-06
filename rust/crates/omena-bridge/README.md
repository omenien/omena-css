# `omena-bridge`

Internal Rust crate for CME-coupled bridge surfaces around Omena semantic graph
products.

`omena-semantic` still owns the generic semantic boundary and keeps legacy graph
entry points for compatibility. This crate is the new boundary for entry points
that combine generic style semantics with CSS Module Explainer source inputs
such as `EngineInputV2`.

Current public products:

- `omena-bridge.cme-semantic-bridge` — bridge boundary summary describing the
  CME-coupled surfaces that should move behind this crate.
- `omena-semantic.style-semantic-graph` — bridge-assembled graph product, kept
  stable for existing host consumers while graph assembly moves behind this
  crate.
- `omena-semantic.selector-references` — bridge-owned selector reference engine
  product, kept stable for existing host consumers while ownership moves behind
  this crate.
- `omena-semantic.design-token-semantics` — generic design-token readiness
  surface forwarded from `omena-semantic` as part of the graph product.
- `omena-semantic.css-modules-semantics` — parser-fact-backed CSS Modules seed
  surface forwarded from `omena-semantic` as part of the graph product while
  cross-file `composes`/`@value`/ICSS resolution remains future graph work.
- `omena-semantic.source-input-evidence` — bridge-owned source evidence product,
  kept stable for existing host consumers while ownership moves behind this
  crate. The evidence includes value-domain derivation counts from the
  source-backed expression-semantics payload.
- `omena-semantic.promotion-evidence` — bridge-owned source-backed promotion
  evidence product, kept stable for existing host consumers while ownership
  moves behind this crate. The evidence includes parser-backed design-token
  seed facts from CSS custom properties.
- `omena-bridge.source-import-declarations` — OXC-backed source import
  declaration producer for CSS Module style imports and `classnames/bind`
  bindings consumed by the Rust LSP source syntax index.
- `omena-bridge.style-resolution` — style specifier resolver for relative
  imports and tsconfig/jsconfig path aliases, including Sass partial and index
  candidate expansion for LSP and bridge consumers.
- `omena-bridge.binder-plugin-boundary` — built-in `BinderPluginV0` boundary
  for source-side class-name tracking. The default plugin keeps the current CSS
  Modules + `classnames/bind`/`classnames`/`clsx` behavior behind one boundary;
  built-in proof points track Tailwind/Uno utility classes, vanilla-extract
  recipe variants, and Vue `useCssModule()` style-module references without
  claiming a CSS Module style source. External plugin ABI is intentionally not
  stable yet.

Primary check:

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-bridge
```

Split boundary check:

```sh
pnpm cme-check bundle rust/omena-bridge/split-boundary
```
