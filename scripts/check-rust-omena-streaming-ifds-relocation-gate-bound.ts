import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

const slopeReportOverride = flagValue("--slope-report-path");
const slopeReportEnv = process.env.OMENA_Z5_COMPLEXITY_SLOPE_REPORT;
const slopeReportPath =
  slopeReportOverride ??
  slopeReportEnv ??
  join(mkdtempSync(join(tmpdir(), "omena-slope-report-")), "report.json");
const approvalReportPath =
  process.env.OMENA_STREAMING_IFDS_RELOCATION_APPROVAL_REPORT ??
  join(mkdtempSync(join(tmpdir(), "omena-relocation-approval-")), "report.json");

if (slopeReportEnv && !existsSync(slopeReportPath)) {
  throw new Error(`expected slope report from previous benchmark step: ${slopeReportPath}`);
}

if (!slopeReportOverride && !slopeReportEnv) {
  runCommand("node", [
    "--import",
    "tsx",
    "./scripts/check-rust-z5-perf-gate-baseline.ts",
    "--complexity-slope",
    "--report-path",
    slopeReportPath,
  ]);
}

const gateArgs = [
  "--import",
  "tsx",
  "./scripts/check-rust-omena-streaming-ifds-relocation-gate.ts",
  "--slope-report-path",
  slopeReportPath,
  "--require-slope",
];

interface GateSummary {
  readonly product: string;
  readonly verdictKind: string;
  readonly demandPrimaryReady: boolean;
  readonly boundary: {
    readonly source: string;
  };
  readonly conjuncts: {
    readonly factKeyGateGreen: boolean;
    readonly deletionCorpusGreen: boolean;
    readonly complexitySlopeGreen: boolean;
    readonly settleAllEqual: boolean;
  };
  readonly approval: {
    readonly source: string;
    readonly green: boolean;
    readonly refusal: string | null;
  };
  readonly route: {
    readonly engine: string;
    readonly relocationGateGreen: boolean;
  };
  readonly equivalence: {
    readonly product: string;
    readonly comparisonKind: string;
    readonly demandFactKeyCount: number;
    readonly eagerFactKeyCount: number;
    readonly demandFactKeySha256: string;
    readonly eagerFactKeySha256: string;
    readonly equivalent: boolean;
  };
}

const approvalCandidateOutput = runCommand("node", gateArgs);
const approvalCandidate = parseJson<GateSummary>(
  approvalCandidateOutput,
  "relocation approval candidate",
);
assertBoundUpstream(approvalCandidate);
assert.equal(approvalCandidate.approval.source, "absent");
assert.equal(approvalCandidate.approval.green, false);
assert.equal(approvalCandidate.approval.refusal, "absent artifact verdict");
assert.equal(approvalCandidate.demandPrimaryReady, false);
assert.equal(approvalCandidate.route.engine, "batch");

mkdirSync(dirname(approvalReportPath), { recursive: true });
writeFileSync(approvalReportPath, approvalCandidateOutput, "utf8");

const invalidApprovalPath = join(
  mkdtempSync(join(tmpdir(), "omena-relocation-approval-invalid-")),
  "report.json",
);
const invalidApproval = {
  ...approvalCandidate,
  boundary: { ...approvalCandidate.boundary, source: "override" },
};
writeFileSync(invalidApprovalPath, JSON.stringify(invalidApproval), "utf8");
const invalidSummary = parseJson<GateSummary>(
  runCommand("node", [...gateArgs, "--approval-report-path", invalidApprovalPath]),
  "invalid relocation approval summary",
);
assert.equal(invalidSummary.approval.green, false);
assert.equal(invalidSummary.approval.refusal, "artifact verdict red");
assert.equal(invalidSummary.demandPrimaryReady, false);
assert.equal(invalidSummary.route.engine, "batch");

const staleApprovalPath = join(
  mkdtempSync(join(tmpdir(), "omena-relocation-approval-stale-")),
  "report.json",
);
const staleApproval = {
  ...approvalCandidate,
  boundary: { ...approvalCandidate.boundary, artifactSha256: "0".repeat(64) },
};
writeFileSync(staleApprovalPath, JSON.stringify(staleApproval), "utf8");
const staleSummary = parseJson<GateSummary>(
  runCommand("node", [...gateArgs, "--approval-report-path", staleApprovalPath]),
  "stale relocation approval summary",
);
assert.equal(staleSummary.approval.green, false);
assert.equal(staleSummary.approval.refusal, "artifact verdict red");
assert.equal(staleSummary.demandPrimaryReady, false);
assert.equal(staleSummary.route.engine, "batch");

const gateOutput = runCommand("node", [...gateArgs, "--approval-report-path", approvalReportPath]);
const gateSummary = parseJson<GateSummary>(gateOutput, "bound relocation gate summary");

assertBoundUpstream(gateSummary);
assert.equal(gateSummary.approval.source, "approval-report");
assert.equal(gateSummary.approval.green, true);
assert.equal(gateSummary.approval.refusal, null);
assert.equal(gateSummary.demandPrimaryReady, true);
assert.equal(gateSummary.route.engine, "demand");
assert.equal(gateSummary.route.relocationGateGreen, true);

console.log(
  JSON.stringify(
    {
      ...gateSummary,
      approvalAdversarialChecks: {
        absentArtifactEngine: approvalCandidate.route.engine,
        invalidBoundarySourceEngine: invalidSummary.route.engine,
        staleArtifactDigestEngine: staleSummary.route.engine,
      },
    },
    null,
    2,
  ),
);

function assertBoundUpstream(summary: GateSummary): void {
  assert.equal(summary.product, "omena-streaming-ifds.relocation-gate");
  assert.equal(summary.boundary.source, "boundary-binary");
  assert.equal(summary.verdictKind, "bound");
  assert.deepEqual(summary.conjuncts, {
    factKeyGateGreen: true,
    deletionCorpusGreen: true,
    complexitySlopeGreen: true,
    settleAllEqual: true,
  });
  assert.equal(summary.equivalence.product, "omena-streaming-ifds.demand-eager-equivalence");
  assert.equal(summary.equivalence.comparisonKind, "demandVsIndependentProjectedBatch");
  assert.equal(summary.equivalence.equivalent, true);
  assert.equal(summary.equivalence.demandFactKeyCount, summary.equivalence.eagerFactKeyCount);
  assert.equal(summary.equivalence.demandFactKeySha256, summary.equivalence.eagerFactKeySha256);
}

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
