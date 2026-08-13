import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { bundleShardNames } from "../packages/check-orchestrator/src/manifest/shards";
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

const workflowLines = readFileSync(".github/workflows/ci.yml", "utf8").split(/\r?\n/u);
const planJob = workflowJobBlock(workflowLines, "rust-product-test-plan");
const crateJob = workflowJobBlock(workflowLines, "rust-product-test-crates");
const contractJob = workflowJobBlock(workflowLines, "rust-product-test-contracts");
const aggregateJob = workflowJobBlock(workflowLines, "rust-product-tests");

assert.match(
  planJob,
  /pnpm omena-check run rust\/product-test-coverage-classguard/u,
  "the product-test plan job must execute this classguard",
);
assert.deepEqual(
  parseInlineMatrix(crateJob, "product-shard").toSorted(),
  RUST_PRODUCT_TEST_SHARDS.map(({ id }) => id).toSorted(),
  "the Cargo matrix must execute every declared product-test shard",
);
assert.match(
  crateJob,
  /pnpm omena-check run rust\/product-test-execution -- \$\{\{ matrix\.product-shard \}\}/u,
  "the Cargo matrix must pass its shard through the canonical product-test gate",
);

const contractShards = bundleShardNames(CONTRACT_BUNDLE_ID);
assert.deepEqual(
  parseInlineMatrix(contractJob, "contract-shard").toSorted(),
  [...contractShards].toSorted(),
  "the contract matrix must execute every declared product-test contract shard",
);
assert.match(
  contractJob,
  /pnpm omena-check bundle rust\/product-test-contracts --summary --shard=\$\{\{ matrix\.contract-shard \}\}/u,
  "the contract matrix must use the orchestrator shard table",
);
assert.match(
  contractJob,
  /taiki-e\/install-action@7f4eb899022d8fe70b20c4f3de697aa85c309026/u,
  "the API-surface lane must retain the pinned prebuilt tool installer",
);
assert.match(
  contractJob,
  /key: rust-api-tools-\$\{\{ runner\.os \}\}-\$\{\{ runner\.arch \}\}-public-api-0\.52\.0-semver-checks-0\.48\.0/u,
  "the API-surface lane must cache its versioned Rust API tools",
);

for (const dependency of [
  "rust-product-test-plan",
  "rust-product-test-crates",
  "rust-product-test-contracts",
]) {
  assert.match(
    aggregateJob,
    new RegExp(`^\\s+- ${dependency}$`, "mu"),
    `rust-product-tests must aggregate ${dependency}`,
  );
}

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

function workflowJobBlock(lines: readonly string[], jobId: string): string {
  const start = lines.findIndex((line) => line === `  ${jobId}:`);
  assert.notEqual(start, -1, `CI workflow must declare job "${jobId}"`);
  const nextJobOffset = lines
    .slice(start + 1)
    .findIndex((line) => /^  [A-Za-z0-9_-]+:\s*$/u.test(line));
  const end = nextJobOffset === -1 ? lines.length : start + 1 + nextJobOffset;
  return lines.slice(start, end).join("\n");
}

function parseInlineMatrix(jobBlock: string, key: string): readonly string[] {
  const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const values = new RegExp(`^\\s+${escapedKey}:\\s*\\[([^\\]]+)\\]\\s*$`, "mu").exec(
    jobBlock,
  )?.[1];
  assert.ok(values, `matrix "${key}" must use an explicit inline list`);
  return values
    .split(",")
    .map((value) => value.trim().replace(/^["']|["']$/gu, ""))
    .filter(Boolean);
}
