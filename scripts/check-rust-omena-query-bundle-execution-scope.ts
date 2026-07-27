import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

const modelPath = "rust/crates/omena-transform-passes/src/model.rs";
const transformPath = "rust/crates/omena-query/src/style/transform.rs";
const queryTypesPath = "rust/crates/omena-query/src/types.rs";
const queryTestPath = "rust/crates/omena-query/src/tests/transform_facade.rs";
const napiPath = "rust/crates/omena-napi/src/lib.rs";
const wasmPath = "rust/crates/omena-wasm/src/lib.rs";
const declarationPath = "packages/css-build-adapter/index.d.ts";
const generatedDeclarationPath = "packages/css-build-adapter/bundler-host-contract.generated.d.ts";
const wireFixturePath = "rust/crates/omena-query/tests/fixtures/bundle-execution-scope-wire.json";
const adapterGatePath = "scripts/check-rust-omena-bundler-adapter-pass-authority.ts";
const adapterTestPath = "test/unit/css-build-adapter/css-build-adapter.test.ts";
const publicApiPath = "rust/crates/omena-query/tests/snapshots/public-api.txt";

const modelSource = readFileSync(modelPath, "utf8");
const transformSource = readFileSync(transformPath, "utf8");
const queryTypesSource = readFileSync(queryTypesPath, "utf8");
let queryTestSource = readFileSync(queryTestPath, "utf8");
let napiSource = readFileSync(napiPath, "utf8");
let wasmSource = readFileSync(wasmPath, "utf8");
let declarationSource = readFileSync(declarationPath, "utf8");
const generatedDeclarationSource = readFileSync(generatedDeclarationPath, "utf8");
const adapterGateSource = readFileSync(adapterGatePath, "utf8");
const adapterTestSource = readFileSync(adapterTestPath, "utf8");
const publicApiSource = readFileSync(publicApiPath, "utf8");
const wireFixture = JSON.parse(readFileSync(wireFixturePath, "utf8")) as Record<string, unknown>;

const injectDropField = process.argv.includes("--inject-drop-field");
const injectFlipScope = process.argv.includes("--inject-flip-scope");
const injectDuplicateCstBuild = process.argv.includes("--inject-duplicate-cst-build");
const injectDeclarationDrift = process.argv.includes("--inject-declaration-drift");
const injectCommentOnlyAssertion = process.argv.includes("--inject-comment-only-assertion");
const injectWireFieldRename = process.argv.includes("--inject-wire-field-rename");
const injectWireEnumRename = process.argv.includes("--inject-wire-enum-rename");
const injectWireOptionalNull = process.argv.includes("--inject-wire-optional-null");
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
const wireDispositions = asArray(wireFixture.sourceMapDispositions).map(asObject);
if (injectWireFieldRename) {
  const fallback = wireDispositions.at(-1);
  assert.notEqual(fallback, undefined);
  if (fallback) {
    fallback.segmentTotal = fallback.segmentCount;
    delete fallback.segmentCount;
  }
}
if (injectWireEnumRename) {
  const anchored = wireDispositions.at(0);
  assert.notEqual(anchored, undefined);
  if (anchored) anchored.granularity = "cst_anchors";
}
if (injectWireOptionalNull) {
  const anchored = wireDispositions.at(0);
  assert.notEqual(anchored, undefined);
  if (anchored) anchored.fallbackReason = null;
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

const fieldScopes = asArray(wireFixture.fieldScopes).map(asObject);
const moduleExecutions = asArray(wireFixture.moduleExecutions).map(asObject);
const bundleComposite = asObject(wireFixture.bundleComposite);
const entryModuleInstance = asObject(wireFixture.entryModuleInstance);
const wireSamplesByInterface = new Map<string, readonly Record<string, unknown>[]>([
  ["OmenaBundleExecutionScopeEvidenceV0", [wireFixture]],
  ["OmenaBundleExecutionFieldScopeV0", fieldScopes],
  ["OmenaBundleModuleExecutionByteFactsV0", moduleExecutions],
  ["OmenaBundleCompositeExecutionByteFactsV0", [bundleComposite]],
  ["OmenaLinkedSourceMapDispositionV0", wireDispositions],
  [
    "OmenaModuleInstanceKeyV0",
    [
      entryModuleInstance,
      ...moduleExecutions.map((sample) => asObject(sample.moduleInstance)),
      ...wireDispositions.map((sample) => asObject(sample.moduleInstance)),
    ],
  ],
]);
const localInterfaces = extractTypeScriptInterfaces(declarationSource);
const generatedInterfaces = extractTypeScriptInterfaces(generatedDeclarationSource);
const allInterfaces = new Map([...generatedInterfaces, ...localInterfaces]);
const wireReachableInterfaces = reachableTypeScriptInterfaces(
  allInterfaces,
  "OmenaBundleExecutionScopeEvidenceV0",
);
assert.deepEqual(
  [...wireSamplesByInterface.keys()].toSorted(),
  [...wireReachableInterfaces].toSorted(),
  "every TypeScript interface reachable from executionScope must have serialized wire samples",
);
for (const interfaceName of wireReachableInterfaces) {
  const body = allInterfaces.get(interfaceName);
  assert.notEqual(body, undefined, `missing TypeScript interface ${interfaceName}`);
  const fields = extractTypeScriptInterfaceFieldDefinitions(body ?? "");
  const samples = wireSamplesByInterface.get(interfaceName);
  assert.ok(samples && samples.length > 0, `${interfaceName} has no wire samples`);
  for (const sample of samples ?? []) {
    const sampleKeys = Object.keys(sample).toSorted();
    const declaredKeys = new Set(fields.map((field) => field.name));
    assert.deepEqual(
      sampleKeys.filter((key) => declaredKeys.has(key)),
      sampleKeys,
      `${interfaceName} wire sample has undeclared keys`,
    );
    for (const field of fields) {
      if (!(field.name in sample)) {
        assert.ok(field.optional, `${interfaceName}.${field.name} is missing from serialized wire`);
        continue;
      }
      assertTypeScriptValueShape(
        sample[field.name],
        field.typeExpression,
        `${interfaceName}.${field.name}`,
      );
    }
  }
  const wireKeys = new Set((samples ?? []).flatMap((sample) => Object.keys(sample)));
  assert.deepEqual(
    [...wireKeys].toSorted(),
    fields.map((field) => field.name).toSorted(),
    `${interfaceName} declaration must match the union of serialized wire keys`,
  );
  for (const field of fields.filter((candidate) => candidate.optional)) {
    assert.ok(
      (samples ?? []).some((sample) => !(field.name in sample)),
      `${interfaceName}.${field.name} optionality needs an omission witness`,
    );
  }
}
assert.deepEqual(
  [...new Set(fieldScopes.map((sample) => sample.scope))].toSorted(),
  extractTypeScriptStringUnion(declarationSource, "OmenaBundleExecutionFieldScopeV0", "scope"),
  "execution scope wire values must match the TypeScript union",
);
assert.deepEqual(
  [...new Set(wireDispositions.map((sample) => sample.granularity))].toSorted(),
  extractTypeScriptStringUnion(
    declarationSource,
    "OmenaLinkedSourceMapDispositionV0",
    "granularity",
  ),
  "source-map granularity wire values must match the TypeScript union",
);
assert.equal(
  fieldScopes.length,
  extractRustEnumVariants(queryTypesSource, "OmenaQueryExecutionEvidenceScopeV0").length,
  "wire fixture must exercise every Rust execution-scope variant",
);
assert.equal(
  wireDispositions.length,
  extractRustEnumVariants(queryTypesSource, "OmenaQueryLinkedSourceMapGranularityV0").length,
  "wire fixture must exercise every Rust source-map granularity variant",
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
runCargoTest("omena-query", "bundle_execution_scope_wire_matches_typescript_fixture");
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
      localTypeScriptInterfaceCount: localInterfaces.size,
      wireReachableInterfaceCount: wireReachableInterfaces.size,
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

function extractRustEnumVariants(input: string, typeName: string): string[] {
  const body = extractTypeBody(input, `enum ${typeName}`);
  return [...body.matchAll(/^\s*([A-Z][A-Za-z0-9]*),?\s*$/gmu)]
    .map((match) => lowerCamel(match[1]))
    .toSorted();
}

function extractRustStructFields(input: string, typeName: string): string[] {
  const body = extractTypeBody(input, `struct ${typeName}`);
  return [...body.matchAll(/^\s*(?:pub\s+)?([a-z][a-z0-9_]*):/gmu)]
    .map((match) => snakeToCamel(match[1]))
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

function extractTypeScriptInterfaces(input: string): Map<string, string> {
  const interfaces = new Map<string, string>();
  for (const match of input.matchAll(/export interface ([A-Z][A-Za-z0-9]*)/gu)) {
    const name = match[1];
    interfaces.set(name, extractTypeBody(input, `interface ${name}`));
  }
  return interfaces;
}

function extractTypeScriptInterfaceFieldDefinitions(
  body: string,
): { readonly name: string; readonly optional: boolean; readonly typeExpression: string }[] {
  return [...body.matchAll(/^\s*readonly\s+([a-z][A-Za-z0-9]*)(\?)?:\s*([^;]+);/gmu)].map(
    (match) => ({
      name: match[1],
      optional: match[2] === "?",
      typeExpression: match[3].trim(),
    }),
  );
}

function reachableTypeScriptInterfaces(
  interfaces: ReadonlyMap<string, string>,
  root: string,
): Set<string> {
  const reached = new Set<string>();
  const pending = [root];
  while (pending.length > 0) {
    const current = pending.pop();
    if (!current || reached.has(current)) continue;
    const body = interfaces.get(current);
    assert.notEqual(body, undefined, `missing TypeScript interface ${current}`);
    reached.add(current);
    for (const reference of body?.match(/\bOmena[A-Z][A-Za-z0-9]*V0\b/gu) ?? []) {
      if (interfaces.has(reference) && !reached.has(reference)) pending.push(reference);
    }
  }
  return reached;
}

function assertTypeScriptValueShape(
  value: unknown,
  rawTypeExpression: string,
  label: string,
): void {
  const typeExpression = rawTypeExpression.replace(/^readonly\s+/u, "").trim();
  if (typeExpression.endsWith("[]")) {
    assert.ok(Array.isArray(value), `${label} must serialize as an array`);
    return;
  }
  const stringLiterals = [...typeExpression.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
  if (stringLiterals.length > 0) {
    assert.equal(typeof value, "string", `${label} must serialize as a string`);
    assert.ok(stringLiterals.includes(value as string), `${label} has an undeclared wire value`);
    return;
  }
  if (typeExpression === "string") {
    assert.equal(typeof value, "string", `${label} must serialize as a string`);
    return;
  }
  if (typeExpression === "number") {
    assert.equal(typeof value, "number", `${label} must serialize as a number`);
    return;
  }
  if (typeExpression === "boolean") {
    assert.equal(typeof value, "boolean", `${label} must serialize as a boolean`);
    return;
  }
  if (/^Omena[A-Z][A-Za-z0-9]*V0$/u.test(typeExpression)) {
    assert.ok(
      typeof value === "object" && value !== null && !Array.isArray(value),
      `${label} must serialize as an object`,
    );
    return;
  }
  assert.fail(`${label} has unsupported TypeScript wire shape ${typeExpression}`);
}

function asObject(value: unknown): Record<string, unknown> {
  assert.ok(
    typeof value === "object" && value !== null && !Array.isArray(value),
    "wire fixture value must be an object",
  );
  return value as Record<string, unknown>;
}

function asArray(value: unknown): unknown[] {
  assert.ok(Array.isArray(value), "wire fixture value must be an array");
  return value;
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
