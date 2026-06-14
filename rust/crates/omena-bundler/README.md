# omena-bundler

`omena-bundler` is the standalone 0.x Rust crate for Omena's CSS bundling
planning surface. It turns parser facts into a transform bundle plan for
`@import` inlining, Sass/Less module evaluation, CSS Modules hashing,
`composes` resolution, `@value` resolution, asset URL rewrite planning, and
code-split chunk metadata.

This crate is intentionally pre-1.0. The public Rust names keep their `V0`
suffix and the surface may change while real adopters exercise the package.

## Install

```toml
[dependencies]
omena-bundler = "0.2.1"
```

## Use the V0 planning surface

```rust
use omena_bundler::summarize_omena_transform_bundle_from_source;
use omena_parser::StyleDialect;

let summary = summarize_omena_transform_bundle_from_source(
    "src/App.module.scss",
    "@use './tokens'; .button { color: red; }",
    StyleDialect::Scss,
);

assert!(summary.module_evaluation_required);
```

For build-tool integrations, most users should still start with the published
JavaScript packages:

- `@omena/vite-plugin`
- `@omena/postcss-plugin`
- `@omena/css-build-adapter`

## Compatibility

The historical `omena-transform-bundle` crate remains as a compatibility
re-export for the same `...V0` Rust surface. Existing consumers can continue to
compile through that path during the 0.x line.
