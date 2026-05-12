# API Reference

This page summarizes the stable public boundaries exposed by the initial
workspace. Use crate rustdoc for full type-level documentation.

## Query Facade

`omena-query` is the default facade for consumers. It exposes query-owned
summaries for parser facts, transform execution, and source/style semantic
lookups while keeping parser and transform crates behind one boundary.

Primary consumers:

- CLI, Node native, and browser bindings.
- Editors and tools that need a stable product surface.
- Integrations that should not depend on lower-level crate internals.

## Parser

`omena-parser` exposes parse and lex results, dialect classification, parser
summaries, CSS Modules intermediate summaries, and canonical producer signals.

Primary consumers:

- Editors and language servers that need style facts.
- Transform engines that need parser-owned source summaries.
- Differential tests that compare token and CST behavior.

## Cascade

`omena-cascade` exposes cascade keys, specificity, declaration winners,
selector-context witnesses, custom-property substitution, and proof helpers for
scope, layer, supports, and box-shorthand rewrites.

Primary consumers:

- Semantic analyzers that need cascade-aware ranking.
- Transform passes that need proof-carrying safety checks.
- Test harnesses that need deterministic cascade witnesses.

## Transform

`omena-transform-cst` defines transform contracts and DAG metadata.
`omena-transform-passes` registers and plans safe mutations.
`omena-transform-bundle`, `omena-transform-target`,
`omena-transform-print`, and `omena-transform-egg` split bundle planning,
target lowering, emission, and equality-saturation concerns.

Primary consumers:

- CSS build tools.
- Editor quick-fix pipelines.
- Benchmark and conformance runners.

## CLI

`omena-cli` exposes the first command-line consumer surface through
`omena-query`:

- `omena check <file>` reports query-owned parser facts and parse-error counts.
- `omena build <file>` runs the conservative transform pipeline.
- `omena build <file> --target-query "ie 11"` plans target-sensitive passes
  from a Browserslist query or named target profile.
- `omena passes` lists accepted transform pass ids.

## Wasm

`omena-wasm` exposes the first browser-side in-memory consumer surface through
`omena-query`:

- `checkStyleSource(source, path)` reports query-owned parser facts.
- `buildStyleSource(source, path, passIds)` runs conservative transform passes.
- `buildStyleSourceForTargetQuery(source, path, targetQuery)` plans
  target-sensitive passes from a Browserslist query or named target profile.
- `listTransformPasses()` lists accepted transform pass ids.

## Node Native Binding

`omena-napi` exposes the first Node native binding substrate:

- `checkStyleSourceJson(source, path)` reports query-owned parser facts as JSON.
- `buildStyleSourceJson(source, path, passIds)` runs conservative transform
  passes and returns JSON.
- `buildStyleSourceForTargetQueryJson(source, path, targetQuery)` plans
  target-sensitive passes from a Browserslist query or named target profile.
- `listTransformPassesJson()` lists accepted transform pass ids as JSON.
