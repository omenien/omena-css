import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

const sourcePath = "rust/crates/omena-query/src/style/transform.rs";
const source = readFileSync(sourcePath, "utf8");
const injectLinkedNeedle = process.argv.includes("--inject-linked-needle");

const bundleBody = extractFunctionBody(
  source,
  "run_omena_query_bundle_with_semantic_inputs_and_options",
);
let linkedSourceMapBody = extractFunctionBody(
  source,
  "summarize_omena_query_linked_bundle_source_map_v3",
);
const linkedSegmentBody = extractFunctionBody(source, "linked_bundle_source_map_segments");
const legacyInlineBody = extractFunctionBody(source, "import_inline_source_map_segments");
const legacyGraphBody = extractFunctionBody(source, "collect_import_graph_source_map_segments");

if (injectLinkedNeedle) {
  linkedSourceMapBody += "\nfind_import_origin_generated_range(";
}

assert.match(bundleBody, /if let Some\(materialization\)/u);
assert.match(bundleBody, /summarize_omena_query_linked_bundle_source_map_v3\s*\(/u);
assert.match(
  bundleBody,
  /summarize_omena_query_consumer_build_source_map_v3_with_resolution_inputs\s*\(/u,
);
assert.match(linkedSourceMapBody, /linked_bundle_source_map_segments\s*\(/u);
assert.doesNotMatch(linkedSourceMapBody, /find_import_origin_generated_range\s*\(/u);
assert.doesNotMatch(linkedSourceMapBody, /import_inline_source_map_segments\s*\(/u);
assert.doesNotMatch(linkedSegmentBody, /find_import_origin_generated_range\s*\(/u);
assert.doesNotMatch(linkedSegmentBody, /import_inline_source_map_segments\s*\(/u);
assert.equal(countCalls(legacyInlineBody, "find_import_origin_generated_range"), 1);
assert.equal(countCalls(legacyGraphBody, "find_import_origin_generated_range"), 1);
assert.equal(countCalls(source, "find_import_origin_generated_range") - 1, 2);

const test = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "linked_bundle_source_map_uses_materialized_module_offsets",
    "--quiet",
  ],
  { encoding: "utf8" },
);
assert.equal(test.status, 0, [test.stdout, test.stderr].filter(Boolean).join("\n"));

console.log(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-query.linked-source-map-boundary",
      linkedSourceMapAuthority: "materializedModuleRegions",
      linkedNeedleCallCount: 0,
      legacyNeedleCallCount: 2,
      exactOffsetTestPassed: true,
    },
    null,
    2,
  ),
);

function extractFunctionBody(input: string, functionName: string): string {
  const declarationStart = input.indexOf(`fn ${functionName}`);
  assert.notEqual(declarationStart, -1, `missing function ${functionName}`);
  const bodyStart = input.indexOf("{", declarationStart);
  assert.notEqual(bodyStart, -1, `missing body for function ${functionName}`);
  let depth = 1;
  let cursor = bodyStart + 1;
  while (cursor < input.length && depth > 0) {
    if (input[cursor] === "{") depth += 1;
    if (input[cursor] === "}") depth -= 1;
    cursor += 1;
  }
  assert.equal(depth, 0, `unterminated function ${functionName}`);
  return input.slice(bodyStart + 1, cursor - 1);
}

function countCalls(input: string, functionName: string): number {
  return [...input.matchAll(new RegExp(`\\b${functionName}\\s*\\(`, "gu"))].length;
}
