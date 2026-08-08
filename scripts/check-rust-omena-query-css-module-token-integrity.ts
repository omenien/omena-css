import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const productSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-query/src/style/transform/token_integrity.rs"),
  "utf8",
);
const transformContractSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-transform-passes/src/model.rs"),
  "utf8",
);
const queryTypesSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-query/src/types.rs"),
  "utf8",
);
const transformFacadeSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-query/src/style/transform.rs"),
  "utf8",
);
const instrumentSource = readFileSync(
  path.join(repoRoot, "rust/crates/omena-diff-test/src/linked_emission.rs"),
  "utf8",
);
const pathScopeEnum =
  /pub enum LinkedEmissionModuleTokenCollisionPathScopeV0\s*\{(?<body>[^}]*)\}/u.exec(
    instrumentSource,
  );
assert.ok(pathScopeEnum?.groups?.body, "the linked-emission pathScope enum is missing");
const pathScopeVocabulary = [
  ...pathScopeEnum.groups.body.matchAll(/^\s*(?<variant>[A-Z][A-Za-z0-9]*),\s*$/gmu),
].map((match) => {
  const variant = match.groups?.variant;
  assert.ok(variant);
  return [variant, `${variant[0].toLowerCase()}${variant.slice(1)}`] as const;
});
assert.ok(pathScopeVocabulary.length > 0, "the linked-emission pathScope enum is empty");

for (const [variant, wireLabel] of pathScopeVocabulary) {
  assert.ok(
    transformContractSource.includes(`Self::${variant} => "${wireLabel}"`),
    `the product gate must consume the linked-emission ${variant}/${wireLabel} pathScope`,
  );
}
for (const field of [
  "module_instances",
  "module_paths",
  "original_names",
  "module_token_collisions",
  "unattributed_emitted_tokens",
  "unavailable_reasons",
]) {
  assert.ok(
    transformContractSource.includes(`pub ${field}:`),
    `the ownership census is missing its ${field} consumer field`,
  );
}
assert.ok(
  transformContractSource.includes("Vec<ModuleInstanceKeyV0>"),
  "the ownership census must carry the existing module-instance identity",
);
assert.ok(
  queryTypesSource.includes("pub struct OmenaQueryBundleTokenOwnershipResultV0"),
  "the additive bundle result must expose the product-readable ownership census",
);
assert.ok(
  transformFacadeSource.includes("run_omena_query_bundle_with_token_ownership_census_and_options"),
  "the product must expose an additive census-returning build entry point",
);
assert.ok(
  productSource.includes("CssModuleTokenOwnershipCensusV0::unavailable"),
  "the producer must distinguish an unavailable census from a complete empty census",
);

const result = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "token_integrity",
    "--",
    "--nocapture",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  },
);
assert.equal(
  result.status,
  0,
  ["CSS Modules token-integrity product tests failed", result.stdout, result.stderr]
    .filter(Boolean)
    .join("\n"),
);
const transcript = `${result.stdout}\n${result.stderr}`;
const passed = [...transcript.matchAll(/test result: ok\. (\d+) passed/gu)].reduce(
  (total, match) => total + Number(match[1]),
  0,
);
assert.ok(passed >= 3, `expected at least three token-integrity tests, observed ${passed}`);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-query.css-module-token-integrity",
      ownershipCensusProduct: "omena-query.css-module-token-ownership-census",
      ownershipIdentity: "omena-parser::ModuleInstanceKeyV0",
      ownershipConsumer: "closed-world admission OwnershipNotSeparable",
      pathScopeAuthority: "omena-diff-test::LinkedEmissionModuleTokenCollisionPathScopeV0",
      pathScopeVocabulary: pathScopeVocabulary.map(([, wireLabel]) => wireLabel),
      productTestCount: passed,
      verificationProfile: "strict",
      emissionPaths: ["importInlineLegacy", "linkedOrder"],
      tierReachable: true,
    },
    null,
    2,
  )}\n`,
);
