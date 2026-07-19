# omena-cli

Start with the [product overview](../../../README.md). External Sass and lockfile
behavior lives in [the compatibility guide](../../../docs/sass-compat.md), and
plan-first source changes live in [the migration guide](../../../docs/migrate-verb.md).

`omena-cli` is the command-line consumer surface for the Omena CSS workspace.

## Commands

<!-- BEGIN GENERATED: OMENA CLI COMMANDS -->
<!-- Generated from product code. Do not edit by hand. -->

| Command                               | Role                | Purpose                                                                            |
| ------------------------------------- | ------------------- | ---------------------------------------------------------------------------------- |
| `omena check`                         | Compatibility alias | Compatibility route through `facts_file`.                                          |
| `omena facts`                         | Specialized command | Parse a CSS-family source and report parser-owned facts.                           |
| `omena lint`                          | Product command     | Run semantic and compatibility lint rules.                                         |
| `omena fmt`                           | Product command     | Format CSS-family sources through the typed CST formatter contract.                |
| `omena minify`                        | Product command     | Minify a stylesheet with an explicit semantic profile and backend.                 |
| `omena bundle`                        | Product command     | Bundle a source entry and emit CSS plus optional evidence.                         |
| `omena modules`                       | Product command     | Emit or verify typed CSS Modules interfaces.                                       |
| `omena sass`                          | Product command     | Inspect Sass module graphs and compatibility diagnostics.                          |
| `omena intel`                         | Product command     | Query workspace style-intelligence providers.                                      |
| `omena migrate`                       | Product command     | Plan a named source migration without applying unsafe edits.                       |
| `omena verify`                        | Product command     | Verify user-workspace product contracts and evidence.                              |
| `omena ci`                            | Product command     | Run the configured CI product workflow.                                            |
| `omena sdk`                           | Specialized command | Execute a generated SDK workflow request against an ephemeral workspace runtime.   |
| `omena explain`                       | Product command     | Explain a diagnostic, transform decision, or retained artifact.                    |
| `omena build`                         | Specialized command | Run the conservative transform pipeline.                                           |
| `omena passes`                        | Specialized command | List transform pass ids accepted by `omena build --pass`.                          |
| `omena compress`                      | Specialized command | Estimate an MDL minimum-description summary for a style source.                    |
| `omena context`                       | Specialized command | Derive transform context from EngineInputV2 semantic reachability.                 |
| `omena expression-flow`               | Specialized command | Analyze cross-language class-value flow from EngineInputV2.                        |
| `omena selector-projection`           | Specialized command | Project expression-domain flow values to target style selectors.                   |
| `omena cascade`                       | Specialized command | Read cascade and custom-property LFP information at a source position.             |
| `omena context-index`                 | Specialized command | Read @layer, @container, and @scope context indexes.                               |
| `omena style-diagnostics`             | Specialized command | Read query-owned style diagnostics for a CSS-family file.                          |
| `omena style-hover-candidates`        | Specialized command | Read query-owned style hover candidates for a CSS-family file.                     |
| `omena style-completion`              | Specialized command | Read query-owned style completions at a source position.                           |
| `omena source-diagnostics`            | Specialized command | Read query-owned source diagnostics from precomputed missing-selector candidates.  |
| `omena dynamic-classname-diagnostics` | Specialized command | Read query-owned dynamic className M-tier diagnostics from an input JSON contract. |
| `omena perceptual-check`              | Specialized command | Emit downstream perceptual-check JSON from Omena style facts.                      |
| `omena lock`                          | Specialized command | Verify local Omena lockfile integrity.                                             |
| `omena sif`                           | Specialized command | Generate local Sass Interface File artifacts.                                      |
| `omena provenance`                    | Specialized command | Inspect deferred/advisory SIF provenance metadata without network access.          |
| `omena report`                        | Specialized command | Report soundiness and diagnostic-noise visibility for a workspace slice.           |
| `omena audit`                         | Specialized command | Run feature-gated audit surfaces.                                                  |

<!-- END GENERATED: OMENA CLI COMMANDS -->

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
