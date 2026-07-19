# Zed

This project now ships a standalone Rust `omena-lsp-server` entrypoint.

Install the editor-agnostic server from crates.io:

```bash
cargo install omena-lsp-server --version 0.2.1
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

## settings.json example

Zed can run an additional language server by:

1. defining a server under `lsp`
2. enabling that server for the languages you want under `languages.<Language>.language_servers`

Example:

```json
{
  "lsp": {
    "omena-css": {
      "binary": {
        "path": "omena-lsp-server",
        "arguments": []
      }
    }
  },
  "languages": {
    "TypeScript": {
      "language_servers": ["omena-css", "..."]
    },
    "TSX": {
      "language_servers": ["omena-css", "..."]
    },
    "JavaScript": {
      "language_servers": ["omena-css", "..."]
    },
    "CSS": {
      "language_servers": ["omena-css", "..."]
    },
    "SCSS": {
      "language_servers": ["omena-css", "..."]
    },
    "Less": {
      "language_servers": ["omena-css", "..."]
    }
  }
}
```

`"..."` keeps Zed's default language servers enabled alongside Omena CSS Modules.

## Notes

- Prefer the crates.io-installed `omena-lsp-server` for non-VS Code editors.
- If you use a repo-local build instead, replace `darwin-arm64` with your packaged `<platform>-<arch>` directory.
- This server is intended to run beside Zed's default TS/JS server, not replace it.
- Start with `TypeScript`, `TSX`, and `SCSS` if you want the smallest config surface.
- The repo-local smoke command for this transport is:

```bash
pnpm omena-check run rust/omena-lsp-server/thin-client-boundary
```

## References

- Zed language configuration docs: https://zed.dev/docs/languages/typescript
- Zed language server settings examples: https://zed.dev/docs/languages/dart
