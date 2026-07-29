# omena-transform-print

## Role

`omena-transform-print` owns identity, pretty, and minified CSS emission plus
Source Map V3 generation and composition.

## Contract

`print_transform_cst_source_with_dialect_and_pretty_options` emits
width-sensitive Doc IR output. Execution-artifact printers preserve transform
provenance, and source-map APIs expose UTF-8 byte and UTF-16 position data,
serialization, parsing, VLQ decoding, and upstream-map composition.

## Consumers

`omena-query` reaches this crate through the transform-runner facade for build
output and maps. `omena-benchmarks` measures its emission paths directly.

## Boundaries

The crate does not resolve modules, select semantic passes, or link bundles.
`Minified` mode handles print-level trivia; semantic minification remains the
transform runtime's responsibility.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-transform-print
```
