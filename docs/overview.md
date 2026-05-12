# Overview

omena-css is a Rust workspace for CSS-family parsing, semantic substrates,
cascade modeling, incremental recomputation, and conservative CSS transforms.

The workspace is split into small crates so parser, cascade, incremental, and
transform responsibilities can be tested and published independently while still
sharing one release train.

## Crate Layers

- Syntax and interning: `omena-syntax`, `omena-interner`
- Parser surface: `omena-parser`
- Incremental substrate: `omena-incremental`
- Cascade substrate: `omena-cascade`
- Transform substrate: `omena-transform-cst`, `omena-transform-passes`,
  `omena-transform-bundle`, `omena-transform-target`,
  `omena-transform-print`, `omena-transform-egg`
- Consumer surface: `omena-cli`

## Current Product Surface

The first public surface focuses on parser and transform foundations:

- CSS, SCSS, Sass, and Less dialect classification.
- Recovery-aware parser summaries for CSS Modules and style facts.
- Cascade ordering, specificity, custom-property substitution, and transform
  proof helpers.
- Conservative transform planning and execution surfaces with explicit
  provenance.

## Design Rules

- Keep parser facts canonical at the parser boundary.
- Keep cascade-sensitive rewrites behind proof helpers.
- Keep source-map provenance attached to every emitted transform result.
- Prefer public crate names and product terms over private planning labels.
