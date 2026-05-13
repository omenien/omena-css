# omena-wasm

`omena-wasm` exposes the first browser-side binding for the Omena CSS
workspace. The binding consumes `omena-query` as the public Rust facade and
keeps parser and transform crates behind that boundary.

The current API is intentionally in-memory:

- `checkStyleSource(source, path)` checks CSS-family source text and returns
  query-owned parser facts.
- `buildStyleSource(source, path, passIds)` runs conservative transform passes
  and returns the execution summary plus output CSS.
- `buildStyleSourceWithContext(source, path, passIds, context)` accepts
  explicit evaluator/provenance context and returns the execution summary plus
  output CSS.
- `buildStyleSourceForTargetQuery(source, path, targetQuery)` plans
  conservative target-sensitive passes from a Browserslist query or named
  target profile.
- `buildStyleSourceForTargetQueryWithOptions(source, path, targetQuery,
targetOptions)` accepts camelCase target transform options for explicit
  lowering opt-ins.
- `buildStyleSourceForTargetQueryWithContext(source, path, targetQuery,
targetOptions, context)` combines target planning with explicit evaluator
  context, including dart-sass-compatible SCSS output.
- `buildStyleSourcesWithContext(targetPath, sources, passIds, context,
packageManifests)` derives import/composes context from an in-memory workspace
  source array, merges explicit evaluator/provenance context, and returns an
  execution summary plus output CSS.
- `buildStyleSourcesForTargetQueryWithContext(targetPath, sources, targetQuery,
targetOptions, context, packageManifests)` combines target planning with
  workspace-derived import/composes context.
- `readCascadeAtPosition(source, path, line, character, input)` reads cascade,
  computed-value, and custom-property LFP information at a `var(...)` reference
  position. Pass `null` or `undefined` for `input` when no EngineInputV2 context
  is needed.
- `readStyleContextIndex(source, path, input)` reads query-owned `@layer`,
  `@container`, and `@scope` context indexes. Pass `null` or `undefined` for
  `input` when no EngineInputV2 context is needed.
- `readStyleDiagnostics(source, path)` reads query-owned style diagnostics for
  a CSS-family file.
- `expressionDomainIncrementalFlow(input)` runs one query-owned
  expression-domain incremental-flow pass for simple browser clients.
- `new ExpressionDomainFlowRuntime().analyze(input)` keeps the query-owned
  incremental-flow runtime alive across calls so browser clients can observe
  graph reuse.
- `expressionDomainSelectorProjection(input)` projects expression-domain flow
  values to target style selectors.
- `listTransformPasses()` lists transform pass ids accepted by
  `buildStyleSource`.

This crate does not read from the filesystem and does not provide the Node or
LSP integration layer. Those stay in native consumer crates.
