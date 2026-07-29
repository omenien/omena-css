---
title: Evidence and transform safety
description: Learn how Omena separates facts, proof evidence, uncertainty, and byte-producing transforms.
kind: explanation
status: stable
products: [transform, evidence, bundler]
owner: architecture
sourceOfTruth: authored
---

# Evidence and transform safety

A transform can be syntactically possible and still be semantically unsafe.
Omena therefore separates the proposed edit from the evidence that admits it.

## Three outcomes

- **Apply** when the required facts and product policy establish the transform.
- **Preserve** when the input is valid but the available evidence is
  insufficient.
- **Reject** when a typed precondition or output invariant fails.

Preservation is not a hidden success. Structured results name unresolved
inputs, rejected preconditions, or retained artifacts so callers can decide
whether to supply more context.

## Why source identity matters

Cross-file transforms depend on module identity, source spans, and emission
order. Omena carries those identities through the transform IR and source-map
path instead of reconstructing them from final strings. This permits byte
comparisons, provenance checks, and fail-closed bundle decisions at product
boundaries.

## External evidence

Sass Interface Files and external-tool attestations can enrich a closed-world
decision, but they remain typed evidence with explicit trust and freshness
boundaries. They do not silently become parser facts.

Use `omena explain` or request build evidence when a transform is preserved.
Do not infer safety from a zero-mutation count alone.
