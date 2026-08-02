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
    productSource.includes(`Self::${variant} => "${wireLabel}"`),
    `the product gate must consume the linked-emission ${variant}/${wireLabel} pathScope`,
  );
}

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
