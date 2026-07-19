# omena-bundler

Start with the [product overview](../../../README.md). JavaScript build-host
integration is documented by the [Vite plugin](../../../packages/vite-plugin/README.md).

`omena-bundler` is the standalone 0.x Rust crate for Omena's CSS bundling
planning surface. It turns parser facts into a transform bundle plan for
`@import` inlining, Sass/Less module evaluation, CSS Modules hashing,
`composes` resolution, `@value` resolution, asset URL rewrite planning, and
code-split chunk metadata.

This crate is intentionally pre-1.0. The public Rust names keep their `V0`
suffix and the surface may change while real adopters exercise the package.

## Install

```toml
[dependencies]
omena-bundler = "0.2.1"
```

## Use the V0 planning surface

```rust
use omena_bundler::summarize_omena_transform_bundle_from_source;
use omena_parser::StyleDialect;

let summary = summarize_omena_transform_bundle_from_source(
    "src/App.module.scss",
    "@use './tokens'; .button { color: red; }",
    StyleDialect::Scss,
);

assert!(summary.module_evaluation_required);
```

For build-tool integrations, most users should still start with the published
JavaScript packages:

- `@omena/vite-plugin`
- `@omena/postcss-plugin`
- `@omena/css-build-adapter`

## Compatibility

The historical `omena-transform-bundle` crate remains as a compatibility
re-export for the same `...V0` Rust surface. Existing consumers can continue to
compile through that path during the 0.x line.

## Design decisions (omena-css#55 / rfcs#76)

The bundle & source-map emission RFC (omena-css#55, now rfcs#76) posed seven
design questions. Their resolutions are recorded here as the durable in-tree
design record (originally landed on `omena-transform-bundle`'s README in
`728316b8`; relocated here when the bundle planning surface moved into this
crate). Each entry is a decision record, not a stability promise — this crate is
still 0.x and its surface may change.

1. **Q1 — bundler crate boundary: SUPERSEDED.** Originally resolved
   _evolve-in-place; do NOT split a new `omena-bundler` crate_ (a split would
   disturb internal `omena-query` call sites and expand the crates.io +
   Trusted-Publishing release set — a recurring publish obligation). **This was
   later superseded:** the standalone `omena-bundler` crate WAS extracted at
   `3d01181a` (2026-06-14) as a deliberate maintainer-override choice to make the
   bundler discoverable/usable standalone at 0.x, consciously accepting the
   "premature module boundary" cost Q1 flagged in exchange for standalone
   shareability. The old `omena-transform-bundle` path stays as a re-export so no
   consumer breaks. Rationale + scope: the standalone-0x packaging decision (kept
   `V0`, made no stability/freeze claim).
2. **Q2 — `sourcesContent` policy: DECIDED.** Keep the always-embed default; the
   bundle path embeds `sourcesContent` via the V3 serializer. A size-driven
   opt-out config knob is deferred until a consumer needs it.
3. **Q3 — provenance carrier: DECIDED-BY-CODE.** Omena provenance ships as V3
   extension fields inside the map (`x_omenaSchemaVersion` / `x_omenaProduct` /
   `x_omenaSegmentCount` / `x_omenaPassIds`), not as a separate JSON metadata
   side-file. The JSON-metadata-only alternative is retired.
4. **Q4 — bundle-time tree shaking: DECIDED-BY-CODE.** Landed as a context-flag +
   pass-append on the existing build path (two call sites, single- and
   split-output), with no new graph-access machinery and no salsa
   re-architecture. The existing substrate suffices.
5. **Q5 — asset hashing ownership: DECIDED.** Resolve-only; the rewrite primitive
   preserves data/fragment/external/absolute refs. Content-hashing / CDN concerns
   stay with downstream tools.
6. **Q6 — split policy: IMPLEMENTED.** Entry/route-driven split config +
   automatic shared-chunk detection landed as the dependency-derived planner plus
   the split-manifest + origin-chain gate surfaces (the `[Q6]` lane). New chunk
   kinds are represented in the manifest schema, not silently mis-bucketed.
7. **Q7 — Vite-plugin depth: DECIDED.** Thin adapter by design (no HMR /
   virtual-module machinery in the plugin); revisit only with a driving user
   request.
