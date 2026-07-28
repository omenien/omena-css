import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

const sourcePath = "rust/crates/omena-query/src/style/transform.rs";
const source = readFileSync(sourcePath, "utf8");
const injectLinkedNeedle = process.argv.includes("--inject-linked-needle");
const injectSharedEntrypoint = process.argv.includes("--inject-shared-entrypoint");

const semanticBundleBody = extractFunctionBody(
  source,
  "run_omena_query_bundle_with_semantic_inputs_and_options",
);
const attributedBundleBody = extractFunctionBody(
  source,
  "run_omena_query_bundle_with_module_reachability_and_options",
);
const semanticScopeBody = extractFunctionBody(
  source,
  "run_omena_query_bundle_with_execution_scope_evidence_and_options",
);
const attributedScopeBody = extractFunctionBody(
  source,
  "run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options",
);
const bundleBody = extractFunctionBody(
  source,
  "run_omena_query_bundle_with_optional_module_reachability",
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

const sharedBundleEntrypointCount =
  countCalls(
    stripRustComments(source),
    "run_omena_query_bundle_with_optional_module_reachability",
  ) -
  1 +
  (injectSharedEntrypoint ? 1 : 0);
const expectedSharedEntrypointCount = [semanticScopeBody, attributedScopeBody].filter(
  (body) => countCalls(body, "run_omena_query_bundle_with_optional_module_reachability") === 1,
).length;
// A new caller of the shared bundle implementation changes the source-wide
// count without changing this expected public-entrypoint population.
assert.equal(sharedBundleEntrypointCount, expectedSharedEntrypointCount);
assert.equal(
  countCalls(
    semanticBundleBody,
    "run_omena_query_bundle_with_execution_scope_evidence_and_options",
  ),
  1,
);
assert.equal(
  countCalls(
    attributedBundleBody,
    "run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options",
  ),
  1,
);
for (const entrypointBody of [
  semanticBundleBody,
  attributedBundleBody,
  semanticScopeBody,
  attributedScopeBody,
]) {
  assert.doesNotMatch(entrypointBody, /summarize_omena_query_linked_bundle_source_map_v3\s*\(/u);
  assert.doesNotMatch(
    entrypointBody,
    /summarize_omena_query_consumer_build_source_map_v3_with_resolution_inputs\s*\(/u,
  );
}
for (const entrypointBody of [semanticScopeBody, attributedScopeBody]) {
  assert.equal(
    countCalls(entrypointBody, "run_omena_query_bundle_with_optional_module_reachability"),
    1,
  );
}
assert.match(bundleBody, /linked_materialization\.as_ref\(\)/u);
assert.match(bundleBody, /linked_module_executions\.as_deref\(\)/u);
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
assert.equal(
  countCalls(linkedSegmentBody, "print_omena_query_transform_source_with_pretty_options"),
  1,
);
assert.doesNotMatch(
  linkedSegmentBody,
  /print_transform_execution_artifact_with_dialect_and_source\s*\(/u,
);
assert.equal(countCalls(legacyInlineBody, "find_import_origin_generated_range"), 1);
assert.equal(countCalls(legacyGraphBody, "find_import_origin_generated_range"), 1);
const linkedNeedleCallCount =
  countCalls(linkedSourceMapBody, "find_import_origin_generated_range") +
  countCalls(linkedSegmentBody, "find_import_origin_generated_range");
const legacyNeedleCallCount =
  countCalls(legacyInlineBody, "find_import_origin_generated_range") +
  countCalls(legacyGraphBody, "find_import_origin_generated_range");
assert.equal(linkedNeedleCallCount, 0);
assert.equal(
  countCalls(stripRustComments(source), "find_import_origin_generated_range") - 1,
  linkedNeedleCallCount + legacyNeedleCallCount,
);

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
const testOutput = [test.stdout, test.stderr].filter(Boolean).join("\n");
assert.equal(test.status, 0, testOutput);
const exactOffsetTestCount = [...testOutput.matchAll(/running ([0-9]+) tests?/gu)]
  .map((match) => Number.parseInt(match[1], 10))
  .reduce((total, count) => total + count, 0);
assert.ok(exactOffsetTestCount > 0, "linked source-map offset filter matched no tests");

console.log(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-query.linked-source-map-boundary",
      linkedSourceMapAuthority: "materializedModuleRegions",
      sharedBundleEntrypointCount,
      linkedNeedleCallCount,
      legacyNeedleCallCount,
      exactOffsetTestCount,
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

function stripRustComments(value: string): string {
  return value.replace(/\/\*[\s\S]*?\*\//gu, "").replace(/\/\/.*$/gmu, "");
}
