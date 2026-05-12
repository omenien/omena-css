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
- `listTransformPassesJson()` lists transform pass ids accepted by
  `buildStyleSourceJson`.

This crate is the native binding substrate for future npm packaging. It does not
ship an npm package by itself.
