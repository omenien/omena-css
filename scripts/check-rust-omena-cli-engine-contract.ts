import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface EngineContractCase {
  readonly name: string;
  readonly args: readonly string[];
  readonly baselineSha256: string;
}

interface EngineContract {
  readonly schemaVersion: string;
  readonly product: string;
  readonly baselineRevision: string;
  readonly normalization: string;
  readonly cases: readonly EngineContractCase[];
}

interface ProcessResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(
  readFileSync(path.join(repoRoot, "rust/crates/omena-cli/engine-command-contract.json"), "utf8"),
) as EngineContract;

assert.equal(contract.schemaVersion, "0");
assert.equal(contract.product, "omena-cli.engine-command-contract");
assert.match(contract.baselineRevision, /^[0-9a-f]{40}$/u);
assert.equal(contract.normalization, "canonical-executable-name-only");
assert.deepEqual(
  contract.cases.map((entry) => entry.name),
  ["build-help", "lock-help", "sif-help", "passes"],
);

runChecked("cargo", ["build", "--manifest-path", "rust/Cargo.toml", "-p", "omena-cli"]);

const binaryPath = path.join(repoRoot, "rust/target/debug/omena");
for (const entry of contract.cases) {
  const result = run(binaryPath, entry.args);
  const normalized = normalizeCanonicalExecutableName(entry, result);
  assert.equal(
    digest(normalized),
    entry.baselineSha256,
    `${entry.name} changed outside the canonical executable name`,
  );
}
assertCompatibilityRoutes();

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-cli-engine-contract",
      baselineRevision: contract.baselineRevision,
      cases: contract.cases.length,
      normalization: contract.normalization,
    },
    null,
    2,
  )}\n`,
);

function normalizeCanonicalExecutableName(
  entry: EngineContractCase,
  result: ProcessResult,
): ProcessResult {
  if (!entry.name.endsWith("-help")) {
    return result;
  }

  const usage = /^Usage: omena /gmu;
  const matches = result.stdout.match(usage) ?? [];
  assert.equal(matches.length, 1, `${entry.name} must expose exactly one canonical usage line`);
  return {
    ...result,
    stdout: result.stdout.replace(usage, "Usage: omena-cli "),
  };
}

function digest(result: ProcessResult): string {
  return createHash("sha256").update(JSON.stringify(result)).digest("hex");
}

function assertCompatibilityRoutes(): void {
  const routes = [
    { args: ["explain", "--help"], expected: "Usage: pnpm explain:expression" },
    { args: ["rename", "--help"], expected: "Usage: pnpm omena rename selector" },
    { args: ["--help"], expected: "Usage: omena <COMMAND>" },
  ] as const;

  for (const route of routes) {
    const result = run(process.execPath, ["--import", "tsx", "scripts/cme.ts", ...route.args]);
    assert.equal(
      result.status,
      0,
      `pnpm compatibility route ${route.args.join(" ")} failed\n${result.stderr}`,
    );
    assert.ok(
      result.stdout.includes(route.expected),
      `pnpm compatibility route ${route.args.join(" ")} did not reach its expected handler`,
    );
  }
}

function runChecked(command: string, args: readonly string[]): void {
  const result = run(command, args);
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
}

function run(command: string, args: readonly string[]): ProcessResult {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 32,
  });
  if (result.error) {
    throw result.error;
  }
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}
