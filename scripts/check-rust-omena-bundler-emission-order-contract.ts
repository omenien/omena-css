import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const injectUnclassified = process.argv.includes("--inject-unclassified-edge-kind");
const crossFileSource = read("rust/crates/omena-cross-file-summary/src/lib.rs");
const bundlerSource = read("rust/crates/omena-bundler/src/lib.rs");
const emissionSource = read("rust/crates/omena-bundler/src/emission_order.rs");
const rawVariants = enumVariants(crossFileSource, "OmenaCrossFileSummaryRawEdgeKindV0");
const bundleVariants = enumVariants(bundlerSource, "TransformBundleEdgeKind");

const cargoArgs = [
  "run",
  "--quiet",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-bundler",
  "--bin",
  "omena-emission-order-contract",
];
if (injectUnclassified) {
  cargoArgs.push("--", "--inject-unclassified-edge-kind");
}
const gate = spawnSync("cargo", cargoArgs, {
  cwd: repoRoot,
  encoding: "utf8",
  maxBuffer: 16 * 1024 * 1024,
});
if (gate.status !== 0) {
  throw new Error(
    `emission-order runtime contract failed (${gate.status ?? "signal"})\n${gate.stderr}${gate.stdout}`,
  );
}

const lines = gate.stdout.trim().split("\n");
const header = lines[0]?.split("\t") ?? [];
assert.deepEqual(header.slice(0, 3), ["report", "0", "omena-bundler.emission-order-contract"]);
assert.equal(Number(header[3]), rawVariants.length);
assert.equal(Number(header[4]), bundleVariants.length);
assert.equal(header[5], "moduleIdLegacy", "default emission order must stay byte-compatible");

const rawRows = lines.filter((line) => line.startsWith("raw\t")).map((line) => line.split("\t"));
const bundleRows = lines
  .filter((line) => line.startsWith("bundle\t"))
  .map((line) => line.split("\t"));
const policyRows = lines
  .filter((line) => line.startsWith("policy\t"))
  .map((line) => line.split("\t"));
const differenceRows = lines
  .filter((line) => line.startsWith("difference\t"))
  .map((line) => line.split("\t"));
assert.equal(rawRows.length, rawVariants.length);
assert.equal(bundleRows.length, bundleVariants.length);
assert.equal(new Set(rawRows.map((row) => row[1])).size, rawRows.length);
assert.equal(new Set(bundleRows.map((row) => row[1])).size, bundleRows.length);
for (const row of [...rawRows, ...bundleRows]) {
  assert.match(row[2] ?? "", /^(orderBearing|orderNeutral)$/);
}
for (const row of bundleRows) {
  assert.ok((row[3] ?? "").length > 0, `missing order-relevance reason for ${row[1]}`);
}
assert.deepEqual(
  policyRows.map((row) => row[1]).sort(),
  ["css", "less", "scss"],
  "the policy differential must span the supported stylesheet dialects",
);
for (const row of policyRows) {
  const fixtureId = row[1] ?? "";
  const moduleIdLegacyRuleCount = Number(row[2]);
  const importOrderRuleCount = Number(row[3]);
  const differenceCount = Number(row[4]);
  assert.ok(moduleIdLegacyRuleCount > 0, `${fixtureId} legacy policy output must be non-empty`);
  assert.equal(importOrderRuleCount, moduleIdLegacyRuleCount);
  assert.ok(differenceCount > 0, `${fixtureId} must distinguish the two policies`);
  assert.equal(row[5], "false", `${fixtureId} policy comparison must report divergence`);
  assert.equal(
    differenceRows.filter((difference) => difference[1] === fixtureId).length,
    differenceCount,
    `${fixtureId} detailed differences must match the derived count`,
  );
}
for (const row of differenceRows) {
  assert.ok(Number.isInteger(Number(row[2])));
  assert.ok((row[3] ?? "").length > 0 && (row[5] ?? "").length > 0);
}

assert.ok(!bundlerSource.includes("build_global_rule_order_from_projection"));
assert.ok(emissionSource.includes("build_global_rule_order_from_plan"));
assert.match(bundlerSource, /#\[serde\(skip_serializing\)\]\s+pub emission_plan: EmissionPlanV0/);

const ledger = JSON.parse(read("rust/omena-emission-order-residual-ledger.json")) as {
  readonly schemaVersion: string;
  readonly product: string;
  readonly entries: ReadonlyArray<{
    readonly id: string;
    readonly status: string;
    readonly owner: string;
    readonly reason: string;
  }>;
};
assert.equal(ledger.schemaVersion, "0");
assert.equal(ledger.product, "omena-bundler.emission-order-residual-ledger");
assert.deepEqual(ledger.entries.map((entry) => entry.id).sort(), [
  "cascade-optimal-ordering",
  "cross-chunk-css-order",
  "dialect-import-semantics",
]);
assert.ok(ledger.entries.every((entry) => entry.owner.length > 0 && entry.reason.length > 0));

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-bundler.emission-order-contract",
      rawEdgeKindCount: rawRows.length,
      bundleEdgeKindCount: bundleRows.length,
      defaultPolicy: header[5],
      differentialFixtureCount: policyRows.length,
      differentialRowCount: differenceRows.length,
      residualEntryCount: ledger.entries.length,
      orderFlowsThroughPlan: true,
    },
    null,
    2,
  )}\n`,
);

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function enumVariants(source: string, enumName: string): string[] {
  const match = new RegExp(`pub enum ${enumName}\\s*\\{([\\s\\S]*?)\\n\\}`).exec(source);
  assert.ok(match, `missing enum ${enumName}`);
  return (match[1] ?? "")
    .split(",")
    .map((variant) => variant.replace(/\/\/.*$/gm, "").trim())
    .filter((variant) => /^[A-Z][A-Za-z0-9_]*$/.test(variant));
}
