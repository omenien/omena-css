# Contributing

Run formatting, tests, and clippy before opening a pull request:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Commit Messages

Use plain imperative commit subjects:

```text
Add parser differential coverage
Tighten transform workspace packaging
Fix source-map segment ordering
```

Do not use internal planning labels, phase names, or issue-triage shorthand in
commit messages. Public history should describe the product change directly.
