import { strict as assert } from "node:assert";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";

const COLLAPSED_CONST_PATH = "rust/crates/omena-query-transform-runner/src/lib.rs";
const PRODUCT_SCRIPT_NAME = "check:rust-product-test-execution";
const EXPLICIT_PRODUCT_TEST_CRATES = ["omena-abstract-value", "omena-cascade-proof"] as const;

const root = process.cwd();
const packageJson = JSON.parse(read("package.json")) as {
  scripts?: Record<string, string>;
};
const productScript = packageJson.scripts?.[PRODUCT_SCRIPT_NAME];

assert.ok(productScript, `${PRODUCT_SCRIPT_NAME} must exist in package.json`);

const testCrates = extractCargoTestPackageList(productScript);
const collapsedCrates = extractCollapsedTransformCrates(read(COLLAPSED_CONST_PATH));
const collapsedCratesWithTests = collapsedCrates.filter(
  (crateName) => testCountForCrate(crateName) > 0,
);
const missingCollapsedCrates = collapsedCratesWithTests.filter(
  (crateName) => !testCrates.has(crateName),
);

assert.deepEqual(
  missingCollapsedCrates,
  [],
  `collapsed transform crate(s) with tests missing from ${PRODUCT_SCRIPT_NAME}: ${missingCollapsedCrates.join(", ")}`,
);

const missingExplicitCrates = EXPLICIT_PRODUCT_TEST_CRATES.filter((crateName) => {
  assert.ok(
    testCountForCrate(crateName) > 0,
    `explicit product-test crate must contain tests: ${crateName}`,
  );
  return !testCrates.has(crateName);
});

assert.deepEqual(
  missingExplicitCrates,
  [],
  `explicit product-test crate(s) missing from ${PRODUCT_SCRIPT_NAME}: ${missingExplicitCrates.join(", ")}`,
);

console.log(
  JSON.stringify({
    schemaVersion: "0",
    product: "rust.product-test-coverage-classguard",
    collapsedTransformCrateCount: collapsedCrates.length,
    collapsedTransformCratesWithTests: collapsedCratesWithTests,
    explicitProductTestCrates: EXPLICIT_PRODUCT_TEST_CRATES,
    productTestCrateCount: testCrates.size,
  }),
);

function extractCargoTestPackageList(script: string): Set<string> {
  const segment = script.match(/\bcargo\s+test\b(?<segment>[\s\S]*?)--no-fail-fast/)?.groups
    ?.segment;
  assert.ok(
    segment,
    `${PRODUCT_SCRIPT_NAME} must contain a cargo test segment ending in --no-fail-fast`,
  );

  return new Set(
    Array.from(segment.matchAll(/(?:^|\s)-p\s+([A-Za-z0-9_-]+)/g), (match) => {
      const crateName = match[1];
      assert.ok(crateName, "cargo test -p entry must include a crate name");
      return crateName;
    }),
  );
}

function extractCollapsedTransformCrates(source: string): string[] {
  const constBody = source.match(
    /OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0\s*:\s*\[&str;\s*\d+\]\s*=\s*\[(?<body>[\s\S]*?)\];/,
  )?.groups?.body;

  assert.ok(
    constBody,
    `could not find OMENA_QUERY_TRANSFORM_RUNNER_COLLAPSED_CRATES_V0 in ${COLLAPSED_CONST_PATH}`,
  );

  const crates = Array.from(constBody.matchAll(/"([^"]+)"/g), (match) => {
    const crateName = match[1];
    assert.ok(crateName, "collapsed transform crate entry must be a string literal");
    return crateName;
  });

  assert.ok(crates.length > 0, "collapsed transform crate const must not be empty");
  return crates;
}

function testCountForCrate(crateName: string): number {
  const srcDir = path.join(root, "rust", "crates", crateName, "src");
  assert.ok(existsSync(srcDir), `crate src directory must exist: ${crateName}`);

  return rustFiles(srcDir)
    .map((filePath) => read(filePath))
    .reduce(
      (count, source) => count + Array.from(source.matchAll(/#\s*\[\s*test\s*\]/g)).length,
      0,
    );
}

function rustFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const entryPath = path.join(dir, entry);
    const stat = statSync(entryPath);
    if (stat.isDirectory()) {
      return rustFiles(entryPath);
    }
    return entryPath.endsWith(".rs") ? [entryPath] : [];
  });
}

function read(filePath: string): string {
  return readFileSync(path.isAbsolute(filePath) ? filePath : path.join(root, filePath), "utf8");
}
