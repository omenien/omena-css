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

  it("STRUCTURE RED-PROOF: deleting rust-product-test-plan REDs (the fixture g131-S3 demands)", () => {
    const jobs = realJobs().filter((job) => job.name !== "rust-product-test-plan");
    expect(findProductTestCiStructureErrors(jobs, EXPECTED).join(";")).toContain(
      'registry must declare job "rust-product-test-plan"',
    );
  });

  it("STRUCTURE RED-PROOF: a needs restructure that drops a product-test result from ci-required REDs", () => {
    // Sever the crates job from its aggregator without touching anything else.
    const jobs = realJobs().map((job) =>
      job.name === "rust-product-tests"
        ? { ...job, needs: job.needs.filter((need) => need !== "rust-product-test-crates") }
        : job,
    );
    expect(findProductTestCiStructureErrors(jobs, EXPECTED).join(";")).toContain(
      'job "rust-product-test-crates" no longer reaches ci-required',
    );
  });

  it("STRUCTURE EQUIVALENCE: deleting the intermediate aggregator while wiring the three jobs directly into ci-required stays GREEN (the legal g131-S3 restructure)", () => {
    const jobs = realJobs()
      .filter((job) => job.name !== "rust-product-tests")
      .map((job) =>
        job.name === "ci-required"
          ? {
              ...job,
              needs: [
                ...job.needs.filter((need) => need !== "rust-product-tests"),
                "rust-product-test-plan",
                "rust-product-test-crates",
                "rust-product-test-contracts",
              ],
            }
          : job,
      );
    expect(findProductTestCiStructureErrors(jobs, EXPECTED)).toEqual([]);
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
