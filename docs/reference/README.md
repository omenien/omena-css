<!-- Generated from product code. Do not edit by hand. -->
# Omena reference

These tables are rendered from the current product contracts and checked in CI.

- [CLI commands and product verbs](./cli.md)
- [Persona presets](./personas.md)
- [Configuration keys](./configuration.md)
- [LSP capabilities](./lsp-capabilities.md)

## Contributor checks

| Command | Contract |
| --- | --- |
| `pnpm omena-check run rust/omena-cli-verb-census` | Keep product commands, dispatch, config ownership, and the verb census aligned. |
| `pnpm omena-check run rust/omena-bridge/boundary` | Exercise the built-in style-intelligence registry through its product boundary. |
| `pnpm omena-check run rust/omena-fact-precision-census` | Classify every production precision source used by intelligence providers. |
| `pnpm omena-check run rust/omena-config-schema-census` | Keep omena.toml tables, product ownership, and schema consumers synchronized. |
| `pnpm omena-check run docs/reference-surface` | Regenerate and verify public CLI, config, persona, SDK, and LSP references. |
