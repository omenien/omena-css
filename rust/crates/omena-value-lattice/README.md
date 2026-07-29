# omena-value-lattice

## Role

`omena-value-lattice` provides a region-local CSS value lens, token stream,
canonical equality, color handling, numeric reduction, and static function
substitution.

## Contract

`declaration_value_lens`, `css_value_component_stream`, and
`canonicalize_css_value` operate on declaration value slices while preserving
base byte offsets.

## Consumers

Transform, query, Sass evaluation, and SIF paths share these value identities
instead of maintaining separate canonicalizers.

## Boundaries

The crate accepts values, not declarations, rules, or stylesheets. Whole-file
property analysis is intentionally unrepresentable at this layer.

## Verification

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-value-lattice
```
