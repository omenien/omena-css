import { execFileSync, spawnSync } from "node:child_process";
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  computeCostLedgerDigest,
  findCostLedgerDiagnostics,
  packMeasuredPartition,
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

function withWriterFixture(
  results: readonly Record<string, unknown>[],
  verify: (fixture: {
    root: string;
    downloads: string;
    run: () => ReturnType<typeof spawnSync>;
  }) => void,
): void {
  const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "omena-cost-writer-"));
  const root = path.join(temporaryRoot, "repo");
  const bin = path.join(temporaryRoot, "bin");
  const downloads = path.join(temporaryRoot, "downloads.jsonl");
  try {
    execFileSync("git", ["worktree", "add", "--detach", root, "HEAD"], {
      cwd: repoRoot,
      stdio: "ignore",
    });
    cpSync(
      path.join(repoRoot, "packages/check-orchestrator/src/cli/main.ts"),
      path.join(root, "packages/check-orchestrator/src/cli/main.ts"),
    );
    symlinkSync(
      path.join(repoRoot, "node_modules"),
      path.join(temporaryRoot, "node_modules"),
      "dir",
    );
    mkdirSync(bin);
    writeFileSync(
      path.join(bin, "gh"),
      `#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const args = process.argv.slice(2);
if (args[0] === "run" && args[1] === "list") {
  process.stdout.write(JSON.stringify([{ databaseId: 1111111111 }]));
} else if (args[0] === "api") {
  process.stdout.write(JSON.stringify({ jobs: [] }));
} else if (args[0] === "run" && args[1] === "download") {
  const dir = args[args.indexOf("--dir") + 1];
  const summary = path.join(dir, "summary", "check-summary-fixture.json");
  if (fs.existsSync(summary)) process.exit(1);
  fs.mkdirSync(path.dirname(summary), { recursive: true });
  fs.writeFileSync(summary, JSON.stringify({ results: ${JSON.stringify(results)} }));
  fs.appendFileSync(${JSON.stringify(downloads)}, JSON.stringify(dir) + "\\n");
} else {
  process.stderr.write("unexpected fixture command: " + JSON.stringify(args));
  process.exit(2);
}
`,
      { mode: 0o755 },
    );
    verify({
      root,
      downloads,
      run: () =>
        spawnSync(
          process.execPath,
          [
            "--import",
            "tsx",
            "./packages/check-orchestrator/src/cli/main.ts",
            "cost-ledger",
            "--write",
            "--",
            "--runs=1",
          ],
          {
            cwd: root,
            encoding: "utf8",
            env: { ...process.env, PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}` },
            timeout: 60_000,
          },
        ),
    });
  } finally {
    spawnSync("git", ["worktree", "remove", "--force", root], { cwd: repoRoot });
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
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

  it("PARTITION: measured pack respects the target, keeps rest non-empty, and REFUSES single-bin packs", () => {
    const minutes = (value: number) => value * 60_000;
    const partition = packMeasuredPartition(
      [
        { id: "a", p95Ms: minutes(6) },
        { id: "b", p95Ms: minutes(5) },
        { id: "c", p95Ms: minutes(3) },
        { id: "d", p95Ms: minutes(2) },
        { id: "e", p95Ms: minutes(1) },
      ],
      minutes(8),
    );
    // Every named bin respects the target; rest is structurally non-empty.
    for (const bin of partition.named) expect(bin.p95TotalMs).toBeLessThanOrEqual(minutes(8));
    expect(partition.rest.members.length).toBeGreaterThanOrEqual(1);
    // Partition property: named ∪ rest == input, disjoint.
    const all = [...partition.named.flatMap((bin) => bin.members), ...partition.rest.members];
    expect([...all].toSorted()).toEqual(["a", "b", "c", "d", "e"]);
    // A single member larger than the target is FLAGGED, never silent
    // (R2-confirm: neutering overTarget to false must fail here).
    const oversized = packMeasuredPartition(
      [
        { id: "huge", p95Ms: minutes(10) },
        { id: "small-a", p95Ms: minutes(2) },
        { id: "small-b", p95Ms: minutes(3) },
      ],
      minutes(8),
    );
    expect(oversized.named.some((bin) => bin.overTarget)).toBe(true);
    expect(oversized.named.find((bin) => bin.members.includes("huge"))?.overTarget).toBe(true);
    // ...and fitting bins are NOT flagged (no false positive).
    expect(partition.named.every((bin) => !bin.overTarget)).toBe(true);
    // RED: a pack that fits one bin would empty rest — the writer refuses.
    expect(() =>
      packMeasuredPartition(
        [
          { id: "a", p95Ms: minutes(1) },
          { id: "b", p95Ms: minutes(1) },
        ],
        minutes(8),
      ),
    ).toThrow(/rest shard would be empty/u);
    expect(() => packMeasuredPartition([], minutes(8))).toThrow(/empty member set/u);
  });

  it("RED-PROOF: deleting the digest key is a governed shape error, not silence", () => {
    const root = ledgerRoot();
    const ledger = coherentLedger(root);
    delete ledger["recordsDigest"];
    writeLedger(root, ledger);
    expect(codesFor(root)).toContain("error:ci-cost-ledger-invalid-shape");
  });
});

describe("cost ledger writer receipts", () => {
  it("reads summaries downloaded after the initial scan snapshot and isolates repeated downloads", () => {
    withWriterFixture(
      [{ title: "docs/smoke", durationMs: 77, status: "pass", timedOut: false }],
      ({ root, downloads, run }) => {
        const ledgerPath = path.join(root, "packages/check-orchestrator/ci-cost-ledger.json");
        const first = run();
        expect(first.status, String(first.stderr)).toBe(0);
        const bytes = readFileSync(ledgerPath);
        expect(JSON.parse(bytes.toString()).gates).toEqual([
          { gateId: "docs/smoke", p50Ms: 77, p95Ms: 77, sampleCount: 1 },
        ]);
        const second = run();
        expect(second.status, String(second.stderr)).toBe(0);
        expect(readFileSync(ledgerPath)).toEqual(bytes);
        const directories = readFileSync(downloads, "utf8")
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line));
        expect(directories).toHaveLength(2);
        expect(new Set(directories).size).toBe(2);
      },
    );
  }, 120_000);

  it("refuses zero passing samples without replacing existing ledger bytes", () => {
    withWriterFixture(
      [
        { title: "docs/failed", durationMs: 77, status: "fail", timedOut: false },
        { title: "docs/timeout", durationMs: 88, status: "pass", timedOut: true },
      ],
      ({ root, run }) => {
        const ledgerPath = path.join(root, "packages/check-orchestrator/ci-cost-ledger.json");
        const before = readFileSync(ledgerPath);
        const result = run();
        expect(result.status).toBe(1);
        expect(String(result.stderr)).toContain(
          "no passing gate summary receipts were read; refusing to replace the cost ledger",
        );
        expect(readFileSync(ledgerPath)).toEqual(before);
      },
    );
  }, 120_000);
});
