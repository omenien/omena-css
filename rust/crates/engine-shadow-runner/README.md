# `engine-shadow-runner`

Internal JSON-lines runner for exercising Omena engine contracts outside the
editor process. It accepts typed engine and query payloads, executes the same
Rust authorities used by product surfaces, and emits deterministic summaries
for parity, differential, and release gates.

The runner is packaged inside the extension's native artifact matrix but is not
published as a standalone crate.
