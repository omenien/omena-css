import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { bundleShardNames } from "../packages/check-orchestrator/src/manifest/shards";
import { loadCiWorkflowRegistry } from "../packages/check-orchestrator/src/manifest/ci-workflow";
import { findProductTestCiStructureErrors } from "./lib/product-test-ci-structure";
import {
  RUST_PRODUCT_TEST_SHARDS,
  rustProductTestCargoArgs,
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

  const cargoArgs = rustProductTestCargoArgs(shard);
  assert.ok(cargoArgs.includes("--all-features"), `${shard.id} must exercise all package features`);
  assert.ok(cargoArgs.includes("--no-fail-fast"), `${shard.id} must preserve failure aggregation`);
  assert.equal(
    cargoArgs.includes("--workspace"),
    shard.workspaceRemainder,
    `${shard.id} workspace selection must match its declared mode`,
  );

  for (const packageName of shardPackages) {
    const owners = packageOwners.get(packageName) ?? [];
    owners.push(shard.id);
    packageOwners.set(packageName, owners);
  }
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

// g131-S0: the CI-structure assertions read the governed job REGISTRY (the
// authority ci.yml is byte-generated from), and the aggregation invariant is
// the transitive needs closure of ci-required — structure-proof against the
// g131-S3 restructures (deleting the intermediate aggregator is legal; a
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
