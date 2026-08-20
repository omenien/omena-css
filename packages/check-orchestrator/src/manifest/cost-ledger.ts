import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import type { CheckDiagnostic } from "./types";
import { loadCiWorkflowRegistry } from "./ci-workflow";

// g131-S1: the committed CI cost ledger. Written LOCALLY (gh credentials, no
// CI commit path, no `actions:` grant) by `omena-check cost-ledger --write`
// from real green-run receipts; checked in CI as an ordinary reachable gate
// so the escape-hatch population is untouched. Every duration is labeled
// with its source run ids and sealed under a digest — a hand-edited number
// without a re-run of the writer is a digest drift, not a quiet lie.

export interface CostLedgerGateRecord {
  readonly gateId: string;
  readonly p50Ms: number;
  readonly p95Ms: number;
  readonly sampleCount: number;
}

export interface CostLedgerJobRecord {
  readonly jobName: string;
  readonly wallP50Ms: number;
  readonly wallP95Ms: number;
  readonly queueP50Ms: number;
  readonly queueP95Ms: number;
  readonly sampleCount: number;
}

export interface CostLedger {
  readonly schemaVersion: string;
  readonly product: string;
  readonly generatedAt: string;
  readonly sourceRunIds: readonly string[];
  readonly gates: readonly CostLedgerGateRecord[];
  readonly jobs: readonly CostLedgerJobRecord[];
  readonly recordsDigest: string;
}

export function costLedgerPath(rootDir: string): string {
  return path.join(rootDir, "packages/check-orchestrator/ci-cost-ledger.json");
}

export function loadCostLedger(rootDir: string): CostLedger | null {
  const ledgerPath = costLedgerPath(rootDir);
  if (!existsSync(ledgerPath)) return null;
  return JSON.parse(readFileSync(ledgerPath, "utf8")) as CostLedger;
}

export function costLedgerToday(): string {
  return process.env.OMENA_COST_LEDGER_TODAY ?? new Date().toISOString().slice(0, 10);
}

export function computeCostLedgerDigest(
  ledger: Pick<CostLedger, "generatedAt" | "sourceRunIds" | "gates" | "jobs">,
): string {
  return createHash("sha256")
    .update(
      JSON.stringify({
        generatedAt: ledger.generatedAt,
        sourceRunIds: [...ledger.sourceRunIds].toSorted(),
        gates: [...ledger.gates].toSorted((left, right) => left.gateId.localeCompare(right.gateId)),
        jobs: [...ledger.jobs].toSorted((left, right) => left.jobName.localeCompare(right.jobName)),
      }),
    )
    .digest("hex");
}

export function percentile(samples: readonly number[], fraction: number): number {
  if (samples.length === 0) return 0;
  const sorted = [...samples].toSorted((left, right) => left - right);
  const index = Math.min(sorted.length - 1, Math.ceil(fraction * sorted.length) - 1);
  return sorted[Math.max(0, index)] ?? 0;
}

export interface MeasuredMember {
  readonly id: string;
  readonly p95Ms: number;
}

export interface MeasuredPartition {
  readonly named: readonly { readonly p95TotalMs: number; readonly members: readonly string[] }[];
  readonly rest: { readonly p95TotalMs: number; readonly members: readonly string[] };
}

// g131-S2: first-fit-decreasing pack into bins of targetMs. The LAST bin is
// the implicit rest shard; the pack REFUSES a result whose rest would be
// empty (single-bin packs included) — the shard resolver hard-fails on an
// empty rest, so the writer must never propose one.
export function packMeasuredPartition(
  members: readonly MeasuredMember[],
  targetMs: number,
): MeasuredPartition {
  if (members.length === 0) throw new Error("cannot partition an empty member set");
  const sorted = [...members].toSorted((left, right) => right.p95Ms - left.p95Ms);
  const bins: { totalMs: number; members: string[] }[] = [];
  for (const member of sorted) {
    const fit = bins.find((bin) => bin.totalMs + member.p95Ms <= targetMs);
    if (fit) {
      fit.totalMs += member.p95Ms;
      fit.members.push(member.id);
    } else {
      bins.push({ totalMs: member.p95Ms, members: [member.id] });
    }
  }
  if (bins.length < 2) {
    throw new Error(
      `pack fits a single ${Math.round((bins[0]?.totalMs ?? 0) / 60_000)}m shard at this target; ` +
        "the rest shard would be empty (the resolver hard-fails on that) — widen the member set or lower the target.",
    );
  }
  const rest = bins[bins.length - 1]!;
  if (rest.members.length === 0) {
    throw new Error("proposed partition empties the rest shard; refusing to emit.");
  }
  return {
    named: bins.slice(0, -1).map((bin) => ({ p95TotalMs: bin.totalMs, members: [...bin.members] })),
    rest: { p95TotalMs: rest.totalMs, members: [...rest.members] },
  };
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}/;
const AGING_DAYS = 30;
const STALE_DAYS = 90;

export function summaryInstrumentedJobNames(rootDir: string): readonly string[] {
  const registry = loadCiWorkflowRegistry(rootDir);
  if (!registry) return [];
  return registry.jobs
    .filter((job) =>
      job.block.some((line) => line.includes("omena-check") && line.includes("--summary")),
    )
    .map((job) => job.name);
}

export function findCostLedgerDiagnostics(rootDir: string): readonly CheckDiagnostic[] {
  const diagnostics: CheckDiagnostic[] = [];
  const ledger = loadCostLedger(rootDir);
  if (!ledger) {
    if (existsSync(path.join(rootDir, "packages/check-orchestrator"))) {
      diagnostics.push({
        severity: "error",
        code: "ci-cost-ledger-missing",
        message:
          "packages/check-orchestrator/ci-cost-ledger.json is absent; run `pnpm omena-check cost-ledger --write` from a credentialed checkout.",
      });
    }
    return diagnostics;
  }

  const shapeErrors: string[] = [];
  if (!ISO_DATE.test(ledger.generatedAt ?? "")) shapeErrors.push("generatedAt must be an ISO date");
  if (!Array.isArray(ledger.sourceRunIds) || ledger.sourceRunIds.length === 0) {
    shapeErrors.push("sourceRunIds must pin at least one source run");
  }
  if (!Array.isArray(ledger.gates) || ledger.gates.length === 0) {
    shapeErrors.push("gates must carry at least one per-gate record");
  }
  if (!Array.isArray(ledger.jobs) || ledger.jobs.length === 0) {
    shapeErrors.push("jobs must carry at least one per-job record");
  }
  if (typeof ledger.recordsDigest !== "string" || !/^[0-9a-f]{64}$/.test(ledger.recordsDigest)) {
    shapeErrors.push("recordsDigest must be a sha256 hex digest");
  }
  if (shapeErrors.length > 0) {
    for (const shapeError of shapeErrors) {
      diagnostics.push({
        severity: "error",
        code: "ci-cost-ledger-invalid-shape",
        message: `ci-cost-ledger.json ${shapeError}.`,
      });
    }
    return diagnostics;
  }

  const digest = computeCostLedgerDigest(ledger);
  if (digest !== ledger.recordsDigest) {
    diagnostics.push({
      severity: "error",
      code: "ci-cost-ledger-digest-drift",
      message:
        `ci-cost-ledger.json records digest to ${digest} but the file pins ${ledger.recordsDigest}; ` +
        "durations must come from the writer over real runs, never a hand edit.",
    });
  }

  const today = costLedgerToday();
  const ageDays = Math.floor(
    (Date.parse(`${today}T00:00:00Z`) -
      Date.parse(`${ledger.generatedAt.slice(0, 10)}T00:00:00Z`)) /
      86_400_000,
  );
  if (ageDays >= STALE_DAYS) {
    diagnostics.push({
      severity: "error",
      code: "ci-cost-ledger-stale",
      message:
        `ci-cost-ledger.json is ${ageDays}d old (>=${STALE_DAYS}d); refresh duty = each goal pickup: ` +
        "run `pnpm omena-check cost-ledger --write` and commit the result.",
    });
  } else if (ageDays >= AGING_DAYS) {
    diagnostics.push({
      severity: "warning",
      code: "ci-cost-ledger-aging",
      message:
        `ci-cost-ledger.json is ${ageDays}d old (>=${AGING_DAYS}d); refresh duty = each goal pickup: ` +
        "run `pnpm omena-check cost-ledger --write` and commit the result.",
    });
  }

  // Coverage floor: every summary-instrumented CI job must carry a job row —
  // a lane silently missing from the ledger is a cost nobody is watching.
  const coveredJobs = new Set(ledger.jobs.map((job) => job.jobName));
  for (const jobName of summaryInstrumentedJobNames(rootDir)) {
    if (!coveredJobs.has(jobName)) {
      diagnostics.push({
        severity: "error",
        code: "ci-cost-ledger-coverage-gap",
        message: `summary-instrumented CI job "${jobName}" has no ci-cost-ledger.json row; re-run the writer over runs that include it.`,
      });
    }
  }
  return diagnostics;
}
