import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const args = new Set(process.argv.slice(2).filter((arg) => arg !== "--"));
const injectReorderedEvaluator = args.has("--inject-reordered-evaluator");
assert.deepEqual(
  [...args].filter((arg) => arg.startsWith("--") && arg !== "--inject-reordered-evaluator"),
  [],
  "unknown custom-property formalization option",
);

const documentPath = "docs/concepts/custom-property-bounded-substitution.md";
const gapRegisterPath = "rust/omena-custom-property-fixed-point-gap-register.json";
const corpusPath =
  "rust/crates/omena-cascade/tests/fixtures/custom-property-fixed-point-witness-v1.json";
const evaluatorPath = "rust/crates/omena-cascade/tests/custom_property_fixed_point_witness.rs";
const customPropertyPath = "rust/crates/omena-cascade/src/custom_property.rs";
const modelPath = "rust/crates/omena-cascade/src/model.rs";

const document = read(documentPath);
const customProperty = read(customPropertyPath);
const model = read(modelPath);
const evaluator = read(evaluatorPath);
const conceptsIndex = read("docs/concepts/README.md");
const conceptsMeta = JSON.parse(read("docs/concepts/meta.json")) as { pages?: string[] };
const gapRegister = JSON.parse(read(gapRegisterPath)) as GapRegister;
const corpus = JSON.parse(read(corpusPath)) as WitnessCorpus;

assert.ok(
  document.includes("sourceOfTruth: authored") &&
    document.includes("# Custom-property bounded substitution"),
  "the shipped-algorithm document must remain an authored concept page",
);
assert.ok(
  conceptsIndex.includes("./custom-property-bounded-substitution.md") &&
    conceptsMeta.pages?.includes("custom-property-bounded-substitution"),
  "the custom-property concept page must remain reachable from the docs navigation",
);

const correspondenceSymbols = [
  "CustomPropertyEnv",
  "CascadeValue",
  "substitute_custom_properties",
  "substitute_custom_properties_inner",
  "compute_custom_property_env_least_fixed_point",
  "resolve_custom_property_env_least_fixed_point",
  "summarize_custom_property_least_fixed_point",
  "visiting",
  "CascadeValue::GuaranteedInvalid",
  "CustomPropertyLeastFixedPointIterationV0",
  "custom_property_iteration_trace_is_monotone",
  "custom_property_bounded_fixed_point_computation_witness",
  "reached_fixed_point",
] as const;
for (const symbol of correspondenceSymbols) {
  assert.ok(document.includes(`\`${symbol}\``), `correspondence table omits ${symbol}`);
  assert.ok(
    customProperty.includes(symbol) || model.includes(symbol),
    `correspondence symbol does not exist in the shipped implementation: ${symbol}`,
  );
}
assert.ok(
  customProperty.includes("let mut current = env.clone();") &&
    customProperty.includes("let max_iterations = env.len().saturating_add(1).max(1);") &&
    customProperty.includes("if next == current") &&
    customProperty.includes("let mut visiting = BTreeSet::new();"),
  "the document guard must observe clone-start, explicit bound, equality, and DFS visiting-set code",
);
assert.ok(
  !/tarjan/iu.test(customProperty),
  "custom-property resolution must not claim Tarjan SCCs",
);
const cascadeValueBlock = extractBetween(
  model,
  "pub enum CascadeValue {",
  "pub enum ComputedCascadeValueStatusV0",
);
assert.ok(!/\bIf\b/u.test(cascadeValueBlock), "CascadeValue unexpectedly gained an if() variant");

assert.equal(gapRegister.schemaVersion, "1");
assert.equal(gapRegister.product, "omena-cascade.custom-property-fixed-point-gap-register");
assert.equal(gapRegister.authority.implementation, customPropertyPath);
assert.equal(gapRegister.authority.valueDomain, `${modelPath}#CascadeValue`);
assert.equal(gapRegister.authority.claimsUnderTest, "rfcs#10");
assert.deepEqual(gapRegister.rows.map((row) => row.id).toSorted(), [
  "conditional-value-domain",
  "cycle-decomposition",
  "initial-approximation",
  "termination-claim",
]);
for (const row of gapRegister.rows) {
  for (const [field, value] of Object.entries(row)) {
    assert.ok(
      typeof value === "string" && value.trim().length > 0,
      `gap row ${row.id} has empty ${field}`,
    );
  }
  assert.ok(document.includes(`\`${row.id}\``), `document gap table omits ${row.id}`);
}
assert.deepEqual(gapRegister.rfcDisposition, {
  tracker: "rfcs#10",
  status: "claims-under-test-not-shipped",
  decisionOwner: "independent-review",
  implementationUpgrade: "research-track-only",
});

const proseWithoutGapRegister = removeBetween(
  document,
  "<!-- gap-register:start -->",
  "<!-- gap-register:end -->",
);
assert.ok(
  !/\bleast fixed point\b/iu.test(proseWithoutGapRegister),
  "unqualified least fixed point claim exists outside the gap register",
);
assert.ok(
  !/knaster[ -]tarski/iu.test(proseWithoutGapRegister),
  "unqualified Knaster-Tarski claim exists outside the gap register",
);

assert.equal(corpus.schemaVersion, "1");
assert.equal(corpus.product, "omena-cascade.custom-property-fixed-point-witness-corpus");
assert.deepEqual(corpus.oracle, {
  statusOrder: "unresolvedBottomLeResolvedValue",
  environmentOrder: "pointwise",
  initialApproximation: "allBottom",
  transfer: "simultaneousOriginalBindingEvaluation",
  unresolvedAtFixedPoint: "guaranteedInvalid",
});
assert.equal(corpus.cases.length, 6);
assert.equal(corpus.cases.filter((entry) => entry.expectedDisposition === "agreement").length, 4);
assert.equal(corpus.cases.filter((entry) => entry.expectedDisposition === "finding").length, 2);
assert.deepEqual(
  corpus.cases
    .filter((entry) => entry.cycleShape !== null)
    .map((entry) => entry.cycleShape)
    .filter((shape) =>
      ["mutuallyRecursiveFallbackChain", "cycleThroughFallback"].includes(shape ?? ""),
    )
    .toSorted(),
  ["cycleThroughFallback", "mutuallyRecursiveFallbackChain"],
);
assert.ok(
  evaluator.includes(
    'const PERTURBATION_ENV: &str = "OMENA_CUSTOM_PROPERTY_WITNESS_PERTURBATION"',
  ) &&
    evaluator.includes('value == "reordered-in-place"') &&
    evaluator.includes("for (name, value) in bindings.iter().rev()") &&
    evaluator.includes("evaluate_fixture_value(value, &in_place)"),
  "the evaluator perturbation must be an order-sensitive in-place transfer mutation",
);

if (injectReorderedEvaluator) {
  const injected = runWitness("reordered-in-place", "inherit");
  process.exitCode = injected.status ?? 1;
} else {
  const normal = runWitness(null, "inherit");
  assert.equal(normal.status, 0, "the independent all-bottom witness corpus must pass");

  const mutation = runWitness("reordered-in-place", "pipe");
  const mutationOutput = `${mutation.stdout ?? ""}\n${mutation.stderr ?? ""}`;
  assert.notEqual(mutation.status, 0, "the reordered in-place evaluator mutation must be RED");
  assert.ok(
    mutationOutput.includes("acyclic-alias-chain evaluator iterations"),
    "the mutation must be caught by the independent alias-chain iteration oracle",
  );

  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: "1",
      product: "omena-cascade.custom-property-fixed-point-formalization",
      correspondenceSymbolCount: correspondenceSymbols.length,
      gapCount: gapRegister.rows.length,
      caseCount: corpus.cases.length,
      agreementCount: 4,
      findingCount: 2,
      novelCycleCaseCount: 2,
      mutation: "reordered-in-place:RED",
      rfcDisposition: gapRegister.rfcDisposition.status,
    })}\n`,
  );
}

function runWitness(
  perturbation: string | null,
  stdio: "inherit" | "pipe",
): ReturnType<typeof spawnSync> {
  const environment = { ...process.env };
  delete environment.OMENA_CUSTOM_PROPERTY_WITNESS_PERTURBATION;
  if (perturbation !== null) {
    environment.OMENA_CUSTOM_PROPERTY_WITNESS_PERTURBATION = perturbation;
  }
  return spawnSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cascade",
      "--test",
      "custom_property_fixed_point_witness",
      "custom_property_fixed_point_witness_corpus_matches_frozen_oracle",
      "--",
      "--nocapture",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: environment,
      stdio,
    },
  );
}

function read(filePath: string): string {
  return readFileSync(path.join(repoRoot, filePath), "utf8");
}

function extractBetween(source: string, start: string, end: string): string {
  const startIndex = source.indexOf(start);
  const endIndex = source.indexOf(end, startIndex + start.length);
  assert.ok(startIndex >= 0 && endIndex > startIndex, `cannot extract ${start} .. ${end}`);
  return source.slice(startIndex, endIndex);
}

function removeBetween(source: string, start: string, end: string): string {
  const startIndex = source.indexOf(start);
  const endIndex = source.indexOf(end, startIndex + start.length);
  assert.ok(startIndex >= 0 && endIndex > startIndex, `cannot remove ${start} .. ${end}`);
  return source.slice(0, startIndex) + source.slice(endIndex + end.length);
}

type GapRegister = {
  schemaVersion: string;
  product: string;
  authority: {
    implementation: string;
    valueDomain: string;
    claimsUnderTest: string;
    upgradeOwner: string;
  };
  rows: Array<{
    id: string;
    gapKind: string;
    shippedState: string;
    claimsUnderTestState: string;
    observableConsequence: string;
    upgradeCost: string;
  }>;
  rfcDisposition: {
    tracker: string;
    status: string;
    decisionOwner: string;
    implementationUpgrade: string;
  };
};

type WitnessCorpus = {
  schemaVersion: string;
  product: string;
  oracle: {
    statusOrder: string;
    environmentOrder: string;
    initialApproximation: string;
    transfer: string;
    unresolvedAtFixedPoint: string;
  };
  cases: Array<{
    id: string;
    cycleShape: string | null;
    expectedDisposition: string;
  }>;
};
