import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { loadCiWorkflowRegistry } from "../../../packages/check-orchestrator/src/manifest/ci-workflow";

const repoRoot = path.resolve(__dirname, "../../..");

// g131-S3: the consolidation's two mandatory arms.
//
// ORDERING ARM — the verify barrier was replaced by direct leaf needs; the
// failure matrix alone cannot catch a mis-rewire that lets package or
// extension-host-smoke START before a former verify leaf finished. The
// golden below asserts the transitive needs closure of each former consumer
// still contains EVERY former verify leaf (verify-docs folded into
// verify-core, so the post-merge leaf set is the golden).
const FORMER_VERIFY_LEAVES = ["verify-core", "verify-native-linux", "build-output"] as const;

function needsClosure(
  jobs: readonly { name: string; needs: readonly string[] }[],
  root: string,
): Set<string> {
  const byName = new Map(jobs.map((job) => [job.name, job.needs]));
  const closure = new Set<string>();
  const queue = [...(byName.get(root) ?? [])];
  while (queue.length > 0) {
    const name = queue.pop();
    if (!name || closure.has(name)) continue;
    closure.add(name);
    queue.push(...(byName.get(name) ?? []));
  }
  return closure;
}

describe("consolidated CI graph (g131-S3)", () => {
  const registry = loadCiWorkflowRegistry(repoRoot);
  if (!registry) throw new Error("ci-workflow.json missing");
  const jobs = registry.jobs.map((job) => ({ name: job.name, needs: job.needs }));

  it("ORDERING ARM: package and extension-host-smoke transitively need every former verify leaf", () => {
    for (const consumer of ["package", "extension-host-smoke"]) {
      const closure = needsClosure(jobs, consumer);
      for (const leaf of FORMER_VERIFY_LEAVES) {
        expect(closure.has(leaf), `${consumer} must wait on ${leaf}`).toBe(true);
      }
    }
  });

  // The golden itself, extracted so the RED-proof exercises THE SAME
  // assertion the green case runs (stage-5 R2: the old arm re-derived the
  // closure by hand and never touched the golden — a legitimate re-route
  // would have failed the arm while the golden stayed correctly green).
  function orderingGoldenErrors(
    graph: readonly { name: string; needs: readonly string[] }[],
  ): string[] {
    const errors: string[] = [];
    for (const consumer of ["package", "extension-host-smoke"]) {
      const closure = needsClosure(graph, consumer);
      for (const leaf of FORMER_VERIFY_LEAVES) {
        if (!closure.has(leaf)) errors.push(`${consumer} no longer waits on ${leaf}`);
      }
    }
    return errors;
  }

  it("ORDERING ARM RED-PROOF: a mis-rewire fails the golden; an equivalent re-route stays green", () => {
    // Mis-rewire: drop a former leaf from package's needs -> golden RED.
    const misWired = jobs.map((job) =>
      job.name === "package"
        ? { ...job, needs: job.needs.filter((need) => need !== "verify-native-linux") }
        : job,
    );
    expect(orderingGoldenErrors(misWired)).toEqual([
      "package no longer waits on verify-native-linux",
    ]);
    // Equivalent re-route: package waits on extension-host-smoke which waits
    // on all three leaves -> transitively ordered, golden stays GREEN.
    const rerouted = jobs.map((job) =>
      job.name === "package"
        ? { ...job, needs: ["extension-host-smoke", "native-runner-matrix"] }
        : job,
    );
    expect(orderingGoldenErrors(rerouted)).toEqual([]);
  });

  it("REQUIRED MODEL: ci-required needs exactly the required-annotated jobs and every job reaches it or is advisory", () => {
    const ciRequired = registry.jobs.find((job) => job.name === "ci-required")!;
    const required = registry.jobs
      .filter((job) => job.requiredAnnotation === true)
      .map((job) => job.name)
      .toSorted();
    expect([...ciRequired.needs].toSorted()).toEqual(required);
    // The five former aggregator-only jobs are gone.
    for (const gone of [
      "verify",
      "rust-product-tests",
      "rust-workspace",
      "benchmark-gates",
      "closure-fast",
    ]) {
      expect(registry.jobs.some((job) => job.name === gone)).toBe(false);
    }
  });

  it("SCCACHE GUARD (g131-S4): the wrapper rides behind the OMENA_SCCACHE switch with a variable kill switch", () => {
    const ci = readFileSync(path.join(repoRoot, ".github/workflows/ci.yml"), "utf8");
    // Kill switch: a repo variable overrides the default without a commit.
    expect(ci).toContain("OMENA_SCCACHE: ${{ vars.OMENA_SCCACHE || 'on' }}");
    const cacheRust = readFileSync(
      path.join(repoRoot, ".github/actions/cache-rust/action.yml"),
      "utf8",
    );
    // Every sccache step is gated on the switch — flipping the variable to
    // "off" must leave ZERO ungated sccache surface in the action.
    const steps = cacheRust
      .split("\n    - ")
      .slice(1)
      .map((segment) =>
        segment
          .split("\n")
          .filter((line) => !line.trim().startsWith("#"))
          .join("\n"),
      );
    for (const step of steps) {
      // Gate the steps that USE sccache (comments naming it are not steps).
      if (!step.includes("sccache-action") && !step.includes("RUSTC_WRAPPER")) continue;
      expect(step, "sccache step must be gated on OMENA_SCCACHE").toContain(
        "if: ${{ env.OMENA_SCCACHE == 'on' }}",
      );
    }
    // The divergence lane exists, is weekly, and escalates on failure.
    const divergence = readFileSync(
      path.join(repoRoot, ".github/workflows/sccache-divergence.yml"),
      "utf8",
    );
    expect(divergence).toContain("cron:");
    expect(divergence).toContain("issues: write");
    expect(divergence).toContain("escalate-ci-failure");
    expect(divergence).toContain("diff /tmp/digest-sccache.txt /tmp/digest-plain.txt");
  });

  it(
    "LEAF-FAILURE MATRIX: the root judge REDs when ANY required leaf fails and stays green on all-success",
    { timeout: 60_000 },
    () => {
      const ciRequired = registry.jobs.find((job) => job.name === "ci-required")!;
      const judge = (results: Record<string, { result: string }>): number => {
        try {
          execFileSync("node", ["./scripts/check-ci-required-results.mjs"], {
            cwd: repoRoot,
            env: { ...process.env, OMENA_CI_REQUIRED_RESULTS: JSON.stringify(results) },
            stdio: "pipe",
          });
          return 0;
        } catch (error) {
          return (error as { status?: number }).status ?? 1;
        }
      };
      const allGreen = Object.fromEntries(
        ciRequired.needs.map((need) => [need, { result: "success" }]),
      );
      expect(judge(allGreen)).toBe(0);
      // Every leaf, in turn: failure AND skipped must both flip the judge RED.
      for (const need of ciRequired.needs) {
        for (const outcome of ["failure", "skipped"]) {
          const mutated = { ...allGreen, [need]: { result: outcome } };
          expect(judge(mutated), `${need}=${outcome} must RED the judge`).not.toBe(0);
        }
      }
    },
  );
});
