import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const slopeReportOverride = flagValue("--slope-report-path");
const slopeReportPath =
  slopeReportOverride ?? join(mkdtempSync(join(tmpdir(), "omena-slope-report-")), "report.json");

if (!slopeReportOverride) {
  runCommand("node", [
    "--import",
    "tsx",
    "./scripts/check-rust-z5-perf-gate-baseline.ts",
    "--complexity-slope",
    "--report-path",
    slopeReportPath,
  ]);
}

const gateOutput = runCommand("node", [
  "--import",
  "tsx",
  "./scripts/check-rust-omena-streaming-ifds-relocation-gate.ts",
  "--slope-report-path",
  slopeReportPath,
  "--require-slope",
]);
const gateSummary = parseJson<{
  readonly product: string;
  readonly verdictKind: string;
  readonly demandPrimaryReady: boolean;
  readonly conjuncts: {
    readonly factKeyGateGreen: boolean;
    readonly deletionCorpusGreen: boolean;
    readonly complexitySlopeGreen: boolean;
    readonly settleAllEqual: boolean;
  };
}>(gateOutput, "bound relocation gate summary");

assert.equal(gateSummary.product, "omena-streaming-ifds.relocation-gate");
assert.equal(gateSummary.verdictKind, "bound");
assert.equal(gateSummary.demandPrimaryReady, true);
assert.deepEqual(gateSummary.conjuncts, {
  factKeyGateGreen: true,
  deletionCorpusGreen: true,
  complexitySlopeGreen: true,
  settleAllEqual: true,
});

console.log(gateOutput);

function flagValue(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  assert.ok(value && !value.startsWith("--"), `${name} requires a value`);
  return value;
}

function runCommand(command: string, args: readonly string[]): string {
  const result = spawnSync(command, args, {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 30,
  });
  assert.equal(
    result.status,
    0,
    `${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return result.stdout;
}

function parseJson<T>(source: string, label: string): T {
  try {
    return JSON.parse(source) as T;
  } catch (error) {
    throw new Error(`failed to parse ${label}: ${(error as Error).message}\n${source}`);
  }
}
