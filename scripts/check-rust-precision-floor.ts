import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

function read(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function rustSources(relativeDirectory: string): string[] {
  const directory = path.join(repoRoot, relativeDirectory);
  return fs
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
const queryCore = read("rust/crates/omena-query-core/src/lib.rs");
const queryTypes = read("rust/crates/omena-query/src/types.rs");
const queryTransform = read("rust/crates/omena-query/src/style/transform.rs");
const queryTransformContext = read("rust/crates/omena-query/src/style/transform/context.rs");
const transformModel = read("rust/crates/omena-transform-passes/src/model.rs");
const transformExecutor = read("rust/crates/omena-transform-passes/src/runtime/executor.rs");

const factPrecisionVariants = topLevelEnumVariants(abstractTypes, "FactPrecision");
assert.deepEqual(factPrecisionVariants, ["Exact", "Conservative", "Heuristic", "Unknown"]);

const classValueVariants = topLevelEnumVariants(abstractTypes, "AbstractClassValueV0");
const classValueAdapter = blockBody(abstractDomain, "pub fn fact_precision_from_class_value");
const mappedClassValueVariants = [
  ...new Set(
    [...classValueAdapter.matchAll(/AbstractClassValueV0::([A-Z][A-Za-z0-9]*)/gu)].map(
      (match) => match[1],
    ),
  ),
].toSorted();
assert.deepEqual(mappedClassValueVariants, classValueVariants.toSorted());
assert.ok(!/(^|[^\w])_\s*=>/u.test(classValueAdapter), "class-value adapter must not catch all");

const producerSources = [queryCore, ...rustSources("rust/crates/omena-query/src")];
const producerValueDomains = new Set<string>();
for (const source of producerSources) {
  for (const match of source.matchAll(/value_domain:\s*"([^"]+)"/gu)) {
    if (match[1]) producerValueDomains.add(match[1]);
  }
  for (const match of source.matchAll(/source_diagnostic_precision\(\s*"([^"]+)"/gu)) {
    if (match[1]) producerValueDomains.add(match[1]);
  }
  if (source.includes("OMENA_QUERY_TYPE_ORACLE_UNKNOWN_VALUE_DOMAIN")) {
    producerValueDomains.add("unknown");
  }
}

const analysisAdapter = blockBody(queryCore, "pub fn fact_precision_from_analysis_precision");
const unmappedProducerValueDomains = [...producerValueDomains].filter(
  (valueDomain) => !analysisAdapter.includes(`"${valueDomain}"`),
);
assert.deepEqual(
  unmappedProducerValueDomains,
  [],
  `query precision producers are not mapped: ${unmappedProducerValueDomains.join(", ")}`,
);
assert.ok(
  analysisAdapter.includes("FactPrecision::Unknown"),
  "open string precision inputs must fail closed to Unknown",
);
assert.ok(
  queryTypes.includes("pub fn fact_precision_from_evidence_analysis_precision"),
  "evidence precision must reuse the query-side precision adapter",
);
assert.ok(queryCore.includes("pub struct OmenaQueryExpressionDomainSelectorPrecisionV0"));
assert.ok(queryCore.includes("pub precision: FactPrecision"));
assert.ok(queryCore.includes("precision: fact_precision_from_class_value(&node.value)"));
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
assert.ok(queryTransformContext.includes("current.bounded_by(projection_precision)"));
assert.ok(
  queryTransform.includes(
    "execute_transform_passes_on_source_with_dialect_context_closed_world_bundle_and_precision",
  ),
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
      queryPrecisionValueDomains: [...producerValueDomains].toSorted(),
      unmappedProducerValueDomains,
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
