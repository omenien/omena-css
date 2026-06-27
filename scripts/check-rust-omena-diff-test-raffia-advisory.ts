import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

interface SeedChunkV0 {
  readonly fixtures: readonly SeedFixtureV0[];
}

interface SeedFixtureV0 {
  readonly id: string;
  readonly source: string;
  readonly dialect?: string;
}

interface OracleInputFixtureV0 {
  readonly id: string;
  readonly corpus: string;
  readonly dialect: string;
  readonly source: string;
}

interface RaffiaOracleSummaryV0 {
  readonly product: string;
  readonly gateMode: "advisory";
  readonly fixtureCount: number;
  readonly reports: readonly RaffiaOracleReportV0[];
}

interface RaffiaOracleReportV0 {
  readonly id: string;
  readonly corpus: string;
  readonly dialect: string;
  readonly omenaCompleteTree: boolean;
  readonly omenaNodeCount: number;
  readonly omenaTokenCount: number;
  readonly omenaBogusKinds: readonly string[];
  readonly omenaErrorCodes: readonly string[];
  readonly raffiaParseOk: boolean;
  readonly raffiaRecoverableErrorCount: number;
  readonly raffiaDebugLen: number;
  readonly raffiaError: string | null;
  readonly relation: string;
}

const repoRoot = process.cwd();
const fixtures = [
  ...readFixtures("wpt", "rust/crates/omena-diff-test/wpt-corpus/css-values.json", "css"),
  ...readFixtures("wpt", "rust/crates/omena-diff-test/wpt-corpus/css-values-advisory.json", "css"),
  ...readFixtures(
    "sass-spec",
    "rust/crates/omena-diff-test/sass-spec-corpus/language-core.json",
    "scss",
  ),
  ...readFixtures("less", "rust/crates/omena-diff-test/less-corpus/language-core.json", "less"),
];

assert.ok(fixtures.length > 0, "raffia advisory fixture list must not be empty");

const result = spawnSync(
  "cargo",
  ["run", "--manifest-path", "tools/raffia-oracle/Cargo.toml", "--quiet"],
  {
    cwd: repoRoot,
    encoding: "utf8",
    input: JSON.stringify(fixtures),
    maxBuffer: 32 * 1024 * 1024,
  },
);

assert.equal(result.status, 0, result.stderr);
assert.equal(result.error, undefined);

const summary = JSON.parse(result.stdout) as RaffiaOracleSummaryV0;
assert.equal(summary.product, "omena-diff-test.raffia-advisory");
assert.equal(summary.gateMode, "advisory");
assert.equal(summary.fixtureCount, fixtures.length);
assert.equal(summary.reports.length, fixtures.length);
assert.ok(
  summary.reports.every((report) => report.omenaCompleteTree),
  "raffia advisory still requires Omena complete-tree coverage",
);
assert.ok(
  summary.reports.every(
    (report) =>
      report.omenaNodeCount > 0 &&
      report.omenaTokenCount > 0 &&
      (report.raffiaParseOk ? report.raffiaDebugLen > 0 : report.raffiaError !== null),
  ),
  "raffia advisory must contain real parser output on every fixture",
);

const artifactPath = path.join(repoRoot, "rust/target/omena-diff-test/raffia-advisory.json");
mkdirSync(path.dirname(artifactPath), { recursive: true });
writeFileSync(artifactPath, `${JSON.stringify(summary, null, 2)}\n`);

process.stdout.write(
  JSON.stringify(
    {
      product: "omena-diff-test.raffia-advisory-check",
      gateMode: "advisory",
      fixtureCount: summary.fixtureCount,
      relationCounts: countRelations(summary.reports),
      artifactPath: path.relative(repoRoot, artifactPath),
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function readFixtures(
  corpus: string,
  relativePath: string,
  defaultDialect: string,
): readonly OracleInputFixtureV0[] {
  const chunk = JSON.parse(readFileSync(path.join(repoRoot, relativePath), "utf8")) as SeedChunkV0;
  return chunk.fixtures.map((fixture) => ({
    id: fixture.id,
    corpus,
    dialect: fixture.dialect ?? defaultDialect,
    source: fixture.source,
  }));
}

function countRelations(reports: readonly RaffiaOracleReportV0[]): Record<string, number> {
  return reports.reduce<Record<string, number>>((counts, report) => {
    counts[report.relation] = (counts[report.relation] ?? 0) + 1;
    return counts;
  }, {});
}
