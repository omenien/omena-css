import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest";
import { strict as assert } from "node:assert";
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

const precisionProducerSources = [
  evidenceGraph,
  abstractDomain,
  queryCore,
  queryTypes,
  ...rustSources("rust/crates/omena-query/src"),
];
const typedProducerValueDomains = new Set(
  precisionProducerSources.flatMap((source) =>
    [...source.matchAll(/value_domain:\s*ValueDomainPrecisionV1::([A-Z][A-Za-z0-9]*)/gu)].map(
      (match) => match[1]!,
    ),
  ),
);
assert.ok(typedProducerValueDomains.size > 0, "typed value-domain producers must be non-vacuous");
const typedAxisProducerMarkers = [
  "analysis_precision_from_class_value_with_witness",
  "FlowPrecisionV1::from_dataflow_mode",
  "ContextPrecisionV1::from_max_context_depth",
  "ProviderCompletenessV1::from_unresolved_count",
  "WorldAssumptionV1::from_closed_world",
  "RevisionIdentityV1::from_revisions",
] as const;
const typedAxisProducerCorpus = precisionProducerSources.join("\n");
for (const marker of typedAxisProducerMarkers) {
  assert.ok(typedAxisProducerCorpus.includes(marker), `missing typed axis producer ${marker}`);
}
function literalConstantAxisAssignments(sources: readonly string[]): string[] {
  const pattern =
    /(?:value_domain|flow(?:_sensitivity)?|context(?:_sensitivity)?|provider_completeness|world_assumption|revision(?:_axis)?):\s*(?:String::from\s*\(\s*)?"[^"]+"/gu;
  return sources.flatMap((source) => [...source.matchAll(pattern)].map((match) => match[0]));
}
const literalAxisAssignments = literalConstantAxisAssignments(precisionProducerSources);
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
const stringKeyedDerivations = stringKeyedPrecisionDerivations(precisionProducerSources);
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
assert.ok(queryTransform.includes("with_closed_world_witness(witnessed.value_domain)"));
for (const [source, arm] of [
  [checkerFixSafety, "fix_safety_closes_when_any_meet_axis_lowers_an_exact_domain"],
  [
    bridgeStyleIntelligence,
    "provider_precision_backing_consumes_completeness_not_only_value_domain",
  ],
  [transformTreeShakeTests, "tree_shake_precision_floor_consumes_the_full_axis_meet"],
  [queryTransform, "sealed_bundle_content_binds_finite_reachability_precision"],
] as const) {
  assert.ok(source.includes(arm), `missing destructive consumer sensitivity arm ${arm}`);
}
for (const producerGateArm of [
  "unresolved_provider.provider_completeness",
  "open_world.world_assumption",
  "stale_revision.revision",
  "shallow_context.context",
  "non_dataflow.flow",
]) {
  assert.ok(
    transformTreeShakeTests.includes(producerGateArm),
    `missing producer-to-closed-gate axis assertion ${producerGateArm}`,
  );
}
assert.ok(
  queryTransform.includes(
    "execute_transform_passes_on_source_with_dialect_context_closed_world_bundle_precision_and_policy",
  ),
);
const precisionLabelDrops = precisionCalibration.cases.flatMap(
  (calibrationCase) => calibrationCase.precisionLabelDrops ?? [],
);
assert.equal(precisionLabelDrops.length, 4, "precision label-drop disclosure count drifted");
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
      typedAxisProducerCount: typedAxisProducerMarkers.length,
      literalConstantAxisAssignmentCount: literalAxisAssignments.length,
      stringKeyedPrecisionDerivationCount: stringKeyedDerivations.length,
      destructiveConsumerSensitivityArms: 4,
      producerToClosedGateAxisArms: 5,
      disclosedPrecisionLabelDropCount: precisionLabelDrops.length,
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
