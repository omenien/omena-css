import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

/**
 * rust/m6-publication-material
 *
 * Model A (direct publish): the monorepo IS the public repo, so the M6
 * publication material ships from `docs/` directly — there is no longer a
 * generated standalone workspace to template these docs into. This gate reads
 * the in-tree `docs/positioning.md`, `docs/paper-draft.md`, and
 * `docs/benchmarks.md` and proves the evidence-boundary discipline holds:
 *
 *   - positioning names the comparison tools (Lightning CSS / PostCSS /
 *     Dart Sass / Biome CSS) with their source-anchor URLs, states the
 *     not-a-build-replacement + same-corpus + staged-substrate boundaries, and
 *     lists the current non-claims;
 *   - paper-draft carries the "M6 Evidence Boundary" + per-substrate hedges and
 *     the "Publication Requirement" evidence list;
 *   - benchmarks carries the same-corpus reporting policy.
 *
 * It also asserts the UNSUPPORTED-claim exclusions are absent from both the
 * positioning and paper-draft docs, so an over-claim can never silently land in
 * the public material. The assertIncludes/assertExcludes helpers are exercised
 * by the boundary strings themselves; this gate no longer runs the (retired)
 * workspace generator nor checks generator-only artifacts.
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
  "https://www.sasscss.com/dart-sass",
  "https://biomejs.dev/",
]) {
  assertIncludes(positioning, sourceUrl);
}
for (const requiredPositioningBoundary of [
  "not positioned as a build-time replacement",
  "External speed comparisons require same-corpus",
  "Research-facing M6 surfaces are staged substrates",
  "No direct speed ranking",
  "No Sass compiler replacement claim",
  "No PostCSS ecosystem replacement claim",
  "No theorem-complete cascade",
  "No public Cargo 1.0 API freeze claim",
]) {
  assertIncludes(positioning, requiredPositioningBoundary);
}

for (const requiredPaperBoundary of [
  "M6 Evidence Boundary",
  "Vue SFC phase 1",
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
]) {
  assertExcludes(positioning, unsupportedClaim);
  assertExcludes(paperDraft, unsupportedClaim);
}

// Self-test: the include/exclude predicates flag a present/absent needle.
{
  assert.ok(includes("alpha beta", "beta"), "self-test: includes must detect a present needle");
  assert.ok(!includes("alpha beta", "gamma"), "self-test: includes must reject an absent needle");
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.m6-publication-material",
      positioningMaterialReady: true,
      paperDraftBoundaryReady: true,
      sameCorpusBenchmarkPolicyReady: true,
      unsupportedClaimGuardReady: true,
      evidenceClaimLevel: "m6PublicationScaffoldWithExplicitBoundaries",
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
