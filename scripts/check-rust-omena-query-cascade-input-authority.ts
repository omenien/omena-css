#!/usr/bin/env node
import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

import { assertRustCfgTestMaskContract, maskRustCfgTestItems } from "./lib/rust-cfg-test-mask";

assertRustCfgTestMaskContract();

const querySourceRoot = "rust/crates/omena-query/src";
const cascadeModulePath = `${querySourceRoot}/style/cascade_checker.rs`;
const scannerPath = `${querySourceRoot}/style/cascade_checker/source_scanner.rs`;
const testOnlyPaths = new Set([
  scannerPath,
  `${querySourceRoot}/style/cascade_checker/declaration_facts_differential.rs`,
]);
const scannerReference =
  /\b(?:source_scanner|collect_query_checker_cascade_declarations_from_scanner|query_prelude_start|find_query_top_level_byte|matching_query_block_end)\b/gu;

assert.doesNotMatch(
  maskRustCfgTestItems(
    `#[cfg(test)]\nfn probe() { let _ = b'{'; let _ = "}"; source_scanner::probe(); }\nfn live() {}`,
  ),
  /source_scanner/u,
  "cfg(test) masking must ignore delimiter bytes inside literals",
);

const cascadeModule = readFileSync(cascadeModulePath, "utf8");
assert.match(
  cascadeModule,
  /#\[cfg\(test\)\]\s*mod source_scanner;/u,
  "the legacy declaration scanner must remain test-only",
);

const rustPaths = [
  ...new Set(
    execFileSync(
      "git",
      ["ls-files", `:(glob)${querySourceRoot}/*.rs`, `:(glob)${querySourceRoot}/**/*.rs`],
      { encoding: "utf8" },
    )
      .trim()
      .split("\n")
      .filter(Boolean),
  ),
].toSorted();

const references: { readonly path: string; readonly line: number; readonly token: string }[] = [];
for (const path of rustPaths) {
  if (testOnlyPaths.has(path)) continue;
  const source = readFileSync(path, "utf8");
  let production = maskRustCfgTestItems(source);
  if (
    process.argv.includes("--inject-scanner-reference") &&
    path === `${querySourceRoot}/style.rs`
  ) {
    production += "\nfn injected_scanner_reference() { source_scanner::query_prelude_start(); }\n";
  }
  for (const match of production.matchAll(scannerReference)) {
    references.push({
      path,
      line: production.slice(0, match.index).split("\n").length,
      token: match[0],
    });
  }
}

assert.deepEqual(
  references,
  [],
  `production cascade input still references the retired scanner: ${JSON.stringify(references)}`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-query.cascade-input-authority",
      scannedRustFileCount: rustPaths.length,
      productionScannerReferenceCount: references.length,
      scannerDisposition: "cfg-test-only-differential-oracle",
    },
    null,
    2,
  )}\n`,
);
