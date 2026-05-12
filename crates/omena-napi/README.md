# omena-napi

`omena-napi` exposes the first Node native binding crate for the Omena CSS
workspace.

The current API returns JSON strings so the binding can stay thin while the Rust
parser and transform contracts settle:

- `checkStyleSourceJson(source, path)` parses CSS-family source text and returns
  parser-owned facts.
- `buildStyleSourceJson(source, path, passIds)` runs conservative transform
  passes and returns an execution summary plus output CSS.
- `listTransformPassesJson()` lists transform pass ids accepted by
  `buildStyleSourceJson`.

This crate is the native binding substrate for future npm packaging. It does not
ship an npm package by itself.
