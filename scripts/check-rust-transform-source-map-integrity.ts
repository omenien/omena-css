import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface SourceMapIntegrityFixtureReport {
  fixture: string;
  passIds: string[];
  mutationCount: number;
  sourceByteLen: number;
  generatedByteLen: number;
  segmentCount: number;
  survivingNodeCount: number;
  mappedSurvivingNodeCount: number;
  mapParsed: boolean;
  decodedSegmentCount: number;
  upstreamDecodedSegmentCount: number;
  upstreamMapApplied: boolean;
  compositionFallbackReason?: string;
  composedMapParsed: boolean;
  noDanglingSegments: boolean;
  complete: boolean;
}

interface SourceMapIntegrityReport {
  schemaVersion: string;
  product: string;
  fixtureCount: number;
  destructiveFixtureCount: number;
  survivingNodeCount: number;
  mappedSurvivingNodeCount: number;
  reports: SourceMapIntegrityFixtureReport[];
  complete: boolean;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourcePath = path.join(repoRoot, "rust/crates/omena-transform-print/src/lib.rs");
const binaryPath = path.join(
  repoRoot,
  "rust/crates/omena-transform-print/src/bin/transform_source_map_integrity.rs",
);
const source = fs.readFileSync(sourcePath, "utf8");
const binary = fs.readFileSync(binaryPath, "utf8");

assert.match(
  source,
  /pub const fn default_print_options\(\) -> TransformPrintOptionsV0 \{[\s\S]*include_source_map: true,/u,
  "the integrity corpus must emit source maps",
);
assert.ok(
  source.includes("summarize_transform_source_map_integrity_for_fixtures"),
  "the integrity report must keep an explicit corpus boundary",
);
assert.ok(
  source.includes("compose_transform_source_map_v3_with_upstream_map"),
  "the integrity report must exercise source-map composition",
);
assert.ok(
  source.includes("parse_transform_source_map_v3_json"),
  "the integrity report must parse serialized maps",
);
assert.ok(
  binary.includes("if report.complete"),
  "the boundary binary must fail when the report is incomplete",
);

const execution = spawnSync(
  "cargo",
  [
    "run",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-transform-print",
    "--bin",
    "transform_source_map_integrity",
    "--quiet",
  ],
  { cwd: repoRoot, encoding: "utf8" },
);
const executionOutput = `${execution.stdout ?? ""}\n${execution.stderr ?? ""}`;
assert.equal(execution.status, 0, executionOutput);

const report = JSON.parse(execution.stdout) as SourceMapIntegrityReport;
assert.equal(report.schemaVersion, "0");
assert.equal(report.product, "omena-transform-print.source-map-integrity");
assert.ok(report.fixtureCount > 0, "the destructive rewrite corpus must be non-empty");
assert.equal(report.destructiveFixtureCount, report.fixtureCount);
assert.equal(report.reports.length, report.fixtureCount);
assert.ok(report.survivingNodeCount > 0);
assert.equal(report.mappedSurvivingNodeCount, report.survivingNodeCount);

for (const fixture of report.reports) {
  assert.ok(fixture.fixture.length > 0);
  assert.ok(fixture.passIds.length > 1);
  assert.ok(fixture.mutationCount > 0, `${fixture.fixture} must execute a destructive rewrite`);
  assert.ok(
    fixture.generatedByteLen < fixture.sourceByteLen,
    `${fixture.fixture} must remove source bytes`,
  );
  assert.ok(fixture.segmentCount > 0);
  assert.ok(fixture.survivingNodeCount > 0);
  assert.equal(fixture.mappedSurvivingNodeCount, fixture.survivingNodeCount);
  assert.equal(fixture.mapParsed, true);
  assert.ok(fixture.decodedSegmentCount > 0);
  assert.ok(fixture.upstreamDecodedSegmentCount > 0);
  assert.equal(fixture.upstreamMapApplied, true);
  assert.equal(fixture.compositionFallbackReason, undefined);
  assert.equal(fixture.composedMapParsed, true);
  assert.equal(fixture.noDanglingSegments, true);
  assert.equal(fixture.complete, true);
}

assert.equal(report.complete, true);
process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
