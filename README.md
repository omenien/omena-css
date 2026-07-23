# omena-css

omena-css is a CSS-family semantic engine with CLI, LSP, SDK, build, and editor surfaces.

## Install

VS Code:

[Install Omena CSS Modules from the Marketplace](https://marketplace.visualstudio.com/items?itemName=omena.omena-css).

CLI:

```bash
cargo install omena-cli --locked
```

Node and browser bindings:

```bash
npm install @omena/napi
npm install @omena/wasm
```

## 60-Second Check

Create `path/to/file.module.css` with no local `@keyframes pulse` declaration:

```css
.button {
  animation: pulse 1s ease;
}
```

Run the zero-config single-file lint path:

```bash
omena lint path/to/file.module.css
```

The report includes this finding:

```text
2:14 missing-keyframes @keyframes 'pulse' not found in this file.
```

The documentation gate executes this fixture with the Rust CLI and compares the
captured finding, so the example changes when product behavior changes.

## What You Get

- CSS, SCSS, Sass, Less, CSS Modules, and source-language fact production.
- Shared diagnostics, navigation, references, rename, explain, and code actions.
- Cascade-, value-, provenance-, and target-aware transform decisions.
- Lint, format, minify, bundle, module, Sass, intelligence, and migration commands.
- NAPI, WASM, CLI, and LSP workflows over snapshot-bound contracts.
- Vite, PostCSS, ESLint, Stylelint, and VS Code integration surfaces.

## Product Guides

| Area         | Guide                                                                                                           |
| ------------ | --------------------------------------------------------------------------------------------------------------- |
| Start here   | [Getting started](docs/getting-started.md)                                                                      |
| CLI          | [Command guide](rust/crates/omena-cli/README.md)                                                                |
| SDK          | [NAPI, WASM, CLI, and LSP workflows](docs/sdk.md)                                                               |
| LSP          | [Rust API docs](https://docs.rs/omena-lsp-server), [Zed](docs/clients/zed.md), [Neovim](docs/clients/neovim.md) |
| Bundler      | [Rust bundler](rust/crates/omena-bundler/README.md), [Vite host](packages/vite-plugin/README.md)                |
| Lint plugins | [ESLint](packages/eslint-plugin/README.md), [Stylelint](packages/stylelint-plugin/README.md)                    |
| Sass and SIF | [Compatibility and provenance](docs/sass-compat.md)                                                             |
| Codemods     | [Using `omena migrate`](docs/migrate-verb.md)                                                                   |
| VS Code      | [Extension behavior and settings](docs/vscode-extension.md)                                                     |
| Positioning  | [Scope, comparisons, and non-goals](docs/positioning.md)                                                        |
| Architecture | [Current codemap](ARCHITECTURE.md)                                                                              |
| Performance  | [Reproduction and evidence](docs/performance.md)                                                                |
| Releases     | [Changelog](CHANGELOG.md)                                                                                       |
| Contributing | [Contributor guide](CONTRIBUTING.md)                                                                            |
| Reference    | [Generated CLI, config, persona, and LSP tables](docs/reference/README.md)                                      |

## Runtime Shape

The packaged VS Code extension starts the Rust `omena-lsp-server`; the extension
host handles process supervision, UI, configuration, and file events. NAPI,
WASM, CLI, and LSP expose the same snapshot-bound query and typed-outcome
contracts. TypeScript adapters and shadow checks protect compatibility without
becoming a second CSS semantic authority.

The architecture, product-path matrix, generated IDL, and public reference
surfaces are checked in CI. Unknown, unresolved, and unsupported cases remain
typed outcomes rather than guessed successes.

## Development

```bash
pnpm install
pnpm check
pnpm test
pnpm build
```

Contributor recipes and the complete generated command inventory live in
[CONTRIBUTING.md](CONTRIBUTING.md) and [docs/reference/](docs/reference/README.md).

## License

[MIT](LICENSE)
