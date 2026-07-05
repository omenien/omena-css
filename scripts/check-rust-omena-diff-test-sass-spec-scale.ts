import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readdirSync } from "node:fs";
import path from "node:path";

interface SassSpecImportScaleChunkReportV0 {
  readonly chunkId: string;
  readonly path: string;
  readonly manifestFixtureCount: number;
  readonly actualFixtureCount: number;
  readonly manifestSha256: string;
  readonly actualSha256: string;
  readonly fixtureCountMatches: boolean;
  readonly sha256Matches: boolean;
}

interface SassSpecImportScaleReportV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly sourcePin: string;
  readonly sourceArchiveRoot: string;
  readonly sourceArchiveCount: number;
  readonly sourceArchiveScanSucceeded: boolean;
  readonly importedFixtureCount: number;
  readonly importedChunkCount: number;
  readonly seedFixtureCount: number;
  readonly perPushSmokeFixtureCount: number;
  readonly perPushSmokeFloorFixtureCount: number;
  readonly staticMustMatchCount: number;
  readonly expectedSoundBailCount: number;
  readonly parserRecoveryCount: number;
  readonly outOfScopeCount: number;
  readonly allImportedCountsMatchManifest: boolean;
  readonly allChunkHashesMatchManifest: boolean;
  readonly allSparsePathCountsMatchManifest: boolean;
  readonly allSourceArchivesUnderSparsePaths: boolean;
  readonly allSourceArchiveCountMatchesImportedFixtures: boolean;
  readonly allSmokeFloorHolds: boolean;
  readonly allImportScaleChecksHold: boolean;
  readonly chunks: readonly SassSpecImportScaleChunkReportV0[];
}

interface OmenaDiffTestBoundarySummaryV0 {
  readonly product: string;
  readonly sassSpecImportedFixtureCount: number;
  readonly sassSpecImportSourceArchiveCount: number;
  readonly sassSpecImportChunkCount: number;
  readonly sassSpecPerPushSmokeFixtureCount: number;
  readonly sassSpecPerPushSmokeFloorFixtureCount: number;
  readonly sassSpecStaticMustMatchCount: number;
  readonly sassSpecExpectedSoundBailCount: number;
  readonly sassSpecParserRecoveryCount: number;
  readonly sassSpecOutOfScopeCount: number;
  readonly allSassSpecImportScaleCountsMatch: boolean;
  readonly allSassSpecSmokeFloorHolds: boolean;
  readonly sassSpecImportScaleReport: SassSpecImportScaleReportV0;
}

const repoRoot = process.cwd();
const result = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-diff-test",
    "--bin",
    "omena-diff-test-boundary",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 32,
  },
);

if (result.error) {
  throw result.error;
}

assert.equal(
  result.status,
  0,
  `omena-diff-test boundary failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
);

const summary = JSON.parse(result.stdout) as OmenaDiffTestBoundarySummaryV0;
const scale = summary.sassSpecImportScaleReport;
const scannedArchiveCount = countHrxArchives(path.join(repoRoot, scale.sourceArchiveRoot));

assert.equal(summary.product, "omena-diff-test.boundary");
assert.equal(scale.schemaVersion, "0");
assert.equal(scale.product, "omena-diff-test.sass-spec-import-scale");
assert.match(scale.sourcePin, /^sass\/sass-spec@[0-9a-f]{40}$/u);
assert.equal(scale.sourceArchiveCount, scannedArchiveCount);
assert.equal(summary.sassSpecImportSourceArchiveCount, scannedArchiveCount);
assert.equal(summary.sassSpecImportedFixtureCount, scale.importedFixtureCount);
assert.equal(summary.sassSpecImportChunkCount, scale.importedChunkCount);
assert.equal(summary.sassSpecPerPushSmokeFixtureCount, scale.perPushSmokeFixtureCount);
assert.equal(summary.sassSpecPerPushSmokeFloorFixtureCount, scale.perPushSmokeFloorFixtureCount);
assert.equal(
  scale.importedFixtureCount,
  scale.staticMustMatchCount +
    scale.expectedSoundBailCount +
    scale.parserRecoveryCount +
    scale.outOfScopeCount,
);
assert.ok(scale.staticMustMatchCount > 0, "static bucket must be non-empty");
assert.ok(scale.expectedSoundBailCount > 0, "sound-bail bucket must be non-empty");
assert.ok(scale.parserRecoveryCount > 0, "parser-recovery bucket must be non-empty");
assert.ok(scale.outOfScopeCount > 0, "out-of-scope bucket must be non-empty");
assert.equal(
  summary.sassSpecImportedFixtureCount,
  summary.sassSpecStaticMustMatchCount +
    summary.sassSpecExpectedSoundBailCount +
    summary.sassSpecParserRecoveryCount +
    summary.sassSpecOutOfScopeCount,
);
assert.equal(summary.sassSpecStaticMustMatchCount, scale.staticMustMatchCount);
assert.equal(summary.sassSpecExpectedSoundBailCount, scale.expectedSoundBailCount);
assert.equal(summary.sassSpecParserRecoveryCount, scale.parserRecoveryCount);
assert.equal(summary.sassSpecOutOfScopeCount, scale.outOfScopeCount);
assert.ok(scale.sourceArchiveScanSucceeded, "source archive scan must succeed");
assert.ok(scale.importedFixtureCount > 0, "imported sass-spec fixture count must be non-empty");
assert.ok(scale.importedChunkCount > 0, "imported sass-spec chunk count must be non-empty");
assert.ok(scale.seedFixtureCount >= 4, "seed fixture floor must include the original seed");
assert.equal(scale.perPushSmokeFloorFixtureCount, scale.seedFixtureCount);
assert.ok(
  scale.perPushSmokeFixtureCount >= scale.perPushSmokeFloorFixtureCount,
  "per-push smoke fixture count must keep the seed floor",
);
assert.ok(scale.allImportedCountsMatchManifest, "manifest fixture counts must match chunks");
assert.ok(scale.allChunkHashesMatchManifest, "manifest chunk hashes must match chunk sources");
assert.ok(scale.allSparsePathCountsMatchManifest, "sparse-path counts must match chunks");
assert.ok(scale.allSourceArchivesUnderSparsePaths, "source archives must stay under sparse paths");
assert.ok(
  scale.allSourceArchiveCountMatchesImportedFixtures,
  "source archive count must match imported fixtures",
);
assert.ok(scale.allSmokeFloorHolds, "smoke fixture floor must hold");
assert.ok(scale.allImportScaleChecksHold, "complete import-scale gate must hold");
assert.ok(summary.allSassSpecImportScaleCountsMatch, "boundary scale gate must hold");
assert.ok(summary.allSassSpecSmokeFloorHolds, "boundary smoke-floor gate must hold");

for (const chunk of scale.chunks) {
  assert.ok(chunk.fixtureCountMatches, `${chunk.chunkId} fixture count mismatch`);
  assert.ok(chunk.sha256Matches, `${chunk.chunkId} sha256 mismatch`);
  assert.match(chunk.manifestSha256, /^[0-9a-f]{64}$/u);
  assert.equal(chunk.manifestSha256, chunk.actualSha256);
}

console.log(
  [
    "checked omena-diff-test sass-spec scale:",
    `sourceArchives=${scale.sourceArchiveCount}`,
    `importedFixtures=${scale.importedFixtureCount}`,
    `chunks=${scale.importedChunkCount}`,
    `smokeFixtures=${scale.perPushSmokeFixtureCount}`,
    `smokeFloor=${scale.perPushSmokeFloorFixtureCount}`,
  ].join(" "),
);

function countHrxArchives(root: string): number {
  let count = 0;
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      count += countHrxArchives(entryPath);
    } else if (entry.isFile() && entry.name.endsWith(".hrx")) {
      count += 1;
    }
  }
  return count;
}
