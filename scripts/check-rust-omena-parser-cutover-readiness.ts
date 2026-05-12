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
  /^scripts\/check-rust-omena-lsp-server-parser-consumer\.ts$/,
  /^scripts\/check-rust-omena-bridge-parser-consumer\.ts$/,
  /^scripts\/check-rust-omena-cascade-parser-consumer\.ts$/,
  /^scripts\/check-rust-omena-parser-cutover-readiness\.ts$/,
  /^scripts\/check-rust-omena-parser-differential-corpus\.ts$/,
  /^scripts\/check-rust-omena-parser-forward-canary\.ts$/,
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
assert.ok(
  lspManifest.includes("omena-query"),
  "omena-lsp-server must consume parser facts through omena-query",
);
assert.ok(
  !lspManifest.includes("engine-style-parser"),
  "omena-lsp-server must not depend on engine-style-parser",
);

const codeActionQuery = readText("server/engine-host-node/src/code-action-query.ts");
assert.ok(
  !codeActionQuery.includes("parseStyleDocument"),
  "code-action-query must consume style documents through the runtime buildStyleDocument choke point",
);
assert.ok(
  codeActionQuery.includes("buildStyleDocumentForCodeAction"),
  "code-action-query must keep an explicit runtime style-document builder boundary",
);

const scssIndex = readText("server/engine-core-ts/src/core/scss/scss-index.ts");
assert.ok(
  scssIndex.includes("export type StyleDocumentBuilder") &&
    scssIndex.includes("buildBaseStyleDocument") &&
    scssIndex.includes("parseStyleDocument(content, filePath)"),
  "StyleIndexCache must expose a single injectable style-document builder seam for parser cutover",
);

const sharedRuntimeCaches = readText(
  "server/engine-host-node/src/runtime/shared-runtime-caches.ts",
);
assert.ok(
  sharedRuntimeCaches.includes("BuildSharedRuntimeCachesOptions") &&
    sharedRuntimeCaches.includes("buildStyleDocument?: StyleDocumentBuilder") &&
    sharedRuntimeCaches.includes("options.buildStyleDocument"),
  "shared runtime caches must thread the parser builder seam into the runtime style index",
);

const serverRuntimeManager = readText(
  "server/engine-host-node/src/runtime/server-runtime-manager.ts",
);
assert.ok(
  serverRuntimeManager.includes("buildStyleDocument?: StyleDocumentBuilder") &&
    serverRuntimeManager.includes("buildSharedRuntimeCaches") &&
    serverRuntimeManager.includes("args.options.buildStyleDocument"),
  "server runtime manager must expose the parser builder seam for future Rust parser injection",
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
assertCutoverGateWiring(
  "G.parse",
  packageJson.includes('"check:rust-omena-parser-boundary"') &&
    packageJson.includes("cargo test --manifest-path rust/Cargo.toml -p omena-parser"),
);
assertCutoverGateWiring(
  "G.tree-shape",
  packageJson.includes('"check:rust-omena-syntax-boundary"') &&
    packageJson.includes("summarizes_parser_cst_equivalence_contract"),
);
assertCutoverGateWiring(
  "G.lsp",
  packageJson.includes('"check:rust-omena-lsp-server-boundary"') &&
    packageJson.includes("rust/omena-lsp-server/parser-consumer"),
);
assertCutoverGateWiring(
  "G.cascade",
  packageJson.includes('"check:rust-omena-cascade-boundary"') &&
    packageJson.includes("rust/omena-cascade/parser-consumer"),
);
assertCutoverGateWiring(
  "G.bridge",
  packageJson.includes('"check:rust-omena-bridge-boundary"') &&
    packageJson.includes("rust/omena-bridge/parser-consumer"),
);
assertCutoverGateWiring(
  "G.differential",
  packageJson.includes('"check:rust-omena-parser-differential-corpus"') &&
    packageJson.includes("rust/omena-parser/differential-corpus"),
);
assertCutoverGateWiring(
  "G.codspeed",
  packageJson.includes('"check:rust-z5-performance-baseline-readiness"') &&
    packageJson.includes("rust/z5-parser-product-cutover"),
);
assertCutoverGateWiring(
  "G.canary",
  packageJson.includes('"check:rust-omena-parser-forward-canary"') &&
    packageJson.includes("rust/omena-parser/forward-canary"),
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
  `validated omena-parser cutover readiness: productCrates=${PRODUCT_CRATE_MANIFESTS.length} parserLaneScripts=${PRODUCT_PARSER_LANE_SCRIPTS.length} cutoverGates=8 allowedLegacyRefs=${legacyReferencePaths.length}\n`,
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
    .toSorted();
}

function assertCutoverGateWiring(gate: string, condition: boolean): void {
  assert.ok(condition, `${gate} must be wired into the omena-parser cutover gate set`);
}
