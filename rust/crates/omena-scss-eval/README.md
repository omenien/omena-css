# omena-scss-eval

## Role

`omena-scss-eval` evaluates statically decidable SCSS, Sass, and Less values,
variables, calls, control flow, and native CSS conditionals.

## Contract

Public entry points derive static module evaluations, typed value resolutions,
control-flow graphs, call-return summaries, and provenance-preserving native
edits. Query and transform consumers use these edits directly where their
evidence is sufficient.

## Consumers

`omena-query` uses the evaluator for module and value workflows.
`omena-transform-passes` consumes the native CSS edit plans.

## Boundaries

This is not a complete Sass or Less compiler. Dynamic interpolation, branches,
loops, and calls that cannot be proven statically remain unchanged. Internal
round-trip oracles do not replace differential testing against external
compilers.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-scss-eval
```
