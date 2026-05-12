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

## Install the CLI

```sh
cargo install omena-cli
omena check path/to/file.module.scss
omena build path/to/file.css --pass whitespace-strip
omena passes
```

Use the checkout form when developing the workspace locally:

```sh
cargo run -p omena-cli -- check path/to/file.module.scss
cargo run -p omena-cli -- build path/to/file.css --pass whitespace-strip
cargo run -p omena-cli -- passes
```

## Use the Browser Binding

`omena-wasm` is an in-memory binding for browser and playground consumers.
It does not read from the filesystem; pass source text and a path-like label so
the dialect can be inferred. Generate a web package with `wasm-pack build
crates/omena-wasm --target web`, then import the generated module:

```js
import init, { checkStyleSource, buildStyleSource } from "./pkg/omena_wasm.js";

await init();
const facts = checkStyleSource(".card { color: red; }", "demo.module.css");
const built = buildStyleSource(".card { color: #ffffff; }", "demo.css", [
  "color-compression",
]);
```

## Publish Readiness

Run the manual GitHub Actions publish workflow in `dry-run` mode first. For a
local check, package the crate you changed:

```sh
cargo package --list --manifest-path crates/omena-parser/Cargo.toml
cargo publish --dry-run --manifest-path crates/omena-parser/Cargo.toml
```
