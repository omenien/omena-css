import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  adoptCiWorkflow,
  checkCiWorkflow,
  renderCiWorkflow,
  resolveCiWorkflowVerdict,
  validateCiWorkflowRegistry,
} from "../../../packages/check-orchestrator/src/manifest/ci-workflow";

const repoRoot = path.resolve(__dirname, "../../..");

const SYNTHETIC = `name: CI
on:
  push:
    branches: [master]
jobs:
  preflight:
    # omena-ci-tier: verify
    # omena-ci-required: false
    runs-on: ubuntu-latest
    steps:
      - id: shards
        run: echo "matrix=$(pnpm --silent omena-check shards rust/closure-fast --json)" >> "$GITHUB_OUTPUT"
  value-matrix:
    # omena-ci-required: false
    needs: preflight
    strategy:
      fail-fast: false
      matrix:
        product-shard: [workspace, differential, benchmarks]
    runs-on: ubuntu-latest
    steps:
      - run: pnpm omena-check run rust/product-test-execution --summary -- \${{ matrix.product-shard }}
  dynamic-matrix:
    # omena-ci-required: false
    needs: preflight
    strategy:
      matrix:
        shard: \${{ fromJSON(needs.preflight.outputs.closure-fast-shards) }}
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - if: \${{ matrix.shard == 'semantic-analysis' }}
        run: cargo build -p engine-shadow-runner
      - name: multi-line body
        run: |
          set -euo pipefail
          echo "verbatim body survives"
      - uses: actions/upload-artifact@abc123
        with:
          name: summary-\${{ matrix.shard }}
          path: .omena-ci/
  reusable:
    # omena-ci-required: true
    uses: ./.github/workflows/_reusable.yml
  aggregate:
    # omena-ci-required: true
    needs:
      - value-matrix
      - dynamic-matrix
    if: \${{ always() }}
    runs-on: ubuntu-latest
    steps:
      - run: node ./scripts/check-ci-required-results.mjs
        env:
          OMENA_CI_REQUIRED_RESULTS: \${{ toJson(needs) }}
  ci-required:
    # omena-ci-required: false
    needs:
      - reusable
      - aggregate
    if: \${{ always() }}
    runs-on: ubuntu-latest
    steps:
      - run: node ./scripts/check-ci-required-results.mjs
`;

describe("ci-workflow generator (g130-S3)", () => {
  it("NO-OP PROOF: adopt(write(committed ci.yml)) round-trips byte-identically on the real repository", () => {
    const committed = readFileSync(path.join(repoRoot, ".github/workflows/ci.yml"), "utf8");
    const registry = adoptCiWorkflow(committed);
    expect(renderCiWorkflow(registry)).toBe(committed);
    expect(validateCiWorkflowRegistry(registry).errors).toEqual([]);
    expect(checkCiWorkflow(repoRoot).ok).toBe(true);
  });

  it("SHAPE COVERAGE: the six job shapes and five attributes survive the round trip on a synthetic workflow", () => {
    const registry = adoptCiWorkflow(SYNTHETIC);
    expect(registry.jobs.map((job) => job.name)).toEqual([
      "preflight",
      "value-matrix",
      "dynamic-matrix",
      "reusable",
      "aggregate",
      "ci-required",
    ]);
    const rendered = renderCiWorkflow(registry);
    expect(rendered).toBe(SYNTHETIC);
    // Attributes preserved verbatim: value matrix, fromJSON indirection,
    // continue-on-error, conditional step, multi-line run body, per-leg
    // artifact name, aggregator always()+toJson(needs), reusable uses.
    for (const needle of [
      "product-shard: [workspace, differential, benchmarks]",
      "fromJSON(needs.preflight.outputs.closure-fast-shards)",
      "continue-on-error: true",
      "if: ${{ matrix.shard == 'semantic-analysis' }}",
      'echo "verbatim body survives"',
      "name: summary-${{ matrix.shard }}",
      "OMENA_CI_REQUIRED_RESULTS: ${{ toJson(needs) }}",
      "uses: ./.github/workflows/_reusable.yml",
    ]) {
      expect(rendered).toContain(needle);
    }
  });

  it("DELETION FIXTURE: removing one registry entry removes exactly that job block (template-hardcoding killer)", () => {
    const registry = adoptCiWorkflow(SYNTHETIC);
    const survivor = {
      ...registry,
      jobs: registry.jobs.filter((job) => job.name !== "dynamic-matrix"),
    };
    const rendered = renderCiWorkflow(survivor);
    expect(rendered).not.toContain("dynamic-matrix:");
    expect(rendered).not.toContain("verbatim body survives");
    expect(rendered).toContain("value-matrix:");
    expect(rendered).toContain("aggregate:");
    // The deleted block is exactly the byte difference.
    const deletedBlock = registry.jobs.find((job) => job.name === "dynamic-matrix");
    expect(SYNTHETIC.length - rendered.length).toBe(
      (deletedBlock?.block.join("\n").length ?? 0) + 1,
    );
  });

  it("VALIDATION: structured fields must agree with the verbatim block", () => {
    const registry = adoptCiWorkflow(SYNTHETIC);
    const lying = {
      ...registry,
      jobs: registry.jobs.map((job) =>
        job.name === "aggregate" ? { ...job, needs: ["value-matrix"] } : job,
      ),
    };
    expect(validateCiWorkflowRegistry(lying).errors.join(";")).toContain(
      'job "aggregate" needs disagree',
    );
  });

  it("VALIDATION: ci-required needs must equal the required-annotated job set", () => {
    const registry = adoptCiWorkflow(SYNTHETIC);
    const drifted = adoptCiWorkflow(
      SYNTHETIC.replace(
        "  reusable:\n    # omena-ci-required: true",
        "  reusable:\n    # omena-ci-required: false",
      ),
    );
    expect(validateCiWorkflowRegistry(registry).errors).toEqual([]);
    expect(validateCiWorkflowRegistry(drifted).errors.join(";")).toContain(
      "must equal the required-annotated jobs",
    );
  });

  it("RED-PROOF: drift on a scratch root REDs with the sanctioned-edit message; break-glass converts to a warning verdict", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-ci-workflow-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    mkdirSync(path.join(root, "packages/check-orchestrator"), { recursive: true });
    const registry = adoptCiWorkflow(SYNTHETIC);
    writeFileSync(
      path.join(root, "packages/check-orchestrator/ci-workflow.json"),
      JSON.stringify(registry, null, 2),
    );
    writeFileSync(
      path.join(root, ".github/workflows/ci.yml"),
      SYNTHETIC.replace("verbatim body survives", "hand edit"),
    );
    const outcome = checkCiWorkflow(root);
    expect(outcome.ok).toBe(false);
    expect(outcome.reason).toContain("hand edits to ci.yml are not sanctioned");
    expect(resolveCiWorkflowVerdict(outcome, undefined)).toBe("drift-error");
    expect(resolveCiWorkflowVerdict(outcome, "  ")).toBe("drift-error");
    expect(resolveCiWorkflowVerdict(outcome, "incident-2026-08-20")).toBe("override-warning");
    expect(resolveCiWorkflowVerdict({ ok: true }, undefined)).toBe("ok");
    // Registry-side incoherence is also drift, not silence.
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), renderCiWorkflow(registry));
    expect(checkCiWorkflow(root).ok).toBe(true);
  });
});
