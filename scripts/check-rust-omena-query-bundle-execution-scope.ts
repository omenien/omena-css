import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const transformPath = "rust/crates/omena-query/src/style/transform.rs";
const queryTypesPath = "rust/crates/omena-query/src/types.rs";
const queryTestPath = "rust/crates/omena-query/src/tests/transform_facade.rs";
const napiPath = "rust/crates/omena-napi/src/lib.rs";
const wasmPath = "rust/crates/omena-wasm/src/lib.rs";
const declarationPath = "packages/css-build-adapter/index.d.ts";
const generatedDeclarationPath = "packages/css-build-adapter/bundler-host-contract.generated.d.ts";
const wireFixturePath = "rust/crates/omena-query/tests/fixtures/bundle-execution-scope-wire.json";
const wireCompatBaselinePath =
  "rust/crates/omena-query/tests/fixtures/bundle-execution-scope-wire-compat-baseline.json";
const bundleModuleWireKeyFixturePath =
  "rust/crates/omena-query/tests/fixtures/bundle-module-execution-wire-keys.json";
const foldContractPath =
  "rust/crates/omena-query/tests/fixtures/bundle-execution-fold-contract.json";
const executionWireFixtureDirectory =
  "rust/crates/omena-transform-passes/tests/fixtures/execution-summary-wire";
const adapterGatePath = "scripts/check-rust-omena-bundler-adapter-pass-authority.ts";
const adapterTestPath = "test/unit/css-build-adapter/css-build-adapter.test.ts";
const publicApiPath = "rust/crates/omena-query/tests/snapshots/public-api.txt";
const TRANSFORM_EXECUTION_SUMMARY_UNDECLARED_WIRE_KEYS = [
  "cascadeProofObligations",
  "closedWorldAdmission",
  "cssImportInlines",
  "cssModuleComposesExports",
  "cssModuleEvaluation",
  "decisions",
  "designTokenRoutes",
  "dischargeLedgerTelemetry",
  "inputByteLen",
  "moduleQualifiedShake",
  "mutationCount",
  "outputByteLen",
  "passPlan",
  "provenanceDerivationForest",
  "provenancePreserved",
  "semanticPreservationTelemetry",
  "semanticRemovals",
  "strictPolicy",
  "structuralIrTransactionTelemetry",
  "winnerEqualityObligations",
].toSorted();
const UNBOUND_LOCAL_TYPESCRIPT_INTERFACES = [
  "OmenaBuildAdapterBundleOptions",
  "OmenaBuildAdapterOptions",
  "OmenaBuildOutput",
  "OmenaBuildState",
  "OmenaBundleArtifactV0",
  "OmenaBundleBuildOutput",
  "OmenaBundleCodeSplitWorkspacePlanOutputV0",
  "OmenaBundleWithEvidenceV0",
  "OmenaConsumerBuildSummaryV0",
  "OmenaPackageManifestInput",
  "OmenaSourceMapV3V0",
  "OmenaStyleSourceInput",
  "OmenaTargetTransformOptionsV0",
  "OmenaTransformBundleSourceSummaryV0",
  "OmenaTransformExecutionContextV0",
].toSorted();

type WireKeySample = {
  readonly interfaceName: string;
  readonly sampleName: string;
  keys: string[];
  readonly wireTypes: Record<string, string>;
};

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
const wireCompatBaseline = JSON.parse(readFileSync(wireCompatBaselinePath, "utf8")) as Record<
  string,
  unknown
>;
const bundleModuleWireKeySample = asWireKeySample(
  JSON.parse(readFileSync(bundleModuleWireKeyFixturePath, "utf8")),
  "bundle-module-execution-wire-keys.json",
);
const foldContract = asObject(JSON.parse(readFileSync(foldContractPath, "utf8")));
const executionWireKeySamples = readdirSync(executionWireFixtureDirectory)
  .filter((fileName) => fileName.endsWith(".json"))
  .toSorted()
  .map((fileName) =>
    asWireKeySample(
      JSON.parse(readFileSync(join(executionWireFixtureDirectory, fileName), "utf8")),
      fileName,
    ),
  );

const injectDropField = process.argv.includes("--inject-drop-field");
const injectFlipScope = process.argv.includes("--inject-flip-scope");
const injectDuplicateCstBuild = process.argv.includes("--inject-duplicate-cst-build");
const injectDeclarationDrift = process.argv.includes("--inject-declaration-drift");
const injectCommentOnlyAssertion = process.argv.includes("--inject-comment-only-assertion");
const injectWireFieldRename = process.argv.includes("--inject-wire-field-rename");
const injectWireEnumRename = process.argv.includes("--inject-wire-enum-rename");
const injectWireOptionalNull = process.argv.includes("--inject-wire-optional-null");
const injectExecutionWireFieldRename = process.argv.includes(
  "--inject-execution-wire-field-rename",
);
const injectExecutionWireFieldAdd = process.argv.includes("--inject-execution-wire-field-add");
const injectExecutionRequiredOmission = process.argv.includes(
  "--inject-execution-required-omission",
);
const injectBundleAggregateFieldAdd = process.argv.includes("--inject-bundle-aggregate-field-add");
const injectBundleFoldRowDrop = process.argv.includes("--inject-bundle-fold-row-drop");
const injectBundleWitnessMissing = process.argv.includes("--inject-bundle-witness-missing");
const injectBundleCarrierAccountingDrop = process.argv.includes(
  "--inject-bundle-carrier-accounting-drop",
);
const injectBundleCarrierPartitionOverlap = process.argv.includes(
  "--inject-bundle-carrier-partition-overlap",
);
const injectBundleDuplicateByteAuthority = process.argv.includes(
  "--inject-bundle-duplicate-byte-authority",
);
const injectBundleWireFieldRename = process.argv.includes("--inject-bundle-wire-field-rename");
const injectBundleWireSampleLoss = process.argv.includes("--inject-bundle-wire-sample-loss");
const injectBundleCompatByteChange = process.argv.includes("--inject-bundle-compat-byte-change");
const injectBundleLegacyEmpty = process.argv.includes("--inject-bundle-legacy-empty");
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
const bundleExecution = asObject(wireFixture.bundleExecution);
const bundleEmissionExecution = asObject(bundleExecution.emissionExecution);
if (injectBundleWireFieldRename) {
  bundleExecution.aggregateMutationTotal = bundleExecution.aggregateMutationCount;
  delete bundleExecution.aggregateMutationCount;
}
if (injectBundleCompatByteChange) {
  wireFixture.product = "omena-query.bundle-execution-scope.changed";
}
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
const executionSummarySamples = executionWireKeySamples.filter(
  (sample) => sample.interfaceName === "OmenaTransformExecutionSummaryV0",
);
assert.ok(executionSummarySamples.length > 1, "execution summary needs conditional wire samples");
assert.deepEqual(
  executionSummarySamples.map((sample) => sample.sampleName).toSorted(),
  ["conditional", "unconditional"],
  "the authored transform-pass key-set authority must retain both serializer states",
);
if (injectExecutionWireFieldRename) {
  for (const sample of executionSummarySamples) {
    const fieldIndex = sample.keys.indexOf("mutationCount");
    assert.notEqual(fieldIndex, -1);
    sample.keys[fieldIndex] = "mutationTotal";
    sample.wireTypes.mutationTotal = sample.wireTypes.mutationCount;
    delete sample.wireTypes.mutationCount;
  }
}
if (injectExecutionWireFieldAdd) {
  for (const sample of executionSummarySamples) {
    sample.keys.push("unclassifiedExecutionField");
    sample.wireTypes.unclassifiedExecutionField = "boolean";
  }
}
if (injectExecutionRequiredOmission) {
  const sample = executionSummarySamples.at(0);
  assert.notEqual(sample, undefined);
  if (sample) {
    sample.keys = sample.keys.filter((key) => key !== "requestedPassIds");
    delete sample.wireTypes.requestedPassIds;
  }
}

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
assert.equal(new Set(scopeRows.map((row) => row.fieldName)).size, scopeRows.length);
assert.ok(scopeRows.every((row) => row.derivation.trim().length > 0));
if (injectFlipScope) {
  const outputCss = scopeRows.find((row) => row.fieldName === "outputCss");
  assert.notEqual(outputCss, undefined);
  if (outputCss) outputCss.scope = "Entry";
}

const executionWireKeys = new Set(executionSummarySamples.flatMap((sample) => sample.keys));
if (injectDropField) executionWireKeys.delete([...executionWireKeys].toSorted().at(-1) ?? "");

const foldRows = asArray(foldContract.foldRows).map(asObject);
const absentRows = asArray(foldContract.absentFields).map(asObject);
const summaryOwnedFields = asArray(foldContract.summaryOwnedFields).map((field) => String(field));
if (injectBundleFoldRowDrop) foldRows.pop();
if (injectBundleWitnessMissing && foldRows[0]) {
  foldRows[0].divergenceWitnessFixture = "missing_bundle_execution_divergence_witness";
}
if (injectBundleCarrierAccountingDrop) absentRows.pop();
if (injectBundleCarrierPartitionOverlap) {
  absentRows.push({ field: "mutationCount", reason: "reListingNotFold" });
}
const rustBundleSummaryFields = extractRustStructFields(
  queryTypesSource,
  "BundleExecutionSummaryV0",
);
const rustAggregateFields = rustBundleSummaryFields
  .filter((field) => field.startsWith("aggregate"))
  .toSorted();
if (injectBundleAggregateFieldAdd) rustAggregateFields.push("aggregateUnbackedCount");
const tableAggregateFields = foldRows.map((row) => String(row.field)).toSorted();
const allowedFoldTokens = new Set(["sum", "orderedUnion"]);
// FALSIFIER: id=bundle-execution-fold-row-shape class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=authored-table reentry=fold-contract-schema-change
assert.ok(
  foldRows.every(
    (row) =>
      typeof row.field === "string" &&
      typeof row.sourceField === "string" &&
      typeof row.fold === "string" &&
      allowedFoldTokens.has(row.fold) &&
      typeof row.meaning === "string" &&
      row.meaning.length > 0 &&
      typeof row.divergenceWitnessFixture === "string",
  ),
  "bundle execution fold rows must carry the complete authored contract",
);
// FALSIFIER: id=bundle-execution-aggregate-without-row class=accounting via=--inject-bundle-aggregate-field-add,--inject-bundle-fold-row-drop producer=can-fail owner=bundle-execution-contract entry=four-authored-folds
assert.deepEqual(
  rustAggregateFields,
  tableAggregateFields,
  `bundle aggregate fields without authored rows: ${rustAggregateFields
    .filter((field) => !tableAggregateFields.includes(field))
    .join(",")}; authored rows without fields: ${tableAggregateFields
    .filter((field) => !rustAggregateFields.includes(field))
    .join(",")}`,
);
for (const row of foldRows) {
  const witness = String(row.divergenceWitnessFixture);
  // FALSIFIER: id=bundle-execution-fold-witness-resolution class=liveness via=--inject-bundle-witness-missing producer=can-fail owner=bundle-execution-contract entry=product-run-witnesses
  assert.match(
    transformSource,
    new RegExp(`\\bfn\\s+${escapeRegExp(witness)}\\s*\\(`, "u"),
    `bundle fold witness does not resolve: ${witness}`,
  );
  const rustField = camelToSnake(String(row.field));
  const fieldOffset = queryTypesSource.indexOf(`pub ${rustField}:`);
  const declarationPrefix = queryTypesSource.slice(Math.max(0, fieldOffset - 180), fieldOffset);
  // FALSIFIER: id=bundle-execution-fold-doc-token class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=doc-table-token reentry=aggregate-field-doc-changes
  assert.match(
    declarationPrefix,
    new RegExp(`Fold: \`${escapeRegExp(String(row.fold))}\``, "u"),
    `${row.field} doc comment must name fold ${row.fold}`,
  );
}
const directFoldSourceFields = foldRows
  .map((row) => String(row.sourceField))
  .filter((field) => !field.includes("."));
const carrierClassificationCounts = new Map<string, number>();
for (const field of [
  ...summaryOwnedFields,
  ...directFoldSourceFields,
  ...absentRows.map((row) => String(row.field)),
]) {
  carrierClassificationCounts.set(field, (carrierClassificationCounts.get(field) ?? 0) + 1);
}
const multiplyClassifiedCarrierFields = [...carrierClassificationCounts]
  .filter(([, count]) => count > 1)
  .map(([field]) => field)
  .toSorted();
// FALSIFIER: id=bundle-execution-carrier-partition class=accounting via=--inject-bundle-carrier-partition-overlap producer=can-fail owner=bundle-execution-contract entry=exclusive-carrier-classification
assert.deepEqual(
  multiplyClassifiedCarrierFields,
  [],
  `execution carrier fields must have exactly one classification: ${multiplyClassifiedCarrierFields.join(",")}`,
);
const accountedCarrierFields = new Set([
  ...summaryOwnedFields,
  ...directFoldSourceFields,
  ...absentRows.map((row) => String(row.field)),
]);
// FALSIFIER: id=bundle-execution-carrier-totality class=accounting via=--inject-bundle-carrier-accounting-drop producer=can-fail owner=bundle-execution-contract entry=twenty-eight-carrier-fields
assert.deepEqual(
  [...accountedCarrierFields].toSorted(),
  [...executionWireKeys].toSorted(),
  "every execution carrier field must be summary-owned, directly folded, or absent with reason",
);
const allowedAbsentReasons = new Set([
  "noBundleMeaning",
  "reListingNotFold",
  "producerEntailedConstant",
  "ownedByByteAuthority",
  "noDivergenceWitness",
]);
// FALSIFIER: id=bundle-execution-absent-reason-vocabulary class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=closed-reason-vocabulary reentry=absent-reason-schema-change
assert.ok(
  absentRows.every(
    (row) => typeof row.field === "string" && allowedAbsentReasons.has(String(row.reason)),
  ),
  "bundle execution absent fields must use the closed reason vocabulary",
);
const absentReasonByField = new Map(
  absentRows.map((row) => [String(row.field), String(row.reason)]),
);
// FALSIFIER: id=bundle-execution-no-meaning-fields class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=plan-policy-forest-absent reentry=producer-gains-bundle-level-meaning
assert.deepEqual(
  ["passPlan", "provenanceDerivationForest", "strictPolicy"].map((field) => [
    field,
    absentReasonByField.get(field),
  ]),
  [
    ["passPlan", "noBundleMeaning"],
    ["provenanceDerivationForest", "noBundleMeaning"],
    ["strictPolicy", "noBundleMeaning"],
  ],
);
// FALSIFIER: id=bundle-execution-measured-absence-policy class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=pickup-measured-classification reentry=product-divergence-corpus-change
assert.deepEqual(
  ["plannedOnlyPassIds", "moduleQualifiedShake", "closedWorldAdmission"].map((field) => [
    field,
    absentReasonByField.get(field),
  ]),
  [
    ["plannedOnlyPassIds", "noBundleMeaning"],
    ["moduleQualifiedShake", "reListingNotFold"],
    ["closedWorldAdmission", "reListingNotFold"],
  ],
);

// A serialized field addition makes the computed domain larger; production
// fixtures can emit that state after any additive summary-field change.
assert.equal(
  scopeRows.length,
  executionWireKeys.size,
  "execution-scope rows must cover the serialized execution domain",
);
assert.deepEqual(
  scopeRows.map((row) => row.fieldName).toSorted(),
  [...executionWireKeys].toSorted(),
  "execution-scope table must classify every serialized execution field exactly once",
);
const bundleFields = scopeRows.filter((row) => row.scope === "Bundle").map((row) => row.fieldName);
const projectionBody = extractFunctionBody(transformSource, "project_linked_bundle_execution");
// This static census sees direct `execution.field = value` writes only. The
// runtime serialization fixtures remain load-bearing for clone, spread, or
// helper-based write styles that this expression cannot discover.
const projectedFields = [...projectionBody.matchAll(/execution\.([a-z][a-z0-9_]*)\s*=/gmu)].map(
  (match) => snakeToCamel(match[1]),
);
assert.deepEqual(
  bundleFields.toSorted(),
  projectedFields.toSorted(),
  "bundle-scoped fields must be derived from the linked execution projection",
);
const emissionExecutionFields = extractRustStructFields(
  queryTypesSource,
  "BundleEmissionExecutionV0",
);
if (injectBundleDuplicateByteAuthority) {
  emissionExecutionFields.push("materializedOutputByteLen");
}
const emissionExecutionBody = extractTypeBody(queryTypesSource, "struct BundleEmissionExecutionV0");
// FALSIFIER: id=bundle-execution-single-byte-authority class=accounting via=--inject-bundle-duplicate-byte-authority producer=can-fail owner=bundle-execution-contract entry=regions-and-counts-only
assert.ok(
  emissionExecutionFields.every((field) => !/(?:css|byte)/iu.test(field)) &&
    !/\bString\b/u.test(emissionExecutionBody),
  "bundle emission execution must not duplicate CSS or byte-total authority",
);
const compatProjection = structuredClone(wireFixture);
delete compatProjection.bundleExecution;
// FALSIFIER: id=bundle-execution-additive-wire-values class=accounting via=--inject-bundle-compat-byte-change producer=can-fail owner=bundle-execution-contract entry=existing-wire-byte-identical
assert.deepEqual(
  compatProjection,
  wireCompatBaseline,
  "existing execution-scope wire values must remain byte-compatible",
);
// FALSIFIER: id=bundle-execution-additive-wire-key class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=single-additive-root-key reentry=scope-evidence-wire-expands
assert.deepEqual(
  Object.keys(wireFixture)
    .filter((key) => !(key in wireCompatBaseline))
    .toSorted(),
  ["bundleExecution"],
  "bundle execution must be the only additive execution-scope root key",
);
const bundleRunBody = extractFunctionBody(
  transformSource,
  "run_omena_query_bundle_with_optional_module_reachability",
);
const legacyExecutionScopeAbsent =
  /OmenaQueryBundleEmissionPathV0::ImportInlineLegacy\s*=>[\s\S]*?summary\.execution,\s*None,\s*OmenaQueryBundleEmissionPathV0::ImportInlineLegacy,\s*None,\s*None,/u.test(
    bundleRunBody,
  ) && !injectBundleLegacyEmpty;
// FALSIFIER: id=bundle-execution-legacy-absence class=placement via=--inject-bundle-legacy-empty producer=can-fail owner=bundle-execution-contract entry=legacy-scope-absent
assert.equal(
  legacyExecutionScopeAbsent,
  true,
  "legacy emission must not claim an empty linked bundle aggregate",
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
  [
    queryTestPath,
    queryTestSource,
    "linked_bundle_and_consumer_build_share_source_identity_but_not_emission_segments",
  ],
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
  if (path !== queryTestPath) {
    // FALSIFIER: id=bundle-execution-client-reachability class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=napi-wasm-product-tests reentry=public-client-test-shape-changes
    assert.match(testBody, /bundle_execution/u);
  }
}

const fieldScopes = asArray(wireFixture.fieldScopes).map(asObject);
const moduleExecutions = asArray(wireFixture.moduleExecutions).map(asObject);
const bundleComposite = asObject(wireFixture.bundleComposite);
const entryModuleInstance = asObject(wireFixture.entryModuleInstance);
const moduleRegionSamples = asArray(bundleEmissionExecution.moduleRegions).map(asObject);
const orderEntryRegionSamples = asArray(bundleEmissionExecution.orderEntryRegions).map(asObject);
const bundleWireSamplesByInterface = new Map<string, readonly Record<string, unknown>[]>([
  ["OmenaBundleExecutionScopeEvidenceV0", [wireFixture]],
  ["OmenaBundleExecutionFieldScopeV0", fieldScopes],
  ["OmenaBundleModuleExecutionByteFactsV0", moduleExecutions],
  ["OmenaBundleCompositeExecutionByteFactsV0", [bundleComposite]],
  ["OmenaBundleExecutionSummaryV0", [bundleExecution]],
  ["OmenaBundleEmissionExecutionV0", [bundleEmissionExecution]],
  ["OmenaLinkedEmissionModuleRegionV0", moduleRegionSamples],
  ["OmenaLinkedEmissionOrderEntryRegionV0", orderEntryRegionSamples],
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
if (injectBundleWireSampleLoss) {
  bundleWireSamplesByInterface.delete("OmenaBundleExecutionSummaryV0");
}
const localInterfaces = extractTypeScriptInterfaces(declarationSource);
const generatedInterfaces = extractTypeScriptInterfaces(generatedDeclarationSource);
const allInterfaces = new Map([...generatedInterfaces, ...localInterfaces]);
const wireReachableInterfaces = new Set(
  [
    ...reachableTypeScriptInterfaces(allInterfaces, "OmenaBundleExecutionScopeEvidenceV0"),
    ...reachableTypeScriptInterfaces(allInterfaces, "OmenaTransformExecutionSummaryV0"),
  ].toSorted(),
);
const keySamplesByInterface = groupWireKeySamplesByInterface([
  ...executionWireKeySamples,
  bundleModuleWireKeySample,
]);
// FALSIFIER: id=bundle-execution-reachable-sample-closure class=accounting via=--inject-bundle-wire-sample-loss producer=can-fail owner=bundle-execution-contract entry=computed-reachable-interface-set
assert.deepEqual(
  [
    ...new Set([...bundleWireSamplesByInterface.keys(), ...keySamplesByInterface.keys()]),
  ].toSorted(),
  [...wireReachableInterfaces].toSorted(),
  "every TypeScript interface reachable from executionScope must have serialized wire samples",
);
for (const interfaceName of wireReachableInterfaces) {
  const body = allInterfaces.get(interfaceName);
  assert.notEqual(body, undefined, `missing TypeScript interface ${interfaceName}`);
  const fields = extractTypeScriptInterfaceFieldDefinitions(body ?? "");
  const valueSamples = bundleWireSamplesByInterface.get(interfaceName);
  const keySamples = keySamplesByInterface.get(interfaceName);
  // FALSIFIER: id=bundle-execution-wire-sample-nonempty class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=reachable-sample-map-entry reentry=sample-registration-representation-changes
  assert.ok(
    (valueSamples?.length ?? 0) + (keySamples?.length ?? 0) > 0,
    `${interfaceName} has no wire samples`,
  );
  if (keySamples) {
    const declaredKeys = new Set(fields.map((field) => field.name));
    const wireKeys = new Set(keySamples.flatMap((sample) => sample.keys));
    for (const sample of keySamples) {
      assert.deepEqual(
        Object.keys(sample.wireTypes).toSorted(),
        sample.keys.toSorted(),
        `${interfaceName}.${sample.sampleName} wire types must cover its serialized keys`,
      );
      for (const field of fields) {
        if (!sample.keys.includes(field.name)) {
          assert.ok(
            field.optional,
            `${interfaceName}.${field.name} is missing from serialized wire`,
          );
          continue;
        }
        assertTypeScriptWireType(
          sample.wireTypes[field.name],
          field.typeExpression,
          `${interfaceName}.${field.name}`,
        );
      }
    }
    const undeclaredWireKeys = [...wireKeys].filter((key) => !declaredKeys.has(key)).toSorted();
    const expectedResidual =
      interfaceName === "OmenaTransformExecutionSummaryV0"
        ? TRANSFORM_EXECUTION_SUMMARY_UNDECLARED_WIRE_KEYS
        : [];
    // A newly serialized but undeclared key makes this false; serde output can
    // produce it independently of the TypeScript declaration.
    assert.deepEqual(
      undeclaredWireKeys,
      expectedResidual,
      `${interfaceName} undeclared wire-key residual changed`,
    );
    for (const field of fields.filter((candidate) => candidate.optional)) {
      // Removing the omission witness makes this false; serde can emit both
      // states only when a production sample actually exercises the field.
      assert.ok(
        keySamples.some((sample) => !sample.keys.includes(field.name)),
        `${interfaceName}.${field.name} optionality needs an omission witness`,
      );
    }
    continue;
  }
  for (const sample of valueSamples ?? []) {
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
  const valueWireKeys = new Set((valueSamples ?? []).flatMap((sample) => Object.keys(sample)));
  assert.deepEqual(
    [...valueWireKeys].toSorted(),
    fields.map((field) => field.name).toSorted(),
    `${interfaceName} declaration must match the union of serialized wire keys`,
  );
  for (const field of fields.filter((candidate) => candidate.optional)) {
    assert.ok(
      (valueSamples ?? []).some((sample) => !(field.name in sample)),
      `${interfaceName}.${field.name} optionality needs an omission witness`,
    );
  }
}
const unboundLocalInterfaces = [...localInterfaces.keys()]
  .filter((interfaceName) => !wireReachableInterfaces.has(interfaceName))
  .toSorted();
// Adding a local declaration or a reference from either wire root changes this
// residual, and both changes are valid declaration-author inputs.
assert.deepEqual(
  unboundLocalInterfaces,
  UNBOUND_LOCAL_TYPESCRIPT_INTERFACES,
  "the named local TypeScript interface residual changed",
);
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
assert.match(stripRustComments(adapterGateSource), /executionScope: null/u);
assert.match(stripRustComments(adapterTestSource), /executionScope/u);

for (const surface of [
  "BundleExecutionSummaryV0",
  "BundleModuleExecutionV0",
  "BundleEmissionExecutionV0",
  "OmenaQueryBundleExecutionScopeEvidenceV0",
  "OmenaQueryBundleExecutionScopeResultV0",
  "run_omena_query_bundle_with_execution_scope_evidence_and_options",
]) {
  assert.ok(publicApiSource.includes(surface), `public API snapshot is missing ${surface}`);
}

// The declaration/sample checks above establish consistency. This production
// serializer test is the separate load-bearing growth and omission authority;
// its position in this process is not part of the contract.
runCargoTest(
  "omena-transform-passes",
  "module_qualified_tree_shake_distinguishes_same_name_owners",
);
runCargoTest("omena-query", "linked_bundle_retains_each_module_execution_before_bundle_projection");
runCargoTest("omena-query", "bundle_execution_scope_wire_matches_typescript_fixture");
runCargoTest("omena-query", "linked_bundle_source_map_");
runCargoTest(
  "omena-query",
  "linked_bundle_and_consumer_build_share_source_identity_but_not_emission_segments",
);
runCargoTest("omena-napi", "bundles_workspace_sources_for_node_clients");
runCargoTest("omena-wasm", "bundles_workspace_sources_for_browser_clients");
runCargoTest("omena-cli", "bundle_command_loads_configured_workspace_sources_without_flags");

console.log(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-query.bundle-execution-scope-gate",
      executionWireFieldCount: executionWireKeys.size,
      scopeRowCount: scopeRows.length,
      entryFieldCount: scopeRows.filter((row) => row.scope === "Entry").length,
      bundleFieldCount: bundleFields.length,
      projectedBundleFieldCount: projectedFields.length,
      bundleAggregateFieldCount: rustAggregateFields.length,
      bundleFoldRowCount: foldRows.length,
      bundleAbsentFieldCount: absentRows.length,
      additiveExecutionScopeWireKeyCount: Object.keys(wireFixture).filter(
        (key) => !(key in wireCompatBaseline),
      ).length,
      retainedExecutionProducerCount,
      perModuleCstBuildCount: cstBuildCount,
      localTypeScriptInterfaceCount: localInterfaces.size,
      wireReachableInterfaceCount: wireReachableInterfaces.size,
      unboundLocalTypeScriptInterfaceCount: unboundLocalInterfaces.length,
      executionSummaryUndeclaredWireKeyCount:
        TRANSFORM_EXECUTION_SUMMARY_UNDECLARED_WIRE_KEYS.length,
      executionSummaryKeyAuthority:
        "omena-transform-passes/execution-summary-wire/conditional+unconditional",
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
  if (typeExpression === "number | null") {
    // FALSIFIER: id=bundle-execution-nullable-number-wire-shape class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=nullable-region-order-index reentry=nullable-number-wire-representation-changes
    assert.ok(
      value === null || typeof value === "number",
      `${label} must serialize as a number or null`,
    );
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

function assertTypeScriptWireType(
  wireType: string | undefined,
  rawTypeExpression: string,
  label: string,
): void {
  const typeExpression = rawTypeExpression.replace(/^readonly\s+/u, "").trim();
  let expectedWireType: string;
  if (typeExpression.endsWith("[]")) {
    expectedWireType = "array";
  } else if ([...typeExpression.matchAll(/"([^"]+)"/gu)].length > 0) {
    expectedWireType = "string";
  } else if (typeExpression === "string") {
    expectedWireType = "string";
  } else if (typeExpression === "number" || typeExpression === "number | null") {
    expectedWireType = "number";
  } else if (typeExpression === "boolean") {
    expectedWireType = "boolean";
  } else if (/^Omena[A-Z][A-Za-z0-9]*V0$/u.test(typeExpression)) {
    expectedWireType = "object";
  } else {
    assert.fail(`${label} has unsupported TypeScript wire shape ${typeExpression}`);
  }
  assert.equal(wireType, expectedWireType, `${label} has the wrong serialized wire type`);
}

function asWireKeySample(value: unknown, fileName: string): WireKeySample {
  const sample = asObject(value);
  assert.equal(sample.schemaVersion, "0", `${fileName} has the wrong schema version`);
  // FALSIFIER: id=bundle-execution-wire-sample-product class=structuralEntailment via=STRUCTURAL producer=entailed owner=bundle-execution-contract entry=two-authoritative-key-sample-producers reentry=wire-key-sample-product-expands
  assert.ok(
    sample.product === "omena-transform-passes.wire-key-sample" ||
      sample.product === "omena-query.bundle-execution-wire-key-sample",
    `${fileName} has the wrong product`,
  );
  assert.equal(typeof sample.interfaceName, "string", `${fileName} lacks interfaceName`);
  assert.equal(typeof sample.sampleName, "string", `${fileName} lacks sampleName`);
  const keys = asArray(sample.keys);
  assert.ok(
    keys.every((key) => typeof key === "string"),
    `${fileName} has a non-string key`,
  );
  const wireTypes = asObject(sample.wireTypes);
  assert.ok(
    Object.values(wireTypes).every(
      (wireType) =>
        typeof wireType === "string" &&
        ["array", "boolean", "null", "number", "object", "string"].includes(wireType),
    ),
    `${fileName} has an unknown wire type`,
  );
  return {
    interfaceName: sample.interfaceName as string,
    sampleName: sample.sampleName as string,
    keys: (keys as string[]).toSorted(),
    wireTypes: wireTypes as Record<string, string>,
  };
}

function groupWireKeySamplesByInterface(
  samples: readonly WireKeySample[],
): Map<string, WireKeySample[]> {
  const grouped = new Map<string, WireKeySample[]>();
  for (const sample of samples) {
    const interfaceSamples = grouped.get(sample.interfaceName) ?? [];
    interfaceSamples.push(sample);
    grouped.set(sample.interfaceName, interfaceSamples);
  }
  return grouped;
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

function camelToSnake(value: string): string {
  return value.replace(/[A-Z]/gu, (character) => `_${character.toLowerCase()}`);
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
