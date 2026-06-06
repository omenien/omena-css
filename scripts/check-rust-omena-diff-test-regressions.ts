import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

type RegressionStatus = "fixed" | "todo";
type IssueState = "OPEN" | "CLOSED";

interface RegressionManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly fixtures: readonly RegressionManifestFixtureV0[];
}

interface RegressionManifestFixtureV0 {
  readonly id: string;
  readonly path: string;
  readonly status: RegressionStatus;
  readonly issue: {
    readonly repository: string;
    readonly number: number;
  };
}

interface ParsedFixtureV0 {
  readonly files: readonly ParsedFixtureFileV0[];
  readonly expectations: readonly ParsedFixtureExpectationV0[];
}

interface ParsedFixtureFileV0 {
  readonly path: string;
  readonly source: string;
}

interface ParsedFixtureExpectationV0 {
  readonly key: string;
  readonly value: string;
}

interface StyleDiagnosticsSummary {
  readonly diagnostics: readonly StyleDiagnostic[];
}

interface StyleDiagnostic {
  readonly code: string;
}

interface FixtureReport {
  readonly id: string;
  readonly status: RegressionStatus;
  readonly issue: string;
  readonly issueState: IssueState;
  readonly evaluatedExpectationCount: number;
  readonly satisfiedExpectationCount: number;
  readonly outcome: "pass" | "expectedFailure";
}

const repoRoot = process.cwd();
const regressionRoot = path.join(repoRoot, "rust/crates/omena-diff-test/regressions");
const manifest = readJson<RegressionManifestV0>(path.join(regressionRoot, "manifest.json"));

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-diff-test.regression-corpus");
assert.ok(manifest.fixtures.length > 0, "regression corpus must not be empty");

const issueStateCache = new Map<string, IssueState>();
const reports = manifest.fixtures.map((fixture) => evaluateRegressionFixture(fixture));

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-diff-test.regression-corpus",
      fixtureCount: reports.length,
      fixedCount: reports.filter((fixtureReport) => fixtureReport.status === "fixed").length,
      todoCount: reports.filter((fixtureReport) => fixtureReport.status === "todo").length,
      reports,
    },
    null,
    2,
  )}\n`,
);

function evaluateRegressionFixture(fixture: RegressionManifestFixtureV0): FixtureReport {
  assertManifestFixturePath(fixture.path, `${fixture.id} path`);
  const issueKey = `${fixture.issue.repository}#${fixture.issue.number}`;
  const issueState = issueStateFor(fixture.issue.repository, fixture.issue.number);
  const parsed = parseFixture(readFileSync(path.join(regressionRoot, fixture.path), "utf8"));
  const workspace = materializeFixture(parsed);
  try {
    const diagnostics = readWorkspaceDiagnostics(workspace, parsed.files);
    const outcomes = parsed.expectations
      .map((expectation) => evaluateExpectation(expectation, diagnostics))
      .filter((outcome): outcome is boolean => outcome !== undefined);
    assert.ok(outcomes.length > 0, `${fixture.id} must declare at least one evaluated expectation`);
    const satisfiedCount = outcomes.filter(Boolean).length;
    const allSatisfied = satisfiedCount === outcomes.length;

    if (fixture.status === "fixed") {
      assert.equal(
        issueState,
        "CLOSED",
        `${fixture.id} is fixed but ${issueKey} is still ${issueState}`,
      );
      assert.ok(allSatisfied, `${fixture.id} fixed regression expectations failed`);
      return report(fixture, issueState, outcomes.length, satisfiedCount, "pass");
    }

    assert.equal(
      issueState,
      "OPEN",
      `${fixture.id} is still marked todo but ${issueKey} is ${issueState}`,
    );
    assert.ok(
      !allSatisfied,
      `${fixture.id} todo regression now passes while ${issueKey} is still open`,
    );
    return report(fixture, issueState, outcomes.length, satisfiedCount, "expectedFailure");
  } finally {
    rmSync(workspace, { force: true, recursive: true });
  }
}

function report(
  fixture: RegressionManifestFixtureV0,
  issueState: IssueState,
  evaluatedExpectationCount: number,
  satisfiedExpectationCount: number,
  outcome: FixtureReport["outcome"],
): FixtureReport {
  return {
    id: fixture.id,
    status: fixture.status,
    issue: `${fixture.issue.repository}#${fixture.issue.number}`,
    issueState,
    evaluatedExpectationCount,
    satisfiedExpectationCount,
    outcome,
  };
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
  const workspace = mkdtempSync(path.join(tmpdir(), "omena-regression-fixture-"));
  for (const file of parsed.files) {
    assertSafeRelativePath(file.path, `fixture file ${file.path}`);
    const filePath = path.join(workspace, file.path);
    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, file.source);
  }
  return workspace;
}

function readWorkspaceDiagnostics(
  workspace: string,
  files: readonly ParsedFixtureFileV0[],
): readonly StyleDiagnostic[] {
  const styleFiles = files
    .map((file) => file.path)
    .filter((filePath) => /\.(?:css|scss|sass|less)$/u.test(filePath));
  assert.ok(styleFiles.length > 0, "regression fixture must include at least one style file");
  const sourceFiles = styleFiles.map((filePath) => path.join(workspace, filePath));
  return sourceFiles.flatMap((targetPath) => {
    const sourceArgs = sourceFiles
      .filter((sourcePath) => sourcePath !== targetPath)
      .flatMap((sourcePath) => ["--source", sourcePath]);
    const summary = JSON.parse(
      run(
        "cargo",
        [
          "run",
          "--quiet",
          "--manifest-path",
          "rust/Cargo.toml",
          "-p",
          "omena-cli",
          "--bin",
          "omena-cli",
          "--",
          "style-diagnostics",
          targetPath,
          ...sourceArgs,
          "--json",
        ],
        { maxBuffer: 1024 * 1024 * 64 },
      ).stdout,
    ) as StyleDiagnosticsSummary;
    return summary.diagnostics;
  });
}

function evaluateExpectation(
  expectation: ParsedFixtureExpectationV0,
  diagnostics: readonly StyleDiagnostic[],
): boolean | undefined {
  const kind = expectation.key.split(/\s+/u)[0] ?? "";
  if (kind === "no-diagnostic") {
    const code = expectation.key.split(/\s+/u)[1] ?? codeFromBody(expectation.value);
    assert.ok(code, `no-diagnostic expectation is missing a code: ${expectation.key}`);
    return diagnostics.every((diagnostic) => diagnostic.code !== code);
  }
  if (kind === "diagnostic") {
    const code = expectation.key.split(/\s+/u)[1] ?? codeFromBody(expectation.value);
    assert.ok(code, `diagnostic expectation is missing a code: ${expectation.key}`);
    return diagnostics.some((diagnostic) => diagnostic.code === code);
  }
  if (kind === "count") {
    const [code, rawExpected] = (expectation.key.split(/\s+/u)[1] ?? "").split(":");
    assert.ok(code && rawExpected, `count expectation must be count <code>:<n>`);
    const expected = Number.parseInt(rawExpected, 10);
    assert.ok(
      Number.isSafeInteger(expected),
      `count expectation has invalid count: ${rawExpected}`,
    );
    return diagnostics.filter((diagnostic) => diagnostic.code === code).length === expected;
  }
  return undefined;
}

function codeFromBody(value: string): string | undefined {
  return value
    .split(/\r?\n/u)
    .find((line) => line.trim().startsWith("code:"))
    ?.trim()
    .replace(/^code:\s*/u, "");
}

function issueStateFor(repository: string, number: number): IssueState {
  const key = `${repository}#${number}`;
  const cached = issueStateCache.get(key);
  if (cached) {
    return cached;
  }
  const result = run("gh", ["issue", "view", String(number), "-R", repository, "--json", "state"]);
  const state = JSON.parse(result.stdout) as { readonly state: IssueState };
  assert.match(state.state, /^(OPEN|CLOSED)$/u, `${key} issue state must be OPEN or CLOSED`);
  issueStateCache.set(key, state.state);
  return state.state;
}

function assertManifestFixturePath(relativePath: string, label: string): void {
  assertSafeRelativePath(relativePath, label);
  assert.ok(existsSync(path.join(regressionRoot, relativePath)), `${label} does not exist`);
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
  if (result.error) {
    throw result.error;
  }
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} exited ${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}
