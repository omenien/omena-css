# Workspace Session Routing

Omena has one semantic workspace authority in `omena-query`, with several host-specific ways to
retain and query that authority. A resident process is an optional transport for repeated CLI
work; it does not replace the language server, create a second semantic engine, or become a
requirement for one-shot commands.

## Consumer Routes

| Consumer | Semantic owner | Repeated-work route | Failure or compatibility route |
| --- | --- | --- | --- |
| `editor` | `omena-lsp-server` | LSP document and workspace state | not applicable |
| `cli-one-shot` | `omena-cli` and `omena-query` | direct process execution | not applicable |
| `cli-watch` | `omena-query` through `omenad` | loopback resident workspace session | direct CLI execution |
| `eslint` | `omena-query` through `@omena/napi` | in-process `CachedWorkspace` | direct CLI diagnostics when NAPI is unavailable |
| `bundler-host` | `omena-query` through the existing bundler-host protocol | NAPI or WASM adapter state | adapter-selected NAPI/WASM compatibility route; no daemon hop |

The `omena check`, `omena lint`, `omena fmt`, and `omena explain` watch modes share the CLI watch
route. Their one-shot forms retain the unified direct dispatcher and do not discover or start a
resident process.

## Configuration

The optional `[workspace.session]` table belongs to the same resolved `omena.toml` snapshot used
by the rest of the CLI:

```toml
[workspace.session]
enabled = true
idleTimeoutMs = 300000
requestDeadlineMs = 30000
maxResponseBytes = 16777216
```

- `enabled` defaults to `true`. Setting it to `false` keeps watch mode on direct execution and
  prevents daemon discovery or startup.
- `idleTimeoutMs` controls how long an inactive spawned process remains available.
- `requestDeadlineMs` bounds each resident read operation.
- `maxResponseBytes` bounds encoded operation responses and cannot exceed the transport frame
  limit.

The canonical workspace root and resolved config-content digest form the session identity. A
different root or config digest cannot attach to an existing session. Every request also names the
workspace snapshot it observed, so stale requests fail instead of reading a newer state silently.

## Lifecycle And Trust Boundary

`omenad` binds an operating-system-selected loopback TCP endpoint and publishes the endpoint in an
atomically replaced local file. It accepts multiple local clients, pins the first workspace and
config identity, bounds request time and response size, and removes its endpoint on idle exit or
shutdown. An automatically started process outlives its initial watch client until that lifecycle
boundary. Watch clients reconnect and synchronize current sources when possible, then fall back to
direct execution if startup, handshake, or a later request fails.

The current transport is local and unauthenticated. It is not a remote service boundary.

## Explicit Residuals

The following are deliberately outside the current contract:

- a single process hosting multiple independent workspace roots;
- remote or multi-machine workspace sessions;
- authenticated or encrypted daemon transport;
- replacing editor LSP lifecycle and document ownership with the daemon;
- making daemon availability mandatory for CI, one-shot CLI, NAPI, WASM, or bundler adapters.
