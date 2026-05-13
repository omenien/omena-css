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
- `omena build <file> --target-query "ie 11" --allow-logical-to-physical`
  opts into compatibility lowerings that are disabled by default.
- `omena build <file> --context-json context.json` accepts explicit evaluator
  and provenance context, including dart-sass-compatible SCSS output.
- `omena build <file> --source other.css` derives import/composes context from
  additional workspace style sources before running requested passes.
- `omena build <file> --package-manifest node_modules/pkg/package.json`
  lets workspace source context resolve package style exports for import
  inlining.
- `omena cascade <file> --line <n> --character <n>` reads cascade,
  computed-value, and custom-property LFP information at a `var(...)`
  reference position.
- `omena passes` lists accepted transform pass ids.

## Wasm

`omena-wasm` exposes the first browser-side in-memory consumer surface through
`omena-query`:

- `checkStyleSource(source, path)` reports query-owned parser facts.
- `buildStyleSource(source, path, passIds)` runs conservative transform passes.
- `buildStyleSourceWithContext(source, path, passIds, context)` accepts
  explicit evaluator/provenance context.
- `buildStyleSourceForTargetQuery(source, path, targetQuery)` plans
  target-sensitive passes from a Browserslist query or named target profile.
- `buildStyleSourceForTargetQueryWithOptions(source, path, targetQuery,
  targetOptions)` accepts explicit target transform opt-ins.
- `buildStyleSourceForTargetQueryWithContext(source, path, targetQuery,
  targetOptions, context)` combines target planning with explicit evaluator
  context.
- `buildStyleSourcesWithContext(targetPath, sources, passIds, context,
  packageManifests)` derives import/composes context from in-memory workspace
  sources and merges explicit evaluator/provenance context.
- `buildStyleSourcesForTargetQueryWithContext(targetPath, sources, targetQuery,
  targetOptions, context, packageManifests)` combines target planning with
  workspace-derived import/composes context.
- `listTransformPasses()` lists accepted transform pass ids.

## Node Native Binding

`omena-napi` exposes the first Node native binding substrate:

- `checkStyleSourceJson(source, path)` reports query-owned parser facts as JSON.
- `buildStyleSourceJson(source, path, passIds)` runs conservative transform
  passes and returns JSON.
- `buildStyleSourceWithContextJson(source, path, passIds, contextJson)`
  accepts explicit evaluator/provenance context and returns JSON.
- `buildStyleSourceForTargetQueryJson(source, path, targetQuery)` plans
  target-sensitive passes from a Browserslist query or named target profile.
- `buildStyleSourceForTargetQueryWithOptionsJson(source, path, targetQuery,
  targetOptionsJson)` accepts explicit target transform opt-ins.
- `buildStyleSourceForTargetQueryWithContextJson(source, path, targetQuery,
  targetOptionsJson, contextJson)` combines target planning with explicit
  evaluator context.
- `buildStyleSourcesWithContextJson(targetPath, sourcesJson, passIds,
  contextJson, packageManifestsJson)` derives import/composes context from
  workspace source JSON and merges explicit evaluator/provenance context.
- `buildStyleSourcesForTargetQueryWithContextJson(targetPath, sourcesJson,
  targetQuery, targetOptionsJson, contextJson, packageManifestsJson)` combines
  target planning with workspace-derived import/composes context.
- `listTransformPassesJson()` lists accepted transform pass ids as JSON.
