import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { isCheckerRuleCode } from "../server/engine-core-ts/src/core/checker/checker-rule-registry";

interface CompatibilityCensus {
  readonly schemaVersion: "0";
  readonly product: "omena-cli.stylelint-compat-census";
  readonly pluginPackage: "@omena/stylelint-plugin";
  readonly pluginPeerRange: string;
  readonly mappings: readonly {
    readonly stylelintRule: string;
    readonly omenaRule: string;
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const census = readJson<CompatibilityCensus>("rust/crates/omena-cli/stylelint-compat-census.json");
const packageJson = readJson<{ readonly peerDependencies: Record<string, string> }>(
  "packages/stylelint-plugin/package.json",
);
const pluginRules = [
  ...read("packages/stylelint-plugin/index.cjs").matchAll(/require\("\.\/lib\/([^"/]+)\.cjs"\)/gu),
]
  .map((match) => {
    const source = read(`packages/stylelint-plugin/lib/${match[1]}.cjs`);
    const ruleName = /const ruleName = "([^"]+)";/u.exec(source)?.[1];
    assert.ok(ruleName, `missing ruleName in ${match[1]}.cjs`);
    return ruleName;
  })
  .toSorted();
const recommendedRules = [
  ...read("packages/stylelint-plugin/recommended.cjs").matchAll(/"(omena\/[^"]+)":/gu),
]
  .map((match) => match[1])
  .toSorted();

const mappings = [...census.mappings];
if (process.env.OMENA_STYLELINT_COMPAT_TEST_UNKNOWN_RULE === "1") {
  mappings.push({ stylelintRule: "omena/injected", omenaRule: "injected-unknown-rule" });
}

assert.equal(census.schemaVersion, "0");
assert.equal(census.product, "omena-cli.stylelint-compat-census");
assert.equal(census.pluginPackage, "@omena/stylelint-plugin");
assert.equal(census.pluginPeerRange, "^17.0.0");
assert.equal(packageJson.peerDependencies.stylelint, census.pluginPeerRange);
assert.equal(new Set(mappings.map(({ stylelintRule }) => stylelintRule)).size, mappings.length);
assert.equal(new Set(mappings.map(({ omenaRule }) => omenaRule)).size, mappings.length);
for (const mapping of mappings) {
  assert.ok(isCheckerRuleCode(mapping.omenaRule), `unknown Omena rule ${mapping.omenaRule}`);
}
const mappedStylelintRules = census.mappings.map(({ stylelintRule }) => stylelintRule).toSorted();
assert.deepEqual(mappedStylelintRules, pluginRules);
assert.deepEqual(recommendedRules, pluginRules);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-stylelint-compat",
      mappedRuleCount: mappings.length,
      pluginRuleCount: pluginRules.length,
      unsupportedPolicy: "reported",
    },
    null,
    2,
  )}\n`,
);

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}

function read(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}
