import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { loadCheckManifest } from "../../../packages/check-orchestrator/src/manifest/index";
import {
  classifyWorkflowCadences,
  computeGateLifecycles,
  findGateLifecycleDiagnostics,
} from "../../../packages/check-orchestrator/src/manifest/lifecycle";
import { collectWorkflowLifecycleView } from "../../../packages/check-orchestrator/src/manifest/workflows";

function fixtureRoot(workflows: Record<string, string>, scripts: Record<string, string>): string {
  const root = mkdtempSync(path.join(os.tmpdir(), "omena-lifecycle-"));
  writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({ name: "omena-css", scripts }, null, 2),
  );
  mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
  for (const [name, text] of Object.entries(workflows)) {
    writeFileSync(path.join(root, ".github/workflows", name), text);
  }
  return root;
}

const CI_YML = `name: CI
on:
  push:
    branches: [master]
jobs:
  leaf:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run docs/site
  aggregate:
    needs: leaf
    runs-on: ubuntu-latest
    steps:
      - run: node ./scripts/check-ci-required-results.mjs
  side:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run docs/smoke
  ci-required:
    needs:
      - aggregate
    runs-on: ubuntu-latest
    steps:
      - run: node ./scripts/check-ci-required-results.mjs
`;

const NIGHTLY_YML = `name: Nightly
on:
  schedule:
    - cron: "0 8 * * *"
jobs:
  soak:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run docs/soak-only
`;

const SCRIPTS = {
  "omena-check": "node ./check.js",
  "check:docs-site": "node ./scripts/check-docs-site.ts",
  "check:docs-smoke": "node ./scripts/check-docs-smoke.ts",
  "check:docs-soak-only": "node ./scripts/check-docs-soak.ts",
};

describe("gate lifecycle derivation (g130-S1)", () => {
  it("derives blocking through the ci-required needs-ancestor closure and advisory outside it", () => {
    const root = fixtureRoot({ "ci.yml": CI_YML, "nightly.yml": NIGHTLY_YML }, SCRIPTS);
    const manifest = loadCheckManifest(root, { declaredGates: [] });
    const lifecycles = computeGateLifecycles(root, manifest.gates).byGateId;

    expect(lifecycles.get("docs/site")).toMatchObject({ cadence: "push", strength: "blocking" });
    // `side` never reaches ci-required: push cadence, advisory strength.
    expect(lifecycles.get("docs/smoke")).toMatchObject({ cadence: "push", strength: "advisory" });
    // scheduled-only gate: nightly + advisory.
    expect(lifecycles.get("docs/soak-only")).toMatchObject({
      cadence: "nightly",
      strength: "advisory",
    });
  });

  it("RED-PROOF: removing the leaf from its aggregator's needs flips its gates to advisory", () => {
    const mutated = CI_YML.replace("  aggregate:\n    needs: leaf\n", "  aggregate:\n");
    const root = fixtureRoot({ "ci.yml": mutated }, SCRIPTS);
    const manifest = loadCheckManifest(root, { declaredGates: [] });
    const lifecycles = computeGateLifecycles(root, manifest.gates).byGateId;
    expect(lifecycles.get("docs/site")?.strength).toBe("advisory");
  });

  it("derivation is total: unreachable gates are manual/advisory and every gate has axes", () => {
    const root = fixtureRoot({ "ci.yml": CI_YML }, SCRIPTS);
    const manifest = loadCheckManifest(root, { declaredGates: [] });
    const lifecycles = computeGateLifecycles(root, manifest.gates).byGateId;
    expect(lifecycles.get("docs/soak-only")).toMatchObject({
      cadence: "manual",
      strength: "advisory",
    });
    for (const gate of manifest.gates) {
      expect(lifecycles.get(gate.id)).toBeDefined();
    }
  });

  it("RED-PROOF: a scheduled-only gate declared blocking without axisException is a mismatch error", () => {
    const root = fixtureRoot({ "ci.yml": CI_YML, "nightly.yml": NIGHTLY_YML }, SCRIPTS);
    const manifest = loadCheckManifest(root, {
      declaredGates: [
        {
          id: "docs/soak-only",
          kind: "gate",
          scope: "docs",
          packageTarget: "check:docs-soak-only",
          ciTier: "scheduled",
          strength: "blocking",
        },
      ],
    });
    expect(manifest.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          severity: "error",
          code: "gate-axis-override-mismatch",
          message: expect.stringContaining('"docs/soak-only" declares strength "blocking"'),
        }),
      ]),
    );

    const excused = loadCheckManifest(root, {
      declaredGates: [
        {
          id: "docs/soak-only",
          kind: "gate",
          scope: "docs",
          packageTarget: "check:docs-soak-only",
          ciTier: "scheduled",
          strength: "blocking",
          axisException: "promotion staged for the next push lane rewire",
        },
      ],
    });
    expect(excused.diagnostics).not.toEqual(
      expect.arrayContaining([expect.objectContaining({ code: "gate-axis-override-mismatch" })]),
    );
    expect(excused.lifecycleByGateId.get("docs/soak-only")).toMatchObject({
      strength: "blocking",
      strengthSource: "declared-override",
    });
  });

  it("RED-PROOF: axisException without any declared axis is an error", () => {
    const root = fixtureRoot({ "ci.yml": CI_YML }, SCRIPTS);
    const manifest = loadCheckManifest(root, {
      declaredGates: [
        {
          id: "docs/site",
          kind: "gate",
          scope: "docs",
          packageTarget: "check:docs-site",
          ciTier: "verify",
          axisException: "unused",
        },
      ],
    });
    expect(manifest.diagnostics).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: "gate-axis-exception-unused" })]),
    );
  });

  it("cadence takes the fastest reaching workflow and workflow_call files inherit callers", () => {
    const CALLER = `name: CI
on:
  push:
    branches: [master]
jobs:
  matrix:
    uses: ./.github/workflows/_reusable.yml
  ci-required:
    needs:
      - matrix
    runs-on: ubuntu-latest
    steps:
      - run: node ./scripts/check-ci-required-results.mjs
`;
    const REUSABLE = `name: Reusable
on:
  workflow_call:
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run docs/site
`;
    const root = fixtureRoot(
      { "ci.yml": CALLER, "_reusable.yml": REUSABLE, "nightly.yml": NIGHTLY_YML },
      SCRIPTS,
    );
    const manifest = loadCheckManifest(root, { declaredGates: [] });
    const view = collectWorkflowLifecycleView(root, manifest.gates);
    const cadences = classifyWorkflowCadences(view.triggers);
    expect(cadences.get("_reusable.yml")).toBe("push");
    const lifecycles = computeGateLifecycles(root, manifest.gates).byGateId;
    expect(lifecycles.get("docs/site")?.cadence).toBe("push");
  });

  it("findGateLifecycleDiagnostics is quiet on the real repository", () => {
    const repoRoot = path.resolve(__dirname, "../../..");
    const manifest = loadCheckManifest(repoRoot);
    expect(findGateLifecycleDiagnostics(repoRoot, manifest.gates)).toEqual([]);
  });
});
