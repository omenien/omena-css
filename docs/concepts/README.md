---
title: Concepts
description: Understand the semantic and evidence contracts behind Omena diagnostics and transformations.
kind: explanation
status: stable
products: [platform]
owner: architecture
sourceOfTruth: authored
---

# Concepts

Omena is not a collection of independent regex checks. It keeps syntax,
identity, cross-file facts, query results, and product rendering in separate
layers so uncertainty can remain visible.

- [Semantic analysis](./semantic-analysis.md) follows facts from source text to
  editor and CLI results.
- [Evidence and transform safety](./evidence-and-transform-safety.md) explains
  why transforms can apply, preserve, or reject without inventing certainty.
- [Custom-property dependency resolution](./custom-property-bounded-substitution.md)
  states the shipped dependency-graph resolver, its independent all-bottom witness, and
  the explicit research gaps.
- [Positioning](../positioning.md) compares the product boundary with adjacent
  CSS tools.
- [Performance evidence](../performance.md) explains how latency claims are
  measured.
