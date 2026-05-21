# WPT Seed Corpus Schema

This directory contains the generated, checked-in WPT-style seed corpus used by
the M4 conformance lane.

`selections.json` is the human-reviewed source selection file.
`scripts/generate-rust-omena-diff-test-wpt-corpus.ts` expands that selection
into `manifest.json` and chunk files. `manifest.json` records the upstream
source pin, chunk files, chunk hashes, generator metadata, and the known-failure
policy path. Chunk files use schema version `0` and contain parser-only,
layout-free CSS fixtures with an expected minified CSS literal.

This seed corpus is intentionally small. It proves the governance shape before a
larger sparse WPT mirror is introduced.

The known-failure policy also records Stage 2 promotion evidence. Stage 1 stays
advisory until the generated corpus reaches the configured fixture threshold,
known failures remain reviewed, and the configured consecutive-green run count
is met.

Refresh generated files with:

```bash
node --import tsx ./scripts/generate-rust-omena-diff-test-wpt-corpus.ts
```
