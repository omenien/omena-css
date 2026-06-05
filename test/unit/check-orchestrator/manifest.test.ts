import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
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
    expect(releaseBundle?.referencedScripts).toContain("check:rust-workspace");
    expect(releaseBundle?.referencedScripts).toContain("check:rust-producer-boundary");

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
        "check:rust-omena-lsp-server-boundary",
        "check:rust-omena-lsp-server-shell",
        "check:rust-omena-lsp-server-provider-parity",
        "check:rust-omena-lsp-server-runtime-loop",
      ]),
    );

    const releaseVerify = resolveGateTarget(manifest, "release/release/verify");
    expect(releaseVerify?.kind).toBe("bundle");
    expect(releaseVerify?.referencedScripts).toEqual(
      expect.arrayContaining([
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
      /\| `rust\/release\/bundle`\s+\| bundle\s+\| package\s+\| `check:rust-release-bundle`\s+\|/,
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
        "    steps:",
        "      - run: pnpm check",
        "      - run: pnpm omena-check run test/test",
      ].join("\n"),
    );

    const diagnostics = runDoctor(loadCheckManifest(root));
    expect(diagnostics).toEqual([
      expect.objectContaining({
        severity: "error",
        code: "workflow-direct-script-call",
        message: expect.stringContaining(
          '.github/workflows/ci.yml:5 calls "check" directly; use "pnpm omena-check run core/check".',
        ),
      }),
    ]);
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
    expect(diagnostics).toEqual([
      expect.objectContaining({
        severity: "warning",
        code: "alias-chain",
        message:
          'Alias "check:rust-parser-index-producer" references alias "check:rust-checker-entrance"; point to "check:rust-checker-bounded-lanes" directly or keep only one public alias.',
      }),
    ]);
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
