# omena-tsgo-client

`omena-tsgo-client` owns the Rust-side contract for talking to TypeScript 7
`tsgo` as source-side ground truth.

The crate is intentionally narrow:

- describe the long-lived tsgo process model;
- describe the type-fact request and result contracts;
- own a workspace-scoped persistent process pool for the tsgo API process;
- encode and drain tsgo's `Content-Length` JSON-RPC transport frames;
- write and read JSON-RPC frames against the managed tsgo process I/O;
- build typed type-fact RPC requests for tsgo's source-fact methods;
- execute the Rust JSON-RPC type-fact provider sequence over a transport;
- reduce tsgo literal and union type responses into CME-compatible type facts;
- keep the LSP request path policy explicit;
- provide a phase-3 boundary gate before the VS Code client becomes thin.

It does not run the full LSP server by itself. `omena-lsp-server` consumes this
boundary as the source-provider migration target.
