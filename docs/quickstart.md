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
cargo add omena-query
```

Most consumers should start with `omena-query`, which owns the public facade
for parser facts, transform execution, and consumer summaries. Lower-level
crates remain public so integrations can opt into smaller boundaries when
needed.

## Install the CLI

```sh
cargo install omena-cli
omena check path/to/file.module.scss
omena build path/to/file.css --pass whitespace-strip
omena build path/to/file.css --target-query "ie 11"
omena build path/to/file.css --target-query "ie 11" --allow-logical-to-physical
omena build path/to/Button.module.css --source path/to/tokens.css --pass import-inline
omena build path/to/Button.module.css --source node_modules/@design/tokens/dist/theme.css --package-manifest node_modules/@design/tokens/package.json --pass import-inline
omena cascade path/to/file.module.css --line 10 --character 16 --json
omena passes
```

Use the checkout form when developing the workspace locally:

```sh
cargo run -p omena-cli -- check path/to/file.module.scss
cargo run -p omena-cli -- build path/to/file.css --pass whitespace-strip
cargo run -p omena-cli -- build path/to/file.css --target-query "ie 11"
cargo run -p omena-cli -- build path/to/file.css --target-query "ie 11" --allow-logical-to-physical
cargo run -p omena-cli -- build path/to/Button.module.css --source path/to/tokens.css --pass import-inline
cargo run -p omena-cli -- build path/to/Button.module.css --source node_modules/@design/tokens/dist/theme.css --package-manifest node_modules/@design/tokens/package.json --pass import-inline
cargo run -p omena-cli -- cascade path/to/file.module.css --line 10 --character 16 --json
cargo run -p omena-cli -- passes
```

## Use the Browser Binding

`omena-wasm` is an in-memory binding for browser and playground consumers.
It does not read from the filesystem; pass source text and a path-like label so
the dialect can be inferred. Generate a web package with `wasm-pack build
crates/omena-wasm --target web`, then import the generated module:

```js
import init, {
  checkStyleSource,
  buildStyleSource,
  buildStyleSourceWithContext,
  buildStyleSourceForTargetQuery,
  buildStyleSourceForTargetQueryWithOptions,
  buildStyleSourceForTargetQueryWithContext,
  buildStyleSourcesWithContext,
  buildStyleSourcesForTargetQueryWithContext,
  readCascadeAtPosition,
} from "./pkg/omena_wasm.js";

await init();
const facts = checkStyleSource(".card { color: red; }", "demo.module.css");
const built = buildStyleSource(".card { color: #ffffff; }", "demo.css", [
  "color-compression",
]);
const legacyBuilt = buildStyleSourceForTargetQuery(
  ".card { display: flex; color: light-dark(#000, #fff); }",
  "demo.css",
  "ie 11",
);
const legacyBuiltWithOptions = buildStyleSourceForTargetQueryWithOptions(
  ".card { margin-inline: 1rem; }",
  "demo.css",
  "ie 11",
  { allowLogicalToPhysical: true },
);
const evaluatedScss = buildStyleSourceForTargetQueryWithContext(
  "$brand: red; .card { color: $brand; }",
  "demo.module.scss",
  "ie 11",
  null,
  {
    scssModuleEvaluation: {
      evaluator: "dart-sass-compatible",
      evaluatedCss: ".card { color: red; }",
    },
  },
);
const bundledModule = buildStyleSourcesWithContext(
  "Button.module.css",
  [
    {
      stylePath: "Button.module.css",
      styleSource:
        '@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; }',
    },
    { stylePath: "tokens.css", styleSource: ":root { --brand: red; }" },
  ],
  ["import-inline", "composes-resolution"],
  {},
  [],
);
const cascade = readCascadeAtPosition(
  ":root { --brand: red; } .button { color: var(--brand); }",
  "Button.module.css",
  0,
  44,
  null,
);
```

## Use the Node Native Binding Substrate

`omena-napi` is the Rust N-API substrate for future npm packaging. It exposes
JSON-string APIs so Node clients can consume the same query-owned parser and
transform contracts without depending on unstable Rust structs. A future npm wrapper can
export this shape:

```js
import {
  checkStyleSourceJson,
  buildStyleSourceJson,
  buildStyleSourceWithContextJson,
  buildStyleSourceForTargetQueryJson,
  buildStyleSourceForTargetQueryWithOptionsJson,
  buildStyleSourceForTargetQueryWithContextJson,
  buildStyleSourcesWithContextJson,
  buildStyleSourcesForTargetQueryWithContextJson,
  readCascadeAtPositionJson,
} from "omena-napi";

const facts = JSON.parse(
  checkStyleSourceJson(".card { color: red; }", "demo.module.css"),
);
const built = JSON.parse(
  buildStyleSourceJson(".card { color: #ffffff; }", "demo.css", [
    "color-compression",
  ]),
);
const legacyBuilt = JSON.parse(
  buildStyleSourceForTargetQueryJson(
    ".card { display: flex; color: light-dark(#000, #fff); }",
    "demo.css",
    "ie 11",
  ),
);
const legacyBuiltWithOptions = JSON.parse(
  buildStyleSourceForTargetQueryWithOptionsJson(
    ".card { margin-inline: 1rem; }",
    "demo.css",
    "ie 11",
    JSON.stringify({ allowLogicalToPhysical: true }),
  ),
);
const evaluatedScss = JSON.parse(
  buildStyleSourceForTargetQueryWithContextJson(
    "$brand: red; .card { color: $brand; }",
    "demo.module.scss",
    "ie 11",
    "{}",
    JSON.stringify({
      scssModuleEvaluation: {
        evaluator: "dart-sass-compatible",
        evaluatedCss: ".card { color: red; }",
      },
    }),
  ),
);
const bundledModule = JSON.parse(
  buildStyleSourcesWithContextJson(
    "Button.module.css",
    JSON.stringify([
      {
        stylePath: "Button.module.css",
        styleSource:
          '@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; }',
      },
      { stylePath: "tokens.css", styleSource: ":root { --brand: red; }" },
    ]),
    ["import-inline", "composes-resolution"],
    "{}",
    "[]",
  ),
);
const cascade = JSON.parse(
  readCascadeAtPositionJson(
    ":root { --brand: red; } .button { color: var(--brand); }",
    "Button.module.css",
    0,
    44,
    "",
  ),
);
```

## Publish Readiness

Run the manual GitHub Actions publish workflow in `dry-run` mode first. For a
local check, package the crate you changed:

```sh
cargo package --list --manifest-path crates/omena-parser/Cargo.toml
cargo publish --dry-run --manifest-path crates/omena-parser/Cargo.toml
```
