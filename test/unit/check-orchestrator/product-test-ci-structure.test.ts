import path from "node:path";
import { describe, expect, it } from "vitest";
import { loadCiWorkflowRegistry } from "../../../packages/check-orchestrator/src/manifest/ci-workflow";
import { bundleShardNames } from "../../../packages/check-orchestrator/src/manifest/shards";
import {
  findProductTestCiStructureErrors,
  type CiStructureJobView,
} from "../../../scripts/lib/product-test-ci-structure";
import { RUST_PRODUCT_TEST_SHARDS } from "../../../scripts/lib/rust-product-test-plan";

const repoRoot = path.resolve(__dirname, "../../..");

const EXPECTED = {
  crateShardIds: RUST_PRODUCT_TEST_SHARDS.map(({ id }) => id),
  contractShardNames: [...bundleShardNames("rust/product-test-contracts")],
};

function realJobs(): CiStructureJobView[] {
  const registry = loadCiWorkflowRegistry(repoRoot);
  if (!registry) throw new Error("ci-workflow.json missing");
  return registry.jobs.map((job) => ({ name: job.name, block: job.block, needs: job.needs }));
}

describe("product-test CI structure (registry-anchored classguard, g131-S0)", () => {
  it("is quiet on the real registry", () => {
    expect(findProductTestCiStructureErrors(realJobs(), EXPECTED)).toEqual([]);
  });

  it("STRUCTURE RED-PROOF: removing the classguard from every job REDs (home-agnostic duty, post-S3)", () => {
    const jobs = realJobs().map((job) => ({
      ...job,
      block: job.block.filter((line) => !line.includes("rust/product-test-coverage-classguard")),
    }));
    expect(findProductTestCiStructureErrors(jobs, EXPECTED).join(";")).toContain(
      "no CI job executes the product-test classguard",
    );
  });

  it("STRUCTURE RED-PROOF: a classguard home that does not reach ci-required REDs", () => {
    // Move the classguard line to an advisory job outside the required graph.
    const jobs = realJobs().map((job) => {
      if (job.name === "rust-product-test-crates") {
        return {
          ...job,
          block: job.block.filter(
            (line) => !line.includes("rust/product-test-coverage-classguard"),
          ),
        };
      }
      if (job.name === "resolver-path-identity-advisory") {
        return {
          ...job,
          block: [
            ...job.block,
            "      - run: pnpm omena-check run rust/product-test-coverage-classguard --summary",
          ],
        };
      }
      return job;
    });
    expect(findProductTestCiStructureErrors(jobs, EXPECTED).join(";")).toContain(
      "does not reach ci-required",
    );
  });

  it("STRUCTURE RED-PROOF: a needs restructure that drops a product-test result from ci-required REDs", () => {
    // Sever the crates job from ci-required (post-S3: it is a DIRECT need).
    const jobs = realJobs().map((job) =>
      job.name === "ci-required"
        ? { ...job, needs: job.needs.filter((need) => need !== "rust-product-test-crates") }
        : job,
    );
    expect(findProductTestCiStructureErrors(jobs, EXPECTED).join(";")).toContain(
      'job "rust-product-test-crates" no longer reaches ci-required',
    );
  });

  it("STRUCTURE EQUIVALENCE: re-introducing an intermediate aggregator between the product-test jobs and ci-required stays GREEN (aggregation-shape agnosticism, both directions)", () => {
    const real = realJobs();
    const ciRequired = real.find((job) => job.name === "ci-required")!;
    const jobs = [
      ...real.map((job) =>
        job.name === "ci-required"
          ? {
              ...job,
              needs: [
                ...job.needs.filter(
                  (need) =>
                    need !== "rust-product-test-crates" && need !== "rust-product-test-contracts",
                ),
                "rust-product-tests",
              ],
            }
          : job,
      ),
      {
        name: "rust-product-tests",
        block: ["  rust-product-tests:", "    runs-on: ubuntu-latest"],
        needs: ["rust-product-test-crates", "rust-product-test-contracts"],
      },
    ];
    void ciRequired;
    expect(findProductTestCiStructureErrors(jobs, EXPECTED)).toEqual([]);
  });

  it("STEP-SHAPE ARMS (stage-5 R5): the classguard duty accepts only EXECUTING steps", () => {
    const CG = "pnpm omena-check run rust/product-test-coverage-classguard --summary";
    const stripped = realJobs().map((job) => ({
      ...job,
      block: job.block.filter((line) => !line.includes("product-test-coverage-classguard")),
    }));
    const withExtra = (extra: readonly string[]) =>
      stripped.map((job) =>
        job.name === "rust-product-test-crates" ? { ...job, block: [...job.block, ...extra] } : job,
      );
    const unenforced = (jobs: ReturnType<typeof realJobs>) =>
      findProductTestCiStructureErrors(jobs, EXPECTED)
        .join(";")
        .includes("no CI job executes the product-test classguard");
    // Executing shapes SATISFY:
    expect(unenforced(withExtra([`      - run: ${CG}`]))).toBe(false);
    expect(unenforced(withExtra(["      - name: classguard", `        run: ${CG}`]))).toBe(false);
    expect(unenforced(withExtra(["      - id: cg", `        run: ${CG}`]))).toBe(false);
    // Non-executing shapes DO NOT:
    expect(unenforced(withExtra([`      # DISABLED - run: ${CG}`]))).toBe(true);
    expect(
      unenforced(
        withExtra(["      - uses: some/action@abc", "        with:", `          run: ${CG}`]),
      ),
    ).toBe(true);
    expect(unenforced(withExtra([`        run: ${CG}`]))).toBe(true);
  });

  it("CONTENT RED-PROOF: dropping a matrix leg or the pinned installer still REDs after the migration", () => {
    const noLeg = realJobs().map((job) =>
      job.name === "rust-product-test-crates"
        ? {
            ...job,
            block: job.block.map((line) =>
              /product-shard:\s*\[/u.test(line) ? line.replace(/,\s*benchmarks/u, "") : line,
            ),
          }
        : job,
    );
    expect(findProductTestCiStructureErrors(noLeg, EXPECTED).join(";")).toContain(
      'job "rust-product-test-crates" matrix',
    );

    const noInstaller = realJobs().map((job) =>
      job.name === "rust-product-test-contracts"
        ? {
            ...job,
            block: job.block.filter((line) => !line.includes("taiki-e/install-action")),
          }
        : job,
    );
    expect(findProductTestCiStructureErrors(noInstaller, EXPECTED).join(";")).toContain(
      "pinned prebuilt tool installer",
    );
  });
});
