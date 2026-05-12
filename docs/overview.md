# Overview

omena-css is a Rust workspace for CSS-family parsing, semantic substrates,
cascade modeling, incremental recomputation, and conservative CSS transforms.

The workspace is split into small crates so parser, cascade, incremental, and
transform responsibilities can be tested and published independently while still
sharing one release train.

## Crate Layers

- Abstract value and producer inputs: `omena-abstract-value`,
  `omena-engine-input-producers`
- Syntax and interning: `omena-syntax`, `omena-interner`
- Parser surface: `omena-parser`
- Incremental substrate: `omena-incremental`
- Cascade substrate: `omena-cascade`
- Semantic bridge: `omena-resolver`, `omena-semantic`, `omena-bridge`
- Query facade: `omena-query`
- Transform substrate: `omena-transform-cst`, `omena-transform-passes`,
  `omena-transform-bundle`, `omena-transform-target`,
  `omena-transform-print`, `omena-transform-egg`
- Consumer surfaces: `omena-cli`, `omena-napi`, `omena-wasm`

## Current Product Surface

The first public surface focuses on parser and transform foundations:

- CSS, SCSS, Sass, and Less dialect classification.
- Recovery-aware parser summaries for CSS Modules and style facts.
- Cascade ordering, specificity, custom-property substitution, and transform
  proof helpers.
- Conservative transform planning and execution surfaces with explicit
  provenance.
- Query-owned consumer facade for CLI, Node native, and browser bindings.
- Node native JSON binding substrate through `omena-napi`.
- Browser-side in-memory query bindings through `omena-wasm`.

## Design Rules

- Keep parser facts canonical at the parser boundary.
- Keep cascade-sensitive rewrites behind proof helpers.
- Keep source-map provenance attached to every emitted transform result.
- Prefer public crate names and product terms over private planning labels.
