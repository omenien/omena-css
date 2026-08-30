import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { bundleShardNames } from "../packages/check-orchestrator/src/manifest/shards";
import { loadCiWorkflowRegistry } from "../packages/check-orchestrator/src/manifest/ci-workflow";
import { resolveSummaryMemberArgs } from "../packages/check-orchestrator/src/cli/summary-args";
import { findProductTestCiStructureErrors } from "./lib/product-test-ci-structure";
import {
  RUST_PRODUCT_TEST_FEATURE_ISOLATION_PACKAGES,
  RUST_PRODUCT_TEST_SHARDS,
  rustProductTestCargoInvocations,
  rustProductTestPackagesByShard,
} from "./lib/rust-product-test-plan";

const PRODUCT_SCRIPT_NAME = "check:rust-product-test-execution";
const PRODUCT_SCRIPT_COMMAND = "node --import tsx ./scripts/run-rust-product-tests.ts";
const CONTRACT_BUNDLE_ID = "rust/product-test-contracts";
const root = process.cwd();
const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
  scripts?: Record<string, string>;
};
const productScript = packageJson.scripts?.[PRODUCT_SCRIPT_NAME];

assert.equal(
  productScript,
  PRODUCT_SCRIPT_COMMAND,
  `${PRODUCT_SCRIPT_NAME} must delegate to the shared sharded runner`,
);

const orchestratorMain = readFileSync("packages/check-orchestrator/src/cli/main.ts", "utf8");
assert.match(
  orchestratorMain,
  /const memberArgs = resolveSummaryMemberArgs\(gate, targetSpec, extraArgs\);/u,
  "summary execution must route member arguments through the tested resolver",
);
assert.deepEqual(
  resolveSummaryMemberArgs(
    { id: "rust/product-test-execution", kind: "gate" },
    { target: "rust/product-test-execution" },
    ["differential"],
  ),
  ["differential"],
  "a summarized leaf gate must receive its CLI shard argument",
);
assert.deepEqual(
  resolveSummaryMemberArgs(
    { id: "rust/product-tests", kind: "bundle" },
    { target: "rust/product-test-execution", args: ["workspace"] },
    ["differential"],
  ),
  ["workspace"],
  "a bundle must not leak its CLI arguments into ordinary dependencies",
);
assert.deepEqual(
  resolveSummaryMemberArgs(
    { id: "rust/product-tests-alias", kind: "alias" },
    { target: "rust/product-test-execution", args: ["workspace"] },
    ["differential"],
  ),
  ["workspace", "differential"],
  "an alias must continue forwarding its CLI arguments to dependencies",
);

interface CargoMetadata {
  readonly packages: readonly {
    readonly id: string;
    readonly name: string;
  }[];
  readonly workspace_members: readonly string[];
}

const metadata = JSON.parse(
  execFileSync(
    "cargo",
    ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
    {
      cwd: root,
      encoding: "utf8",
    },
  ),
) as CargoMetadata;
const workspaceMemberIds = new Set(metadata.workspace_members);
const workspaceCrates = metadata.packages
  .filter((rustPackage) => workspaceMemberIds.has(rustPackage.id))
  .map((rustPackage) => rustPackage.name)
  .toSorted();

assert.ok(workspaceCrates.length > 0, "Cargo metadata must expose workspace members");
assert.equal(
  workspaceCrates.length,
  workspaceMemberIds.size,
  "every Cargo workspace member must resolve to package metadata",
);

const packagesByShard = rustProductTestPackagesByShard(workspaceCrates);
const packageOwners = new Map<string, string[]>();
for (const shard of RUST_PRODUCT_TEST_SHARDS) {
  const shardPackages = packagesByShard.get(shard.id) ?? [];
  assert.ok(shardPackages.length > 0, `Rust product-test shard "${shard.id}" must not be empty`);

  const cargoInvocations = rustProductTestCargoInvocations(shard);
  assert.ok(cargoInvocations.length > 0, `${shard.id} must have a Cargo invocation`);
  for (const invocation of cargoInvocations) {
    assert.ok(
      invocation.args.includes("--all-features"),
      `${shard.id}/${invocation.id} must exercise all selected package features`,
    );
    assert.ok(
      invocation.args.includes("--no-fail-fast"),
      `${shard.id}/${invocation.id} must preserve failure aggregation`,
    );
  }
  const workspaceInvocationCount = cargoInvocations.filter((invocation) =>
    invocation.args.includes("--workspace"),
  ).length;
  assert.equal(
    workspaceInvocationCount,
    shard.workspaceRemainder ? 1 : 0,
    `${shard.id} workspace selection must match its declared mode`,
  );

  for (const packageName of shardPackages) {
    const owners = packageOwners.get(packageName) ?? [];
    owners.push(shard.id);
    packageOwners.set(packageName, owners);
  }
}

const workspaceShard = RUST_PRODUCT_TEST_SHARDS.find((shard) => shard.workspaceRemainder);
assert.ok(workspaceShard, "exactly one product-test shard must own the workspace remainder");
const workspaceInvocations = rustProductTestCargoInvocations(workspaceShard);
const workspaceRemainderInvocation = workspaceInvocations.find((invocation) =>
  invocation.args.includes("--workspace"),
);
assert.ok(workspaceRemainderInvocation, "the workspace remainder invocation must exist");
for (const packageName of RUST_PRODUCT_TEST_FEATURE_ISOLATION_PACKAGES) {
  assert.ok(
    workspaceRemainderInvocation.args.some(
      (value, index) =>
        value === "--exclude" && workspaceRemainderInvocation.args[index + 1] === packageName,
    ),
    `${packageName} must be excluded from workspace-wide feature unification`,
  );
  assert.ok(
    workspaceInvocations.some((invocation) =>
      invocation.args.some(
        (value, index) => value === "-p" && invocation.args[index + 1] === packageName,
      ),
    ),
    `${packageName} must retain an isolated all-features product-test invocation`,
  );
}

const missingPackages = workspaceCrates.filter((packageName) => !packageOwners.has(packageName));
const unknownPackages = [...packageOwners.keys()].filter(
  (packageName) => !workspaceCrates.includes(packageName),
);
const duplicatePackages = [...packageOwners.entries()]
  .filter(([, owners]) => owners.length !== 1)
  .map(([packageName, owners]) => `${packageName}:${owners.join("+")}`);

assert.deepEqual(missingPackages, [], `workspace packages missing from shards: ${missingPackages}`);
assert.deepEqual(unknownPackages, [], `shards name unknown workspace packages: ${unknownPackages}`);
assert.deepEqual(
  duplicatePackages,
  [],
  `workspace packages assigned to multiple shards: ${duplicatePackages}`,
);

// The CI-structure assertions read the governed job registry (the
// authority ci.yml is byte-generated from), and the aggregation invariant is
// the transitive needs closure of ci-required. Deleting the intermediate aggregator is legal; a
// product-test result no longer reaching ci-required is not).
const registry = loadCiWorkflowRegistry(root);
assert.ok(registry, "packages/check-orchestrator/ci-workflow.json must exist");
const contractShards = bundleShardNames(CONTRACT_BUNDLE_ID);
const structureErrors = findProductTestCiStructureErrors(registry.jobs, {
  crateShardIds: RUST_PRODUCT_TEST_SHARDS.map(({ id }) => id),
  contractShardNames: [...contractShards],
});
assert.deepEqual(structureErrors, [], `CI structure violations: ${structureErrors.join(" | ")}`);

console.log(
  JSON.stringify({
    schemaVersion: "2",
    product: "rust.product-test-coverage-classguard",
    executionMode: "exact-partition-all-features",
    workspaceCrateCount: workspaceCrates.length,
    crateShards: Object.fromEntries(packagesByShard),
    contractShards,
  }),
);
