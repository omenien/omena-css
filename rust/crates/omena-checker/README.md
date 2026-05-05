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
- `evaluate_omena_checker_dynamic_class_domain` - first Rust-owned dynamic
  class domain rule body. It consumes `omena-abstract-value` selector projection
  and returns the current `missing-resolved-class-values` /
  `missing-resolved-class-domain` policy decision.

The TypeScript checker runtime still owns the full diagnostics pipeline today.
This crate owns the rule/bundle registry plus the first pure rule-evaluation
body so later slices can move execution by rule family without changing the
public rule catalog.

Primary check:

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-checker
```
