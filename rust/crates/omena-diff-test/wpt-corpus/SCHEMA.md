# WPT Seed Corpus Schema

This directory contains the generated, checked-in WPT-style seed corpus used by
the M4 conformance lane.

`manifest.json` is the source of truth. It records the upstream source pin,
chunk files, chunk hashes, and the known-failure policy path. Chunk files use
schema version `0` and contain parser-only, layout-free CSS fixtures with an
expected minified CSS literal.

This seed corpus is intentionally small. It proves the governance shape before a
larger sparse WPT mirror is introduced.
