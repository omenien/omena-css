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

`expectations/` mirrors every extracted WPT module path, including modules with
zero tier-zero tuples. Expectation reasons come from one bounded vocabulary;
`unstable` entries must use the quarantined status. Every manifest is byte-bound
to `adjudications/reviewed-expectations.json`. Updating observed outcomes never
updates these files: an expectation change requires a matching maintainer review
record carrying the new manifest SHA-256. The runner forces reviewed expectations
into its deterministic sample and rejects stale expected failures that now pass.

The WPT module rows in `omena-spec-audit/data/spec-sources.json` are independent
source pins. They share the first extraction epoch by reviewed policy, but each
row points to its own path-mirrored expectation manifest so later module epochs
can advance independently.

`omena-spec-audit/data/omena-conformance-dashboard.json` is a generated join of
the full extracted WPT run, the five-axis capability ledger, and the runtime
transform-conformance ledger. It reports per-module and per-CSS-spec X/Y counts
with both WPT and Webref pins. Non-CSS `rel=help` references and unlinked cases
remain separate, and the schema intentionally has no headline percentage. The
check command reruns both runtime producers and rejects any hand-edited number.

The known-failure policy records Stage 2 promotion evidence for the blocking
chunk. Additional Stage 1 advisory chunks can be checked in beside it to expand
coverage without invalidating existing green-run evidence. Advisory fixtures
still run through the three-way Omena/lightningcss/WPT checker; they simply do
not count toward the current blocking chunk's reviewed green-run hash until
promoted.

`module-promotion-policy.json` applies one reviewed promotion template to the
existing primary seed chunk and every extracted module. A module becomes
blocking only when its fixture floor, zero-failure ceiling, review interval, and
consecutive scheduled-run evidence all pass. The existing blocking chunk is
re-certified through this path; newly extracted modules begin advisory.
`adjudications/reviewed-promotion-template.json` pins the template constants, so
editing a threshold without a matching review digest fails the promotion gate.

The scheduled benchmark workflow records full-corpus wall time and normalized
microseconds per evaluated tuple against
`omena-benchmarks/baselines/wpt-case-count-baseline-v0.json`. The first baseline
is explicitly recorder-only. It cannot become threshold-enforcing until two
comparable scheduled Linux samples exist; recorder mode must retain that named
exit condition.

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
