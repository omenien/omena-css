import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * The monorepo is the public source, so publication material ships directly
 * from `docs/`. This gate keeps the comparison boundaries and same-corpus
 * benchmark policy aligned without generating a second tree.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const readDoc = (relativePath: string): string =>
  readFileSync(path.join(repoRoot, relativePath), "utf8");

const positioning = readDoc("docs/positioning.md");
const benchmarks = readDoc("docs/performance.md");

for (const tool of ["Lightning CSS", "PostCSS", "Dart Sass", "Biome CSS"]) {
  assertIncludes(positioning, tool);
}
for (const sourceUrl of [
  "https://lightningcss.dev/",
  "https://postcss.org/",
  "https://sass-lang.com/dart-sass/",
  "https://biomejs.dev/",
]) {
  assertIncludes(positioning, sourceUrl);
}
for (const requiredPositioningBoundary of [
  "not positioned as a build-time replacement",
  "build, bundle, minify",
  "External speed comparisons require same-corpus",
  "Research-facing semantic substrates remain bounded",
  "publishes speed comparisons only with same-corpus, same-machine",
  "does not compile Sass",
  "not a general PostCSS plugin host",
  "has no 1.0 freeze",
]) {
  assertIncludes(positioning, requiredPositioningBoundary);
}

for (const benchmarkBoundary of [
  "must report the command, input set, machine class, and comparison baseline",
  "Do not treat a single synthetic benchmark as product readiness",
]) {
  assertIncludes(benchmarks, benchmarkBoundary);
}

for (const unsupportedClaim of [
  "replaces Lightning CSS",
  "replaces PostCSS",
  "replaces Dart Sass",
  "is faster than Lightning CSS",
  "theorem-proven cascade semantics",
  "sheaf",
  "cosheaf",
  "modal",
  "Datalog",
  "egglog",
  "perceptual",
  "full WPT/spec conformance",
  "Cargo 1.0.0 API freeze is complete",
  "Cascade-Proven",
  "M6 Evidence Boundary",
  "Research-facing M6",
  "Vue SFC phase 1",
]) {
  assertExcludes(positioning, unsupportedClaim);
}

assert.ok(includes("alpha beta", "beta"), "self-test: includes must detect a present needle");
assert.ok(!includes("alpha beta", "gamma"), "self-test: includes must reject an absent needle");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.publication-material",
      positioningMaterialReady: true,
      sameCorpusBenchmarkPolicyReady: true,
      unsupportedClaimGuardReady: true,
      evidenceClaimLevel: "publicationScaffoldWithExplicitBoundaries",
    },
    null,
    2,
  )}\n`,
);

function includes(haystack: string, needle: string): boolean {
  return haystack.includes(needle);
}

function assertIncludes(haystack: string, needle: string): void {
  assert.ok(includes(haystack, needle), `expected publication material to include ${needle}`);
}

function assertExcludes(haystack: string, needle: string): void {
  assert.ok(!includes(haystack, needle), `publication material must not claim ${needle}`);
}
