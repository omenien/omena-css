---
title: Architecture guide
description: Navigate Omena product facades, semantic layers, host adapters, and repository contracts.
kind: explanation
status: stable
products: [architecture]
owner: architecture
sourceOfTruth: authored
---

# Architecture guide

The repository [architecture codemap](../../ARCHITECTURE.md) is the
file-oriented authority. This page gives the product view.

| Layer      | Responsibility                                      | Primary surface                |
| ---------- | --------------------------------------------------- | ------------------------------ |
| Syntax     | lossless CSS-family parse facts and spans           | `omena-parser`, `omena-syntax` |
| Semantics  | stable selector, module, Sass, and value identities | `omena-semantic`               |
| Resolution | specifiers, packages, and cross-file closure        | resolver and summary crates    |
| Query      | memoized workspace analysis and typed results       | `omena-query`                  |
| Transform  | typed IR, planning, execution, and printing         | transform crates               |
| Products   | CLI, LSP, SDK, bundler, and lint adapters           | public hosts                   |
| Evidence   | differential checks, snapshots, and attestations    | test and evidence crates       |

Dependencies should point from product facades toward lower-level authorities,
not from syntax or semantic substrates back into CLI or host adapters. The
[crate catalog](../reference/crates.md) provides the complete generated list.
