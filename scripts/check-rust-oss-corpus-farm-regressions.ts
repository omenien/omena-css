// omena-verification-scope: engine-self
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { runCheckerCli } from "../server/checker-cli/src";

interface RegressionManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly fixtures: readonly RegressionManifestFixtureV0[];
}

interface RegressionManifestFixtureV0 {
  readonly id: string;
  readonly path: string;
  readonly status: string;
  readonly sourceProvenance?: {
    readonly repository: string;
    readonly pin: string;
  };
  readonly minimization?: string;
}

interface ParsedFixtureV0 {
  readonly files: readonly ParsedFixtureFileV0[];
  readonly expectations: readonly ParsedFixtureExpectationV0[];
}

interface ParsedFixtureFileV0 {
  readonly path: string;
  readonly source: string;
  readonly markers: readonly unknown[];
}

interface ParsedFixtureExpectationV0 {
  readonly key: string;
  readonly value: string;
}

const repoRoot = process.cwd();
const regressionRoot = path.join(repoRoot, "rust/crates/omena-diff-test/regressions");
const manifest = readJson<RegressionManifestV0>(path.join(regressionRoot, "manifest.json"));

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-diff-test.regression-corpus");
assertEncodedRawFixtureRoundTrip();
assertClassifierBranches();
assertBoundedPathPolicy();

void (async () => {
  const rawFixtures = manifest.fixtures.filter((fixture) => fixture.status === "raw");
  const reports = await Promise.all(rawFixtures.map(replayRawFixture));

  process.stdout.write(
    `${JSON.stringify(
      {
        product: "omena-diff-test.oss-corpus-farm.raw-regressions",
        rawFixtureCount: rawFixtures.length,
        replayedCount: reports.length,
        reports,
      },
      null,
      2,
    )}\n`,
  );
})();

function assertEncodedRawFixtureRoundTrip(): void {
  const source = [
    ".card {",
    "  //---- divider comment",
    '  content: "--- file: not-a-header /*|*/ /*at:point*/ /*<range>*/ /*</range>*/";',
    "}",
    "/*",
  ].join("\n");
  const fixture = [
    "--- expect: raw-reproducer",
    "exitCode: 0",
    "stdoutJson: parseable",
    "--- file: src/Card.module.scss encoding:hex",
    Buffer.from(source, "utf8").toString("hex"),
  ].join("\n");
  const parsed = parseFixture(`${fixture}\n`);
  assert.equal(parsed.files.length, 1);
  assert.equal(parsed.files[0]?.path, "src/Card.module.scss");
  assert.equal(parsed.files[0]?.source, source);
  assert.deepEqual(parsed.files[0]?.markers, []);
}

function assertClassifierBranches(): void {
  const result = run("node", [
    "--import",
    "tsx",
    "./scripts/oss-corpus-farm.ts",
    "--classifier-fixture",
  ]);
  const report = JSON.parse(result.stdout) as {
    readonly passCount: number;
    readonly pinChangeCount: number;
    readonly regressionCount: number;
    readonly missingBaselineCount: number;
    readonly reports: readonly { readonly id: string; readonly diffKind: string }[];
  };
  assert.equal(report.passCount, 1);
  assert.equal(report.pinChangeCount, 1);
  assert.equal(report.regressionCount, 1);
  assert.equal(report.missingBaselineCount, 1);
  assert.equal(
    report.reports.find((entry) => entry.id === "pin-change")?.diffKind,
    "pin-change",
    "a changed source pin must not pass solely because the fact-set hash is unchanged",
  );
}

function assertBoundedPathPolicy(): void {
  const result = run("node", [
    "--import",
    "tsx",
    "./scripts/oss-corpus-farm.ts",
    "--path-policy-fixture",
  ]);
  assert.deepEqual(JSON.parse(result.stdout), [
    ["src", true],
    ["src/styles", true],
    [".", false],
    ["", false],
    ["../outside", false],
    ["/absolute", false],
  ]);
}

async function replayRawFixture(fixture: RegressionManifestFixtureV0) {
  assert.ok(fixture.sourceProvenance, `${fixture.id} raw fixture must cite source provenance`);
  assert.match(fixture.sourceProvenance.repository, /^https:\/\/github\.com\//u);
  assert.match(fixture.sourceProvenance.pin, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+@[0-9a-f]{40}$/u);
  assert.equal(fixture.minimization, "raw");
  assertSafeRelativePath(fixture.path, `${fixture.id} path`);

  const parsed = parseFixture(readFileSync(path.join(regressionRoot, fixture.path), "utf8"));
  const workspace = materializeFixture(parsed);
  try {
    let stdout = "";
    let stderr = "";
    const exitCode = await runCheckerCli(
      [workspace, "--preset", "ci", "--fail-on", "none", "--format", "json"],
      {
        stdout: (message) => {
          stdout += message;
        },
        stderr: (message) => {
          stderr += message;
        },
        cwd: () => repoRoot,
      },
    );
    const expected = rawExpectation(parsed, "raw-reproducer");
    assert.ok(expected, `${fixture.id} raw fixture must declare a raw-reproducer expectation`);
    assert.equal(exitCode, Number.parseInt(expected.exitCode ?? "0", 10));
    if (expected.stdoutJson === "unparseable") {
      assert.throws(() => JSON.parse(stdout));
    } else if (expected.stdoutJson === "parseable") {
      JSON.parse(stdout);
    }
    return {
      id: fixture.id,
      repository: fixture.sourceProvenance.repository,
      pin: fixture.sourceProvenance.pin,
      exitCode,
      stdoutBytes: stdout.length,
      stderrBytes: stderr.length,
    };
  } finally {
    rmSync(workspace, { force: true, recursive: true });
  }
}

function rawExpectation(parsed: ParsedFixtureV0, key: string): Record<string, string> | undefined {
  const expectation = parsed.expectations.find((candidate) => candidate.key === key);
  if (!expectation) return undefined;
  return Object.fromEntries(
    expectation.value
      .split(/\r?\n/u)
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line) => {
        const split = line.indexOf(":");
        assert.ok(split > 0, `raw expectation line must be key: value, got ${line}`);
        return [line.slice(0, split).trim(), line.slice(split + 1).trim()];
      }),
  );
}

function parseFixture(raw: string): ParsedFixtureV0 {
  const result = run(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-testkit",
      "--bin",
      "omena-testkit-parse-fixture",
    ],
    { input: raw, maxBuffer: 1024 * 1024 * 8 },
  );
  return JSON.parse(result.stdout) as ParsedFixtureV0;
}

function materializeFixture(parsed: ParsedFixtureV0): string {
  const workspace = mkdtempSync(path.join(tmpdir(), "omena-oss-corpus-raw-"));
  for (const file of parsed.files) {
    assertSafeRelativePath(file.path, `fixture file ${file.path}`);
    const filePath = path.join(workspace, file.path);
    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, file.source);
  }
  return workspace;
}

function assertSafeRelativePath(relativePath: string, label: string): void {
  assert.ok(!path.isAbsolute(relativePath), `${label} must be relative`);
  assert.ok(!relativePath.includes(".."), `${label} must stay inside the regression root`);
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function run(
  command: string,
  args: readonly string[],
  options: {
    readonly input?: string;
    readonly maxBuffer?: number;
  } = {},
): { readonly stdout: string } {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    input: options.input,
    maxBuffer: options.maxBuffer ?? 1024 * 1024,
  });
  if (result.error) throw result.error;
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} exited ${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}
