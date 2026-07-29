# `omena-cst-typed`

## Role

`omena-cst-typed` is a borrowed typed projection over the parser-owned CSS
syntax tree.

## Contract

`parsed_typed_cst` exposes typed node accessors without cloning the tree.
`parse_tree_data` and `parse_style_document_typed_v0` provide the generic egress
shape, while the structural manifest records supported children and tokens.

## Consumers

Query and syntax-facing product boundaries use these accessors when raw CST
navigation would duplicate grammar knowledge.

## Boundaries

`omena-parser` remains the only tree authority. This crate neither reparses
source nor owns semantic analysis, mutation, or printing.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-cst-typed
```
