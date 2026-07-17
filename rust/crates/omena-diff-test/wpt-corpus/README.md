# WPT-Derived Conformance Corpus

This directory is the single checked-in WPT-derived corpus used by `omena-diff-test`. It contains the original reviewed seed fixtures and deterministic extracts from pinned WPT snapshots. The testharness HTML is never executed; extraction reads statically representable helper call sites and records their provenance.

## Provenance

- Upstream: [web-platform-tests/wpt](https://github.com/web-platform-tests/wpt)
- License and notice: [LICENSE.md](./LICENSE.md) and [NOTICE.md](./NOTICE.md)
- Current source pins and generated chunk hashes: [manifest.json](./manifest.json)
- Human-reviewed seed selections: [selections.json](./selections.json)
- Schema and regeneration instructions: [SCHEMA.md](./SCHEMA.md)

Pins are full WPT commit hashes. Extracted modules may advance independently, but every generated tuple and every reported count must identify the module pin that produced it. Updating a pin regenerates a reviewable corpus diff; runtime outcomes never update expected results automatically.

The existing 65 seed fixtures remain reviewed evidence. Extracted tuples extend this corpus beside them rather than replacing it.
