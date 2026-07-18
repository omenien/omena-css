import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  buildDeclaredGates,
  buildCheckPlan,
  buildCheckSurfaceReport,
  loadCheckManifest,
  renderCheckInventory,
  renderCheckPlan,
  renderCheckSurfaceReport,
  resolveGateTarget,
  runDoctor,
  type CheckDiagnostic,
  type CheckGate,
  type DeclaredCheckGateV0,
} from "../../../packages/check-orchestrator/src";

describe("check orchestrator manifest", () => {
  const manifest = loadCheckManifest();
  const repoRoot = process.cwd();

  it("mirrors the current root scripts without doctor errors", () => {
    expect(manifest.gates.length).toBeGreaterThan(150);
    expect(runDoctor(manifest).filter((diagnostic) => diagnostic.severity === "error")).toEqual([]);
  });

  it("assigns stable ids for representative scopes", () => {
    expect(resolveGateTarget(manifest, "rust/selected-query/consumers")?.scriptName).toBe(
      "check:rust-selected-query-consumers",
    );
    expect(resolveGateTarget(manifest, "rust/design-system/universality-class")?.scriptName).toBe(
      "check:rust-design-system-universality-class",
    );
    expect(resolveGateTarget(manifest, "rust/omena-query/adapter-capabilities")?.scriptName).toBe(
      "check:rust-omena-query-adapter-capabilities",
    );
    expect(resolveGateTarget(manifest, "rust/omena-resolver/boundary")?.scriptName).toBe(
      "check:rust-omena-resolver-boundary",
    );
    expect(resolveGateTarget(manifest, "rust/omena-resolver/split-boundary")?.scriptName).toBe(
      "check:rust-omena-resolver-split-boundary",
    );
    expect(
      resolveGateTarget(manifest, "rust/omena-syntax-authority-raw-scan-census")?.scriptName,
    ).toBe("check:rust-omena-syntax-authority-raw-scan-census");
    expect(
      resolveGateTarget(manifest, "rust/omena-syntax-authority-raw-scan-census:update"),
    ).toMatchObject({
      scriptName: "update:rust-omena-syntax-authority-raw-scan-census",
      ciTier: "manual",
    });
    expect(resolveGateTarget(manifest, "rust/omena-syntax/boundary")?.scriptName).toBe(
      "check:rust-omena-syntax-boundary",
    );
    expect(resolveGateTarget(manifest, "rust/omena-lsp-server/boundary")?.scriptName).toBe(
      "check:rust-omena-lsp-server-boundary",
    );
    expect(resolveGateTarget(manifest, "rust/omena-lsp-server/shell")?.scriptName).toBe(
      "check:rust-omena-lsp-server-shell",
    );
    expect(resolveGateTarget(manifest, "rust/omena-lsp-server/provider-parity")?.scriptName).toBe(
      "check:rust-omena-lsp-server-provider-parity",
    );
    expect(resolveGateTarget(manifest, "rust/omena-lsp-server/runtime-loop")?.scriptName).toBe(
      "check:rust-omena-lsp-server-runtime-loop",
    );
    expect(
      resolveGateTarget(manifest, "rust/omena-lsp-server/explain-hover-trace")?.scriptName,
    ).toBe("check:rust-omena-lsp-server-explain-hover-trace");
    expect(
      resolveGateTarget(manifest, "rust/omena-lsp-server/sass-alias-diagnostics")?.scriptName,
    ).toBe("check:rust-omena-lsp-server-sass-alias-diagnostics");
    expect(
      resolveGateTarget(manifest, "rust/omena-lsp-server/style-provider-parity")?.scriptName,
    ).toBe("check:rust-omena-lsp-server-style-provider-parity");
    expect(resolveGateTarget(manifest, "rust/omena-lsp-server/lane")?.scriptName).toBe(
      "check:rust-omena-lsp-server-lane",
    );
    expect(resolveGateTarget(manifest, "ts7/phase-b/protocol@tsgo")?.scriptName).toBe(
      "check:ts7-phase-b-protocol-tsgo",
    );
    expect(resolveGateTarget(manifest, "ts7/phase-c/watch@tsgo")?.scriptName).toBe(
      "check:ts7-phase-c-watch-tsgo",
    );
    expect(resolveGateTarget(manifest, "tsgo/release-batch")?.scriptName).toBe(
      "check:release-batch-tsgo",
    );
    expect(resolveGateTarget(manifest, "tsgo/real-project-corpus")?.scriptName).toBe(
      "check:real-project-corpus-tsgo",
    );
    expect(resolveGateTarget(manifest, "tsgo/lsp-server-smoke")?.scriptName).toBe(
      "check:lsp-server-smoke-tsgo",
    );
    expect(resolveGateTarget(manifest, "tsgo/release/bundle")?.scriptName).toBe(
      "check:tsgo-release-bundle",
    );
    expect(resolveGateTarget(manifest, "editor/provider-host-routing-boundary")?.scriptName).toBe(
      "check:provider-host-routing-boundary",
    );
    expect(resolveGateTarget(manifest, "tooling/orchestrator-doctor")?.scriptName).toBe(
      "check:orchestrator-doctor",
    );
    expect(resolveGateTarget(manifest, "tooling/orchestrator-inventory")?.scriptName).toBe(
      "check:orchestrator-inventory",
    );
    expect(
      resolveGateTarget(manifest, "release/check/packaged-engine-shadow-runner")?.scriptName,
    ).toBe("check:packaged-engine-shadow-runner");
    expect(
      resolveGateTarget(manifest, "release/check/packaged-engine-shadow-runner-matrix")?.scriptName,
    ).toBe("check:packaged-engine-shadow-runner-matrix");
    expect(
      resolveGateTarget(manifest, "release/check/packaged-selected-query-default")?.scriptName,
    ).toBe("check:packaged-selected-query-default");
  });

  it("tracks bundle script references", () => {
    const releaseBundle = resolveGateTarget(manifest, "rust/release/bundle");
    expect(releaseBundle?.kind).toBe("bundle");
    expect(releaseBundle?.origin).toBe("declared");
    expect(releaseBundle?.referencedScripts).toContain("check:rust-workspace");
    expect(releaseBundle?.referencedScripts).toContain("check:rust-producer-boundary");
    expect(releaseBundle?.referencedTargetSpecs).toContainEqual({
      target: "rust/gate/evidence",
      args: ["--variant", "tsgo", "--repeat", "1", "--json"],
    });

    const rustLaneBundle = resolveGateTarget(manifest, "rust/lane/bundle");
    expect(rustLaneBundle?.kind).toBe("bundle");
    expect(rustLaneBundle?.origin).toBe("declared");
    expect(rustLaneBundle?.referencedScripts).toEqual(
      expect.arrayContaining([
        "check:rust-omena-syntax-boundary",
        "check:rust-producer-boundary",
        "check:rust-theory-claim-levels",
      ]),
    );

    const phaseADecisionReady = resolveGateTarget(manifest, "ts7/phase-a/decision-ready");
    expect(phaseADecisionReady?.kind).toBe("bundle");
    expect(phaseADecisionReady?.referencedScripts).toEqual(
      expect.arrayContaining(["check:ts7-phase-a-shadow-review", "check:ts7-phase-a-tsgo-lane"]),
    );

    const tsgoReleaseBundle = resolveGateTarget(manifest, "tsgo/release/bundle");
    expect(tsgoReleaseBundle?.kind).toBe("alias");
    expect(tsgoReleaseBundle?.referencedScripts).toEqual(["check:tsgo-operational-lane"]);

    const checkerReleaseGateShadow = resolveGateTarget(
      manifest,
      "rust/checker/release-gate-shadow",
    );
    expect(checkerReleaseGateShadow?.referencedScripts).toEqual(
      expect.arrayContaining(["check:rust-checker-release-gate-readiness"]),
    );

    const selectedQueryDefaultCandidate = resolveGateTarget(
      manifest,
      "rust/selected-query/default-candidate",
    );
    expect(selectedQueryDefaultCandidate?.referencedScripts).toEqual(
      expect.arrayContaining(["check:rust-selected-query-workspace"]),
    );

    const phase2SwapReadiness = resolveGateTarget(manifest, "rust/phase-2-swap-readiness");
    expect(phase2SwapReadiness?.kind).toBe("bundle");
    expect(phase2SwapReadiness?.referencedScripts).toEqual(
      expect.arrayContaining([
        "check:provider-host-routing-boundary",
        "check:rust-omena-lsp-server-lane",
        "check:rust-selected-query-default-candidate",
        "check:rust-checker-release-gate-shadow",
      ]),
    );

    const rustLspLane = resolveGateTarget(manifest, "rust/omena-lsp-server/lane");
    expect(rustLspLane?.kind).toBe("bundle");
    expect(rustLspLane?.referencedScripts).toEqual(
      expect.arrayContaining([
        "check:rust-omena-lsp-server-shell",
        "check:rust-omena-lsp-server-cancellation",
        "check:rust-omena-lsp-server-provider-parity",
        "check:rust-omena-lsp-server-type-fact-protocol",
        "check:rust-omena-lsp-server-runtime-loop",
        "check:rust-omena-lsp-server-external-sif-runtime",
        "check:rust-omena-lsp-server-diagnostics-coalescing",
        "check:rust-omena-lsp-server-explain-hover-trace",
        "check:rust-omena-lsp-server-sass-alias-diagnostics",
        "check:rust-omena-lsp-server-resolver-cache-runtime",
        "check:rust-omena-lsp-server-resolver-identity-index",
      ]),
    );

    const releaseVerify = resolveGateTarget(manifest, "release/release/verify");
    expect(releaseVerify?.kind).toBe("bundle");
    expect(releaseVerify?.origin).toBe("declared");
    expect(releaseVerify?.referencedScripts).toEqual(
      expect.arrayContaining([
        "@declared/release/sync-server-version",
        "check",
        "check:plugin-consumer-example",
        "check:plugin-consumers",
        "check:rust-release-bundle",
        "check:tsgo-release-bundle",
        "package",
        "test",
      ]),
    );
  });

  it("loads declared closure aliases with compatibility metadata", () => {
    const runtimeQueryGate = resolveGateTarget(manifest, "rust/runtime-query-api-hardening");
    expect(runtimeQueryGate).toMatchObject({
      kind: "alias",
      origin: "declared",
      referencedTargets: ["rust/m1-runtime-query-api-hardening"],
      referencedScripts: ["check:rust-m1-runtime-query-api-hardening"],
      ciTier: "closure-fast",
      ciGroup: "closure-fast",
    });

    expect(resolveGateTarget(manifest, "rust/m1-runtime-query-api-hardening")).toMatchObject({
      deprecatedBy: "rust/runtime-query-api-hardening",
    });
  });

  it("annotates the closure-fast package-derived gates with declared metadata", () => {
    expect(resolveGateTarget(manifest, "rust/omena-query/boundary")).toMatchObject({
      origin: "package+declared",
      ciTier: "closure-fast",
      ciGroup: "closure-fast",
      tags: ["closure-fast"],
    });
    expect(resolveGateTarget(manifest, "release/check/release-tag-grammar")).toMatchObject({
      origin: "package+declared",
      ciTier: "closure-fast",
      ciGroup: "closure-fast",
      tags: ["closure-fast"],
    });
  });

  it("renders the declared closure-fast bundle plan over workflow gate deps", () => {
    const closureFast = resolveGateTarget(manifest, "rust/closure-fast");
    expect(closureFast).toMatchObject({
      kind: "bundle",
      origin: "declared",
      ciTier: "none",
      ciGroup: "closure-fast",
      tags: ["closure-fast", "ci-unreachable-allowed"],
    });

    const plan = buildCheckPlan(manifest, closureFast!);
    expect(plan.steps).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ id: "rust/runtime-query-api-hardening", depth: 1 }),
        expect.objectContaining({ id: "rust/product-facing-capability", depth: 1 }),
        expect.objectContaining({ id: "rust/theory-generalization", depth: 1 }),
        expect.objectContaining({ id: "rust/closure-fast-aggregation-complete", depth: 1 }),
      ]),
    );
  });

  it("uses declared deps for the release Rust bundle while preserving its public script", () => {
    const releaseBundle = resolveGateTarget(manifest, "rust/release/bundle");
    expect(releaseBundle).toMatchObject({
      kind: "bundle",
      origin: "declared",
      scriptName: "check:rust-release-bundle",
      ciTier: "manual",
      ciGroup: "release",
      tags: ["release"],
    });

    const plan = buildCheckPlan(manifest, releaseBundle!);
    expect(plan.steps).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ id: "rust/workspace", depth: 1 }),
        expect.objectContaining({ id: "rust/producer-boundary", depth: 1 }),
        expect.objectContaining({ id: "rust/gate/evidence", depth: 1 }),
      ]),
    );
  });

  it("uses declared deps for the Rust lane bundle while preserving its public script", () => {
    const laneBundle = resolveGateTarget(manifest, "rust/lane/bundle");
    expect(laneBundle).toMatchObject({
      kind: "bundle",
      origin: "declared",
      scriptName: "check:rust-lane-bundle",
      ciTier: "manual",
      ciGroup: "rust",
      tags: ["rust", "lane"],
    });

    const plan = buildCheckPlan(manifest, laneBundle!);
    expect(plan.steps).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ id: "rust/omena-syntax/boundary", depth: 1 }),
        expect.objectContaining({ id: "rust/producer-boundary", depth: 1 }),
        expect.objectContaining({ id: "rust/theory-claim-levels", depth: 1 }),
      ]),
    );
  });

  it("uses declared deps for Omena CSS readiness while keeping scheduled workflow reachability", () => {
    const readiness = resolveGateTarget(manifest, "rust/omena-css/h1-readiness");
    expect(readiness).toMatchObject({
      kind: "bundle",
      origin: "declared",
      scriptName: "check:rust-omena-css-h1-readiness",
      ciTier: "scheduled",
      ciGroup: "drift",
      tags: ["rust", "omena-css", "readiness"],
    });

    const plan = buildCheckPlan(manifest, readiness!);
    expect(plan.steps).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ id: "rust/omena-syntax/boundary", depth: 1 }),
        expect.objectContaining({ id: "rust/omena-diff-test-boundary", depth: 1 }),
        expect.objectContaining({ id: "rust/omena-lsp-server/split-boundary", depth: 1 }),
        expect.objectContaining({ id: "rust/z5-performance-baseline-readiness", depth: 1 }),
        expect.objectContaining({ id: "rust/omena-css/cargo-fuzz", depth: 1 }),
        expect.objectContaining({ id: "rust/omena-css/rustdoc-coverage", depth: 1 }),
      ]),
    );

    const workflow = readFileSync(
      path.join(repoRoot, ".github/workflows/omena-css-drift.yml"),
      "utf8",
    );
    expect(workflow).toContain("pnpm omena-check run rust/omena-css/h1-readiness");
  });

  it("uses declared deps for release verification while preserving its public script", () => {
    const releaseVerify = resolveGateTarget(manifest, "release/release/verify");
    expect(releaseVerify).toMatchObject({
      kind: "bundle",
      origin: "declared",
      scriptName: "release:verify",
      ciTier: "manual",
      ciGroup: "release",
      tags: ["release"],
    });

    const syncVersion = resolveGateTarget(manifest, "release/sync-server-version");
    expect(syncVersion).toMatchObject({
      kind: "command",
      origin: "declared",
      scriptName: "@declared/release/sync-server-version",
      commandParts: ["./scripts/release.sh"],
      ciTier: "manual",
      ciGroup: "release",
      tags: ["release"],
    });

    const plan = buildCheckPlan(manifest, releaseVerify!);
    expect(plan.steps).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ id: "release/sync-server-version", depth: 1 }),
        expect.objectContaining({ id: "release/check/release-m5-api-freeze-audit", depth: 1 }),
        expect.objectContaining({ id: "core/build", depth: 1 }),
        expect.objectContaining({ id: "core/check", depth: 1 }),
        expect.objectContaining({ id: "rust/release/bundle", depth: 1 }),
        expect.objectContaining({ id: "tsgo/release/bundle", depth: 1 }),
        expect.objectContaining({ id: "release/package", depth: 1 }),
      ]),
    );
  });

  it("surfaces declared replacements as preserved compatibility scripts", () => {
    const compatibilityScripts = [
      ["release/release/verify", "release:verify"],
      ["rust/release/bundle", "check:rust-release-bundle"],
      ["rust/lane/bundle", "check:rust-lane-bundle"],
      ["rust/omena-css/h1-readiness", "check:rust-omena-css-h1-readiness"],
    ] as const;

    for (const [id, scriptName] of compatibilityScripts) {
      expect(resolveGateTarget(manifest, id)).toMatchObject({
        origin: "declared",
        scriptName,
      });
      expect(resolveGateTarget(manifest, scriptName)?.id).toBe(id);
    }

    const inventory = renderCheckInventory(manifest);
    expect(inventory).toMatch(
      /\| `release\/release\/verify`\s+\| bundle\s+\| declared\s+\| `release:verify`\s+\|[^|]*\| compatibility script;/,
    );
    expect(inventory).toMatch(
      /\| `rust\/lane\/bundle`\s+\| bundle\s+\| declared\s+\| `check:rust-lane-bundle`\s+\|[^|]*\| compatibility script;/,
    );
  });

  it("resolves deprecated package aliases to canonical declared gates", () => {
    expect(resolveGateTarget(manifest, "check:rust-m1-runtime-query-api-hardening")?.id).toBe(
      "rust/runtime-query-api-hardening",
    );
    expect(resolveGateTarget(manifest, "rust/m1-runtime-query-api-hardening")).toMatchObject({
      deprecatedBy: "rust/runtime-query-api-hardening",
    });
  });

  it("protects documented public scripts unless a package script or declared alias exposes them", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(path.join(root, "README.md"), "Use `pnpm release:verify` before publishing.\n");

    const missing = runDoctor(loadCheckManifest(root, { declaredGates: [] }));
    expect(missing).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "documented-public-script-missing",
          message: expect.stringContaining(
            'README.md:1 documents "pnpm release:verify", but no package script or declared compatibility alias exposes it.',
          ),
        }),
      ]),
    );

    const covered = runDoctor(
      loadCheckManifest(root, {
        declaredGates: [
          {
            id: "release/release/verify",
            kind: "command",
            scope: "release",
            command: ["node", "--version"],
            deprecatedAliases: ["release:verify"],
            ciTier: "manual",
          },
        ],
      }),
    );
    expect(covered).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "documented-public-script-missing",
        }),
      ]),
    );
  });

  it("protects documented omena-check targets from pointing at removed gates", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(
      path.join(root, "README.md"),
      [
        "Use `pnpm omena-check run tooling/known` for the known gate.",
        "Do not document `pnpm omena-check run tooling/missing`.",
        "Do not document `pnpm omena-check bundle tooling/known` as a bundle.",
      ].join("\n"),
    );

    const diagnostics = runDoctor(
      loadCheckManifest(root, {
        declaredGates: [
          {
            id: "tooling/known",
            kind: "command",
            scope: "tooling",
            command: ["node", "--version"],
            ciTier: "manual",
          },
        ],
      }),
    );

    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "documented-omena-check-target-missing",
          message: expect.stringContaining(
            'README.md:2 documents "pnpm omena-check run tooling/missing", but no manifest gate exposes that target.',
          ),
        }),
        expect.objectContaining({
          severity: "error",
          code: "documented-omena-check-target-not-bundle",
          message: expect.stringContaining(
            'README.md:3 documents "pnpm omena-check bundle tooling/known", but target "tooling/known" is a command.',
          ),
        }),
      ]),
    );
  });

  it("lets declared gates explicitly replace package-derived gate definitions", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "check:rust-release-bundle": "echo package source",
            "check:rust-workspace": "echo workspace",
          },
        },
        null,
        2,
      ),
    );

    const replacementManifest = loadCheckManifest(root, {
      declaredGates: [
        {
          id: "rust/release/bundle",
          kind: "bundle",
          scope: "rust",
          replacesPackageTarget: "rust/release/bundle",
          deps: [{ target: "rust/workspace", args: ["--release"] }],
          ciTier: "manual",
          ciReason: "Synthetic replacement fixture is manually invoked.",
        },
      ],
    });

    expect(
      replacementManifest.gates.filter((gate) => gate.id === "rust/release/bundle"),
    ).toHaveLength(1);
    expect(resolveGateTarget(replacementManifest, "rust/release/bundle")).toMatchObject({
      origin: "declared",
      scriptName: "check:rust-release-bundle",
      referencedScripts: ["check:rust-workspace"],
      referencedTargetSpecs: [{ target: "rust/workspace", args: ["--release"] }],
    });
    expect(
      runDoctor(replacementManifest).filter((diagnostic) => diagnostic.severity === "error"),
    ).toEqual([]);
  });

  it("builds valid declared command gates", () => {
    const diagnostics: CheckDiagnostic[] = [];
    const gates = buildDeclaredGates(
      [],
      [
        {
          id: "tooling/declared-example",
          kind: "command",
          scope: "tooling",
          command: ["node", "--version"],
          ciTier: "manual",
        },
      ],
      diagnostics,
    );

    expect(diagnostics).toEqual([]);
    expect(gates).toEqual([
      expect.objectContaining({
        id: "tooling/declared-example",
        scriptName: "@declared/tooling/declared-example",
        origin: "declared",
        commandParts: ["node", "--version"],
      }),
    ]);
  });

  it("resolves declared aliases to package-derived deps", () => {
    const diagnostics: CheckDiagnostic[] = [];
    const gates = buildDeclaredGates(
      [testPackageGate({ id: "rust/package-target", scriptName: "check:rust-package-target" })],
      [
        {
          id: "rust/declared-alias",
          kind: "alias",
          scope: "rust",
          deps: ["rust/package-target"],
        },
      ],
      diagnostics,
    );

    expect(diagnostics).toEqual([]);
    expect(gates[0]).toMatchObject({
      referencedTargets: ["rust/package-target"],
      referencedScripts: ["check:rust-package-target"],
    });
  });

  it("reports declared gate duplicate ids, unknown deps, cycles, and invalid ci tiers", () => {
    const diagnostics: CheckDiagnostic[] = [];
    buildDeclaredGates(
      [],
      [
        {
          id: "rust/duplicate",
          kind: "command",
          scope: "rust",
          command: ["node", "--version"],
        },
        {
          id: "rust/duplicate",
          kind: "command",
          scope: "rust",
          command: ["node", "--version"],
        },
        {
          id: "rust/unknown-dep",
          kind: "alias",
          scope: "rust",
          deps: ["rust/missing"],
        },
        {
          id: "rust/cycle-a",
          kind: "alias",
          scope: "rust",
          deps: ["rust/cycle-b"],
        },
        {
          id: "rust/cycle-b",
          kind: "alias",
          scope: "rust",
          deps: ["rust/cycle-a"],
        },
        {
          id: "rust/invalid-tier",
          kind: "command",
          scope: "rust",
          command: ["node", "--version"],
          ciTier: "fast" as DeclaredCheckGateV0["ciTier"],
        },
      ],
      diagnostics,
    );

    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ code: "duplicate-declared-gate-id" }),
        expect.objectContaining({ code: "declared-gate-unknown-dep" }),
        expect.objectContaining({ code: "declared-gate-cycle" }),
        expect.objectContaining({ code: "declared-gate-unknown-ci-tier" }),
      ]),
    );
  });

  it("reports declared ciTier gates that are not reachable from their workflow tier", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
            check: "echo check",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(
      path.join(root, ".github/workflows/ci.yml"),
      [
        "name: CI",
        "jobs:",
        "  closure-fast:",
        "    # omena-ci-tier: closure-fast",
        "    steps:",
        "      - run: pnpm omena-check run tooling/wired-closure",
        "  verify:",
        "    # omena-ci-tier: verify",
        "    steps:",
        "      - run: pnpm omena-check run core/check",
      ].join("\n"),
    );

    const diagnostics = runDoctor(
      loadCheckManifest(root, {
        declaredGates: [
          {
            id: "tooling/wired-closure",
            kind: "command",
            scope: "tooling",
            command: ["node", "--version"],
            ciTier: "closure-fast",
          },
          {
            id: "tooling/missing-closure",
            kind: "command",
            scope: "tooling",
            command: ["node", "--version"],
            ciTier: "closure-fast",
          },
          {
            id: "tooling/missing-verify",
            kind: "command",
            scope: "tooling",
            command: ["node", "--version"],
            ciTier: "verify",
          },
        ],
      }),
    );

    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "ci-tier-unreachable",
          message:
            'Gate "tooling/missing-closure" declares ciTier "closure-fast" but is not reachable from that workflow tier.',
        }),
        expect.objectContaining({
          severity: "error",
          code: "ci-tier-unreachable",
          message:
            'Gate "tooling/missing-verify" declares ciTier "verify" but is not reachable from that workflow tier.',
        }),
      ]),
    );
    expect(diagnostics).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          message: expect.stringContaining("tooling/wired-closure"),
        }),
      ]),
    );
  });

  it("reports package-origin gates that are outside every workflow tier and leaf classification", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
            "check:rust-parser-reachable": "echo reachable",
            "check:rust-parser-orphan": "echo orphan",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(
      path.join(root, ".github/workflows/ci.yml"),
      [
        "name: CI",
        "jobs:",
        "  verify:",
        "    # omena-ci-tier: verify",
        "    steps:",
        "      - run: pnpm omena-check run rust/parser/reachable",
      ].join("\n"),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "ci-tier-unclassified",
          message:
            'Package gate "rust/parser/orphan" is not reachable from any workflow tier and has no governed leaf classification.',
        }),
      ]),
    );
    expect(diagnostics).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          message: expect.stringContaining("rust/parser/reachable"),
        }),
      ]),
    );
  });

  // rfcs#60: the per-PR rust-workspace job (the rfcs#56 strict clippy/fmt gate) is bound to
  // its own ci tier, so deleting the ci.yml job — or its `omena-check run rust/workspace`
  // step — must surface as ci-tier-unreachable instead of passing silently.
  it("guards the per-PR rust-workspace job with the ci-tier reachability check", () => {
    const declaredGates: DeclaredCheckGateV0[] = [
      {
        id: "rust/workspace",
        kind: "command",
        scope: "rust",
        command: ["node", "--version"],
        ciTier: "rust-workspace",
      },
    ];
    const packageJson = JSON.stringify(
      {
        name: "omena-css",
        scripts: {
          "omena-check": "node ./check.js",
        },
      },
      null,
      2,
    );

    const wiredRoot = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(wiredRoot, ".github/workflows"), { recursive: true });
    writeFileSync(path.join(wiredRoot, "package.json"), packageJson);
    writeFileSync(
      path.join(wiredRoot, ".github/workflows/ci.yml"),
      [
        "name: CI",
        "jobs:",
        "  rust-workspace:",
        "    # omena-ci-tier: rust-workspace",
        "    steps:",
        "      - run: pnpm omena-check run rust/workspace",
      ].join("\n"),
    );
    const wiredDiagnostics = runDoctor(loadCheckManifest(wiredRoot, { declaredGates }));
    expect(wiredDiagnostics).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          message: expect.stringContaining("rust/workspace"),
        }),
      ]),
    );

    // Delete-simulation: the same declaration with the rust-workspace job removed from ci.yml.
    const deletedRoot = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(deletedRoot, ".github/workflows"), { recursive: true });
    writeFileSync(path.join(deletedRoot, "package.json"), packageJson);
    writeFileSync(
      path.join(deletedRoot, ".github/workflows/ci.yml"),
      [
        "name: CI",
        "jobs:",
        "  verify:",
        "    # omena-ci-tier: verify",
        "    steps:",
        "      - run: pnpm omena-check run core/check",
      ].join("\n"),
    );
    const deletedDiagnostics = runDoctor(loadCheckManifest(deletedRoot, { declaredGates }));
    expect(deletedDiagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "ci-tier-unreachable",
          message:
            'Gate "rust/workspace" declares ciTier "rust-workspace" but is not reachable from that workflow tier.',
        }),
      ]),
    );
  });

  it("requires explicit ciTier handling for declared gates", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), ["name: CI", "jobs:"].join("\n"));

    const diagnostics = runDoctor(
      loadCheckManifest(root, {
        declaredGates: [
          {
            id: "tooling/missing-tier",
            kind: "command",
            scope: "tooling",
            command: ["node", "--version"],
          },
          {
            id: "tooling/not-allowed-none",
            kind: "command",
            scope: "tooling",
            command: ["node", "--version"],
            ciTier: "none",
          },
          {
            id: "tooling/allowed-none",
            kind: "command",
            scope: "tooling",
            command: ["node", "--version"],
            ciTier: "none",
            tags: ["ci-unreachable-allowed"],
            ciReason: "Synthetic fixture for allowed none-tier handling.",
          },
        ],
      }),
    );

    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: "declared-gate-missing-ci-tier",
          message: 'Declared gate "tooling/missing-tier" must set ciTier explicitly.',
        }),
        expect.objectContaining({
          code: "ci-tier-none-not-allowed",
          message:
            'Declared gate "tooling/not-allowed-none" uses ciTier "none" without the ci-unreachable-allowed tag.',
        }),
      ]),
    );
    expect(diagnostics).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          message: expect.stringContaining("tooling/allowed-none"),
        }),
      ]),
    );
  });

  it("reports non-exact or skewed OXC tool pins", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, "packages/oxlint-plugin"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: { "omena-check": "node ./check.js" },
          devDependencies: {
            oxfmt: "^0.54.0",
            oxlint: "^1.69.0",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(
      path.join(root, "packages/oxlint-plugin/package.json"),
      JSON.stringify(
        {
          name: "@omena/oxlint-plugin",
          peerDependencies: {
            oxlint: "^1.60.0",
          },
        },
        null,
        2,
      ),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "tool-pin-not-exact",
          message: 'package.json devDependencies.oxfmt must be exact-pinned, got "^0.54.0".',
        }),
        expect.objectContaining({
          severity: "error",
          code: "tool-pin-version-skew",
          message:
            "oxlint must use one exact version across package manifests: package.json=^1.69.0, packages/oxlint-plugin/package.json=^1.60.0.",
        }),
      ]),
    );
  });

  it("rejects VS Code types newer than the minimum supported editor", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: { "omena-check": "node ./check.js" },
          devDependencies: { "@types/vscode": "~1.125.0" },
          engines: { vscode: "^1.115.0" },
        },
        null,
        2,
      ),
    );

    expect(runDoctor(loadCheckManifest(root))).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "vscode-types-engine-skew",
          message:
            "package.json devDependencies.@types/vscode (~1.125.0) exceeds the engines.vscode minimum (^1.115.0); align the types with the minimum supported editor or deliberately raise the engine floor.",
        }),
      ]),
    );
  });

  it("keeps selected-query consumer coverage on Rust graph host and provider surfaces", () => {
    const selectedQueryConsumers = resolveGateTarget(manifest, "rust/selected-query/consumers");

    expect(selectedQueryConsumers?.command).toContain(
      "test/unit/runtime/style-semantic-graph-query-backend.test.ts",
    );
    expect(selectedQueryConsumers?.command).toContain(
      "test/unit/providers/scss-diagnostics.test.ts",
    );
  });

  it("renders a deterministic check inventory", () => {
    const inventory = renderCheckInventory(manifest);
    expect(inventory).toContain("# Check Inventory");
    expect(inventory).toContain("Generated by `pnpm omena-check inventory --write`");
    expect(inventory).toMatch(/\| Scope\s+\| Gates \| Bundles \| Aliases \| Commands \|/);
    expect(inventory).toMatch(
      /\| ID\s+\| Kind\s+\| Origin\s+\| Script\s+\| References\s+\| Status\s+\|/,
    );
    expect(inventory).toMatch(
      /\| `tsgo\/release\/bundle`\s+\| alias\s+\| package\s+\| `check:tsgo-release-bundle`\s+\|/,
    );
    expect(inventory).toMatch(
      /\| `rust\/release\/bundle`\s+\| bundle\s+\| declared\s+\| `check:rust-release-bundle`\s+\|/,
    );
    expect(inventory).toMatch(
      /\| `rust\/lane\/bundle`\s+\| bundle\s+\| declared\s+\| `check:rust-lane-bundle`\s+\|/,
    );
    expect(inventory).toMatch(
      /\| `rust\/omena-css\/h1-readiness`\s+\| bundle\s+\| declared\s+\| `check:rust-omena-css-h1-readiness`\s+\|/,
    );
    expect(inventory).toMatch(
      /\| `release\/release\/verify`\s+\| bundle\s+\| declared\s+\| `release:verify`\s+\|/,
    );
    expect(inventory).toMatch(
      /\| `release\/sync-server-version`\s+\| command\s+\| declared\s+\| `@declared\/release\/sync-server-version`\s+\|/,
    );
  });

  it("builds a readable nested plan for aggregate gates", () => {
    const target = resolveGateTarget(manifest, "release/release/verify");
    expect(target).toBeTruthy();

    const plan = buildCheckPlan(manifest, target!);
    expect(plan.steps[0]?.scriptName).toBe("release:verify");
    expect(plan.steps).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          depth: 1,
          scriptName: "@declared/release/sync-server-version",
        }),
        expect.objectContaining({
          depth: 1,
          scriptName: "check:rust-release-bundle",
        }),
        expect.objectContaining({
          depth: 1,
          scriptName: "check:tsgo-release-bundle",
        }),
        expect.objectContaining({
          scriptName: "build",
        }),
      ]),
    );

    const rendered = renderCheckPlan(plan);
    expect(rendered).toContain("Check plan: release/release/verify (release:verify)");
    expect(rendered).toContain("- release/release/verify (release:verify, bundle)");
    expect(rendered).toContain("  - release/sync-server-version");
    expect(rendered).toContain("  - rust/release/bundle (check:rust-release-bundle, bundle)");
  });

  it("reports aggregate surface size for cleanup planning", () => {
    const report = buildCheckSurfaceReport(manifest);
    expect(report.totalGates).toBeGreaterThan(150);
    expect(report.aliasChains).toEqual([]);
    expect(report.largestBundles[0]).toEqual(
      expect.objectContaining({
        id: "release/release/verify",
        scriptName: "release:verify",
      }),
    );

    const rendered = renderCheckSurfaceReport(report);
    expect(rendered).toContain("Check surface");
    expect(rendered).toContain("Alias chains: 0");
    expect(rendered).toContain("- release/release/verify");
  });

  it("reports workflow direct script calls that bypass omena-check", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
            check: "echo check",
            test: "echo test",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(
      path.join(root, ".github/workflows/ci.yml"),
      [
        "name: CI",
        "jobs:",
        "  direct:",
        "    # omena-ci-tier: verify",
        "    steps:",
        "      - run: pnpm check",
        "      - run: node --import tsx ./scripts/unregistered-check.ts",
        "      - run: pnpm omena-check run test/test",
      ].join("\n"),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "workflow-direct-script-call",
          message: expect.stringContaining(
            '.github/workflows/ci.yml:6 calls "check" directly; use "pnpm omena-check run core/check".',
          ),
        }),
        expect.objectContaining({
          severity: "error",
          code: "workflow-direct-node-script-call",
          message: expect.stringContaining(
            '.github/workflows/ci.yml:7 calls "scripts/unregistered-check.ts" through node directly',
          ),
        }),
      ]),
    );
  });

  it("requires scheduled workflows to declare a failure escalation path", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify({ name: "omena-css", scripts: { "omena-check": "node ./check.js" } }, null, 2),
    );
    writeFileSync(
      path.join(root, ".github/workflows/nightly.yml"),
      [
        "name: Nightly",
        "on:",
        "  schedule:",
        '    - cron: "0 0 * * *"',
        "permissions:",
        "  contents: read",
        "jobs:",
        "  nightly:",
        "    runs-on: ubuntu-latest",
        "    steps:",
        "      - run: echo nightly",
      ].join("\n"),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "scheduled-workflow-missing-issue-permission",
        }),
        expect.objectContaining({
          severity: "error",
          code: "scheduled-workflow-missing-failure-condition",
        }),
        expect.objectContaining({
          severity: "error",
          code: "scheduled-workflow-missing-failure-escalation",
        }),
      ]),
    );
  });

  it("accepts scheduled workflows with deduplicated issue escalation", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify({ name: "omena-css", scripts: { "omena-check": "node ./check.js" } }, null, 2),
    );
    writeFileSync(
      path.join(root, ".github/workflows/nightly.yml"),
      [
        "name: Nightly",
        "on:",
        "  schedule:",
        '    - cron: "0 0 * * *"',
        "permissions:",
        "  contents: read",
        "  issues: write",
        "jobs:",
        "  nightly:",
        "    runs-on: ubuntu-latest",
        "    steps:",
        "      - run: echo nightly",
        "      - name: Escalate scheduled failure",
        "        if: ${{ failure() }}",
        "        uses: ./.github/actions/escalate-ci-failure",
      ].join("\n"),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          code: expect.stringMatching(/^scheduled-workflow-/),
        }),
      ]),
    );
  });

  it("reports invalid omena-check targets before CI reaches runtime", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
            check: "echo check",
            "release:verify": "pnpm omena-check bundle check",
            test: "echo test",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(
      path.join(root, ".github/workflows/ci.yml"),
      [
        "name: CI",
        "jobs:",
        "  invalid:",
        "    steps:",
        "      - run: pnpm omena-check run missing-target",
      ].join("\n"),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "non-bundle-omena-check-target",
          message: 'Script "release:verify" uses omena-check bundle for non-bundle target "check".',
        }),
        expect.objectContaining({
          severity: "error",
          code: "workflow-unknown-omena-check-target",
          message: expect.stringContaining(
            '.github/workflows/ci.yml:5 references unknown omena-check target "missing-target".',
          ),
        }),
      ]),
    );
  });

  it("warns on alias chains so public check surfaces stay flat", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
            "check:rust-checker-bounded-lanes": "echo checker",
            "check:rust-checker-entrance": "pnpm omena-check run rust/checker/bounded-lanes",
            "check:rust-parser-index-producer": "pnpm omena-check run rust/checker/entrance",
          },
        },
        null,
        2,
      ),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "warning",
          code: "alias-chain",
          message:
            'Alias "check:rust-parser-index-producer" references alias "check:rust-checker-entrance"; point to "check:rust-checker-bounded-lanes" directly or keep only one public alias.',
        }),
      ]),
    );
  });

  it("reports non-canonical omena-check targets in checked surfaces", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-check-orchestrator-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "omena-css",
          scripts: {
            "omena-check": "node ./check.js",
            "release:verify": "pnpm omena-check run test",
            test: "echo test",
          },
        },
        null,
        2,
      ),
    );
    writeFileSync(
      path.join(root, ".github/workflows/ci.yml"),
      [
        "name: CI",
        "jobs:",
        "  invalid:",
        "    steps:",
        "      - run: pnpm omena-check run test",
      ].join("\n"),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "non-canonical-omena-check-target",
          message:
            'Script "release:verify" references omena-check target "test"; use canonical gate id "test/test".',
        }),
        expect.objectContaining({
          severity: "error",
          code: "workflow-non-canonical-omena-check-target",
          message: expect.stringContaining(
            '.github/workflows/ci.yml:5 references omena-check target "test"; use canonical gate id "test/test".',
          ),
        }),
      ]),
    );
  });
});

function testPackageGate({
  id,
  scriptName,
}: {
  readonly id: string;
  readonly scriptName: string;
}): CheckGate {
  return {
    id,
    scriptName,
    command: "echo ok",
    scope: "rust",
    kind: "gate",
    origin: "package",
    referencedScripts: [],
  };
}
