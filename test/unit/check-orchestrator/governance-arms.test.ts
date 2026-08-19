import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  loadCheckManifest,
  renderCheckInventory,
} from "../../../packages/check-orchestrator/src/manifest/index";
import { findCiRequiredAggregationDiagnostics } from "../../../packages/check-orchestrator/src/manifest/workflows";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";

const repoRoot = path.resolve(__dirname, "../../..");

describe("g130 S4/S5 governance arms", () => {
  it("S4 DIFF-SIZE ARM: adding one gate changes at most 3 inventory lines", () => {
    const manifest = loadCheckManifest(repoRoot);
    const before = renderCheckInventory(manifest).split("\n");
    const synthetic = {
      ...manifest,
      gates: [
        ...manifest.gates,
        {
          ...manifest.gates.find((gate) => gate.id === "docs/site")!,
          id: "docs/zz-synthetic-diff-probe",
          scriptName: "check:docs-zz-synthetic-diff-probe",
        },
      ],
    };
    const after = renderCheckInventory(synthetic).split("\n");
    const beforeSet = new Map<string, number>();
    for (const line of before) beforeSet.set(line, (beforeSet.get(line) ?? 0) + 1);
    let changed = 0;
    for (const line of after) {
      const count = beforeSet.get(line) ?? 0;
      if (count > 0) beforeSet.set(line, count - 1);
      else changed += 1;
    }
    expect(changed).toBeLessThanOrEqual(3);
  });

  it("S4 RED-PROOF: dep args on a dependencies-executor bundle and CLI-level flags are load-time errors", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-dep-args-"));
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify({
        name: "omena-css",
        scripts: {
          "omena-check": "node ./check.js",
          "check:docs-site": "node ./scripts/a.ts",
          "check:docs-smoke": "node ./scripts/b.ts",
        },
      }),
    );
    const manifest = loadCheckManifest(root, {
      declaredGates: [
        {
          id: "docs/pack",
          kind: "bundle",
          scope: "docs",
          deps: ["docs/site", "docs/smoke"],
          ciTier: "manual",
          ciReason: "fixture",
        },
        {
          id: "docs/bad-args",
          kind: "bundle",
          scope: "docs",
          deps: [{ target: "docs/pack", args: ["--json"] }],
          ciTier: "manual",
          ciReason: "fixture",
        },
        {
          id: "docs/bad-shard",
          kind: "bundle",
          scope: "docs",
          deps: [{ target: "docs/site", args: ["--shard=ghost", "--summary"] }],
          ciTier: "manual",
          ciReason: "fixture",
        },
        {
          id: "docs/good-args",
          kind: "bundle",
          scope: "docs",
          deps: [{ target: "docs/site", args: ["--variant", "x"] }],
          ciTier: "manual",
          ciReason: "fixture",
        },
      ],
    });
    const codes = manifest.diagnostics.map((diagnostic) => diagnostic.code);
    expect(codes).toContain("declared-dep-args-not-forwardable");
    expect(codes).toContain("declared-dep-cli-level-args");
    expect(
      manifest.diagnostics.filter(
        (diagnostic) =>
          diagnostic.code === "declared-dep-args-not-forwardable" &&
          diagnostic.message.includes("docs/good-args"),
      ),
    ).toEqual([]);
  });

  it("S5 RED-PROOF (isolated from the drift gate): stripping every required annotation is now ci-required-model-missing, not silence", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-required-model-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    const annotated = [
      "name: CI",
      "jobs:",
      "  leaf:",
      "    # omena-ci-required: true",
      "    runs-on: ubuntu-latest",
      "  ci-required:",
      "    # omena-ci-required: false",
      "    needs:",
      "      - leaf",
      "    if: ${{ always() }}",
      "    runs-on: ubuntu-latest",
      "    steps:",
      "      - run: node ./scripts/check-ci-required-results.mjs",
    ].join("\n");
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), annotated);
    expect(
      findCiRequiredAggregationDiagnostics(root).filter(
        (diagnostic) => diagnostic.code === "ci-required-model-missing",
      ),
    ).toEqual([]);

    const stripped = annotated
      .split("\n")
      .filter((line) => !line.includes("omena-ci-required"))
      .join("\n");
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), stripped);
    const diagnostics = findCiRequiredAggregationDiagnostics(root);
    expect(diagnostics).toEqual([
      expect.objectContaining({ severity: "error", code: "ci-required-model-missing" }),
    ]);
  });

  it("S5 DIAGNOSTIC-CODE CENSUS: every code literal in src is in the committed inventory and vice versa", () => {
    const src = path.join(repoRoot, "packages/check-orchestrator/src");
    const found = new Set<string>();
    const walk = (dir: string): void => {
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) walk(full);
        else if (entry.name.endsWith(".ts")) {
          for (const match of readFileSync(full, "utf8").matchAll(/code: "([a-z0-9-]+)"/g)) {
            found.add(match[1] ?? "");
          }
        }
      }
    };
    walk(src);
    const committed = (
      JSON.parse(
        readFileSync(
          path.join(repoRoot, "packages/check-orchestrator/diagnostic-codes.json"),
          "utf8",
        ),
      ) as { codes: string[] }
    ).codes;
    expect([...found].toSorted()).toEqual(committed);
  });
});
