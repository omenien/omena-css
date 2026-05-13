# omena-napi

`omena-napi` exposes the first Node native binding crate for the Omena CSS
workspace. The binding consumes `omena-query` as the public Rust facade and
keeps parser and transform crates behind that boundary.

The current API returns JSON strings so the binding can stay thin while the
query, parser, and transform contracts settle:

- `checkStyleSourceJson(source, path)` checks CSS-family source text and
  returns query-owned parser facts.
- `buildStyleSourceJson(source, path, passIds)` runs conservative transform
  passes and returns an execution summary plus output CSS.
- `buildStyleSourceWithContextJson(source, path, passIds, contextJson)` accepts
  explicit evaluator/provenance context and returns an execution summary plus
  output CSS.
- `buildStyleSourceForTargetQueryJson(source, path, targetQuery)` plans
  conservative target-sensitive passes from a Browserslist query or named
  target profile.
- `buildStyleSourceForTargetQueryWithOptionsJson(source, path, targetQuery,
targetOptionsJson)` accepts camelCase target transform options for explicit
  lowering opt-ins.
- `buildStyleSourceForTargetQueryWithContextJson(source, path, targetQuery,
targetOptionsJson, contextJson)` combines target planning with explicit
  evaluator context, including dart-sass-compatible SCSS output.
- `buildStyleSourcesWithContextJson(targetPath, sourcesJson, passIds,
contextJson, packageManifestsJson)` derives import/composes context from a
  workspace source array, merges explicit evaluator/provenance context, and
  returns an execution summary plus output CSS.
- `buildStyleSourcesForTargetQueryWithContextJson(targetPath, sourcesJson,
targetQuery, targetOptionsJson, contextJson, packageManifestsJson)` combines
  target planning with workspace-derived import/composes context.
- `readCascadeAtPositionJson(source, path, line, character, inputJson)` reads
  cascade, computed-value, and custom-property LFP information at a `var(...)`
  reference position. Pass an empty string for `inputJson` when no EngineInputV2
  context is needed.
- `readStyleContextIndexJson(source, path, inputJson)` reads query-owned
  `@layer`, `@container`, and `@scope` context indexes. Pass an empty string for
  `inputJson` when no EngineInputV2 context is needed.
- `readStyleDiagnosticsJson(source, path)` reads query-owned style diagnostics
  for a CSS-family file.
- `readSourceDiagnosticsJson(sourceUri, candidatesJson)` reads query-owned
  source diagnostics from precomputed missing-selector candidates.
- `expressionDomainIncrementalFlowJson(inputJson)` runs one query-owned
  expression-domain incremental-flow pass for simple Node clients.
- `new ExpressionDomainFlowRuntime().analyzeJson(inputJson)` keeps the
  query-owned incremental-flow runtime alive across calls so Node clients can
  observe graph reuse.
- `expressionDomainSelectorProjectionJson(inputJson)` projects
  expression-domain flow values to target style selectors.
- `listTransformPassesJson()` lists transform pass ids accepted by
  `buildStyleSourceJson`.

This crate is the native binding substrate for future npm packaging. It does not
ship an npm package by itself.
