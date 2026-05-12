import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const PRODUCT_CRATE_MANIFESTS = [
  "rust/crates/omena-bridge/Cargo.toml",
  "rust/crates/omena-query/Cargo.toml",
  "rust/crates/omena-semantic/Cargo.toml",
] as const;

const PRODUCT_PARSER_LANE_SCRIPTS = [
  "scripts/check-rust-parser-parity-lite.ts",
  "scripts/check-rust-parser-index-bridge.ts",
  "scripts/check-rust-parser-canonical-candidate.ts",
  "scripts/check-rust-parser-evaluator-candidates.ts",
  "scripts/check-rust-parser-canonical-producer.ts",
  "scripts/check-rust-parser-consumer-boundary.ts",
] as const;

const ALLOWED_LEGACY_REFERENCE_PATTERNS = [
  /^package\.json$/,
  /^rust\/Cargo\.toml$/,
  /^rust\/crates\/engine-style-parser\//,
  /^rust\/crates\/omena-benchmarks\//,
  /^rust\/crates\/omena-parser\/README\.md$/,
  /^rust\/crates\/omena-parser\/src\/lib\.rs$/,
  /^rust\/crates\/omena-query\/src\/tests\.rs$/,
  /^scripts\/check-rust-omena-lsp-server-boundary\.ts$/,
  /^scripts\/check-rust-omena-parser-cutover-readiness\.ts$/,
  /^scripts\/check-rust-omena-parser-differential-corpus\.ts$/,
  /^scripts\/check-rust-omena-parser-style-facts-parity\.ts$/,
  /^scripts\/check-rust-parser-git-consumer\.sh$/,
  /^scripts\/check-rust-split-publish-readiness\.ts$/,
  /^scripts\/prepare-engine-style-parser-subtree\.sh$/,
  /^scripts\/update-rust-split-consumer-pins\.ts$/,
] as const;

const packageJson = readText("package.json");

for (const manifestPath of PRODUCT_CRATE_MANIFESTS) {
  const manifest = readText(manifestPath);
  assert.ok(
    manifest.includes("omena-parser"),
    `${manifestPath} must consume parser facts through omena-parser`,
  );
  assert.ok(
    !manifest.includes("engine-style-parser"),
    `${manifestPath} must not depend on engine-style-parser`,
  );
}

const lspManifest = readText("rust/crates/omena-lsp-server/Cargo.toml");
assert.ok(lspManifest.includes("omena-query"), "omena-lsp-server must consume parser facts through omena-query");
assert.ok(
  !lspManifest.includes("engine-style-parser"),
  "omena-lsp-server must not depend on engine-style-parser",
);

for (const scriptPath of PRODUCT_PARSER_LANE_SCRIPTS) {
  const source = readText(scriptPath);
  assert.ok(source.includes("omena-parser"), `${scriptPath} must invoke omena-parser`);
  assert.ok(
    !source.includes("engine-style-parser"),
    `${scriptPath} must not invoke engine-style-parser in the product parser lane`,
  );
}

assert.ok(
  packageJson.includes('"check:rust-omena-parser-boundary"') &&
    packageJson.includes("rust/omena-parser/cutover-readiness"),
  "rust/omena-parser/boundary must include cutover-readiness",
);
assert.ok(
  packageJson.includes('"check:rust-parser-public-product"') &&
    packageJson.includes("rust/parser/consumer-boundary"),
  "rust/parser/public-product must include the parser consumer boundary",
);

const legacyReferencePaths = findLegacyReferencePaths();
const unexpectedLegacyReferences = legacyReferencePaths.filter(
  (filePath) => !ALLOWED_LEGACY_REFERENCE_PATTERNS.some((pattern) => pattern.test(filePath)),
);
assert.deepEqual(
  unexpectedLegacyReferences,
  [],
  `unexpected engine-style-parser references outside oracle/compat/baseline paths: ${unexpectedLegacyReferences.join(", ")}`,
);

process.stdout.write(
  `validated omena-parser cutover readiness: productCrates=${PRODUCT_CRATE_MANIFESTS.length} parserLaneScripts=${PRODUCT_PARSER_LANE_SCRIPTS.length} allowedLegacyRefs=${legacyReferencePaths.length}\n`,
);

function readText(filePath: string): string {
  return readFileSync(filePath, "utf8");
}

function findLegacyReferencePaths(): string[] {
  const result = spawnSync(
    "rg",
    [
      "-l",
      "engine-style-parser|engine_style_parser",
      "package.json",
      "scripts",
      "rust/Cargo.toml",
      "rust/crates",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
    },
  );

  if (result.status !== 0 && result.status !== 1) {
    throw new Error(`rg failed while scanning legacy parser references:\n${result.stderr}`);
  }

  return result.stdout
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .sort();
}
