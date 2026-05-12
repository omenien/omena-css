# Contributing

Run formatting, tests, and clippy before opening a pull request:

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
