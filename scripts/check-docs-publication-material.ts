import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * The monorepo is the public source, so publication material ships directly
 * from `docs/`. This gate keeps the comparison boundaries, research framing,
 * and same-corpus benchmark policy aligned without generating a second tree.
 */

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const readDoc = (relativePath: string): string =>
  readFileSync(path.join(repoRoot, relativePath), "utf8");

const positioning = readDoc("docs/positioning.md");
const paperDraft = readDoc("docs/paper-draft.md");
const benchmarks = readDoc("docs/benchmarks.md");

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
  "Publication evidence requirements: docs/paper-draft.md",
  "(paper-draft.md)",
  "External speed comparisons require same-corpus",
  "Research-facing semantic substrates remain bounded",
  "No direct speed ranking",
  "No Sass compiler replacement claim",
  "No PostCSS ecosystem replacement claim",
  "No theorem-complete cascade",
  "No public Cargo 1.0 API freeze claim",
]) {
  assertIncludes(positioning, requiredPositioningBoundary);
}

for (const requiredPaperBoundary of [
  "Current Evidence Boundary",
  "Vue SFC source-language bridge",
  "Cascade-family work is framing-neutral substrate",
  "Dimensional/refinement work bridges cascade-family values",
  "does not fork a unit system",
  "complete SMT refinement",
  "Liquid-Haskell-style inference",
  "Edit-distance and cascade-margin work is fixture-witness substrate",
  "Contextual equality saturation is scaffold-only",
  "perceptual-check",
  "WCAG contrast bound for exact sRGB color/background pairs",
  "does not implement APCA, OKLab",
  "Publication Requirement",
  "source-controlled gate command",
  "benchmark corpus and machine record",
  "workspace publish dry-run",
]) {
  assertIncludes(paperDraft, requiredPaperBoundary);
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
  "full WPT/spec conformance",
  "Cargo 1.0.0 API freeze is complete",
  "Cascade-Proven",
  "M6 Evidence Boundary",
  "Research-facing M6",
  "Vue SFC phase 1",
]) {
  assertExcludes(positioning, unsupportedClaim);
  assertExcludes(paperDraft, unsupportedClaim);
}

assert.ok(includes("alpha beta", "beta"), "self-test: includes must detect a present needle");
assert.ok(!includes("alpha beta", "gamma"), "self-test: includes must reject an absent needle");

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.publication-material",
      positioningMaterialReady: true,
      paperDraftBoundaryReady: true,
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
