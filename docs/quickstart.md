# Quickstart

## Verify the Workspace

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Use a Crate

Add the crate that matches the layer you need:

```sh
cargo add omena-parser
cargo add omena-cascade
cargo add omena-transform-passes
```

Most consumers should start with `omena-parser` for source facts or
`omena-transform-passes` for transform planning. Lower-level crates remain
public so integrations can opt into smaller boundaries when needed.

## Use the CLI from a Checkout

```sh
cargo run -p omena-cli -- check path/to/file.module.scss
cargo run -p omena-cli -- build path/to/file.css --pass whitespace-strip
cargo run -p omena-cli -- passes
```

## Publish Readiness

Run the manual GitHub Actions publish workflow in `dry-run` mode first. For a
local check, package the crate you changed:

```sh
cargo package --list --manifest-path crates/omena-parser/Cargo.toml
cargo publish --dry-run --manifest-path crates/omena-parser/Cargo.toml
```
