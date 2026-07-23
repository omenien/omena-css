# Editor Setup

One language server, several hosts. Pick your editor:

| Editor            | Guide                                                                                                                                                   |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| VS Code           | [Extension behavior and settings](../vscode-extension.md) — bundled server, no separate install                                                         |
| Zed               | [Zed setup](zed.md) — standalone `omena-lsp-server`                                                                                                     |
| Neovim            | [Neovim setup](neovim.md) — standalone `omena-lsp-server`                                                                                               |
| Other LSP clients | Any client can start the standalone server binary; the capability surface is the generated [LSP capability reference](../reference/lsp-capabilities.md) |

Host resolution environment variables (all hosts):

- `OMENA_LSP_SERVER_PATH` — absolute path to a server binary, checked first.
- `OMENA_LSP_SERVER_COMMAND` — command name resolved from `PATH`.

Smoke-check a standalone install against the thin-client boundary:

```bash
pnpm omena-check run rust/omena-lsp-server/thin-client-boundary
```
