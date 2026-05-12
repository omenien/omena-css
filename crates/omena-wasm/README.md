# omena-wasm

`omena-wasm` exposes the first browser-side binding for the Omena CSS
workspace. The binding consumes `omena-query` as the public Rust facade and
keeps parser and transform crates behind that boundary.

The current API is intentionally in-memory:

- `checkStyleSource(source, path)` checks CSS-family source text and returns
  query-owned parser facts.
- `buildStyleSource(source, path, passIds)` runs conservative transform passes
  and returns the execution summary plus output CSS.
- `buildStyleSourceForTargetQuery(source, path, targetQuery)` plans
  conservative target-sensitive passes from a Browserslist query or named
  target profile.
- `buildStyleSourceForTargetQueryWithOptions(source, path, targetQuery,
targetOptions)` accepts camelCase target transform options for explicit
  lowering opt-ins.
- `listTransformPasses()` lists transform pass ids accepted by
  `buildStyleSource`.

This crate does not read from the filesystem and does not provide the Node or
LSP integration layer. Those stay in native consumer crates.
