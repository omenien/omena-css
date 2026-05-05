# `omena-checker`

Internal Rust crate for checker rule metadata and diagnostic policy boundaries.

Current public products:

- `omena-checker.boundary` - boundary summary for the checker rule registry,
  named rule bundles, and migration policy away from TypeScript-owned checker
  metadata.
- `list_omena_checker_rule_descriptors` - stable V0 rule descriptor order for
  current checker diagnostics.
- `list_omena_checker_code_bundles` - named rule bundles used by CI and focused
  checker lanes.

The TypeScript checker runtime still executes diagnostics today. This crate
establishes the Rust-owned registry boundary first so later slices can move rule
execution without changing the public rule catalog.

Primary check:

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-checker
```
