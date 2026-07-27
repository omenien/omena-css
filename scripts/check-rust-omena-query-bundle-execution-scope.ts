import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

const modelPath = "rust/crates/omena-transform-passes/src/model.rs";
const transformPath = "rust/crates/omena-query/src/style/transform.rs";
const queryTypesPath = "rust/crates/omena-query/src/types.rs";
const queryTestPath = "rust/crates/omena-query/src/tests/transform_facade.rs";
const parserContractPath = "rust/crates/omena-parser/src/closed_world/contract.rs";
const napiPath = "rust/crates/omena-napi/src/lib.rs";
const wasmPath = "rust/crates/omena-wasm/src/lib.rs";
const declarationPath = "packages/css-build-adapter/index.d.ts";
const generatedDeclarationPath = "packages/css-build-adapter/bundler-host-contract.generated.d.ts";
const adapterGatePath = "scripts/check-rust-omena-bundler-adapter-pass-authority.ts";
const adapterTestPath = "test/unit/css-build-adapter/css-build-adapter.test.ts";
const publicApiPath = "rust/crates/omena-query/tests/snapshots/public-api.txt";

const modelSource = readFileSync(modelPath, "utf8");
const transformSource = readFileSync(transformPath, "utf8");
const queryTypesSource = readFileSync(queryTypesPath, "utf8");
let queryTestSource = readFileSync(queryTestPath, "utf8");
const parserContractSource = readFileSync(parserContractPath, "utf8");
let napiSource = readFileSync(napiPath, "utf8");
let wasmSource = readFileSync(wasmPath, "utf8");
let declarationSource = readFileSync(declarationPath, "utf8");
const generatedDeclarationSource = readFileSync(generatedDeclarationPath, "utf8");
const adapterGateSource = readFileSync(adapterGatePath, "utf8");
const adapterTestSource = readFileSync(adapterTestPath, "utf8");
const publicApiSource = readFileSync(publicApiPath, "utf8");

const injectDropField = process.argv.includes("--inject-drop-field");
const injectFlipScope = process.argv.includes("--inject-flip-scope");
const injectDuplicateCstBuild = process.argv.includes("--inject-duplicate-cst-build");
const injectDeclarationDrift = process.argv.includes("--inject-declaration-drift");
const injectCommentOnlyAssertion = process.argv.includes("--inject-comment-only-assertion");
if (injectDeclarationDrift) {
  declarationSource = declarationSource.replace(
    "readonly segmentCount: number;",
    "readonly segmentTotal: number;",
  );
}
if (injectCommentOnlyAssertion) {
  for (const source of [queryTestSource, napiSource, wasmSource]) {
    assert.match(source, /field\.field_name == "outcomes"/u);
  }
  queryTestSource = queryTestSource.replace(
    'field.field_name == "outcomes"',
    '// field.field_name == "outcomes"',
  );
  napiSource = napiSource.replace(
    'field.field_name == "outcomes"',
    '// field.field_name == "outcomes"',
  );
  wasmSource = wasmSource.replace(
    'field.field_name == "outcomes"',
    '// field.field_name == "outcomes"',
  );
}

const executionBody = extractTypeBody(modelSource, "pub struct TransformExecutionSummaryV0");
const executionFields = [...executionBody.matchAll(/^\s*pub\s+([a-z][a-z0-9_]*):/gmu)].map(
  (match) => snakeToCamel(match[1]),
);
assert.equal(executionFields.length, 27, "unexpected transform execution field count");
if (injectDropField) executionFields.pop();

const scopeBody = extractFunctionBody(transformSource, "bundle_execution_field_scopes");
const scopeRows = [
  ...scopeBody.matchAll(
    /execution_field_scope\(\s*"([^"]+)"\s*,\s*(Entry|Bundle)\s*,\s*"([^"]+)"\s*,?\s*\)/gmu,
  ),
].map((match) => ({
  fieldName: match[1],
  scope: match[2],
  derivation: match[3],
}));
assert.equal(scopeRows.length, 27, "unexpected execution-scope row count");
assert.equal(new Set(scopeRows.map((row) => row.fieldName)).size, scopeRows.length);
assert.ok(scopeRows.every((row) => row.derivation.trim().length > 0));
if (injectFlipScope) {
  const outputCss = scopeRows.find((row) => row.fieldName === "outputCss");
  assert.notEqual(outputCss, undefined);
  if (outputCss) outputCss.scope = "Entry";
}

assert.deepEqual(
  scopeRows.map((row) => row.fieldName).toSorted(),
  executionFields.toSorted(),
  "execution-scope table must classify every published field exactly once",
);
const bundleFields = scopeRows.filter((row) => row.scope === "Bundle").map((row) => row.fieldName);
const projectionBody = extractFunctionBody(transformSource, "project_linked_bundle_execution");
const projectedFields = [...projectionBody.matchAll(/execution\.([a-z][a-z0-9_]*)\s*=/gmu)].map(
  (match) => snakeToCamel(match[1]),
);
assert.deepEqual(
  bundleFields.toSorted(),
  projectedFields.toSorted(),
  "bundle-scoped fields must be derived from the linked execution projection",
);
assert.equal(scopeRows.find((row) => row.fieldName === "moduleQualifiedShake")?.scope, "Entry");
assert.equal(scopeRows.find((row) => row.fieldName === "passPlan")?.scope, "Entry");

const linkedExecutionBody = extractFunctionBody(transformSource, "execute_linked_bundle_modules");
const retainedExecutionProducerCount = countCalls(
  linkedExecutionBody,
  "execute_omena_query_consumer_build_style_module_with_context_and_closed_world_bundle",
);
assert.equal(retainedExecutionProducerCount, 1);
assert.equal(countCalls(linkedExecutionBody, "module_executions.push"), 1);

const evidenceBody = extractFunctionBody(
  transformSource,
  "summarize_linked_bundle_execution_scope",
);
assert.match(evidenceBody, /linked\.module_executions/u);
assert.doesNotMatch(evidenceBody, /execute_omena_query_consumer_build_style_module/u);
assert.doesNotMatch(evidenceBody, /find_target_style_source\s*\(/u);

const sourceMapBody = extractFunctionBody(transformSource, "linked_bundle_source_map_segments");
const cstBuildCount =
  countCalls(sourceMapBody, "print_omena_query_transform_source_with_pretty_options") +
  (injectDuplicateCstBuild ? 1 : 0);
assert.equal(cstBuildCount, 1, "linked source-map loop must build one CST artifact per module");
assert.match(sourceMapBody, /module_executions/u);
assert.doesNotMatch(sourceMapBody, /execute_omena_query_consumer_build_style_module/u);
assert.doesNotMatch(
  sourceMapBody,
  /print_transform_execution_artifact_with_dialect_and_source\s*\(/u,
);

for (const [path, source, testName] of [
  [queryTestPath, queryTestSource, "bundle_operation_facade_matches_consumer_build_source_map"],
  [napiPath, napiSource, "bundles_workspace_sources_for_node_clients"],
  [wasmPath, wasmSource, "bundles_workspace_sources_for_browser_clients"],
] as const) {
  const testBody = stripRustComments(extractFunctionBody(source, testName));
  assert.equal(
    countOccurrences(testBody, 'field.field_name == "outcomes"'),
    1,
    `${path} must assert the entry scope of transform outcomes`,
  );
  assert.match(testBody, /entry\.input_byte_len/u);
  assert.match(testBody, /artifact\.per_pass_provenance/u);
}

const serdeDeclarationContracts = [
  {
    rustSource: parserContractSource,
    rustName: "ModuleInstanceKeyV0",
    tsSource: generatedDeclarationSource,
    tsName: "OmenaModuleInstanceKeyV0",
  },
  {
    rustSource: queryTypesSource,
    rustName: "OmenaQueryExecutionFieldScopeV0",
    tsSource: declarationSource,
    tsName: "OmenaBundleExecutionFieldScopeV0",
  },
  {
    rustSource: queryTypesSource,
    rustName: "OmenaQueryBundleModuleExecutionByteFactsV0",
    tsSource: declarationSource,
    tsName: "OmenaBundleModuleExecutionByteFactsV0",
  },
  {
    rustSource: queryTypesSource,
    rustName: "OmenaQueryBundleCompositeExecutionByteFactsV0",
    tsSource: declarationSource,
    tsName: "OmenaBundleCompositeExecutionByteFactsV0",
  },
  {
    rustSource: queryTypesSource,
    rustName: "OmenaQueryLinkedSourceMapDispositionV0",
    tsSource: declarationSource,
    tsName: "OmenaLinkedSourceMapDispositionV0",
  },
  {
    rustSource: queryTypesSource,
    rustName: "OmenaQueryBundleExecutionScopeEvidenceV0",
    tsSource: declarationSource,
    tsName: "OmenaBundleExecutionScopeEvidenceV0",
  },
] as const;
for (const contract of serdeDeclarationContracts) {
  assert.deepEqual(
    extractRustStructFields(contract.rustSource, contract.rustName),
    extractTypeScriptInterfaceFields(contract.tsSource, contract.tsName),
    `${contract.tsName} must match the serialized Rust field set`,
  );
}
assert.deepEqual(
  extractRustEnumVariants(queryTypesSource, "OmenaQueryExecutionEvidenceScopeV0"),
  extractTypeScriptStringUnion(declarationSource, "OmenaBundleExecutionFieldScopeV0", "scope"),
);
assert.deepEqual(
  extractRustEnumVariants(queryTypesSource, "OmenaQueryLinkedSourceMapGranularityV0"),
  extractTypeScriptStringUnion(
    declarationSource,
    "OmenaLinkedSourceMapDispositionV0",
    "granularity",
  ),
);
assert.deepEqual(extractRustStructFields(napiSource, "OmenaNapiBundleExecutionScopeResultV0"), [
  "bundle",
  "executionScope",
]);
assert.deepEqual(extractRustStructFields(wasmSource, "OmenaWasmBundleExecutionScopeResultV0"), [
  "bundle",
  "executionScope",
]);
for (const [source, typeName] of [
  [napiSource, "OmenaNapiBundleExecutionScopeResultV0"],
  [wasmSource, "OmenaWasmBundleExecutionScopeResultV0"],
] as const) {
  const resultBody = extractTypeBody(source, `struct ${typeName}`);
  assert.match(resultBody, /#\[serde\(flatten\)\]\s*pub bundle:/u);
}
assert.match(
  declarationSource,
  /export interface OmenaBundleWithEvidenceV0 extends OmenaBundleArtifactV0/u,
);
assert.match(
  declarationSource,
  /readonly executionScope: OmenaBundleExecutionScopeEvidenceV0 \| null/u,
);
assert.match(adapterGateSource, /executionScope: null/u);
assert.match(adapterTestSource, /executionScope/u);

for (const surface of [
  "OmenaQueryBundleExecutionScopeEvidenceV0",
  "OmenaQueryBundleExecutionScopeResultV0",
  "run_omena_query_bundle_with_execution_scope_evidence_and_options",
]) {
  assert.ok(publicApiSource.includes(surface), `public API snapshot is missing ${surface}`);
}

runCargoTest("omena-query", "linked_bundle_retains_each_module_execution_before_bundle_projection");
runCargoTest("omena-query", "linked_bundle_source_map_");
runCargoTest("omena-query", "bundle_operation_facade_matches_consumer_build_source_map");
runCargoTest("omena-napi", "bundles_workspace_sources_for_node_clients");
runCargoTest("omena-wasm", "bundles_workspace_sources_for_browser_clients");
runCargoTest("omena-cli", "bundle_command_loads_configured_workspace_sources_without_flags");

console.log(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-query.bundle-execution-scope-gate",
      executionFieldCount: executionFields.length,
      scopeRowCount: scopeRows.length,
      entryFieldCount: scopeRows.filter((row) => row.scope === "Entry").length,
      bundleFieldCount: bundleFields.length,
      projectedBundleFieldCount: projectedFields.length,
      retainedExecutionProducerCount,
      perModuleCstBuildCount: cstBuildCount,
      serdeDeclarationContractCount: serdeDeclarationContracts.length,
    },
    null,
    2,
  ),
);

function runCargoTest(packageName: string, filter: string): void {
  const test = spawnSync(
    "cargo",
    ["test", "--manifest-path", "rust/Cargo.toml", "-p", packageName, filter, "--quiet"],
    { encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  const output = [test.stdout, test.stderr].filter(Boolean).join("\n");
  assert.equal(test.status, 0, output);
  assert.match(output, /running [1-9][0-9]* test/u, `${packageName}:${filter} matched no tests`);
}

function extractTypeBody(input: string, declaration: string): string {
  const declarationStart = input.indexOf(declaration);
  assert.notEqual(declarationStart, -1, `missing declaration ${declaration}`);
  return extractBracedBody(input, declarationStart, declaration);
}

function extractRustStructFields(input: string, typeName: string): string[] {
  const body = extractTypeBody(input, `struct ${typeName}`);
  return [...body.matchAll(/^\s*(?:pub\s+)?([a-z][a-z0-9_]*):/gmu)]
    .map((match) => snakeToCamel(match[1]))
    .toSorted();
}

function extractRustEnumVariants(input: string, typeName: string): string[] {
  const body = extractTypeBody(input, `enum ${typeName}`);
  return [...body.matchAll(/^\s*([A-Z][A-Za-z0-9]*),?\s*$/gmu)]
    .map((match) => lowerCamel(match[1]))
    .toSorted();
}

function extractTypeScriptInterfaceFields(input: string, typeName: string): string[] {
  const body = extractTypeBody(input, `interface ${typeName}`);
  return [...body.matchAll(/^\s*readonly\s+([a-z][A-Za-z0-9]*)\??:/gmu)]
    .map((match) => match[1])
    .toSorted();
}

function extractTypeScriptStringUnion(
  input: string,
  typeName: string,
  fieldName: string,
): string[] {
  const body = extractTypeBody(input, `interface ${typeName}`);
  const field = new RegExp(`readonly\\s+${fieldName}\\??:\\s*([^;]+);`, "u").exec(body);
  assert.notEqual(field, null, `missing ${typeName}.${fieldName}`);
  return [...(field?.[1] ?? "").matchAll(/"([^"]+)"/gu)].map((match) => match[1]).toSorted();
}

function extractFunctionBody(input: string, functionName: string): string {
  const declarationStart = input.indexOf(`fn ${functionName}`);
  assert.notEqual(declarationStart, -1, `missing function ${functionName}`);
  return extractBracedBody(input, declarationStart, functionName);
}

function extractBracedBody(input: string, declarationStart: number, label: string): string {
  const bodyStart = input.indexOf("{", declarationStart);
  assert.notEqual(bodyStart, -1, `missing body for ${label}`);
  let depth = 1;
  let cursor = bodyStart + 1;
  while (cursor < input.length && depth > 0) {
    if (input[cursor] === "{") depth += 1;
    if (input[cursor] === "}") depth -= 1;
    cursor += 1;
  }
  assert.equal(depth, 0, `unterminated body for ${label}`);
  return input.slice(bodyStart + 1, cursor - 1);
}

function snakeToCamel(value: string): string {
  return value.replace(/_([a-z0-9])/gu, (_match, character: string) => character.toUpperCase());
}

function lowerCamel(value: string): string {
  return `${value.slice(0, 1).toLowerCase()}${value.slice(1)}`;
}

function stripRustComments(value: string): string {
  return value.replace(/\/\*[\s\S]*?\*\//gu, "").replace(/\/\/.*$/gmu, "");
}

function countCalls(input: string, functionName: string): number {
  return [...input.matchAll(new RegExp(`\\b${escapeRegExp(functionName)}\\s*\\(`, "gu"))].length;
}

function countOccurrences(input: string, needle: string): number {
  return input.split(needle).length - 1;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
