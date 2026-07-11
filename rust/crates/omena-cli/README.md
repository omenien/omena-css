# omena-cli

`omena-cli` is the command-line consumer surface for the Omena CSS workspace.

Current commands:

- `omena facts <file>` parses a CSS-family file and reports parser-owned facts.
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
- `omena style-diagnostics <file>` reads query-owned style diagnostics for a
  CSS-family file.
- `omena style-hover-candidates <file>` reads query-owned style hover
  candidates for a CSS-family file.
- `omena style-completion <file> --line <n> --character <n>` reads query-owned
  style completions at a source position.
- `omena source-diagnostics <source-uri> --candidates-json candidates.json`
  reads query-owned source diagnostics from precomputed missing-selector
  candidates.
- `omena expression-flow --engine-input-json input.json` analyzes
  cross-language class-value flow through the query-owned incremental runtime.
- `omena selector-projection --engine-input-json input.json` projects
  expression-domain values to target style selectors.
- `omena perceptual-check <file> --json` emits downstream perceptual-tool JSON
  from Omena facts, including a fixture-witnessed WCAG contrast bound for exact
  sRGB color/background pairs. This is not a complete APCA/OKLab/full
  perceptual algorithm or a public-safety claim.
- `omena passes` lists the transform pass ids accepted by `omena build`.

Install the published CLI with Cargo:

```sh
cargo install omena-cli
omena facts path/to/file.module.scss
omena build path/to/file.css --pass whitespace-strip
omena build path/to/file.css --target-query "ie 11"
omena build path/to/file.css --target-query "ie 11" --allow-logical-to-physical
omena cascade path/to/file.module.css --line 10 --character 16 --json
omena context-index path/to/file.module.scss --json
omena style-diagnostics path/to/file.module.scss --json
omena style-hover-candidates path/to/file.module.scss --json
omena style-completion path/to/file.module.scss --line 10 --character 16 --json
omena source-diagnostics file:///workspace/src/App.tsx --candidates-json candidates.json --json
omena expression-flow --engine-input-json input.json --json
omena selector-projection --engine-input-json input.json --json
omena perceptual-check path/to/file.module.css --json
omena passes
```

## Configuration

`omena.toml` is the canonical configuration file. The CLI finds the nearest
file while walking toward the workspace root and loads every product section
into one typed snapshot. Existing `omena.config.toml` and
`omena.config.json` build configurations remain compatible; if more than one
candidate exists in the selected directory, Omena reports the shadowed files.

```toml
extends = "./config/base.toml"

[lint]
profile = "recommended"

[format]
mode = "stable"
lineWidth = 100

[build]
minify = true
output = "dist/app.css"

[[overrides]]
pattern = "*.module.scss"

[overrides.format]
lineWidth = 120
```

Extended files use deterministic table merge and scalar/array replacement.
`[[overrides]]` applies after that merge. `.editorconfig` may supply
`indent_size` and `max_line_length` defaults for `[format]`, while explicit
`omena.toml` values win. Environment interpolation is fail-closed and limited
to path-bearing fields such as `extends`, workspace roots, and build input or
output paths. Every config source file, matching override, EditorConfig input, and
environment value contributes to the configuration digest.

Digest paths are normalized relative to the selected config directory, so the
same project snapshot produces the same key after being checked out elsewhere.
This makes the digest suitable for native or WASM request envelopes and remote
cache keys without making executable JavaScript configs part of the canonical
reproducibility contract.

Unknown keys are reported rather than ignored. Sections whose product command
is not wired yet are retained and reported as `notYetConsumed`, so declaring a
future setting never silently discards it.

The CLI intentionally consumes `omena-query` as the public facade instead of
calling parser or transform crates directly. Checker-grade diagnostics can be
layered in through the same query boundary as those checks become part of the
standalone surface.
