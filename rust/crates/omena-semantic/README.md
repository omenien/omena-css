# omena-semantic

## Role

`omena-semantic` lifts parser facts and `EngineInputV2` evidence into stable
selector, CSS Modules, Sass module, design-token, and style-context summaries.

## Contract

The crate exposes style semantic graph and runtime index summaries, selector
identity and reference engines, CSS Modules cross-file closure and resolution,
Sass module graph closure and resolution, and lossless-CST evidence. Entry
points include `summarize_style_semantic_graph_from_source`,
`summarize_css_modules_cross_file_resolution`, and
`summarize_sass_module_graph_resolution`.

## Consumers

`omena-bridge` presents the semantic facade, `omena-query` consumes runtime and
cross-file facts, and transform passes use layer and cascade evidence.

## Boundaries

The crate does not parse source independently of `omena-parser`, evaluate Sass
values, mutate CSS, render diagnostics, or own LSP and CLI transport.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-semantic
```
