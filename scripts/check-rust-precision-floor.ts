import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function rustSources(relativeDirectory: string): string[] {
  const directory = path.join(repoRoot, relativeDirectory);
  return evidenceScanSurface
    .readdirSync(directory, { recursive: true, encoding: "utf8" })
    .filter((entry) => entry.endsWith(".rs"))
    .map((entry) => fs.readFileSync(path.join(directory, entry), "utf8"));
}

interface RustSourceEntry {
  readonly relativePath: string;
  readonly source: string;
}

function rustSourceEntries(relativeDirectory: string): RustSourceEntry[] {
  const directory = path.join(repoRoot, relativeDirectory);
  return evidenceScanSurface
    .readdirSync(directory, { recursive: true, encoding: "utf8" })
    .filter((entry) => entry.endsWith(".rs"))
    .map((entry) => ({
      relativePath: path.posix.join(relativeDirectory, entry.replaceAll(path.sep, "/")),
      source: fs.readFileSync(path.join(directory, entry), "utf8"),
    }));
}

function productionRustSource(entry: RustSourceEntry): string | undefined {
  if (entry.relativePath.endsWith("/tests.rs") || entry.relativePath.includes("/tests/")) {
    return undefined;
  }
  const characters = entry.source.split("");
  const pattern = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+[A-Za-z0-9_]+\s*\{/gu;
  for (const match of entry.source.matchAll(pattern)) {
    const open = entry.source.indexOf("{", match.index);
    let depth = 1;
    let cursor = open + 1;
    while (cursor < entry.source.length && depth > 0) {
      if (entry.source[cursor] === "{") depth += 1;
      if (entry.source[cursor] === "}") depth -= 1;
      cursor += 1;
    }
    for (let index = match.index; index < cursor; index += 1) {
      if (characters[index] !== "\n" && characters[index] !== "\r") characters[index] = " ";
    }
  }
  return characters.join("");
}

interface StructLiteralEntry {
  readonly body: string;
  readonly start: number;
}

function structLiteralEntries(source: string, typeName: string): StructLiteralEntry[] {
  const entries: StructLiteralEntry[] = [];
  const pattern = new RegExp(`\\b${typeName}\\s*\\{`, "gu");
  for (const match of source.matchAll(pattern)) {
    const open = source.indexOf("{", match.index);
    let depth = 1;
    let cursor = open + 1;
    while (cursor < source.length && depth > 0) {
      if (source[cursor] === "{") depth += 1;
      if (source[cursor] === "}") depth -= 1;
      cursor += 1;
    }
    assert.equal(depth, 0, `unterminated ${typeName} literal`);
    entries.push({ body: source.slice(open + 1, cursor - 1), start: match.index });
  }
  return entries;
}

function structLiteralBodies(source: string, typeName: string): string[] {
  return structLiteralEntries(source, typeName).map((entry) => entry.body);
}

function enclosingFunctionName(source: string, offset: number): string {
  const prefix = source.slice(0, offset);
  const functions = [
    ...prefix.matchAll(/(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?fn\s+([A-Za-z0-9_]+)/gu),
  ];
  const name = functions.at(-1)?.[1];
  assert.ok(
    name !== undefined,
    `precision constructor at byte ${offset} must be inside a function`,
  );
  return name;
}

function normalizedExpression(expression: string): string {
  return expression.replaceAll(/\s+/gu, "");
}

function topLevelFieldExpressions(body: string): ReadonlyMap<string, string> {
  const fields = new Map<string, string>();
  let start = 0;
  let round = 0;
  let square = 0;
  let curly = 0;
  const segments: string[] = [];
  for (let index = 0; index <= body.length; index += 1) {
    const character = body[index];
    if (character === "(") round += 1;
    if (character === ")") round -= 1;
    if (character === "[") square += 1;
    if (character === "]") square -= 1;
    if (character === "{") curly += 1;
    if (character === "}") curly -= 1;
    if (
      (character === "," || index === body.length) &&
      round === 0 &&
      square === 0 &&
      curly === 0
    ) {
      segments.push(body.slice(start, index).trim());
      start = index + 1;
    }
  }
  for (const segment of segments) {
    const match = segment.match(/^([a-z_]+)\s*:\s*([\s\S]+)$/u);
    if (match?.[1] && match[2]) fields.set(match[1], match[2].trim());
  }
  return fields;
}

function blockBody(source: string, marker: string): string {
  const start = source.indexOf(marker);
  assert.ok(start >= 0, `missing ${marker}`);
  const open = source.indexOf("{", start);
  assert.ok(open >= 0, `missing body for ${marker}`);
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(open + 1, index);
  }
  throw new Error(`unterminated body for ${marker}`);
}

function topLevelEnumVariants(source: string, enumName: string): string[] {
  const body = blockBody(source, `pub enum ${enumName}`);
  const variants: string[] = [];
  let depth = 0;
  for (const line of body.split("\n")) {
    const trimmed = line.trim();
    if (depth === 0) {
      const match = trimmed.match(/^([A-Z][A-Za-z0-9]*)\b/u);
      if (match?.[1]) variants.push(match[1]);
    }
    depth += [...line].filter((char) => char === "{").length;
    depth -= [...line].filter((char) => char === "}").length;
  }
  return [...new Set(variants)];
}

function callBodies(source: string, marker: string): string[] {
  const bodies: string[] = [];
  let searchFrom = 0;
  while (true) {
    const start = source.indexOf(marker, searchFrom);
    if (start < 0) return bodies;
    const open = source.indexOf("(", start);
    let depth = 0;
    for (let index = open; index < source.length; index += 1) {
      if (source[index] === "(") depth += 1;
      if (source[index] === ")") depth -= 1;
      if (depth === 0) {
        bodies.push(source.slice(open + 1, index));
        searchFrom = index + 1;
        break;
      }
    }
  }
}

const abstractTypes = read("rust/crates/omena-abstract-value/src/types.rs");
const abstractDomain = read("rust/crates/omena-abstract-value/src/domain.rs");
const evidenceGraph = read("rust/crates/omena-evidence-graph/src/lib.rs");
const queryCore = read("rust/crates/omena-query-core/src/lib.rs");
const queryTypes = read("rust/crates/omena-query/src/types.rs");
const queryTransform = read("rust/crates/omena-query/src/style/transform.rs");
const queryTransformContext = read("rust/crates/omena-query/src/style/transform/context.rs");
const checkerFixSafety = read("rust/crates/omena-checker/src/fix_safety.rs");
const bridgeStyleIntelligence = read("rust/crates/omena-bridge/src/style_intelligence.rs");
const transformModel = read("rust/crates/omena-transform-passes/src/model.rs");
const transformExecutor = read("rust/crates/omena-transform-passes/src/runtime/executor.rs");
const transformTreeShakeTests = read(
  "rust/crates/omena-transform-passes/src/tests/tree_shake_classes.rs",
);
const precisionCalibration = JSON.parse(read("rust/omena-precision-calibration-report.json")) as {
  readonly cases: readonly {
    readonly precisionLabelDrops?: readonly {
      readonly output: string;
      readonly before: string;
      readonly after: string;
      readonly loweringAxis: string;
    }[];
  }[];
  readonly differentialSweep: {
    readonly baselinePin: string;
    readonly method: string;
    readonly baselineObservations: readonly {
      readonly output: string;
      readonly before: string;
    }[];
    readonly rows: readonly {
      readonly caseId: string;
      readonly output: string;
      readonly before: string;
      readonly after: string;
      readonly loweringAxes: readonly string[];
      readonly disposition: "disclosed" | "reverted";
      readonly semverIntentId: string | null;
    }[];
  };
};
interface DirectLiteralJustification {
  readonly sourcePath: string;
  readonly constructor: string;
  readonly field: string;
  readonly expression: string;
  readonly reason: string;
}
interface ProducerGateArm {
  readonly id: string;
  readonly axis: string;
  readonly producerPath: string;
  readonly observed: string;
  readonly effective: string;
  readonly requiredFloor: string;
  readonly gateOpen: boolean;
}
const precisionAuthority = JSON.parse(read("rust/omena-precision-floor-authority.json")) as {
  readonly schemaVersion: string;
  readonly executableManifestTest: string;
  readonly directLiteralJustifications: readonly DirectLiteralJustification[];
  readonly producerGateArms: readonly ProducerGateArm[];
  readonly mutationProbes: readonly {
    readonly id: string;
    readonly sourcePath: string;
    readonly from: string;
    readonly to: string;
    readonly command: readonly string[];
    readonly expectedFailure: string;
  }[];
};
const semverIntent = JSON.parse(read("rust/omena-rust-semver-intent.json")) as {
  readonly intents: readonly {
    readonly crate: string;
    readonly expectedRuntimeValueChanges?: readonly { readonly id: string }[];
  }[];
};

const factPrecisionVariants = topLevelEnumVariants(abstractTypes, "FactPrecision");
assert.deepEqual(factPrecisionVariants, ["Exact", "Conservative", "Heuristic", "Unknown"]);

const classValueVariants = topLevelEnumVariants(abstractTypes, "AbstractClassValueV0");
const classValueAdapter = blockBody(
  abstractDomain,
  "pub fn analysis_precision_from_class_value_with_witness",
);
const mappedClassValueVariants = [
  ...new Set(
    [...classValueAdapter.matchAll(/AbstractClassValueV0::([A-Z][A-Za-z0-9]*)/gu)].map(
      (match) => match[1],
    ),
  ),
].toSorted();
assert.deepEqual(mappedClassValueVariants, classValueVariants.toSorted());
assert.ok(!/(^|[^\w])_\s*=>/u.test(classValueAdapter), "class-value adapter must not catch all");

const precisionAxisTypes = [
  "ValueDomainPrecisionV1",
  "FlowPrecisionV1",
  "ContextPrecisionV1",
  "ProviderCompletenessV1",
  "WorldAssumptionV1",
  "RevisionIdentityV1",
] as const;
for (const axisType of precisionAxisTypes) {
  assert.ok(
    evidenceGraph.includes(`pub enum ${axisType}`),
    `${axisType} must be owned by omena-evidence-graph`,
  );
  const declarations = rustSources("rust/crates").reduce(
    (count, source) =>
      count + [...source.matchAll(new RegExp(`pub enum ${axisType}\\s*\\{`, "gu"))].length,
    0,
  );
  assert.equal(declarations, 1, `${axisType} must have one authority`);
}
const analysisPrecision = blockBody(evidenceGraph, "pub struct AnalysisPrecisionV1");
for (const [field, axisType] of [
  ["value_domain", "ValueDomainPrecisionV1"],
  ["flow", "FlowPrecisionV1"],
  ["context", "ContextPrecisionV1"],
  ["provider_completeness", "ProviderCompletenessV1"],
  ["world_assumption", "WorldAssumptionV1"],
  ["revision", "RevisionIdentityV1"],
] as const) {
  assert.match(analysisPrecision, new RegExp(`pub ${field}: ${axisType}`, "u"));
}
const analysisPrecisionImpl = blockBody(evidenceGraph, "impl AnalysisPrecisionV1");
const effectivePrecision = blockBody(
  analysisPrecisionImpl,
  "pub const fn effective_precision(self)",
);
for (const field of [
  "value_domain",
  "flow",
  "context",
  "provider_completeness",
  "world_assumption",
  "revision",
]) {
  assert.ok(effectivePrecision.includes(`self.${field}`), `meet must consume ${field}`);
}
assert.ok(effectivePrecision.includes(".meet("), "effective precision must be a meet");
assert.ok(
  evidenceGraph.includes("precision_meet_obeys_lattice_laws_and_unknown_absorbs"),
  "lattice laws must stay executable",
);

const precisionConsumerCrates = [
  "omena-abstract-value",
  "omena-query-core",
  "omena-query",
  "omena-bridge",
  "omena-checker",
  "omena-transform-passes",
] as const;
const precisionAuthorityAndConsumerEntries = [
  ...rustSourceEntries("rust/crates/omena-evidence-graph/src"),
  ...precisionConsumerCrates.flatMap((crateName) =>
    rustSourceEntries(`rust/crates/${crateName}/src`),
  ),
];
const productionPrecisionEntries = precisionAuthorityAndConsumerEntries.flatMap((entry) => {
  const source = productionRustSource(entry);
  return source === undefined ? [] : [{ ...entry, source }];
});
const precisionProducerSources = productionPrecisionEntries.map((entry) => entry.source);
for (const crateName of precisionConsumerCrates) {
  assert.ok(
    productionPrecisionEntries.some((entry) =>
      entry.relativePath.startsWith(`rust/crates/${crateName}/src/`),
    ),
    `precision producer/consumer census must include ${crateName}`,
  );
}
const typedProducerValueDomains = new Set(
  precisionProducerSources.flatMap((source) =>
    [...source.matchAll(/value_domain:\s*ValueDomainPrecisionV1::([A-Z][A-Za-z0-9]*)/gu)].map(
      (match) => match[1]!,
    ),
  ),
);
assert.ok(typedProducerValueDomains.size > 0, "typed value-domain producers must be non-vacuous");
const precisionAxisFields = [
  "value_domain",
  "flow",
  "context",
  "provider_completeness",
  "world_assumption",
  "revision",
] as const;
const analysisPrecisionConstructors = productionPrecisionEntries.flatMap((entry) =>
  structLiteralEntries(entry.source, "AnalysisPrecisionV1")
    .map(({ body, start }) => ({
      sourcePath: entry.relativePath,
      constructor: enclosingFunctionName(entry.source, start),
      fields: topLevelFieldExpressions(body),
    }))
    .filter((constructor) => constructor.fields.size > 0),
);
const analysisPrecisionAxisAssignments = analysisPrecisionConstructors.flatMap((constructor) =>
  precisionAxisFields.flatMap((field) => {
    const expression = constructor.fields.get(field);
    return expression === undefined ? [] : [{ ...constructor, field, expression }];
  }),
);
const typedJustifiedAxisAssignments = analysisPrecisionAxisAssignments.filter(({ expression }) =>
  /^(?:ValueDomainPrecisionV1|FlowPrecisionV1|ContextPrecisionV1|ProviderCompletenessV1|WorldAssumptionV1|RevisionIdentityV1)::[A-Z][A-Za-z0-9]*$/u.test(
    expression,
  ),
);
assert.equal(precisionAuthority.schemaVersion, "1");
const authorizedDirectLiterals = precisionAuthority.directLiteralJustifications.map(
  ({ sourcePath, constructor, field, expression, reason }) => {
    assert.ok(reason.trim().length > 0, `${sourcePath}:${constructor}.${field} needs a reason`);
    return {
      sourcePath,
      constructor,
      field,
      expression: normalizedExpression(expression),
    };
  },
);
const discoveredDirectLiterals = typedJustifiedAxisAssignments.map(
  ({ sourcePath, constructor, field, expression }) => ({
    sourcePath,
    constructor,
    field,
    expression: normalizedExpression(expression),
  }),
);
const directLiteralSort = (entry: {
  readonly sourcePath: string;
  readonly constructor: string;
  readonly field: string;
  readonly expression: string;
}) => `${entry.sourcePath}\u0000${entry.constructor}\u0000${entry.field}\u0000${entry.expression}`;
assert.deepEqual(
  discoveredDirectLiterals.toSorted((left, right) =>
    directLiteralSort(left).localeCompare(directLiteralSort(right)),
  ),
  authorizedDirectLiterals.toSorted((left, right) =>
    directLiteralSort(left).localeCompare(directLiteralSort(right)),
  ),
  "production direct literal assignments must have a bijective direct literal justification authority",
);
const derivedAxisAssignments = analysisPrecisionAxisAssignments.filter(
  (assignment) => !typedJustifiedAxisAssignments.includes(assignment),
);
assert.ok(
  analysisPrecisionConstructors.length > 0,
  "precision constructor census must be non-vacuous",
);
assert.equal(
  typedJustifiedAxisAssignments.length + derivedAxisAssignments.length,
  analysisPrecisionAxisAssignments.length,
  "every production typed-axis assignment must be classified",
);
assert.ok(
  analysisPrecisionAxisAssignments.some((assignment) =>
    assignment.sourcePath.includes("/omena-bridge/"),
  ),
  "producer census must measure bridge metadata assignments",
);
assert.equal(
  analysisPrecisionAxisAssignments.some(
    (assignment) =>
      assignment.sourcePath.includes("/omena-checker/") &&
      !assignment.sourcePath.includes("/tests/"),
  ),
  false,
  "omena-checker consumes precision but must not become a production axis producer",
);
function literalConstantAxisAssignments(sources: readonly string[]): string[] {
  const pattern =
    /(?:value_domain|flow(?:_sensitivity)?|context(?:_sensitivity)?|provider_completeness|world_assumption|revision(?:_axis)?):\s*(?:String::from\s*\(\s*)?"[^"]+"/gu;
  return sources.flatMap((source) => [...source.matchAll(pattern)].map((match) => match[0]));
}
const precisionIdentitySources = [
  evidenceGraph,
  abstractDomain,
  queryCore,
  queryTypes,
  bridgeStyleIntelligence,
  checkerFixSafety,
  ...rustSources("rust/crates/omena-query/src"),
];
const literalAxisAssignments = literalConstantAxisAssignments(precisionIdentitySources);
assert.deepEqual(
  literalAxisAssignments,
  [],
  "precision axes must be computed by typed producers, not assigned literal labels",
);
assert.equal(
  literalConstantAxisAssignments(['flow_sensitivity: "incrementalDataflow"']).length,
  1,
  "the literal-axis falsifier must be detected",
);
function stringKeyedPrecisionDerivations(sources: readonly string[]): string[] {
  const patterns = [
    /OMENA_QUERY_ANALYSIS_FACT_PRECISION_BY_VALUE_DOMAIN/gu,
    /source_diagnostic_precision\(\s*"/gu,
    /(?:value_domain|flow_sensitivity|context_sensitivity|revision_axis):\s*(?:String|"[^"]+")/gu,
    /precision\.value_domain\s*==/gu,
  ];
  return sources.flatMap((source) =>
    patterns.flatMap((pattern) => [...source.matchAll(pattern)].map((match) => match[0])),
  );
}
const stringKeyedDerivations = stringKeyedPrecisionDerivations(precisionIdentitySources);
assert.deepEqual(
  stringKeyedDerivations,
  [],
  "precision identity and gates must not use String axes",
);
assert.equal(
  stringKeyedPrecisionDerivations([
    "const OMENA_QUERY_ANALYSIS_FACT_PRECISION_BY_VALUE_DOMAIN: &[(&str, FactPrecision)] = &[];",
  ]).length,
  1,
  "the String-keyed derivation falsifier must be detected",
);

const analysisAdapter = blockBody(queryCore, "pub fn fact_precision_from_analysis_precision");
assert.ok(
  analysisAdapter.includes("project_fact_precision_from_analysis_precision(&precision.axes)"),
  "query precision must project the complete typed axes through the abstract-value authority",
);
assert.ok(
  queryTypes.includes("pub fn fact_precision_from_evidence_analysis_precision"),
  "evidence precision must reuse the query-side precision adapter",
);
assert.ok(queryTypes.includes("precision.axes"), "the evidence bridge must carry every axis");
assert.ok(queryCore.includes("pub struct OmenaQueryExpressionDomainSelectorPrecisionV0"));
assert.ok(queryCore.includes("pub precision: AnalysisPrecisionV1"));
assert.ok(queryCore.includes("precision: analysis_precision_from_class_value(&node.value)"));
assert.ok(
  transformExecutor.includes(
    "execute_transform_passes_on_source_with_dialect_context_closed_world_bundle_and_precision",
  ),
);
assert.ok(
  queryTransformContext.includes(
    "summarize_omena_query_expression_domain_selector_projection_with_precision",
  ),
);
assert.ok(queryTransformContext.includes("reachability_precisions.push(projection_precision)"));
assert.ok(
  queryTransform.includes(
    "closed_world_precision_witness_from_class_value(&value, Some(&witness))",
  ),
);
assert.ok(queryTransform.includes("witnessed.apply_to(*precision)"));
assert.equal(
  evidenceGraph.includes("with_closed_world_witness"),
  false,
  "the evidence graph must not expose an arbitrary closed-world axis raiser",
);
const destructiveConsumerSensitivityArmCandidates = [
  [checkerFixSafety, "fix_safety_closes_when_any_meet_axis_lowers_an_exact_domain"],
  [
    bridgeStyleIntelligence,
    "provider_precision_backing_consumes_completeness_not_only_value_domain",
  ],
  [transformTreeShakeTests, "tree_shake_precision_floor_consumes_the_full_axis_meet"],
  [queryTransform, "sealed_bundle_content_binds_finite_reachability_precision"],
] as const;
const destructiveConsumerSensitivityArms = destructiveConsumerSensitivityArmCandidates.filter(
  ([source, arm]) => source.includes(arm),
);
assert.equal(
  destructiveConsumerSensitivityArms.length,
  destructiveConsumerSensitivityArmCandidates.length,
  "every destructive precision consumer must retain its measured sensitivity arm",
);
for (const [, arm] of destructiveConsumerSensitivityArms) {
  assert.ok(arm.length > 0);
}
const treeShakeMeetSensitivityBody = blockBody(
  transformTreeShakeTests,
  "fn tree_shake_precision_floor_consumes_the_full_axis_meet",
);
const literalFixtureGateAxes = new Set<string>();
for (const body of structLiteralBodies(treeShakeMeetSensitivityBody, "AnalysisPrecisionV1")) {
  if (!body.includes("..exact")) continue;
  for (const field of topLevelFieldExpressions(body).keys()) literalFixtureGateAxes.add(field);
}
assert.deepEqual(
  [...literalFixtureGateAxes].toSorted(),
  [...precisionAxisFields].toSorted(),
  "the destructive gate fixture must lower each of the six axes",
);
assert.ok(
  queryTransform.includes(
    "execute_transform_passes_on_source_with_dialect_context_closed_world_bundle_precision_and_policy",
  ),
);
const precisionLabelDrops = precisionCalibration.cases.flatMap(
  (calibrationCase) => calibrationCase.precisionLabelDrops ?? [],
);
assert.ok(
  precisionLabelDrops.every(
    (drop) =>
      drop.output.length > 0 &&
      drop.before !== drop.after &&
      precisionAxisTypes.some((axis) =>
        axis.toLowerCase().startsWith(drop.loweringAxis.toLowerCase()),
      ),
  ),
  "every precision label drop must name its lowering axis",
);
assert.equal(
  precisionCalibration.differentialSweep.baselinePin,
  "db5a53366e8ab695457d23dad6535450bcf84cd1",
);
assert.equal(
  precisionCalibration.differentialSweep.method,
  "pinnedBaselineObservationTranscriptionPlusExecutableProductFixtureManifest",
);
const semverRuntimeChangeIds = new Set(
  semverIntent.intents.flatMap((intent) =>
    (intent.expectedRuntimeValueChanges ?? []).map((change) => `${intent.crate}:${change.id}`),
  ),
);
const precisionSweepRows = precisionCalibration.differentialSweep.rows;
assert.equal(
  new Set(precisionSweepRows.map((row) => row.output)).size,
  precisionSweepRows.length,
  "differential sweep outputs must be unique",
);
function validateDifferentialSweepRows(
  rows: typeof precisionCalibration.differentialSweep.rows,
): void {
  for (const row of rows) {
    const changed = row.before !== row.after;
    if (changed) {
      assert.equal(row.disposition, "disclosed", `${row.output} changed without disclosure`);
      assert.ok(row.loweringAxes.length > 0, `${row.output} must name its lowering axis`);
      assert.ok(row.semverIntentId !== null, `${row.output} must name a semver intent`);
      assert.ok(
        row.semverIntentId !== null && semverRuntimeChangeIds.has(row.semverIntentId),
        `${row.output} names a missing semver intent`,
      );
    } else {
      assert.equal(row.disposition, "reverted", `${row.output} must be marked reverted`);
      assert.deepEqual(row.loweringAxes, [], `${row.output} reverted row must not claim a drop`);
      assert.equal(
        row.semverIntentId,
        null,
        `${row.output} reverted row must not claim semver drift`,
      );
    }
  }
}
validateDifferentialSweepRows(precisionSweepRows);
const baselineObservations = precisionCalibration.differentialSweep.baselineObservations;
assert.equal(
  new Set(baselineObservations.map((observation) => observation.output)).size,
  baselineObservations.length,
  "pinned baseline observation outputs must be unique",
);
assert.equal(
  new Set(precisionAuthority.producerGateArms.map((arm) => arm.id)).size,
  precisionAuthority.producerGateArms.length,
  "producer-to-gate receipt ids must be unique",
);
assert.deepEqual(
  precisionAuthority.producerGateArms.map((arm) => arm.axis).toSorted(),
  ["context", "flow", "providerCompleteness", "revision", "valueDomain", "worldAssumption"],
  "the executable producer manifest must cover every precision axis exactly once",
);
for (const arm of precisionAuthority.producerGateArms) {
  assert.equal(arm.gateOpen, false, `${arm.id} must close the conservative gate`);
  assert.equal(arm.requiredFloor, "conservative", `${arm.id} must exercise the product floor`);
}
assert.equal(
  new Set(precisionAuthority.mutationProbes.map((probe) => probe.id)).size,
  precisionAuthority.mutationProbes.length,
  "precision mutation probe ids must be unique",
);
for (const probe of precisionAuthority.mutationProbes) {
  const source = read(probe.sourcePath);
  if (process.env.OMENA_PRECISION_MUTATION_PROBE === probe.id) {
    assert.ok(source.includes(probe.to), `${probe.id} active mutation must be present`);
  } else {
    assert.equal(
      source.split(probe.from).length - 1,
      1,
      `${probe.id} mutation source must match exactly once`,
    );
  }
  assert.notEqual(probe.from, probe.to, `${probe.id} mutation must change its source`);
  assert.ok(probe.command.length > 0, `${probe.id} mutation command must be executable`);
  assert.ok(probe.expectedFailure.length > 0, `${probe.id} must bind an expected failure`);
}

const executableManifestRun = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    precisionAuthority.executableManifestTest,
    "--",
    "--exact",
    "--nocapture",
  ],
  { cwd: repoRoot, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
);
const executableManifestTranscript = `${executableManifestRun.stdout ?? ""}\n${executableManifestRun.stderr ?? ""}`;
assert.equal(
  executableManifestRun.status,
  0,
  `executable precision manifest failed:\n${executableManifestTranscript}`,
);
const executableManifestPrefix = "OMENA_PRECISION_FLOOR_EXECUTABLE_MANIFEST=";
const executableManifestLines = executableManifestTranscript
  .split(/\r?\n/gu)
  .filter((line) => line.startsWith(executableManifestPrefix));
assert.equal(executableManifestLines.length, 1, "the executable precision manifest must emit once");
const executableManifest = JSON.parse(
  executableManifestLines[0]!.slice(executableManifestPrefix.length),
) as {
  readonly schemaVersion: string;
  readonly test: string;
  readonly observations: readonly { readonly output: string; readonly after: string }[];
  readonly producerGateArms: readonly ProducerGateArm[];
};
assert.equal(executableManifest.schemaVersion, "1");
assert.equal(executableManifest.test, precisionAuthority.executableManifestTest);
assert.deepEqual(
  executableManifest.producerGateArms,
  precisionAuthority.producerGateArms,
  "executed producer-to-gate receipts must exactly match their reviewed authority",
);
assert.equal(
  new Set(executableManifest.observations.map((observation) => observation.output)).size,
  executableManifest.observations.length,
  "executable observation outputs must be unique",
);
const baselineByOutput = new Map(
  baselineObservations.map((observation) => [observation.output, observation.before]),
);
const rowsByOutput = new Map(precisionSweepRows.map((row) => [row.output, row]));
const executableByOutput = new Map(
  executableManifest.observations.map((observation) => [observation.output, observation.after]),
);
const exactObservationKeys = [...rowsByOutput.keys()].toSorted();
assert.deepEqual(
  [...baselineByOutput.keys()].toSorted(),
  exactObservationKeys,
  "pinned baseline transcription must cover the exact differential output set",
);
assert.deepEqual(
  [...executableByOutput.keys()].toSorted(),
  exactObservationKeys,
  "executable product fixtures must cover the exact differential output set",
);
for (const [output, row] of rowsByOutput) {
  assert.equal(
    baselineByOutput.get(output),
    row.before,
    `${output} before value must match the pinned baseline transcription`,
  );
  assert.equal(
    executableByOutput.get(output),
    row.after,
    `${output} executable observation changed without a matching disclosure row`,
  );
}
const sweptPrecisionChanges = precisionSweepRows
  .filter(
    (row) => row.before !== row.after && row.output !== "analysisPrecision.contextSensitivity",
  )
  .map(({ output, before, after }) => ({ output, before, after }))
  .toSorted((left, right) => left.output.localeCompare(right.output));
const calibratedPrecisionChanges = precisionLabelDrops
  .map(({ output, before, after }) => ({ output, before, after }))
  .toSorted((left, right) => left.output.localeCompare(right.output));
assert.deepEqual(
  calibratedPrecisionChanges,
  sweptPrecisionChanges,
  "the calibration disclosure must be keyed to the differential sweep output",
);

const factPrecisionDeclarations = rustSources("rust/crates").reduce(
  (count, source) => count + [...source.matchAll(/pub enum FactPrecision\s*\{/gu)].length,
  0,
);
assert.equal(factPrecisionDeclarations, 1, "FactPrecision must have one authority");

const structuralHandlersStart = transformExecutor.indexOf("static STRUCTURAL_PASS_HANDLERS");
assert.ok(structuralHandlersStart >= 0, "missing structural handler manifest");
const structuralHandlersEnd = transformExecutor.indexOf("];", structuralHandlersStart);
assert.ok(
  structuralHandlersEnd > structuralHandlersStart,
  "unterminated structural handler manifest",
);
const structuralHandlersBody = transformExecutor.slice(
  structuralHandlersStart,
  structuralHandlersEnd,
);
const structuralHandlers = [
  ...structuralHandlersBody.matchAll(
    /kind:\s*TransformPassKind::([A-Za-z0-9]+),\s*run:\s*([a-z0-9_]+)/gu,
  ),
].map((match) => ({ pass: match[1]!, run: match[2]! }));
assert.ok(structuralHandlers.length > 0, "structural handler census must be non-vacuous");

const policyStart = transformModel.indexOf("pub const TRANSFORM_STRUCTURAL_DECISION_POLICIES_V0");
assert.ok(policyStart >= 0, "missing structural decision policy manifest");
const policyEnd = transformModel.indexOf("];", policyStart);
assert.ok(policyEnd > policyStart, "unterminated structural decision policy manifest");
const policyBody = transformModel.slice(policyStart, policyEnd);
const policyCalls = callBodies(policyBody, "TransformStructuralDecisionPolicyV0::new(");
const policies = [
  ...policyBody.matchAll(
    /TransformStructuralDecisionPolicyV0::new\(\s*TransformPassKind::([A-Za-z0-9]+),\s*TransformStructuralDecisionClassV0::([A-Za-z0-9]+)/gu,
  ),
].map((match) => ({ pass: match[1]!, className: match[2]! }));

assert.deepEqual(
  policies.map((policy) => policy.pass).toSorted(),
  structuralHandlers.map((handler) => handler.pass).toSorted(),
  "every structural handler must have exactly one decision policy",
);
const classCounts = Object.fromEntries(
  ["FactConsuming", "StaticExact", "ObligationDischarge", "NonRemovalRewrite"].map((className) => [
    className,
    policies.filter((policy) => policy.className === className).length,
  ]),
);
assert.deepEqual(classCounts, {
  FactConsuming: 4,
  StaticExact: 8,
  ObligationDischarge: 2,
  NonRemovalRewrite: 7,
});
const factConsumingPolicies = policyCalls.filter((call) =>
  call.includes("TransformStructuralDecisionClassV0::FactConsuming"),
);
assert.equal(factConsumingPolicies.length, 4);
for (const policy of factConsumingPolicies) {
  assert.ok(
    policy.includes("required_precision: FactPrecision::Conservative"),
    "each fact-consuming policy must declare a conservative floor",
  );
}

const reachabilityConsumers = structuralHandlers.filter((handler) => {
  const body = blockBody(transformExecutor, `fn ${handler.run}`);
  return body.includes(".reachability()");
});
const reachabilityCallCount = reachabilityConsumers.reduce(
  (total, handler) =>
    total + blockBody(transformExecutor, `fn ${handler.run}`).split(".reachability()").length - 1,
  0,
);
assert.equal(reachabilityConsumers.length, 4);
assert.equal(reachabilityCallCount, 9);
for (const consumer of reachabilityConsumers) {
  const policy = policies.find((entry) => entry.pass === consumer.pass);
  assert.equal(policy?.className, "FactConsuming", `${consumer.pass} must declare a biting floor`);
  const body = blockBody(transformExecutor, `fn ${consumer.run}`);
  assert.ok(
    body.indexOf("input.precision_blocker()") >= 0 &&
      body.indexOf("input.precision_blocker()") < body.indexOf(".reachability()"),
    `${consumer.pass} must enforce its floor before consuming reachability`,
  );
}
assert.ok(
  topLevelEnumVariants(transformModel, "TransformBlockedReasonV0").includes("PrecisionBelowFloor"),
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-transform.precision-floor",
      factPrecisionVariants,
      classValueVariantCount: classValueVariants.length,
      mappedClassValueVariantCount: mappedClassValueVariants.length,
      unmappedClosedCurrencyVariantCount:
        classValueVariants.length - mappedClassValueVariants.length,
      precisionAxisTypes,
      typedProducerValueDomains: [...typedProducerValueDomains].toSorted(),
      typedAxisProducerCount: executableManifest.producerGateArms.length,
      precisionConsumerCrateCount: precisionConsumerCrates.length,
      analysisPrecisionConstructorCount: analysisPrecisionConstructors.length,
      productionAxisAssignmentCount: analysisPrecisionAxisAssignments.length,
      typedJustifiedAxisAssignmentCount: typedJustifiedAxisAssignments.length,
      derivedAxisAssignmentCount: derivedAxisAssignments.length,
      literalConstantAxisAssignmentCount: literalAxisAssignments.length,
      stringKeyedPrecisionDerivationCount: stringKeyedDerivations.length,
      destructiveConsumerSensitivityArms: destructiveConsumerSensitivityArms.length,
      literalFixtureGateSensitivityArms: literalFixtureGateAxes.size,
      producerToClosedGateAxisArms: executableManifest.producerGateArms.length,
      disclosedPrecisionLabelDropCount: precisionLabelDrops.length,
      executableDifferentialObservationCount: executableManifest.observations.length,
      registeredSourceMutationProbeCount: precisionAuthority.mutationProbes.length,
      differentialSweepRowCount: precisionSweepRows.length,
      differentialSweepChangedRowCount: precisionSweepRows.filter((row) => row.before !== row.after)
        .length,
      differentialSweepRevertedRowCount: precisionSweepRows.filter(
        (row) => row.disposition === "reverted",
      ).length,
      belowFloorCauseClasses: ["heuristicReachability", "unknownReachability"],
      structuralPassCount: structuralHandlers.length,
      structuralDecisionClassCounts: classCounts,
      reachabilityConsumerCount: reachabilityConsumers.length,
      reachabilityCallCount,
      complete: true,
    },
    null,
    2,
  )}\n`,
);
