import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  buildEvidenceAffectedPlan,
  orderAffectedArtifacts,
} from "../../../packages/check-orchestrator/src/evidence/affected-evidence";
import { assertSurfaceNarrowingReason } from "../../../packages/check-orchestrator/src/evidence/scan-surface-registry";
import {
  analyzeScannerSource,
  findUnroutedScannerCallSites,
  SCANNER_DETECTION_PATTERN,
} from "../../../packages/check-orchestrator/src/evidence/scanner-analysis";
import {
  defineScanSurface,
  resolveScanSurface,
  scanSurfaceMatchesPath,
} from "../../../packages/check-orchestrator/src/evidence/scan-surface";
import {
  resolveEvidenceWriterCommand,
  runDigestPinnedWriter,
} from "../../../packages/check-orchestrator/src/evidence/writer-runner";
import {
  buildEvidencePreviewBudgetPlan,
  EVIDENCE_PREVIEW_BUDGET_MS,
  EVIDENCE_PREVIEW_EMPTY_BUDGET_MS,
  executeEvidencePreviewBudget,
  isEvidencePreviewCheckGate,
} from "../../../packages/check-orchestrator/src/evidence/preview-budget";
import type { EvidenceScanSurfaceManifestV0 } from "../../../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import {
  buildEvidenceWriterRegistry,
  type EvidenceArtifactRowV0,
  type EvidenceWriterRegistryV0,
} from "../../../packages/check-orchestrator/src/evidence/writer-registry";
import type {
  CheckGate,
  CheckManifest,
} from "../../../packages/check-orchestrator/src/manifest/types";
import type { CostLedger } from "../../../packages/check-orchestrator/src/manifest/cost-ledger";

const predicateModules = {
  "git-metadata": { modulePath: "p/git.ts", sha256: "a".repeat(64) },
  "node-modules": { modulePath: "p/node.ts", sha256: "b".repeat(64) },
  "personal-docs": { modulePath: "p/personal.ts", sha256: "c".repeat(64) },
  "rust-build-output": { modulePath: "p/rust.ts", sha256: "d".repeat(64) },
  "test-only-rust": { modulePath: "p/test.ts", sha256: "e".repeat(64) },
} as const;

function scannerManifest(): EvidenceScanSurfaceManifestV0 {
  return {
    schemaVersion: "0",
    generatedBy: "pnpm omena-check evidence-surfaces --write",
    detector: { fileCount: 3, patternSha256: "f".repeat(64) },
    resolverSha256: "0".repeat(64),
    predicateModules,
    scanners: [
      {
        scannerPath: "scripts/check-rust.ts",
        disposition: "MIGRATED",
        surfaceModulePath: "scripts/surfaces/check-rust.surface.ts",
        surfaceModuleSha256: "1".repeat(64),
        spec: defineScanSurface({
          scannerPath: "scripts/check-rust.ts",
          mode: "workingTree",
          pathspecs: ["rust/**"],
          includeUntracked: false,
          excludes: [],
        }),
        gateIds: ["rust/check"],
      },
      {
        scannerPath: "scripts/check-docs.ts",
        disposition: "MIGRATED",
        surfaceModulePath: "scripts/surfaces/check-docs.surface.ts",
        surfaceModuleSha256: "2".repeat(64),
        spec: defineScanSurface({
          scannerPath: "scripts/check-docs.ts",
          mode: "workingTree",
          pathspecs: ["docs/**"],
          includeUntracked: false,
          excludes: [],
        }),
        gateIds: ["docs/check"],
      },
      {
        scannerPath: "scripts/check-external.ts",
        disposition: "UNMIGRATED",
        effectiveSurface: "**",
        reason: "external-checkout",
        evidenceNeedle: "checkoutRoot",
        gateIds: ["external/check"],
      },
    ],
  };
}

function artifact(
  artifactPath: string,
  overrides: Partial<EvidenceArtifactRowV0> = {},
): EvidenceArtifactRowV0 {
  return {
    artifactPath,
    classification: "W2",
    writerNodeKind: "normal",
    writerScripts: [],
    writeCommand: ["node", `${artifactPath}.writer.mjs`],
    inputScannerPaths: [],
    inputArtifactPaths: [],
    writerGateIds: [],
    consumerPaths: [],
    consumerGateIds: [],
    ...overrides,
  };
}

function writerRegistry(): EvidenceWriterRegistryV0 {
  return {
    schemaVersion: "0",
    generatedBy: "pnpm omena-check evidence-writers --write",
    artifacts: [
      artifact("rust/a.json", {
        inputScannerPaths: ["scripts/check-rust.ts"],
        writerGateIds: ["rust/write"],
      }),
      artifact("rust/b.json", {
        inputArtifactPaths: ["rust/a.json"],
        consumerGateIds: ["rust/consume"],
      }),
      artifact("rust/docs.json", { inputScannerPaths: ["scripts/check-docs.ts"] }),
      artifact("rust/product-surface-boundary-reviews.json", {
        classification: "W3",
        procedure: [
          "commit the source change",
          "run --measure and hand-edit the measured review row",
          "commit the refreshed evidence",
        ],
      }),
    ],
    notPreviewableInputs: [
      { ownerId: "clock-gate", kind: "calendar-time", detail: "wall clock" },
      { ownerId: "env-gate", kind: "environment", detail: "environment" },
      { ownerId: "history-gate", kind: "git-history", detail: "history" },
      { ownerId: "writer", kind: "concurrent-worktree", detail: "skew" },
      {
        ownerId: "external-gate",
        kind: "network-or-external-checkout",
        detail: "external",
      },
      { ownerId: "binary-gate", kind: "built-binary", detail: "binary" },
      { ownerId: "tool-gate", kind: "toolchain-bytes", detail: "tool" },
    ],
  };
}

describe("evidence affected closure", () => {
  it("closes artifacts in dependency order and names a non-empty excluded set", () => {
    const plan = buildEvidenceAffectedPlan({
      changedPaths: ["rust/crates/omena-query/src/lib.rs"],
      scanManifest: scannerManifest(),
      writerRegistry: writerRegistry(),
    });
    expect(plan.affectedScannerPaths).toEqual([
      "scripts/check-external.ts",
      "scripts/check-rust.ts",
    ]);
    expect(plan.excludedScannerPaths).toEqual(["scripts/check-docs.ts"]);
    expect(plan.writerOrder).toEqual(["rust/a.json", "rust/b.json"]);
    expect(plan.gateIds).toEqual(["external/check", "rust/check", "rust/consume", "rust/write"]);
    expect(plan.notPreviewableInputs.map((entry) => entry.kind)).toHaveLength(7);
    expect(plan.commitThenRefresh).toEqual([
      {
        artifactPath: "rust/product-surface-boundary-reviews.json",
        steps: [
          "commit the source change",
          "run --measure and hand-edit the measured review row",
          "commit the refreshed evidence",
        ],
      },
    ]);
  });

  it("fails closed when a detected scanner lacks a surface row", () => {
    const plan = buildEvidenceAffectedPlan({
      changedPaths: ["docs/guide.md"],
      scanManifest: scannerManifest(),
      writerRegistry: writerRegistry(),
      expectedScannerPaths: ["scripts/check-rust.ts", "scripts/new-scanner.ts"],
    });
    expect(plan.fallbackToFullEvidence).toBe(true);
    expect(plan.affectedArtifactPaths).toEqual([
      "rust/a.json",
      "rust/b.json",
      "rust/docs.json",
      "rust/product-surface-boundary-reviews.json",
    ]);
    expect(plan.evidenceRefreshSufficientAlone).toBe(false);
  });

  it("enforces FULL path-plane implication while retaining a legitimate narrow control", () => {
    const full = buildEvidenceAffectedPlan({
      changedPaths: ["package.json"],
      scanManifest: scannerManifest(),
      writerRegistry: writerRegistry(),
    });
    expect(full.pathPlane.requiresFullCi).toBe(true);
    expect(full.evidenceRefreshSufficientAlone).toBe(false);
    expect(() =>
      buildEvidenceAffectedPlan({
        changedPaths: ["package.json"],
        scanManifest: scannerManifest(),
        writerRegistry: writerRegistry(),
        forceNarrowSufficiencyForTest: true,
      }),
    ).toThrow(/requires FULL CI/u);

    const narrow = buildEvidenceAffectedPlan({
      changedPaths: ["rust/crates/omena-query/src/lib.rs"],
      scanManifest: scannerManifest(),
      writerRegistry: writerRegistry(),
    });
    expect(narrow.pathPlane.requiresFullCi).toBe(false);
    expect(narrow.evidenceRefreshSufficientAlone).toBe(true);
  });

  it("allows a declared self-ratchet loop once and rejects ordinary cycles", () => {
    const self = artifact("rust/self.json", {
      writerNodeKind: "self-ratchet",
      inputArtifactPaths: ["rust/self.json"],
    });
    expect(orderAffectedArtifacts([self], new Set([self.artifactPath]))).toEqual([
      "rust/self.json",
    ]);
    const left = artifact("rust/left.json", { inputArtifactPaths: ["rust/right.json"] });
    const right = artifact("rust/right.json", { inputArtifactPaths: ["rust/left.json"] });
    expect(() =>
      orderAffectedArtifacts([left, right], new Set([left.artifactPath, right.artifactPath])),
    ).toThrow(/DAG cycle/u);
  });

  it("closes every output owned by one writer command", () => {
    const registry = writerRegistry();
    const sharedCommand = ["node", "shared-writer.mjs", "--write"];
    const plan = buildEvidenceAffectedPlan({
      changedPaths: ["rust/first.json"],
      scanManifest: scannerManifest(),
      writerRegistry: {
        ...registry,
        artifacts: [
          ...registry.artifacts,
          artifact("rust/first.json", { writeCommand: sharedCommand }),
          artifact("rust/second.json", { writeCommand: sharedCommand }),
        ].toSorted((left, right) =>
          left.artifactPath < right.artifactPath
            ? -1
            : left.artifactPath > right.artifactPath
              ? 1
              : 0,
        ),
      },
    });
    expect(plan.affectedArtifactPaths).toEqual([
      "rust/a.json",
      "rust/b.json",
      "rust/first.json",
      "rust/second.json",
    ]);
    expect(plan.writerOrder).toEqual([
      "rust/a.json",
      "rust/b.json",
      "rust/first.json",
      "rust/second.json",
    ]);
  });
});

describe("scan surface falsifiers", () => {
  it("rejects a direct enumeration beside the resolver and accepts the routed control", () => {
    expect(findUnroutedScannerCallSites("scripts/check.ts", "readdirSync(root);")).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/check.ts",
        "const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url); evidenceScanSurface.readdirSync(root);",
      ),
    ).toEqual([]);
  });

  it("pins every detector token and does not manufacture a scanner from ordinary source", () => {
    for (const token of [
      "ls-files",
      "readdirSync",
      "readdir(",
      "opendirSync",
      "globSync",
      "fast-glob",
      "globby",
      "trackedProductionSources",
    ]) {
      expect(SCANNER_DETECTION_PATTERN.test(token), token).toBe(true);
    }
    expect(analyzeScannerSource("scripts/plain.ts", "export const value = 1;").rawMatchCount).toBe(
      0,
    );
  });

  it("classifies comment-only matches as false-positive candidates and escalates only exec-family template residue", () => {
    const falsePositive = analyzeScannerSource(
      "scripts/comment-only.ts",
      "// a prose mention of globby must not invent an enumeration call\nexport const value = 1;",
    );
    expect(falsePositive.rawMatchCount).toBe(1);
    expect(falsePositive.callSites).toEqual([]);
    expect(falsePositive.tertiaryExecFamilySites).toEqual([]);

    const control = analyzeScannerSource(
      "scripts/control.ts",
      "console.log(`git ls-files is documented here`);",
    );
    expect(control.tertiaryExecFamilySites).toEqual([]);
    const residue = analyzeScannerSource("scripts/residue.ts", "execSync(`git ls-files ${root}`);");
    expect(residue.tertiaryExecFamilySites).toHaveLength(1);
  });

  it("keeps index and working-tree dialects distinct and refuses outside-root enumeration", () => {
    expect(() =>
      defineScanSurface({
        scannerPath: "scripts/check.ts",
        mode: "workingTree",
        pathspecs: [":(glob)rust/**/*.rs"],
        includeUntracked: false,
        excludes: [],
      }),
    ).toThrow(/cannot use git magic/u);
    expect(() =>
      defineScanSurface({
        scannerPath: "scripts/check.ts",
        mode: "workingTree",
        pathspecs: ["rust/**"],
        includeUntracked: true,
        excludes: [],
      }),
    ).toThrow(/cannot set includeUntracked/u);

    const root = mkdtempSync(path.join(os.tmpdir(), "omena-surface-root-"));
    mkdirSync(path.join(root, "rust"), { recursive: true });
    writeFileSync(path.join(root, "rust/input.rs"), "");
    const surface = resolveScanSurface(
      defineScanSurface({
        scannerPath: "scripts/check.ts",
        mode: "workingTree",
        pathspecs: ["rust/**"],
        includeUntracked: false,
        excludes: [],
      }),
      { repoRoot: root },
    );
    expect(surface.paths).toEqual(["rust/input.rs"]);
    expect(() => surface.readdirSync(os.tmpdir())).toThrow(/outside repoRoot/u);
    const magic = defineScanSurface({
      scannerPath: "scripts/check.ts",
      mode: "index",
      pathspecs: [":(glob)rust/crates/**/*.rs"],
      includeUntracked: false,
      excludes: [],
    });
    expect(scanSurfaceMatchesPath(magic, "rust/crates/top.rs")).toBe(true);
    expect(scanSurfaceMatchesPath(magic, "rust/crates/pkg/src/lib.rs")).toBe(true);
  });

  it("requires a reason for a real narrowing and accepts the same narrowing with a reason", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-surface-narrowing-"));
    mkdirSync(path.join(root, "rust"), { recursive: true });
    mkdirSync(path.join(root, "docs"), { recursive: true });
    writeFileSync(path.join(root, "rust/input.rs"), "");
    writeFileSync(path.join(root, "docs/input.md"), "");
    const oldSpec = defineScanSurface({
      scannerPath: "scripts/check.ts",
      mode: "workingTree",
      pathspecs: ["**"],
      includeUntracked: false,
      excludes: [],
    });
    const newSpec = defineScanSurface({
      scannerPath: "scripts/check.ts",
      mode: "workingTree",
      pathspecs: ["rust/**"],
      includeUntracked: false,
      excludes: [],
    });
    expect(() => assertSurfaceNarrowingReason(root, oldSpec, newSpec)).toThrow(
      /narrowed without a reason/u,
    );
    expect(() =>
      assertSurfaceNarrowingReason(root, oldSpec, newSpec, "scanner reads Rust only"),
    ).not.toThrow();
  });
});

describe("writer registry portability", () => {
  it("builds from the governed index when ripgrep is unavailable", () => {
    const originalPath = process.env.PATH;
    const executableName = process.platform === "win32" ? "git.exe" : "git";
    const gitExecutable = (originalPath ?? "")
      .split(path.delimiter)
      .map((directory) => path.join(directory, executableName))
      .find((candidate) => existsSync(candidate));
    expect(gitExecutable).toBeTruthy();
    const isolatedBin = mkdtempSync(path.join(os.tmpdir(), "omena-writer-path-"));
    symlinkSync(gitExecutable!, path.join(isolatedBin, executableName));
    process.env.PATH = isolatedBin;
    try {
      const registry = buildEvidenceWriterRegistry(path.resolve(import.meta.dirname, "../../.."));
      expect(
        registry.artifacts.some((row) => row.artifactPath === "rust/evidence-writer-registry.json"),
      ).toBe(true);
    } finally {
      if (originalPath === undefined) delete process.env.PATH;
      else process.env.PATH = originalPath;
      rmSync(isolatedBin, { recursive: true, force: true });
    }
  }, 10_000);
});

describe("digest-pinned writer wrapper", () => {
  it("refuses missing external inputs and expands only declared environment keys", () => {
    const command = ["writer", "${CORPUS_ROOT}"];
    expect(() => resolveEvidenceWriterCommand(command, ["CORPUS_ROOT"], {})).toThrow(
      /NOT-PREVIEWABLE external inputs are absent/u,
    );
    expect(
      resolveEvidenceWriterCommand(command, ["CORPUS_ROOT"], { CORPUS_ROOT: "/data" }),
    ).toEqual(["writer", "/data"]);
    expect(() => resolveEvidenceWriterCommand(command, [], { CORPUS_ROOT: "/data" })).toThrow(
      /undeclared environment key/u,
    );
    expect(() => resolveEvidenceWriterCommand(["${UNDECLARED}"], [], {})).toThrow(
      /undeclared environment key/u,
    );
  });

  it("accepts a clean writer and rejects persistent mid-run skew while restoring output", async () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-writer-pin-"));
    writeFileSync(path.join(root, "input.txt"), "before");
    writeFileSync(path.join(root, "output.txt"), "old");
    const command = [
      process.execPath,
      "-e",
      "setTimeout(() => require('fs').writeFileSync('output.txt', 'new'), 80)",
    ];
    execFileSync(command[0]!, command.slice(1), { cwd: root });
    const manuallyWritten = readFileSync(path.join(root, "output.txt"));
    writeFileSync(path.join(root, "output.txt"), "old");
    await expect(
      runDigestPinnedWriter({
        repoRoot: root,
        command,
        inputPaths: ["input.txt"],
        outputPaths: ["output.txt"],
      }),
    ).resolves.toMatchObject({ exitCode: 0 });
    expect(readFileSync(path.join(root, "output.txt"))).toEqual(manuallyWritten);

    writeFileSync(path.join(root, "output.txt"), "old");
    await expect(
      runDigestPinnedWriter({
        repoRoot: root,
        command,
        inputPaths: ["input.txt"],
        outputPaths: ["output.txt"],
        onStartedForTest: async () => {
          await new Promise((resolve) => setTimeout(resolve, 20));
          writeFileSync(path.join(root, "input.txt"), "changed");
        },
      }),
    ).rejects.toThrow(/concurrent-skew/u);
    expect(readFileSync(path.join(root, "output.txt"), "utf8")).toBe("old");
  });
});

describe("pre-push evidence budget", () => {
  const gate = (id: string, command = `node ${id}.mjs`): CheckGate => ({
    id,
    scriptName: `check:${id}`,
    command,
    scope: "tooling",
    kind: "gate",
    origin: "declared",
    executor: "direct",
    referencedScripts: [],
  });
  const gates = [
    gate("b"),
    gate("a"),
    gate("large"),
    gate("unmeasured"),
    gate("writer", "node writer.mjs --write"),
  ];
  const manifest = {
    rootDir: "/repo",
    gates,
    bundles: [],
    diagnostics: [],
    lifecycleByGateId: new Map(),
  } satisfies CheckManifest;
  const ledger = {
    schemaVersion: "1",
    product: "test",
    generatedAt: "2026-08-31",
    sourceRunIds: ["1"],
    gates: [
      { gateId: "a", p50Ms: 10, p95Ms: 20_000, sampleCount: 2 },
      { gateId: "b", p50Ms: 10, p95Ms: 20_000, sampleCount: 2 },
      { gateId: "large", p50Ms: 10, p95Ms: 30_000, sampleCount: 2 },
      { gateId: "writer", p50Ms: 10, p95Ms: 1, sampleCount: 2 },
    ],
    jobs: [],
    recordsDigest: "0".repeat(64),
  } satisfies CostLedger;

  it("runs the cheapest deterministic prefix, lists the remainder, and never selects writers", () => {
    const plan = buildEvidencePreviewBudgetPlan(
      gates.map((entry) => entry.id),
      manifest,
      ledger,
    );
    expect(plan.ranPrefix.map((entry) => entry.gateId)).toEqual(["a", "b"]);
    expect(plan.estimatedRunMs).toBe(40_000);
    expect(plan.skipped).toEqual([
      { gateId: "large", p95Ms: 30_000 },
      { gateId: "unmeasured", p95Ms: null },
    ]);
    expect(plan.omittedWriteModeGateIds).toEqual(["writer"]);
    expect(isEvidencePreviewCheckGate(gates.at(-1)!)).toBe(false);
    for (const command of [
      "node writer.mjs --write-snapshots",
      "node writer.mjs --write-any-future-form",
      "node writer.mjs --update-census",
      "node writer.mjs --update-snapshots",
    ]) {
      expect(isEvidencePreviewCheckGate(gate("mutation", command)), command).toBe(false);
    }
    expect(plan.budgetMs).toBe(EVIDENCE_PREVIEW_BUDGET_MS);
    expect(EVIDENCE_PREVIEW_EMPTY_BUDGET_MS).toBe(5_000);
  });

  it("blocks on the first failing selected gate", async () => {
    const plan = buildEvidencePreviewBudgetPlan(["a", "b"], manifest, ledger);
    const observed: string[] = [];
    await expect(
      executeEvidencePreviewBudget(plan, async (gateId) => {
        observed.push(gateId);
        return gateId === "a" ? 1 : 0;
      }),
    ).rejects.toThrow(/preview gate failed/u);
    expect(observed).toEqual(["a"]);
  });
});
