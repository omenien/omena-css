# `omena-lsp-server`

Rust language-server runtime for Omena CSS Modules. It owns the LSP transport
boundary, document and workspace state, diagnostics scheduling, query reuse,
disk-cache policy, source-provider routing, and editor-facing requests such as
hover, completion, references, rename, and explanations.

The server communicates over JSON-RPC using stdio or IPC and does not perform
network access. The VS Code extension packages the binary as a thin-client
endpoint; other editors can install the published crate's binary independently.

Primary verification:

```sh
cargo test --manifest-path rust/Cargo.toml -p omena-lsp-server
pnpm omena-check run rust/omena-lsp-server/split-boundary
```
