import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

interface AmbiguityFixtureV0 {
  readonly source: string;
  readonly sourceTokens: readonly string[];
  readonly generatedTokens: readonly string[];
  readonly matchingWindowStarts: readonly number[];
  readonly expectedSegment: {
    readonly originalStart: number;
    readonly originalEnd: number;
    readonly generatedStart: number;
    readonly generatedEnd: number;
  };
}

const sourcePath = "rust/crates/omena-query/src/style/transform.rs";
const fixturePath =
  "rust/crates/omena-query/tests/fixtures/linked-source-map-fallback-ambiguity.json";
const source = readFileSync(sourcePath, "utf8");
const fixture = JSON.parse(readFileSync(fixturePath, "utf8")) as AmbiguityFixtureV0;

const injectMatchCount = process.argv.includes("--inject-ambiguous-match-count");
const injectReasonDrift = process.argv.includes("--inject-fallback-reason-drift");
const injectSegmentDrift = process.argv.includes("--inject-fallback-segment-drift");

const matchingWindowStarts = injectMatchCount ? [0] : fixture.matchingWindowStarts;
// FALSIFIER: id=linked-source-map-ambiguous-window class=accounting via=--inject-ambiguous-match-count producer=can-fail entry=two-canonical-token-windows owner=linked-source-map-fallback
assert.deepEqual(matchingWindowStarts, [0, fixture.generatedTokens.length]);

const reasonEntries = [
  ...source.matchAll(/const\s+(LINKED_FALLBACK_[A-Z_]+_REASON):\s*&str\s*=\s*"([^"]+)";/gu),
].map((match) => ({ authority: match[1], value: match[2] }));
const reasonAuthority = Object.fromEntries(
  reasonEntries.map(({ authority, value }) => [
    authority,
    countOccurrences(source, value) + (injectReasonDrift ? 1 : 0),
  ]),
);
// FALSIFIER: id=linked-source-map-reason-authority class=accounting via=--inject-fallback-reason-drift producer=can-fail entry=single-source-reason-vocabulary owner=linked-source-map-fallback
assert.deepEqual(reasonAuthority, {
  LINKED_FALLBACK_EXACT_TOKEN_REASON: 1,
  LINKED_FALLBACK_AMBIGUOUS_TOKEN_REASON: 1,
  LINKED_FALLBACK_SOURCE_START_REASON: 1,
});

const observedSegment = {
  ...fixture.expectedSegment,
  ...(injectSegmentDrift ? { originalEnd: fixture.expectedSegment.originalEnd - 1 } : {}),
};
// FALSIFIER: id=linked-source-map-ambiguous-segment class=placement via=--inject-fallback-segment-drift producer=can-fail entry=source-start-to-source-end owner=linked-source-map-fallback
assert.deepEqual(observedSegment, {
  originalStart: 0,
  originalEnd: fixture.source.length,
  generatedStart: 0,
  generatedEnd: 15,
});

const test = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "linked_bundle_source_map_fallback",
    "--quiet",
  ],
  { encoding: "utf8" },
);
if (test.status !== 0) {
  throw new Error([test.stdout, test.stderr].filter(Boolean).join("\n"));
}

console.log(
  JSON.stringify({
    schemaVersion: "0",
    product: "omena-query.linked-source-map-fallback",
    canonicalWindowCount: matchingWindowStarts.length,
    reasonAuthorityCount: reasonEntries.length,
    productTestStatus: "passed",
  }),
);

function countOccurrences(input: string, needle: string): number {
  return input.split(needle).length - 1;
}
