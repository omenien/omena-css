# omena-cli

`omena-cli` is the command-line consumer surface for the Omena CSS workspace.

Current commands:

- `omena check <file>` parses a CSS-family file and reports parser-owned facts.
- `omena build <file>` runs the conservative transform pipeline and writes CSS
  output.
- `omena build <file> --target-query "ie 11"` plans conservative
  target-sensitive passes from a Browserslist query or named target profile.
- `omena build <file> --target-query "ie 11" --allow-logical-to-physical`
  opts into blocked target-sensitive lowering classes when a project has chosen
  that compatibility tradeoff.
- `omena cascade <file> --line <n> --character <n>` reads cascade,
  computed-value, and custom-property LFP information at a `var(...)`
  reference position.
- `omena context-index <file>` reads semantic-owned `@layer`, `@container`,
  and `@scope` indexes for cascade-aware consumers.
- `omena expression-flow --engine-input-json input.json` analyzes
  cross-language class-value flow through the query-owned incremental runtime.
- `omena selector-projection --engine-input-json input.json` projects
  expression-domain values to target style selectors.
- `omena passes` lists the transform pass ids accepted by `omena build`.

Install the published CLI with Cargo:

```sh
cargo install omena-cli
omena check path/to/file.module.scss
omena build path/to/file.css --pass whitespace-strip
omena build path/to/file.css --target-query "ie 11"
omena build path/to/file.css --target-query "ie 11" --allow-logical-to-physical
omena cascade path/to/file.module.css --line 10 --character 16 --json
omena context-index path/to/file.module.scss --json
omena expression-flow --engine-input-json input.json --json
omena selector-projection --engine-input-json input.json --json
omena passes
```

The CLI intentionally consumes `omena-query` as the public facade instead of
calling parser or transform crates directly. Checker-grade diagnostics can be
layered in through the same query boundary as those checks become part of the
standalone surface.
