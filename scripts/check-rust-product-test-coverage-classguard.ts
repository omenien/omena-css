import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const PRODUCT_SCRIPT_NAME = "check:rust-product-test-execution";
const root = process.cwd();
const packageJson = JSON.parse(readFileSync("package.json", "utf8")) as {
  scripts?: Record<string, string>;
};
const productScript = packageJson.scripts?.[PRODUCT_SCRIPT_NAME];

assert.ok(productScript, `${PRODUCT_SCRIPT_NAME} must exist in package.json`);

const cargoTestSegment = productScript.match(/\bcargo\s+test\b(?<segment>[\s\S]*?)--no-fail-fast/)
  ?.groups?.segment;
assert.ok(
  cargoTestSegment,
  `${PRODUCT_SCRIPT_NAME} must contain a cargo test segment ending in --no-fail-fast`,
);
assert.match(
  cargoTestSegment,
  /(?:^|\s)--workspace(?:\s|$)/,
  `${PRODUCT_SCRIPT_NAME} must execute every Cargo workspace member`,
);
assert.match(
  cargoTestSegment,
  /(?:^|\s)--all-features(?:\s|$)/,
  `${PRODUCT_SCRIPT_NAME} must exercise feature-gated test surfaces`,
);
assert.doesNotMatch(
  cargoTestSegment,
  /(?:^|\s)-p\s+[A-Za-z0-9_-]+/,
  `${PRODUCT_SCRIPT_NAME} must not regress to a hand-maintained package allowlist`,
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

console.log(
  JSON.stringify({
    schemaVersion: "1",
    product: "rust.product-test-coverage-classguard",
    executionMode: "workspace-all-features",
    workspaceCrateCount: workspaceCrates.length,
    workspaceCrates,
  }),
);
