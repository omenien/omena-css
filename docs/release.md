# Release Process

omena-css uses one workspace release train for the public crates in this repo.
Patch releases may be crate-specific when only one crate needs a compatibility
or packaging fix.

## Required Checks

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Run the `Publish Crates` GitHub Actions workflow in `dry-run` mode before
publishing. Publish only after CI is green and dependency order has been
checked.

## Commit Messages

Use plain imperative commit subjects:

```text
Add parser differential coverage
Tighten transform workspace packaging
Fix source-map segment ordering
```

Avoid private planning labels in public history, docs, rustdoc, and release
notes.
