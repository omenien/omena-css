# WPT-Derived Corpus Schema

This directory contains the single generated, checked-in WPT-derived corpus used by
the conformance lane. It has two additive evidence populations: reviewed seed
fixtures and statically extracted tier-zero subtests.

`selections.json` is the human-reviewed source selection file.
`scripts/generate-rust-omena-diff-test-wpt-corpus.ts` expands that selection
into `manifest.json` and chunk files. `manifest.json` records the upstream
source pin, chunk files, per-chunk stage, chunk hashes, generator metadata, and
the known-failure policy path. The manifest also records generated sparse-path
fixture counts at the corpus and chunk levels. Chunk files use schema version
`0` and contain parser-only, layout-free CSS fixtures with an expected minified
CSS literal. The boundary report verifies that every pinned sparse WPT path is
represented by at least one checked fixture and that generated sparse-path
counts match the checked fixture set, so sparse paths cannot drift into
manifest-only metadata.

`extracted/tier-zero-tuples.json` is generated from a checkout at the extraction
pin recorded in `manifest.json`. Each tuple records the helper call, property or
syntax subject, source value, permitted serialization set, validity, upstream
path and line, source-text hash, helper class, and every `rel=help` link from the
source document. `extracted/tier-zero-coverage.json` accounts for every target
module in three honest buckets: extracted subtests, skipped dynamic calls with a
bounded reason, and files that do not import a tier-zero helper. Tentative and
optional files have separate excluded counters.

The extractor parses HTML and JavaScript but never executes testharness. Static
calls are classified at subtest granularity, so files that mix test tiers do not
force a file-wide classification. Permitted WPT serializations are always stored
as a set, including when the upstream helper supplies one canonical string.

The existing runner evaluates the reviewed seed population unchanged. It also
evaluates a deterministic module-stratified extract sample against Omena,
Lightning CSS, and the WPT expected set. Set `OMENA_WPT_FULL_CORPUS=1` to run all
extracted tuples through that same path.

The known-failure policy records Stage 2 promotion evidence for the blocking
chunk. Additional Stage 1 advisory chunks can be checked in beside it to expand
coverage without invalidating existing green-run evidence. Advisory fixtures
still run through the three-way Omena/lightningcss/WPT checker; they simply do
not count toward the current blocking chunk's reviewed green-run hash until
promoted.

Refresh generated files with:

```bash
node --import tsx ./scripts/generate-rust-omena-diff-test-wpt-corpus.ts
```

Regenerate the tier-zero extract from a checkout whose `HEAD` matches the
committed extraction pin, then refresh the manifest:

```bash
pnpm update:rust-omena-diff-test-wpt-extraction --wpt-root /path/to/wpt
node --import tsx ./scripts/generate-rust-omena-diff-test-wpt-corpus.ts
```

Use the corresponding `check:rust-omena-diff-test-wpt-extraction-source` command
with `--wpt-root` to prove byte equality without rewriting committed artifacts.
