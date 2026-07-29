---
title: Semantic analysis
description: Follow Omena facts from lossless parsing through identity, cross-file resolution, and product queries.
kind: explanation
status: stable
products: [parser, semantic, query, lsp]
owner: architecture
sourceOfTruth: authored
---

# Semantic analysis

Omena separates five responsibilities.

1. The parser records lossless syntax and source spans for CSS, SCSS, Sass, and
   Less without deciding product behavior.
2. Semantic crates derive stable identities for selectors, CSS Modules edges,
   Sass symbols, custom properties, and contextual rules.
3. Resolution layers connect files using caller-supplied workspace and package
   inputs.
4. Query layers memoize derived graphs and expose diagnostics, hover,
   completion, transforms, and bundle plans.
5. Product hosts render those typed results for the CLI, LSP, NAPI, WASM, and
   build adapters.

This boundary matters when a source is incomplete. A parser can preserve an
unknown construct, the semantic layer can report that identity is
indeterminate, and the product can avoid a false diagnostic without pretending
the syntax was valid or invalid.

## Cross-file work

CSS Modules `composes` and `@value`, Sass `@use` and `@forward`, package
resolution, and source-to-style references all require a workspace view.
Browser WASM therefore needs caller-supplied files, while the LSP can maintain
an incremental index over editor and disk state.

## Public entry points

Most users should enter through a product surface rather than a substrate crate:

- `omena` for scripts and CI;
- `omena-lsp-server` for editors;
- `@omena/napi` or `@omena/wasm` for embedded hosts;
- `omena-query` as the Rust facade.

The [crate catalog](../reference/crates.md) records lower-level ownership
without turning every internal crate into a standalone product.
