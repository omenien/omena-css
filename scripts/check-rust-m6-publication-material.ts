import { execFileSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface GeneratedWorkspaceReport {
  readonly destination: string;
  readonly crateCount: number;
  readonly publishDryRun: boolean;
  readonly verified: boolean;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspacePath = mkdtempSync(path.join(tmpdir(), "omena-css-m6-publication-"));

try {
  const prepareOutput = execFileSync(
    process.execPath,
    ["scripts/prepare-omena-css-workspace.mjs", "--dest", workspacePath, "--force"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  const generated = JSON.parse(prepareOutput) as GeneratedWorkspaceReport;
  assert.equal(generated.destination, workspacePath);
  assert.equal(generated.crateCount, 39);
  assert.equal(generated.publishDryRun, false);
  assert.equal(generated.verified, false);

  const readGenerated = (relativePath: string): string =>
    readFileSync(path.join(workspacePath, relativePath), "utf8");

  const readme = readGenerated("README.md");
  const benchmarks = readGenerated("docs/benchmarks.md");
  const positioning = readGenerated("docs/positioning.md");
  const paperDraft = readGenerated("docs/paper-draft.md");
  const release = readGenerated("docs/release.md");
  const publishWorkflow = readGenerated(".github/workflows/publish.yml");

  assertIncludes(readme, "[Positioning](docs/positioning.md)");
  assertIncludes(readme, "[Paper draft outline](docs/paper-draft.md)");

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
    "does not implement WCAG, APCA, OKLab",
    "Publication Requirement",
    "source-controlled gate command",
    "benchmark corpus and machine record",
    "generated standalone workspace verification",
  ]) {
    assertIncludes(paperDraft, requiredPaperBoundary);
  }

  for (const benchmarkBoundary of [
    "must report the command, input set, machine class, and comparison baseline",
    "Do not treat a single synthetic benchmark as product readiness",
  ]) {
    assertIncludes(benchmarks, benchmarkBoundary);
  }
  assertIncludes(release, "Avoid private planning labels");
  assertOrder(
    publishWorkflow,
    'version="$(crate_version "$manifest")"',
    'if [[ "$PUBLISH_MODE" == "dry-run" ]]',
  );
  assertIncludes(publishWorkflow, 'if crate_version_exists "$package" "$version"; then');

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

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "rust.m6-publication-material",
        generatedWorkspace: true,
        generatedCrateCount: generated.crateCount,
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
} finally {
  rmSync(workspacePath, { force: true, recursive: true });
}

function assertIncludes(haystack: string, needle: string): void {
  assert.ok(haystack.includes(needle), `expected generated material to include ${needle}`);
}

function assertOrder(haystack: string, before: string, after: string): void {
  const beforeIndex = haystack.indexOf(before);
  const afterIndex = haystack.indexOf(after);
  assert.ok(beforeIndex >= 0, `expected generated material to include ${before}`);
  assert.ok(afterIndex >= 0, `expected generated material to include ${after}`);
  assert.ok(beforeIndex < afterIndex, `expected ${before} to appear before ${after}`);
}

function assertExcludes(haystack: string, needle: string): void {
  assert.ok(!haystack.includes(needle), `generated material must not claim ${needle}`);
}
