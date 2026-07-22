# Neovim

This project now ships a standalone Rust `omena-lsp-server` entrypoint.

Install the editor-agnostic server from crates.io:

```bash
cargo install omena-lsp-server --version 0.3.0
```

The standalone server entrypoint is then:

```text
omena-lsp-server
```

Source repository: https://github.com/omenien/omena-css

For a local checkout, you can also build the repo directly:

```bash
pnpm install
pnpm build
```

The repo-local server entrypoint is:

```text
<repo>/dist/bin/<platform>-<arch>/omena-lsp-server
```

## Neovim 0.11+

Neovim's built-in LSP client can define a config with `vim.lsp.config()` and enable it with `vim.lsp.enable()`.

Example:

```lua
vim.lsp.config("omena_css", {
  cmd = {
    "omena-lsp-server",
  },
  filetypes = {
    "typescript",
    "typescriptreact",
    "javascript",
    "javascriptreact",
    "css",
    "scss",
    "less",
  },
  root_markers = {
    "tsconfig.json",
    "package.json",
    ".git",
  },
})

vim.lsp.enable("omena_css")
```

## Notes

- Prefer the crates.io-installed `omena-lsp-server` for non-VS Code editors.
- If you use a repo-local build instead, replace `darwin-arm64` with your packaged `<platform>-<arch>` directory.
- This server complements your main JS/TS language server. Keep `ts_ls`, `vtsls`, or your existing TypeScript server enabled.
- The Omena CSS Modules server provides:
  - hover
  - definition
  - references
  - rename
  - diagnostics
  - code actions
    for CSS Modules semantics across JS/TS and style files.
- The repo-local smoke command for this transport is:

```bash
pnpm omena-check run rust/omena-lsp-server/thin-client-boundary
```

## References

- Neovim LSP docs: https://neovim.io/doc/user/lsp.html
