import { execFileSync, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  AFFECTED_PATH_RULE_MODULE_PATH,
  AFFECTED_PATH_RULES,
} from "../../../packages/check-orchestrator/src/affected";
import {
  buildEvidenceAffectedPlan,
  orderAffectedArtifacts,
} from "../../../packages/check-orchestrator/src/evidence/affected-evidence";
import {
  assertHistoricalSurfaceNarrowing,
  assertSurfaceNarrowingReason,
  buildEvidenceScanSurfaceManifest,
  evidenceScanSurfaceFreshnessDiagnostics,
  resolveEvidenceHistoricalAuthorityRef,
} from "../../../packages/check-orchestrator/src/evidence/scan-surface-registry";
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
  loadCommittedEvidenceScanSurfaceManifest,
  renderEvidenceScanSurfaceManifest,
  sha256Text,
} from "../../../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import {
  assertEvidenceWriterAuthorityCoverage,
  assertEvidenceWriterOutputAuthorityCoverage,
  assertUpdateCommandWriterCoverage,
  buildEvidenceWriterRegistry,
  discoverEvidenceArtifactPaths,
  discoverEvidenceWriterOutputAuthority,
  EVIDENCE_WRITER_REGISTRY_PATH,
  loadEvidenceWriterRegistry,
  renderEvidenceWriterRegistry,
  type EvidenceArtifactRowV0,
  type EvidenceWriterRegistryV0,
} from "../../../packages/check-orchestrator/src/evidence/writer-registry";
import {
  EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH,
  EVIDENCE_WRITER_COMMAND_DECLARATIONS,
} from "../../../packages/check-orchestrator/src/evidence/writer-command-authority";
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

const knownGateIds = [
  "docs/check",
  "external/check",
  "rust/check",
  "rust/consume",
  "rust/write",
] as const;

function expectedScannerPaths(manifest = scannerManifest()): readonly string[] {
  return manifest.scanners
    .filter((row) => row.disposition !== "RETIRED")
    .map((row) => row.scannerPath);
}

function scannerManifest(): EvidenceScanSurfaceManifestV0 {
  return {
    schemaVersion: "0",
    generatedBy: "pnpm omena-check evidence-surfaces --write",
    detector: { fileCount: 3, patternSha256: "f".repeat(64) },
    resolverSha256: "0".repeat(64),
    predicateAuthority: {
      modulePath: "packages/check-orchestrator/src/evidence/predicates/index.ts",
      sha256: "9".repeat(64),
    },
    predicateModules,
    pathPlaneAuthority: {
      modulePath: AFFECTED_PATH_RULE_MODULE_PATH,
      sha256: "8".repeat(64),
      rules: AFFECTED_PATH_RULES,
    },
    scanners: [
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
    inputPaths: [],
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
    commandAuthority: {
      modulePath: EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH,
      sha256: "a".repeat(64),
      declaredCommandCount: 1,
      declaredOutputCount: 1,
    },
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
      { ownerId: "scripts/clock.ts", kind: "calendar-time", detail: "wall clock" },
      { ownerId: "scripts/env.ts", kind: "environment", detail: "environment" },
      { ownerId: "scripts/history.ts", kind: "git-history", detail: "history" },
      { ownerId: "scripts/writer.ts", kind: "concurrent-worktree", detail: "skew" },
      {
        ownerId: "scripts/external.ts",
        kind: "network-or-external-checkout",
        detail: "external",
      },
      { ownerId: "scripts/binary.ts", kind: "built-binary", detail: "binary" },
      { ownerId: "scripts/tool.ts", kind: "toolchain-bytes", detail: "tool" },
    ],
  };
}

describe("evidence affected closure", () => {
  it("closes artifacts in dependency order and names a non-empty excluded set", () => {
    const plan = buildEvidenceAffectedPlan({
      changedPaths: ["rust/crates/omena-query/src/lib.rs"],
      scanManifest: scannerManifest(),
      writerRegistry: writerRegistry(),
      expectedScannerPaths: expectedScannerPaths(),
      knownGateIds,
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
      knownGateIds,
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

  it("fails closed when an evidence row names a gate outside the live gate universe", () => {
    const registry = writerRegistry();
    const plan = buildEvidenceAffectedPlan({
      changedPaths: ["rust/crates/omena-query/src/lib.rs"],
      scanManifest: scannerManifest(),
      writerRegistry: {
        ...registry,
        artifacts: registry.artifacts.map((row) =>
          row.artifactPath === "rust/a.json"
            ? { ...row, writerGateIds: ["brand-new/unknown-gate"] }
            : row,
        ),
      },
      expectedScannerPaths: expectedScannerPaths(),
      knownGateIds,
    });
    expect(plan.fallbackToFullEvidence).toBe(true);
    expect(plan.fallbackReasons).toContain(
      "evidence manifest references unknown gate: brand-new/unknown-gate",
    );
    expect(plan.evidenceRefreshSufficientAlone).toBe(false);
  });

  it("enforces FULL path-plane implication while retaining a legitimate narrow control", () => {
    const full = buildEvidenceAffectedPlan({
      changedPaths: ["package.json"],
      scanManifest: scannerManifest(),
      writerRegistry: writerRegistry(),
      expectedScannerPaths: expectedScannerPaths(),
      knownGateIds,
    });
    expect(full.pathPlane.requiresFullCi).toBe(true);
    expect(full.evidenceRefreshSufficientAlone).toBe(false);
    expect(() =>
      buildEvidenceAffectedPlan({
        changedPaths: ["package.json"],
        scanManifest: scannerManifest(),
        writerRegistry: writerRegistry(),
        expectedScannerPaths: expectedScannerPaths(),
        knownGateIds,
        forceNarrowSufficiencyForTest: true,
      }),
    ).toThrow(/requires FULL CI/u);

    const narrow = buildEvidenceAffectedPlan({
      changedPaths: ["rust/crates/omena-query/src/lib.rs"],
      scanManifest: scannerManifest(),
      writerRegistry: writerRegistry(),
      expectedScannerPaths: expectedScannerPaths(),
      knownGateIds,
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
      expectedScannerPaths: expectedScannerPaths(),
      knownGateIds,
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

    const aggregateCommand = ["node", "aggregate.mjs", "--write"];
    const focusedCommand = ["node", "focused.mjs", "--write"];
    const alternateRegistry = {
      ...registry,
      artifacts: [
        ...registry.artifacts,
        artifact("rust/aggregate.json", { writeCommand: aggregateCommand }),
        artifact("rust/focused.json", {
          writeCommand: focusedCommand,
          alternateWriteCommands: [aggregateCommand],
        }),
      ].toSorted((left, right) =>
        left.artifactPath < right.artifactPath
          ? -1
          : left.artifactPath > right.artifactPath
            ? 1
            : 0,
      ),
    } satisfies EvidenceWriterRegistryV0;
    const aggregatePlan = buildEvidenceAffectedPlan({
      changedPaths: ["rust/aggregate.json"],
      scanManifest: scannerManifest(),
      writerRegistry: alternateRegistry,
      expectedScannerPaths: expectedScannerPaths(),
      knownGateIds,
    });
    expect(aggregatePlan.affectedArtifactPaths).toContain("rust/focused.json");
    const focusedPlan = buildEvidenceAffectedPlan({
      changedPaths: ["rust/focused.json"],
      scanManifest: scannerManifest(),
      writerRegistry: alternateRegistry,
      expectedScannerPaths: expectedScannerPaths(),
      knownGateIds,
    });
    expect(focusedPlan.affectedArtifactPaths).not.toContain("rust/aggregate.json");
  });
});

describe("scan surface falsifiers", () => {
  it("rejects a direct enumeration beside the resolver and accepts the routed control", () => {
    expect(findUnroutedScannerCallSites("scripts/check.ts", "readdirSync(root);")).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/check.ts",
        'import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url); evidenceScanSurface.readdirSync(root);',
      ),
    ).toEqual([]);
  });

  it("binds canonical lexical symbols while rejecting aliases, shadows, and look-alikes", () => {
    expect(
      findUnroutedScannerCallSites(
        "scripts/aliased.ts",
        'import { readdirSync as enumerate } from "node:fs"; enumerate(root);',
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/computed.ts",
        'import * as fs from "node:fs"; fs["readdirSync"](root);',
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/look-alike.ts",
        "const scanSurface = { readdirSync() { return []; } }; scanSurface.readdirSync(root);",
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/ordinary.ts",
        "const collection = { read() { return []; } }; collection.read(root);",
      ),
    ).toEqual([]);
    expect(
      findUnroutedScannerCallSites(
        "scripts/forged-type.ts",
        "function scan(scanSurface: ReturnType<typeof resolveScanSurfaceForScanner>) { scanSurface.readdirSync(root); }",
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/typed-control.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; function scan(surface: ReturnType<typeof resolveSurface>) { surface["readdirSync"](root); }',
      ),
    ).toEqual([]);
    expect(
      findUnroutedScannerCallSites(
        "scripts/fake-relative.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "./fake/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); surface.readdirSync(root);',
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/shadowed.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); function scan(surface: any) { surface.readdirSync(root); } scan(surface);',
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/structural-forgery.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; type Decoy = { resolver: typeof resolveSurface; readdirSync(root: string): string[] }; function scan(surface: Decoy) { surface.readdirSync(root); }',
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/fake-package.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "@fake/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); surface.readdirSync(root);',
      ),
    ).toHaveLength(1);
    expect(
      findUnroutedScannerCallSites(
        "scripts/factory-control.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; function makeSurface() { return resolveSurface(import.meta.url); } const surface = makeSurface(); surface.readdirSync(root);',
      ),
    ).toEqual([]);
    for (const [fixturePath, source] of [
      [
        "scripts/loop-resolver-shadow.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); for (const resolveSurface of fakeFactories) { const local = resolveSurface(); local.readdirSync(root); }',
      ],
      [
        "scripts/loop-surface-shadow.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); for (const surface of fakeSurfaces) { surface.readdirSync(root); }',
      ],
      [
        "scripts/catch-surface-shadow.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); try { throw fakeSurface; } catch (surface) { surface.readdirSync(root); }',
      ],
      [
        "scripts/destructured-surface-shadow.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); function scan({ surface }: { surface: any }) { surface.readdirSync(root); }',
      ],
      [
        "scripts/nested-array-shadow.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; const surface = resolveSurface(import.meta.url); const [{ surface: nestedSurface = surface }] = fakeSurfaces; nestedSurface.readdirSync(root);',
      ],
    ] as const) {
      expect(findUnroutedScannerCallSites(fixturePath, source), fixturePath).toHaveLength(1);
    }
    expect(
      findUnroutedScannerCallSites(
        "scripts/destructured-control.ts",
        'import { resolveScanSurfaceForScanner as resolveSurface } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest"; function scan({ input }: { input: string }) { const surface = resolveSurface(import.meta.url); surface.readdirSync(input); }',
      ),
    ).toEqual([]);
  });

  it("digest-binds the predicate behavior map and owner-bearing affected rules", () => {
    const baseline = scannerManifest();
    const rendered = renderEvidenceScanSurfaceManifest(baseline);
    expect(rendered).toContain('"predicateAuthority"');
    expect(rendered).toContain('"pathPlaneAuthority"');
    expect(
      renderEvidenceScanSurfaceManifest({
        ...baseline,
        predicateAuthority: { ...baseline.predicateAuthority!, sha256: "7".repeat(64) },
      }),
    ).not.toBe(rendered);
    expect(() =>
      renderEvidenceScanSurfaceManifest({
        ...baseline,
        pathPlaneAuthority: { ...baseline.pathPlaneAuthority!, rules: [] },
      }),
    ).toThrow(/rules must be non-empty/u);
    expect(() =>
      renderEvidenceScanSurfaceManifest({
        ...baseline,
        pathPlaneAuthority: {
          ...baseline.pathPlaneAuthority!,
          rules: [
            {
              ...baseline.pathPlaneAuthority!.rules[0]!,
              ownerModulePath: "not-a-module-label",
            },
          ],
        },
      }),
    ).toThrow(/non-module owner/u);
  }, 10_000);

  it("pins live authority bytes and rejects a stale predicate-dispatch digest", () => {
    const repoRoot = path.resolve(import.meta.dirname, "../../..");
    const live = JSON.parse(
      readFileSync(path.join(repoRoot, "rust/evidence-scan-surfaces.json"), "utf8"),
    ) as EvidenceScanSurfaceManifestV0;
    const predicateSource = readFileSync(
      path.join(repoRoot, "packages/check-orchestrator/src/evidence/predicates/index.ts"),
      "utf8",
    );
    expect(live.predicateAuthority?.sha256).toBe(sha256Text(predicateSource));
    expect(live.pathPlaneAuthority?.rules).toEqual(AFFECTED_PATH_RULES);
    expect(evidenceScanSurfaceFreshnessDiagnostics(repoRoot, live)).toEqual([]);
    expect(
      evidenceScanSurfaceFreshnessDiagnostics(repoRoot, {
        ...live,
        predicateAuthority: { ...live.predicateAuthority!, sha256: "0".repeat(64) },
      }),
    ).toContain(
      "predicate dispatch digest is stale: packages/check-orchestrator/src/evidence/predicates/index.ts",
    );
  });

  it("keeps a coherent committed narrowing RED against the verified parent authority", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-evidence-history-"));
    const manifestPath = path.join(root, "rust/evidence-scan-surfaces.json");
    const resolverPath = path.join(
      root,
      "packages/check-orchestrator/src/evidence/scan-surface.ts",
    );
    const surfaceModulePath = path.join(root, "scripts/surfaces/history.surface.ts");
    mkdirSync(path.dirname(manifestPath), { recursive: true });
    mkdirSync(path.dirname(resolverPath), { recursive: true });
    mkdirSync(path.dirname(surfaceModulePath), { recursive: true });
    mkdirSync(path.join(root, ".changeset"), { recursive: true });
    mkdirSync(path.join(root, "rust"), { recursive: true });
    writeFileSync(surfaceModulePath, "export default {};\n");
    writeFileSync(path.join(root, ".changeset/README.md"), "history control\n");
    writeFileSync(path.join(root, "rust/input.rs"), "// input\n");
    execFileSync("git", ["init", "-q"], { cwd: root });
    execFileSync("git", ["config", "user.name", "Evidence Test"], { cwd: root });
    execFileSync("git", ["config", "user.email", "evidence@example.invalid"], { cwd: root });
    const commitAuthority = (
      resolverSource: string,
      spec: ReturnType<typeof defineScanSurface>,
      message: string,
    ): string => {
      writeFileSync(resolverPath, resolverSource);
      writeFileSync(
        manifestPath,
        `${JSON.stringify({
          schemaVersion: "0",
          generatedBy: "pnpm omena-check evidence-surfaces --write",
          detector: { fileCount: 0, patternSha256: "0".repeat(64) },
          resolverSha256: sha256Text(resolverSource),
          predicateModules: {},
          scanners: [
            {
              scannerPath: "scripts/history.ts",
              disposition: "MIGRATED",
              surfaceModulePath: "scripts/surfaces/history.surface.ts",
              surfaceModuleSha256: sha256Text("export default {};\n"),
              spec,
              gateIds: [],
            },
          ],
        })}\n`,
      );
      execFileSync("git", ["add", "."], { cwd: root });
      execFileSync("git", ["commit", "-q", "-m", message], { cwd: root });
      return execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
    };
    const oldSpec = defineScanSurface({
      scannerPath: "scripts/history.ts",
      mode: "workingTree",
      pathspecs: ["**"],
      includeUntracked: false,
      excludes: [],
    });
    const newSpec = defineScanSurface({
      scannerPath: "scripts/history.ts",
      mode: "workingTree",
      pathspecs: ["rust/**"],
      includeUntracked: false,
      excludes: [],
    });
    const oldAuthority = commitAuthority(
      "export const authority = 'old';\n",
      oldSpec,
      "old authority",
    );
    const candidate = commitAuthority(
      "export const authority = 'candidate';\n",
      newSpec,
      "candidate",
    );
    expect(candidate).not.toBe(oldAuthority);
    const resolvedAuthority = resolveEvidenceHistoricalAuthorityRef(root);
    expect(resolvedAuthority).toBe(oldAuthority);
    const oldManifest = loadCommittedEvidenceScanSurfaceManifest(root, resolvedAuthority);
    const newManifest = JSON.parse(
      readFileSync(manifestPath, "utf8"),
    ) as EvidenceScanSurfaceManifestV0;
    const oldRow = oldManifest?.scanners[0];
    const newRow = newManifest.scanners[0];
    expect(oldRow?.disposition).toBe("MIGRATED");
    expect(newRow?.disposition).toBe("MIGRATED");
    if (oldRow?.disposition !== "MIGRATED" || newRow?.disposition !== "MIGRATED") {
      throw new Error("history fixture lost its migrated rows");
    }
    expect(() => assertSurfaceNarrowingReason(root, oldRow.spec, newRow.spec)).toThrow(
      /first removed path \.changeset\/README\.md/u,
    );
  }, 10_000);

  it("keeps coherent predicate narrowing wired through the production manifest builder", async () => {
    const repoRoot = path.resolve(import.meta.dirname, "../../..");
    const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "omena-production-narrowing-"));
    const worktree = path.join(temporaryRoot, "repo");
    execFileSync("git", ["worktree", "add", "--detach", worktree, "HEAD"], {
      cwd: repoRoot,
      stdio: "ignore",
    });
    try {
      symlinkSync(
        path.join(repoRoot, "node_modules"),
        path.join(worktree, "node_modules"),
        process.platform === "win32" ? "junction" : "dir",
      );
      await expect(buildEvidenceScanSurfaceManifest(worktree)).resolves.toBeDefined();
      const predicatePath = "packages/check-orchestrator/src/evidence/predicates/personal-docs.ts";
      const predicateSource = readFileSync(path.join(worktree, predicatePath), "utf8");
      expect(predicateSource).toContain('candidate === ".personal_docs"');
      writeFileSync(
        path.join(worktree, predicatePath),
        predicateSource.replace(
          'candidate === ".personal_docs"',
          'candidate === ".personal_docs" || candidate === "docs" || candidate.startsWith("docs/")',
        ),
      );
      execFileSync("git", ["add", predicatePath], { cwd: worktree });
      execFileSync("git", ["commit", "-q", "-m", "narrow evidence predicate"], {
        cwd: worktree,
      });
      await expect(buildEvidenceScanSurfaceManifest(worktree)).rejects.toThrow(
        /narrowed without a new reason.*docs\//u,
      );
    } finally {
      execFileSync("git", ["worktree", "remove", "--force", worktree], {
        cwd: repoRoot,
        stdio: "ignore",
      });
      rmSync(temporaryRoot, { recursive: true, force: true });
    }
  }, 60_000);

  it("keeps a coherent committed predicate behavior narrowing RED until a new reason is present", async () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-predicate-history-"));
    const manifestPath = path.join(root, "rust/evidence-scan-surfaces.json");
    const resolverPath = path.join(
      root,
      "packages/check-orchestrator/src/evidence/scan-surface.ts",
    );
    const predicateAuthorityPath = path.join(
      root,
      "packages/check-orchestrator/src/evidence/predicates/index.ts",
    );
    const predicatePath = path.join(
      root,
      "packages/check-orchestrator/src/evidence/predicates/personal-docs.ts",
    );
    const surfaceModulePath = path.join(root, "scripts/surfaces/history.surface.ts");
    for (const target of [
      manifestPath,
      resolverPath,
      predicateAuthorityPath,
      predicatePath,
      surfaceModulePath,
    ]) {
      mkdirSync(path.dirname(target), { recursive: true });
    }
    mkdirSync(path.join(root, "docs"), { recursive: true });
    mkdirSync(path.join(root, "rust"), { recursive: true });
    writeFileSync(path.join(root, "docs/input.md"), "predicate control\n");
    writeFileSync(path.join(root, "rust/input.rs"), "// retained\n");
    const resolverSource = "export const authority = 'resolver';\n";
    const predicateAuthoritySource = "export const authority = 'predicates';\n";
    const surfaceSource = "export default {};\n";
    const oldPredicateSource =
      "const excludesPersonalDocs = () => false; export default excludesPersonalDocs;\n";
    const newPredicateSource =
      'const excludesPersonalDocs = (candidate) => candidate.startsWith("docs/"); export default excludesPersonalDocs;\n';
    writeFileSync(resolverPath, resolverSource);
    writeFileSync(predicateAuthorityPath, predicateAuthoritySource);
    writeFileSync(surfaceModulePath, surfaceSource);
    const spec = defineScanSurface({
      scannerPath: "scripts/history.ts",
      mode: "index",
      pathspecs: ["**"],
      includeUntracked: false,
      excludes: ["personal-docs"],
    });
    const manifestFor = (
      predicateSource: string,
      narrowingReason?: string,
    ): EvidenceScanSurfaceManifestV0 =>
      ({
        schemaVersion: "0",
        generatedBy: "pnpm omena-check evidence-surfaces --write",
        detector: { fileCount: 0, patternSha256: "0".repeat(64) },
        resolverSha256: sha256Text(resolverSource),
        predicateAuthority: {
          modulePath: "packages/check-orchestrator/src/evidence/predicates/index.ts",
          sha256: sha256Text(predicateAuthoritySource),
        },
        predicateModules: {
          "personal-docs": {
            modulePath: "packages/check-orchestrator/src/evidence/predicates/personal-docs.ts",
            sha256: sha256Text(predicateSource),
          },
        },
        scanners: [
          {
            scannerPath: "scripts/history.ts",
            disposition: "MIGRATED",
            surfaceModulePath: "scripts/surfaces/history.surface.ts",
            surfaceModuleSha256: sha256Text(surfaceSource),
            spec,
            gateIds: [],
            ...(narrowingReason ? { narrowingReason } : {}),
          },
        ],
      }) as EvidenceScanSurfaceManifestV0;

    execFileSync("git", ["init", "-q"], { cwd: root });
    execFileSync("git", ["config", "user.name", "Evidence Test"], { cwd: root });
    execFileSync("git", ["config", "user.email", "evidence@example.invalid"], { cwd: root });
    writeFileSync(predicatePath, oldPredicateSource);
    writeFileSync(manifestPath, `${JSON.stringify(manifestFor(oldPredicateSource))}\n`);
    execFileSync("git", ["add", "."], { cwd: root });
    execFileSync("git", ["commit", "-q", "-m", "old predicate authority"], { cwd: root });
    writeFileSync(predicatePath, newPredicateSource);
    const narrowedManifest = manifestFor(newPredicateSource);
    writeFileSync(manifestPath, `${JSON.stringify(narrowedManifest)}\n`);
    execFileSync("git", ["add", "."], { cwd: root });
    execFileSync("git", ["commit", "-q", "-m", "candidate predicate authority"], {
      cwd: root,
    });

    const historicalAuthority = resolveEvidenceHistoricalAuthorityRef(root);
    const oldManifest = loadCommittedEvidenceScanSurfaceManifest(root, historicalAuthority);
    expect(oldManifest).not.toBeNull();
    const imported = (await import(
      `data:text/javascript;base64,${Buffer.from(newPredicateSource).toString("base64")}`
    )) as { readonly default: (candidate: string) => boolean };
    const newPredicates = { "personal-docs": imported.default };
    await expect(
      assertHistoricalSurfaceNarrowing(
        root,
        historicalAuthority,
        oldManifest,
        narrowedManifest,
        newPredicates,
      ),
    ).rejects.toThrow(/first removed path docs\/input\.md/u);
    await expect(
      assertHistoricalSurfaceNarrowing(
        root,
        historicalAuthority,
        oldManifest,
        manifestFor(newPredicateSource, "exclude reviewed documentation inputs"),
        newPredicates,
      ),
    ).resolves.toBeUndefined();
    rmSync(root, { recursive: true, force: true });
  }, 10_000);

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
  it("discovers a tracked nested-only JSON writer without a root artifact seed", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-nested-writer-"));
    const scriptPath = path.join(root, "scripts/write-nested.ts");
    const importedPathModule = path.join(root, "scripts/nested-output.ts");
    const importedWriterPath = path.join(root, "scripts/write-imported.ts");
    const artifactPath = path.join(root, "rust/crates/example/nested-census.json");
    const textArtifactPath = path.join(root, "rust/crates/example/nested-evidence.txt");
    mkdirSync(path.dirname(scriptPath), { recursive: true });
    mkdirSync(path.dirname(artifactPath), { recursive: true });
    writeFileSync(artifactPath, '{"schemaVersion":"0"}\n');
    writeFileSync(textArtifactPath, "evidence\n");
    writeFileSync(
      scriptPath,
      'import { writeFileSync } from "node:fs"; import path from "node:path"; const output = path.join(process.cwd(), "rust", "crates", "example", "nested-census.json"); writeFileSync(output, "{}\\n");\n',
    );
    writeFileSync(
      importedPathModule,
      'export const NESTED_OUTPUT = "rust/crates/example/nested-evidence.txt";\n',
    );
    writeFileSync(
      importedWriterPath,
      'import { writeFileSync } from "node:fs"; import path from "node:path"; import { NESTED_OUTPUT } from "./nested-output"; const output = path.join(process.cwd(), NESTED_OUTPUT); writeFileSync(output, "evidence\\n");\n',
    );
    execFileSync("git", ["init", "-q"], { cwd: root });
    execFileSync("git", ["add", "."], { cwd: root });
    expect(discoverEvidenceArtifactPaths(root)).toContain("rust/crates/example/nested-census.json");
    expect(discoverEvidenceArtifactPaths(root)).toContain(
      "rust/crates/example/nested-evidence.txt",
    );
    writeFileSync(
      scriptPath,
      'import path from "node:path"; const output = path.join(process.cwd(), "rust", "crates", "example", "nested-census.json"); process.stdout.write(output);\n',
    );
    expect(discoverEvidenceArtifactPaths(root)).not.toContain(
      "rust/crates/example/nested-census.json",
    );
    writeFileSync(
      importedWriterPath,
      'import { NESTED_OUTPUT } from "./nested-output"; process.stdout.write(NESTED_OUTPUT);\n',
    );
    expect(discoverEvidenceArtifactPaths(root)).not.toContain(
      "rust/crates/example/nested-evidence.txt",
    );
    rmSync(root, { recursive: true, force: true });
  });

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
      const repoRoot = path.resolve(import.meta.dirname, "../../..");
      const registry = buildEvidenceWriterRegistry(repoRoot);
      expect(registry.artifacts.map((row) => row.artifactPath)).toEqual(
        discoverEvidenceArtifactPaths(repoRoot),
      );
      expect(registry.commandAuthority).toMatchObject({
        modulePath: EVIDENCE_WRITER_COMMAND_AUTHORITY_PATH,
        declaredCommandCount: EVIDENCE_WRITER_COMMAND_DECLARATIONS.length,
        declaredOutputCount: new Set(
          EVIDENCE_WRITER_COMMAND_DECLARATIONS.flatMap((row) => row.outputPaths),
        ).size,
      });
      expect(() => assertEvidenceWriterAuthorityCoverage(registry)).not.toThrow();
      const packageScripts = (
        JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8")) as {
          readonly scripts: Readonly<Record<string, string>>;
        }
      ).scripts;
      expect(() =>
        assertUpdateCommandWriterCoverage(packageScripts, registry.artifacts),
      ).not.toThrow();
      expect(() =>
        assertUpdateCommandWriterCoverage(
          {
            ...packageScripts,
            "update:unregistered-output": "node ./scripts/unregistered-writer.mjs --write",
          },
          registry.artifacts,
        ),
      ).toThrow(/no total writer-output authority/u);
      const outputAuthority = discoverEvidenceWriterOutputAuthority(repoRoot);
      expect(outputAuthority.rootArtifactPaths.length).toBeGreaterThan(40);
      expect(outputAuthority.writeCallOutputPaths.length).toBeGreaterThan(20);
      expect(outputAuthority.declaredCommandOutputPaths).toHaveLength(
        new Set(EVIDENCE_WRITER_COMMAND_DECLARATIONS.flatMap((row) => row.outputPaths)).size,
      );
      expect(() =>
        assertEvidenceWriterOutputAuthorityCoverage(registry, outputAuthority),
      ).not.toThrow();
      const grammarDeclaration = EVIDENCE_WRITER_COMMAND_DECLARATIONS.find(
        (row) => row.commandId === "spec-audit-webref-grammar",
      )!;
      expect(grammarDeclaration.outputPaths).toEqual([
        "rust/crates/omena-spec-audit/data/webref-grammar.json",
        "rust/crates/omena-spec-audit/data/webref-registry-delta.json",
      ]);
      const lineIndexDeclaration = EVIDENCE_WRITER_COMMAND_DECLARATIONS.find(
        (row) => row.commandId === "syntax-line-index-authority",
      )!;
      expect(lineIndexDeclaration.outputPaths).toContain(
        "rust/crates/omena-syntax/tests/snapshots/public-api.txt",
      );
      for (const declaration of EVIDENCE_WRITER_COMMAND_DECLARATIONS) {
        for (const outputPath of declaration.outputPaths) {
          const row = registry.artifacts.find((candidate) => candidate.artifactPath === outputPath);
          expect(
            [row?.writeCommand, ...(row?.alternateWriteCommands ?? [])].some(
              (command) => JSON.stringify(command) === JSON.stringify(declaration.writeCommand),
            ),
            `${declaration.commandId}:${outputPath}`,
          ).toBe(true);
        }
      }
      expect(() =>
        assertEvidenceWriterAuthorityCoverage({
          ...registry,
          artifacts: registry.artifacts.filter(
            (row) => row.artifactPath !== "rust/crates/omena-spec-audit/data/webref-grammar.json",
          ),
        }),
      ).toThrow(/authority output is absent/u);
      expect(() =>
        assertEvidenceWriterOutputAuthorityCoverage(
          {
            ...registry,
            artifacts: registry.artifacts.filter(
              (row) => row.artifactPath !== outputAuthority.rootArtifactPaths[0],
            ),
          },
          outputAuthority,
        ),
      ).toThrow(/output is absent from registry/u);
      expect(() =>
        assertEvidenceWriterAuthorityCoverage({
          ...registry,
          artifacts: registry.artifacts.map((row) =>
            row.artifactPath === "rust/crates/omena-syntax/tests/snapshots/public-api.txt"
              ? { ...row, writeCommand: ["node", "wrong-writer.mjs"] }
              : row,
          ),
        }),
      ).toThrow(/authority command drifted/u);
      expect(
        registry.artifacts.filter((row) => !["W1", "W2", "W3", "W4"].includes(row.classification)),
      ).toEqual([]);
      expect(
        registry.artifacts.some((row) => row.artifactPath === "rust/evidence-writer-registry.json"),
      ).toBe(true);
      const linkedBaseline = registry.artifacts.find(
        (row) => row.artifactPath === "rust/omena-linked-emission-byte-differential-baseline.json",
      );
      const linkedCensus = registry.artifacts.find(
        (row) =>
          row.artifactPath ===
          "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-coverage-census.json",
      );
      expect(linkedCensus?.writeCommand).toEqual(linkedBaseline?.writeCommand);
      const nestedCliCensus = registry.artifacts.find(
        (row) => row.artifactPath === "rust/crates/omena-cli/json-output-census.json",
      );
      expect(nestedCliCensus?.classification).toBe("W2");
      expect(nestedCliCensus?.writerScripts).toEqual([
        "scripts/check-rust-omena-cli-json-output-census.ts",
      ]);
      expect(nestedCliCensus?.writeCommand).toEqual([
        "node",
        "--import",
        "tsx",
        "./scripts/check-rust-omena-cli-json-output-census.ts",
        "--write",
      ]);
      const parserEditSlope = registry.artifacts.find(
        (row) =>
          row.artifactPath ===
          "rust/crates/omena-benchmarks/baselines/parser-edit-slope-baseline-v0.json",
      );
      expect(parserEditSlope?.writerScripts).toEqual([
        "scripts/check-rust-z5-perf-gate-baseline.ts",
        "scripts/lib/parser-edit-slope-gate.ts",
      ]);
      expect(parserEditSlope?.writeCommand).toEqual([
        "node",
        "--import",
        "tsx",
        "./scripts/check-rust-z5-perf-gate-baseline.ts",
        "--write",
      ]);
      const published = registry.artifacts.find(
        (row) => row.artifactPath === "rust/omena-published-crate-surface-register.json",
      );
      expect(published?.writeCommand).toContain("--initialize-from");
      expect(published?.writeCommand).toContain("${OMENA_PUBLISHED_CRATE_REGISTRY_STATE}");
      expect(published?.requiredEnvironmentKeys).toContain("OMENA_PUBLISHED_CRATE_REGISTRY_STATE");
      const ffi = registry.artifacts.find(
        (row) => row.artifactPath === "rust/omena-ffi-boundary-typing-census.json",
      );
      expect(ffi?.inputPaths).toEqual([
        "rust/crates/omena-napi/src/lib.rs",
        "rust/crates/omena-napi/src/sdk_workspace.rs",
        "rust/crates/omena-wasm/src/lib.rs",
        "rust/crates/omena-wasm/src/sdk_workspace.rs",
      ]);
      const parityPath = "rust/omena-cross-surface-parity-golden.json";
      const sdkErrorPath = "rust/omena-sdk-error-mapping-census.json";
      expect(
        registry.artifacts.find((row) => row.artifactPath === parityPath)?.inputArtifactPaths,
      ).toContain(sdkErrorPath);
      expect(
        orderAffectedArtifacts(registry.artifacts, new Set([parityPath, sdkErrorPath])),
      ).toEqual([sdkErrorPath, parityPath]);
      const liveScanManifest = JSON.parse(
        readFileSync(path.join(repoRoot, "rust/evidence-scan-surfaces.json"), "utf8"),
      ) as EvidenceScanSurfaceManifestV0;
      const liveKnownGateIds = [
        ...new Set([
          ...registry.artifacts.flatMap((row) => [...row.writerGateIds, ...row.consumerGateIds]),
          ...liveScanManifest.scanners.flatMap((row) => ("gateIds" in row ? row.gateIds : [])),
        ]),
      ];
      const ffiPlan = buildEvidenceAffectedPlan({
        changedPaths: ["rust/crates/omena-napi/src/lib.rs"],
        scanManifest: liveScanManifest,
        writerRegistry: registry,
        expectedScannerPaths: liveScanManifest.scanners
          .filter((row) => row.disposition !== "RETIRED")
          .map((row) => row.scannerPath),
        knownGateIds: liveKnownGateIds,
      });
      expect(ffiPlan.affectedArtifactPaths).toContain("rust/omena-ffi-boundary-typing-census.json");
      expect(ffiPlan.gateIds).toContain("rust/omena-ffi-boundary-typing-census");
      expect(() =>
        renderEvidenceWriterRegistry({
          ...registry,
          artifacts: registry.artifacts.map((row) =>
            row.artifactPath === "rust/omena-published-crate-surface-register.json"
              ? {
                  ...row,
                  writeCommand: [
                    "node",
                    "./scripts/check-rust-published-crate-surface-register.ts",
                  ],
                }
              : row,
          ),
        }),
      ).toThrow(/must bind a measured registry-state input/u);
      expect(() =>
        renderEvidenceWriterRegistry({
          ...registry,
          notPreviewableInputs: registry.notPreviewableInputs.map((entry, index) =>
            index === 0 ? { ...entry, ownerId: "calendar-gate" } : entry,
          ),
        }),
      ).toThrow(/repository module path/u);
      expect(() =>
        renderEvidenceWriterRegistry({
          ...registry,
          artifacts: registry.artifacts.map((row) =>
            row.artifactPath === "rust/omena-ffi-boundary-typing-census.json"
              ? {
                  ...row,
                  inputPaths: [],
                  inputScannerPaths: [],
                  inputArtifactPaths: [],
                }
              : row,
          ),
        }),
      ).toThrow(/unexplained empty input set/u);
    } finally {
      if (originalPath === undefined) delete process.env.PATH;
      else process.env.PATH = originalPath;
      rmSync(isolatedBin, { recursive: true, force: true });
    }
  }, 30_000);

  it("executes the declared published-register recipe from a fresh output and rejects redirection", async () => {
    const repoRoot = path.resolve(import.meta.dirname, "../../..");
    const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "omena-writer-reproduction-"));
    const worktree = path.join(temporaryRoot, "repo");
    execFileSync("git", ["worktree", "add", "--detach", worktree, "HEAD"], {
      cwd: repoRoot,
      stdio: "ignore",
    });
    try {
      symlinkSync(
        path.join(repoRoot, "node_modules"),
        path.join(worktree, "node_modules"),
        process.platform === "win32" ? "junction" : "dir",
      );
      const outputPath = "rust/omena-published-crate-surface-register.json";
      // The detached worktree starts at HEAD while this test also runs during a
      // pre-commit evidence refresh. Copy the candidate bytes whose recipe is
      // under test rather than silently exercising the previous commit.
      writeFileSync(path.join(worktree, outputPath), readFileSync(path.join(repoRoot, outputPath)));
      writeFileSync(
        path.join(worktree, EVIDENCE_WRITER_REGISTRY_PATH),
        readFileSync(path.join(repoRoot, EVIDENCE_WRITER_REGISTRY_PATH)),
      );
      const registry = loadEvidenceWriterRegistry(worktree);
      expect(registry).not.toBeNull();
      const publishedRow = registry!.artifacts.find((row) => row.artifactPath === outputPath);
      expect(publishedRow?.writeCommand).toBeDefined();
      expect(publishedRow?.freshReproductionRequired).toBe(true);
      const scriptPath = publishedRow!.writerScripts[0]!;
      const baselineBytes = readFileSync(path.join(worktree, outputPath));
      const baseline = JSON.parse(baselineBytes.toString("utf8")) as {
        readonly rows: readonly {
          readonly crate: string;
          readonly registryBaselineAtRegistration: "present" | "firstPublish";
        }[];
      };
      const statePath = "published-registry-state.json";
      writeFileSync(
        path.join(worktree, statePath),
        `${JSON.stringify({
          registered: baseline.rows
            .filter((row) => row.registryBaselineAtRegistration === "present")
            .map((row) => row.crate),
          unregistered: baseline.rows
            .filter((row) => row.registryBaselineAtRegistration === "firstPublish")
            .map((row) => row.crate),
        })}\n`,
      );
      const command = resolveEvidenceWriterCommand(
        publishedRow!.writeCommand!,
        publishedRow!.requiredEnvironmentKeys ?? [],
        { OMENA_PUBLISHED_CRATE_REGISTRY_STATE: statePath },
      );
      await expect(
        runDigestPinnedWriter({
          repoRoot: worktree,
          command,
          inputPaths: [EVIDENCE_WRITER_REGISTRY_PATH, scriptPath, statePath],
          outputPaths: [outputPath],
          requireFreshReproduction: true,
        }),
      ).resolves.toMatchObject({ exitCode: 0 });
      const freshlyGenerated = readFileSync(path.join(worktree, outputPath));
      expect(freshlyGenerated.length).toBeGreaterThan(0);
      await expect(
        runDigestPinnedWriter({
          repoRoot: worktree,
          command,
          inputPaths: [EVIDENCE_WRITER_REGISTRY_PATH, scriptPath, statePath],
          outputPaths: [outputPath],
          requireFreshReproduction: true,
        }),
      ).resolves.toMatchObject({ exitCode: 0 });
      expect(readFileSync(path.join(worktree, outputPath))).toEqual(freshlyGenerated);

      const wrongRegistry = {
        ...registry!,
        artifacts: registry!.artifacts.map((row) =>
          row.artifactPath === outputPath
            ? {
                ...row,
                writeCommand: [
                  "node",
                  "--import",
                  "tsx",
                  "./scripts/check-rust-publish-train-closure.ts",
                  "--initialize-from",
                  "${OMENA_PUBLISHED_CRATE_REGISTRY_STATE}",
                ],
              }
            : row,
        ),
      } satisfies EvidenceWriterRegistryV0;
      writeFileSync(
        path.join(worktree, EVIDENCE_WRITER_REGISTRY_PATH),
        renderEvidenceWriterRegistry(wrongRegistry),
      );
      const redirected = loadEvidenceWriterRegistry(worktree)!.artifacts.find(
        (row) => row.artifactPath === outputPath,
      )!;
      const redirectedCommand = resolveEvidenceWriterCommand(
        redirected.writeCommand!,
        redirected.requiredEnvironmentKeys ?? [],
        { OMENA_PUBLISHED_CRATE_REGISTRY_STATE: statePath },
      );
      await expect(
        runDigestPinnedWriter({
          repoRoot: worktree,
          command: redirectedCommand,
          inputPaths: [EVIDENCE_WRITER_REGISTRY_PATH, ...redirected.writerScripts, statePath],
          outputPaths: [outputPath],
          requireFreshReproduction: true,
        }),
      ).rejects.toThrow(/successful no-op/u);
    } finally {
      execFileSync("git", ["worktree", "remove", "--force", worktree], {
        cwd: repoRoot,
        stdio: "ignore",
      });
      rmSync(temporaryRoot, { recursive: true, force: true });
    }
  }, 30_000);
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
    writeFileSync(path.join(root, "sibling.txt"), "old-sibling");
    const command = [
      process.execPath,
      "-e",
      "setTimeout(() => { const fs = require('fs'); fs.writeFileSync('output.txt', 'new'); fs.writeFileSync('sibling.txt', 'new-sibling'); }, 80)",
    ];
    execFileSync(command[0]!, command.slice(1), { cwd: root });
    const manuallyWritten = readFileSync(path.join(root, "output.txt"));
    const manuallyWrittenSibling = readFileSync(path.join(root, "sibling.txt"));
    writeFileSync(path.join(root, "output.txt"), "old");
    writeFileSync(path.join(root, "sibling.txt"), "old-sibling");
    await expect(
      runDigestPinnedWriter({
        repoRoot: root,
        command,
        inputPaths: ["input.txt"],
        outputPaths: ["output.txt", "sibling.txt"],
      }),
    ).resolves.toMatchObject({ exitCode: 0 });
    expect(readFileSync(path.join(root, "output.txt"))).toEqual(manuallyWritten);
    expect(readFileSync(path.join(root, "sibling.txt"))).toEqual(manuallyWrittenSibling);

    writeFileSync(path.join(root, "output.txt"), "old");
    writeFileSync(path.join(root, "sibling.txt"), "old-sibling");
    await expect(
      runDigestPinnedWriter({
        repoRoot: root,
        command,
        inputPaths: ["input.txt"],
        outputPaths: ["output.txt", "sibling.txt"],
        onStartedForTest: async () => {
          await new Promise((resolve) => setTimeout(resolve, 20));
          writeFileSync(path.join(root, "input.txt"), "changed");
        },
      }),
    ).rejects.toThrow(/concurrent-skew/u);
    expect(readFileSync(path.join(root, "output.txt"), "utf8")).toBe("old");
    expect(readFileSync(path.join(root, "sibling.txt"), "utf8")).toBe("old-sibling");
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
    expect(plan.pricedSkippedMs).toBe(30_000);
    expect(plan.unboundedSkippedCount).toBe(1);
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
      "node writer.mjs --write=true",
      "node writer.mjs --write:path",
      "node writer.mjs --update=snapshot",
    ]) {
      expect(isEvidencePreviewCheckGate(gate("mutation", command)), command).toBe(false);
    }
    expect(plan.budgetMs).toBe(EVIDENCE_PREVIEW_BUDGET_MS);
    expect(EVIDENCE_PREVIEW_EMPTY_BUDGET_MS).toBe(5_000);
    expect(() => buildEvidencePreviewBudgetPlan(["unknown"], manifest, ledger)).toThrow(
      /unknown gate/u,
    );
  });

  it("rejects mixed write and preview CLI mode before any writer executes", () => {
    const result = spawnSync(
      process.execPath,
      [
        "--import",
        "tsx",
        "./packages/check-orchestrator/src/cli/main.ts",
        "affected",
        "--evidence",
        "--write",
        "--preview",
        "--base=HEAD",
      ],
      {
        cwd: path.resolve(import.meta.dirname, "../../.."),
        encoding: "utf8",
        env: process.env,
      },
    );
    expect(result.status).toBe(1);
    expect(result.stderr).toContain("--write and --preview are separate modes");
  }, 10_000);

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
