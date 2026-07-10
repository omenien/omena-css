import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";
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
  readonly upstreamArchiveCount: number;
  readonly upstreamScaleArtifactMatchesManifest: boolean;
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
  readonly allUpstreamArchiveCountExceedsImportedFixtures: boolean;
  readonly allUpstreamArchiveCountExceedsSourceArchives: boolean;
  readonly allSmokeFloorHolds: boolean;
  readonly allImportScaleChecksHold: boolean;
  readonly chunks: readonly SassSpecImportScaleChunkReportV0[];
}

interface SassSpecUpstreamScaleArtifactV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
  };
  readonly archiveExtension: ".hrx";
  readonly archiveCount: number;
  readonly sparsePathArchiveCounts: readonly {
    readonly sparsePath: string;
    readonly archiveCount: number;
  }[];
  readonly importedSourceArchiveCount: number;
  readonly importedSourceArchiveByteMatchCount: number;
  readonly allImportedSourceArchivesMatchUpstream: boolean;
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
  readonly sassSpecStaticMatchCaseCount: number;
  readonly sassSpecStaticMatchCheckedCaseCount: number;
  readonly sassSpecStaticMatchDeclarationValueMatchCount: number;
  readonly allSassSpecStaticMatchChecksHold: boolean;
  readonly allSassSpecImportScaleCountsMatch: boolean;
  readonly allSassSpecSmokeFloorHolds: boolean;
  readonly sassSpecImportScaleReport: SassSpecImportScaleReportV0;
  readonly sassSpecStaticMatchReport: SassSpecStaticMustMatchReportV0;
}

interface SassSpecStaticMustMatchReportV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly caseCount: number;
  readonly checkedCaseCount: number;
  readonly declarationValueCount: number;
  readonly matchedDeclarationValueCount: number;
  readonly allStaticValuesMatchOracle: boolean;
  readonly allStaticMatchChecksHold: boolean;
}

const repoRoot = process.cwd();
const diffTestLibPath = path.join(repoRoot, "rust/crates/omena-diff-test/src/lib.rs");
const scssEvalSourceRoot = path.join(repoRoot, "rust/crates/omena-scss-eval/src");
const configuredBailSiteFiles = readConfiguredBailSiteFiles(diffTestLibPath);
const discoveredBailSiteFiles = recursiveRustFiles(scssEvalSourceRoot)
  .filter((filePath) =>
    /UnsupportedDynamic|FuelExhausted|::Unsupported/u.test(readFileSync(filePath, "utf8")),
  )
  .map((filePath) => path.relative(repoRoot, filePath).split(path.sep).join("/"))
  .toSorted();
assert.deepEqual(
  configuredBailSiteFiles,
  discoveredBailSiteFiles,
  "the sass-spec bail-site source census must include every evaluator source with a bail marker",
);
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
const upstreamScale = readJson<SassSpecUpstreamScaleArtifactV0>(
  path.join(repoRoot, "rust/crates/omena-diff-test/sass-spec-corpus/upstream-scale.json"),
);

assert.equal(summary.product, "omena-diff-test.boundary");
assert.equal(scale.schemaVersion, "0");
assert.equal(scale.product, "omena-diff-test.sass-spec-import-scale");
assert.match(scale.sourcePin, /^sass\/sass-spec@[0-9a-f]{40}$/u);
assert.equal(scale.sourceArchiveCount, scannedArchiveCount);
assert.equal(upstreamScale.product, "omena-diff-test.sass-spec-upstream-scale");
assert.equal(upstreamScale.source.pin, scale.sourcePin);
assert.equal(scale.upstreamArchiveCount, upstreamScale.archiveCount);
assert.equal(upstreamScale.importedSourceArchiveCount, scale.sourceArchiveCount);
assert.equal(
  upstreamScale.importedSourceArchiveByteMatchCount,
  upstreamScale.importedSourceArchiveCount,
);
assert.ok(
  upstreamScale.allImportedSourceArchivesMatchUpstream,
  "imported source archives must match pinned upstream bytes",
);
assert.ok(
  scale.upstreamArchiveCount > scale.importedFixtureCount,
  "upstream archive count must exceed imported smoke fixtures",
);
assert.ok(
  scale.upstreamArchiveCount > scale.sourceArchiveCount,
  "upstream archive count must exceed checked-in source sample",
);
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
assert.equal(summary.sassSpecStaticMatchReport.product, "omena-diff-test.sass-spec-static-match");
assert.equal(summary.sassSpecStaticMatchCaseCount, summary.sassSpecStaticMatchReport.caseCount);
assert.equal(
  summary.sassSpecStaticMatchCheckedCaseCount,
  summary.sassSpecStaticMatchReport.checkedCaseCount,
);
assert.equal(
  summary.sassSpecStaticMatchDeclarationValueMatchCount,
  summary.sassSpecStaticMatchReport.matchedDeclarationValueCount,
);
assert.ok(
  summary.sassSpecStaticMatchReport.caseCount > 0,
  "static match case count must be non-empty",
);
assert.equal(
  summary.sassSpecStaticMatchReport.checkedCaseCount,
  summary.sassSpecStaticMatchReport.caseCount,
  "static match fixtures must reach omena resolution",
);
assert.equal(
  summary.sassSpecStaticMatchReport.matchedDeclarationValueCount,
  summary.sassSpecStaticMatchReport.declarationValueCount,
  "static match declaration values must match omena rendered values",
);
assert.ok(
  summary.sassSpecStaticMatchReport.allStaticValuesMatchOracle,
  "static match values must agree with dart-sass oracle values",
);
assert.ok(
  summary.sassSpecStaticMatchReport.allStaticMatchChecksHold,
  "complete static match gate must hold",
);
assert.ok(summary.allSassSpecStaticMatchChecksHold, "boundary static match gate must hold");
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
assert.ok(
  scale.upstreamScaleArtifactMatchesManifest,
  "upstream scale artifact must match manifest",
);
assert.ok(
  scale.allUpstreamArchiveCountExceedsImportedFixtures,
  "upstream archive count must exceed imported fixtures",
);
assert.ok(
  scale.allUpstreamArchiveCountExceedsSourceArchives,
  "upstream archive count must exceed source archive sample",
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
    `upstreamArchives=${scale.upstreamArchiveCount}`,
    `importedFixtures=${scale.importedFixtureCount}`,
    `chunks=${scale.importedChunkCount}`,
    `smokeFixtures=${scale.perPushSmokeFixtureCount}`,
    `smokeFloor=${scale.perPushSmokeFloorFixtureCount}`,
    `bailSiteFiles=${configuredBailSiteFiles.length}`,
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

function recursiveRustFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...recursiveRustFiles(entryPath));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(entryPath);
    }
  }
  return files;
}

function readConfiguredBailSiteFiles(filePath: string): string[] {
  const source = readFileSync(filePath, "utf8");
  const start = source.indexOf("const SASS_SPEC_BAIL_SITE_SOURCE_FILES");
  assert.notEqual(start, -1, "sass-spec bail-site source census must exist");
  const end = source.indexOf("\n];", start);
  assert.notEqual(end, -1, "sass-spec bail-site source census must have a closed array");
  return [...source.slice(start, end).matchAll(/file:\s*"([^"]+)"/gu)]
    .map((match) => match[1])
    .toSorted();
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}
