import { cpSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  computeCostLedgerDigest,
  findCostLedgerDiagnostics,
  summaryInstrumentedJobNames,
} from "../../../packages/check-orchestrator/src/manifest/cost-ledger";

const repoRoot = path.resolve(__dirname, "../../..");

function ledgerRoot(): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "omena-cost-ledger-"));
  mkdirSync(path.join(root, "packages/check-orchestrator"), { recursive: true });
  cpSync(
    path.join(repoRoot, "packages/check-orchestrator/ci-workflow.json"),
    path.join(root, "packages/check-orchestrator/ci-workflow.json"),
  );
  return root;
}

function coherentLedger(root: string): Record<string, unknown> {
  const body = {
    generatedAt: "2026-08-20",
    sourceRunIds: ["1111111111"],
    gates: [{ gateId: "docs/smoke", p50Ms: 77, p95Ms: 80, sampleCount: 3 }],
    jobs: summaryInstrumentedJobNames(root).map((jobName) => ({
      jobName,
      wallP50Ms: 60_000,
      wallP95Ms: 90_000,
      queueP50Ms: 5_000,
      queueP95Ms: 9_000,
      sampleCount: 3,
    })),
  };
  return {
    schemaVersion: "1",
    product: "omena.check-orchestrator.ci-cost-ledger",
    ...body,
    recordsDigest: computeCostLedgerDigest(body),
  };
}

function writeLedger(root: string, ledger: Record<string, unknown> | null): void {
  const target = path.join(root, "packages/check-orchestrator/ci-cost-ledger.json");
  if (ledger) writeFileSync(target, JSON.stringify(ledger));
}

function codesFor(root: string, today = "2026-08-20"): string[] {
  process.env.OMENA_COST_LEDGER_TODAY = today;
  return findCostLedgerDiagnostics(root).map(
    (diagnostic) => `${diagnostic.severity}:${diagnostic.code}`,
  );
}

afterEach(() => {
  delete process.env.OMENA_COST_LEDGER_TODAY;
});

describe("ci cost ledger (g131-S1)", () => {
  it("is quiet on a coherent, fresh, covering ledger", () => {
    const root = ledgerRoot();
    writeLedger(root, coherentLedger(root));
    expect(codesFor(root)).toEqual([]);
  });

  it("RED-PROOF: a missing ledger in an orchestrator-bearing root REDs", () => {
    const root = ledgerRoot();
    expect(codesFor(root)).toContain("error:ci-cost-ledger-missing");
  });

  it("RED-PROOF: a forged duration without a writer re-run is a digest drift", () => {
    const root = ledgerRoot();
    const ledger = coherentLedger(root);
    (ledger["gates"] as { p50Ms: number }[])[0]!.p50Ms = 1;
    writeLedger(root, ledger);
    expect(codesFor(root)).toContain("error:ci-cost-ledger-digest-drift");
  });

  it("FRESHNESS: warn at >=30d, error at >=90d, injectable clock", () => {
    const root = ledgerRoot();
    writeLedger(root, coherentLedger(root));
    expect(codesFor(root, "2026-09-20")).toContain("warning:ci-cost-ledger-aging");
    expect(codesFor(root, "2026-11-19")).toContain("error:ci-cost-ledger-stale");
    expect(codesFor(root, "2026-08-25")).toEqual([]);
  });

  it("RED-PROOF: a summary-instrumented job with no ledger row is a coverage gap", () => {
    const root = ledgerRoot();
    const ledger = coherentLedger(root);
    (ledger["jobs"] as { jobName: string }[]).shift();
    const body = {
      generatedAt: ledger["generatedAt"],
      sourceRunIds: ledger["sourceRunIds"],
      gates: ledger["gates"],
      jobs: ledger["jobs"],
    };
    ledger["recordsDigest"] = computeCostLedgerDigest(body as never);
    writeLedger(root, ledger);
    expect(codesFor(root)).toContain("error:ci-cost-ledger-coverage-gap");
  });

  it("RED-PROOF: deleting the digest key is a governed shape error, not silence", () => {
    const root = ledgerRoot();
    const ledger = coherentLedger(root);
    delete ledger["recordsDigest"];
    writeLedger(root, ledger);
    expect(codesFor(root)).toContain("error:ci-cost-ledger-invalid-shape");
  });
});
