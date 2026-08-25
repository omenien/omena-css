import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { copyFileSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

const repoRoot = process.cwd();
const scriptPath = path.join(
  repoRoot,
  "scripts/check-rust-omena-custom-property-fixed-point-formalization.ts",
);
const valueGrammarDifferentialPath = path.join(
  repoRoot,
  "scripts/check-rust-omena-value-grammar-differential.ts",
);
const diagnosticTransitionAttributionPath = path.join(
  repoRoot,
  "scripts/measure-rust-omena-diagnostic-transition-attribution.ts",
);
const sharedMutationTargetEnvironment = "OMENA_CUSTOM_PROPERTY_FORMALIZATION_MUTATION_TARGET_DIR";
let sharedMutationTargetDirectory: string | null = null;
const args = new Set(process.argv.slice(2).filter((argument) => argument !== "--"));
const allowedOptions = new Set([
  "--inject-reordered-evaluator",
  "--inject-case-replacement",
  "--inject-oracle-weakening",
  "--inject-fallback-rescue",
  "--inject-delete-scc",
  "--inject-always-valid-validator",
  "--inject-literal-only-verdicts",
  "--inject-matcher-coverage-promotion",
  "--inject-nonconverged-return",
  "--inject-cycle-member-shrink",
  "--inject-closed-world-certificate-disable",
  "--inject-close-open-function-kind",
  "--inject-paint-context-keyword-removal",
  "--inject-linear-component-rescan",
  "--inject-keyword-reference-closure-drop",
  "--inject-accepted-keyword-removal",
  "--inject-owned-wrong-definite",
]);
assert.deepEqual(
  [...args].filter((argument) => argument.startsWith("--") && !allowedOptions.has(argument)),
  [],
  "unknown custom-property formalization option",
);
assert.ok(args.size <= 1, "inject exactly one custom-property mutation at a time");

const documentPath = "docs/concepts/custom-property-bounded-substitution.md";
const gapRegisterPath = "rust/omena-custom-property-fixed-point-gap-register.json";
const diagnosticCensusPath = "rust/omena-custom-property-diagnostic-census.json";
const corpusPath =
  "rust/crates/omena-cascade/tests/fixtures/custom-property-fixed-point-witness-v1.json";
const evaluatorPath = "rust/crates/omena-cascade/tests/custom_property_fixed_point_witness.rs";
const customPropertyPath = "rust/crates/omena-cascade/src/custom_property.rs";
const computedValuePath = "rust/crates/omena-cascade/src/computed_value.rs";
const cascadeTestsPath = "rust/crates/omena-cascade/src/tests.rs";
const cascadeLibPath = "rust/crates/omena-cascade/src/lib.rs";
const modelPath = "rust/crates/omena-cascade/src/model.rs";
const grammarPath = "rust/crates/omena-abstract-value/src/value_grammar.rs";
const grammarOverridePath = "rust/crates/omena-spec-audit/data/value-grammar-overrides.json";
const cliLintPath = "rust/crates/omena-cli/src/lint.rs";
const queryCorePath = "rust/crates/omena-query-core/src/lib.rs";
const salsaPath = "rust/crates/omena-query/src/style/salsa_memo.rs";
const cascadePositionPath = "rust/crates/omena-query/src/style/cascade_position.rs";
const compatibilityProjectionPath = "rust/crates/omena-rg-flow/src/lib.rs";

const document = read(documentPath);
let customProperty = read(customPropertyPath);
const computedValue = read(computedValuePath);
const model = read(modelPath);
let evaluator = read(evaluatorPath);
const rgFlow = read(compatibilityProjectionPath);
const conceptsIndex = read("docs/concepts/README.md");
const conceptsMeta = JSON.parse(read("docs/concepts/meta.json")) as { pages?: string[] };
const gapRegister = JSON.parse(read(gapRegisterPath)) as GapRegister;
const diagnosticCensus = JSON.parse(read(diagnosticCensusPath)) as DiagnosticCensus;
const corpus = JSON.parse(read(corpusPath)) as WitnessCorpus;
const cliLint = read(cliLintPath);

validateDiagnosticCensus(diagnosticCensus, cliLint);

if (args.has("--inject-case-replacement")) {
  corpus.cases = corpus.cases.filter((entry) => entry.id !== "cycle-through-fallback");
  corpus.cases.push({
    ...corpus.cases[0],
    id: "replacement-direct-literal",
  });
}
if (args.has("--inject-oracle-weakening")) {
  evaluator = replaceExactly(
    evaluator,
    `Some(OracleStatus::Resolved(FixtureValue::GuaranteedInvalid)) | None => {
                fallback.as_deref().map_or(
                    OracleStatus::Resolved(FixtureValue::GuaranteedInvalid),
                    |fallback| evaluate_fixture_value(fallback, approximation),
                )
            }`,
    `Some(OracleStatus::Resolved(FixtureValue::GuaranteedInvalid)) => {
                OracleStatus::Resolved(FixtureValue::GuaranteedInvalid)
            }
            None => {
                fallback.as_deref().map_or(
                    OracleStatus::Resolved(FixtureValue::GuaranteedInvalid),
                    |fallback| evaluate_fixture_value(fallback, approximation),
                )
            }`,
  );
}
if (args.has("--inject-cycle-member-shrink")) {
  const plainCycle = corpus.cases.find((entry) => entry.id === "plain-two-cycle");
  assert.ok(plainCycle, "plain-two-cycle fixture is absent");
  delete plainCycle.bindings["--b"];
}

assert.ok(
  document.includes("sourceOfTruth: authored") &&
    document.includes("# Custom-property dependency resolution"),
  "the shipped-algorithm document must remain an authored concept page",
);
assert.ok(
  conceptsIndex.includes("[Custom-property dependency resolution]") &&
    conceptsIndex.includes("./custom-property-bounded-substitution.md") &&
    conceptsMeta.pages?.includes("custom-property-bounded-substitution"),
  "the custom-property concept page must remain reachable from the docs navigation",
);

const correspondenceSymbols = [
  "CanonicalCustomPropertyNameV0",
  "PropertyNameV0",
  "CustomPropertyEnv",
  "CascadeValue",
  "custom_property_dependency_graph",
  "collect_custom_property_reference_indices",
  "strongly_connected_components",
  "dependency_ordered_components",
  "component_is_cyclic",
  "substitute_custom_properties_against_resolved_env",
  "substitute_custom_properties",
  "resolve_custom_property_env_least_fixed_point",
  "summarize_custom_property_least_fixed_point",
  "CustomPropertyLeastFixedPointIterationV0",
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
  customProperty.includes("let dependency_graph = custom_property_dependency_graph(env);") &&
    customProperty.includes("let components = strongly_connected_components(&dependency_graph);") &&
    customProperty.includes(
      "let component_schedule = dependency_ordered_components(&dependency_graph, &components);",
    ) &&
    customProperty.includes("if component_is_cyclic(component, &dependency_graph)") &&
    customProperty.includes("resolved_values[*node] = Some(resolved);") &&
    customProperty.includes("Some(CustomPropertyGuaranteedInvalidReasonV0::CycleMember);"),
  "the product must execute graph, SCC, cyclic-set invalidation, and component scheduling",
);
assert.ok(
  customProperty.includes("if let Some(fallback) = fallback") &&
    customProperty.includes(
      "collect_custom_property_reference_indices(fallback, index_by_name, references);",
    ) &&
    customProperty.includes("HashMap<&str, usize>") &&
    customProperty.includes("edges: Vec<Vec<usize>>"),
  "fallback edges and graph nodes must use the shared canonical custom-property key",
);
assert.ok(
  customProperty.includes("let mut stack = vec![(start, 0usize)];") &&
    customProperty.includes("let mut stack = vec![node];") &&
    customProperty.includes("let mut ready = (0..components.len())") &&
    !customProperty.includes("fn finish_visit(") &&
    !customProperty.includes("fn collect_reverse_component(") &&
    !customProperty.includes("fn visit_component("),
  "SCC partition and component scheduling must remain iterative",
);
for (const retiredNeedle of [
  "let mut current = env.clone();",
  "max_iterations",
  "reached_fixed_point: false",
  "substitute_custom_properties_inner",
  "let mut visiting",
]) {
  assert.ok(
    !customProperty.includes(retiredNeedle),
    `retired iterate-and-rescue code remains: ${retiredNeedle}`,
  );
}
assert.ok(
  customProperty.includes(
    '"the SCC schedule must evaluate every custom-property binding exactly once"',
  ) &&
    customProperty.includes(
      '"the acyclic component schedule must eliminate every var() reference"',
    ) &&
    customProperty.includes("reached_fixed_point: true"),
  "the no-non-converged-value invariant must be executable",
);

const cascadeValueBlock = extractBetween(
  model,
  "pub enum CascadeValue {",
  "pub enum ComputedCascadeValueStatusV0",
);
assert.ok(!/\bIf\b/u.test(cascadeValueBlock), "CascadeValue unexpectedly gained an if() variant");

assert.ok(
  computedValue.includes("pub trait CascadeStandardValueValidatorV0") &&
    computedValue.includes("compute_cascade_computed_value_with_standard_value_validator_v0") &&
    computedValue.includes("standardPropertySyntaxVerdictUnavailable") &&
    computedValue.includes("postSubstitutionStandardPropertySyntaxUnmatched") &&
    computedValue.includes("postSubstitutionStandardPropertySyntaxIndeterminate"),
  "computed-value resolution must expose and consume the post-substitution grammar port",
);
assert.ok(
  computedValue.includes("standard_syntax_verdict_unavailable") &&
    computedValue.includes(
      "ComputedCascadeIndeterminateReasonV0::StandardPropertySyntaxIndeterminate",
    ),
  "an absent standard-property verdict must fail closed as typed indeterminate",
);
assert.ok(
  read(grammarPath).includes(
    "impl CascadeStandardValueValidatorV0 for SpecStandardPropertyValueValidatorV0",
  ) &&
    read(grammarPath).includes(
      "validate_standard_property_value_v0(property.canonical_name(), value).class",
    ) &&
    computedValue.includes("property: &PropertyNameV0") &&
    read(queryCorePath).includes("SpecStandardPropertyValueValidatorV0") &&
    read(salsaPath).includes("source_element_static_custom_property_env") &&
    read(salsaPath).includes("CascadeStandardValueVerdictV0::Unknown"),
  "the spec grammar authority and salsa-fed custom-property environment must reach the port",
);
assert.ok(
  rgFlow.includes("let trace_is_component_schedule") &&
    rgFlow.includes("recompute_input_component_count_from_entries") &&
    [
      "iteration.settled_count == summary.input_count",
      "!cascade_value_contains_var_reference(&entry.resolved)",
      "fn beta_estimate_reads_cascade_component_schedule_without_mutating_cascade()",
    ].every((needle) => rgFlow.includes(needle)) &&
    document.includes("component-schedule observation rather than a Kleene iteration"),
  "downstream RG-flow must not relabel a non-empty component schedule as a Kleene certificate",
);

assert.equal(gapRegister.schemaVersion, "1");
assert.equal(gapRegister.product, "omena-cascade.custom-property-fixed-point-gap-register");
assert.equal(gapRegister.authority.implementation, customPropertyPath);
assert.equal(gapRegister.authority.valueDomain, `${modelPath}#CascadeValue`);
assert.equal(gapRegister.authority.claimsUnderTest, "rfcs#10");
assert.deepEqual(
  gapRegister.rows.map((row) => row.id),
  ["conditional-value-domain"],
);
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
  status: "structural-cycle-semantics-shipped",
  decisionOwner: "product-implementation",
  implementationUpgrade: "dependency-graph-scc-shipped",
});
const proseWithoutGapRegister = removeBetween(
  document,
  "<!-- gap-register:start -->",
  "<!-- gap-register:end -->",
);
assert.ok(
  !/\bleast fixed point\b/iu.test(proseWithoutGapRegister),
  "unqualified least fixed point claim exists outside the residual register",
);
assert.ok(
  !/knaster[ -]tarski/iu.test(proseWithoutGapRegister),
  "unqualified Knaster-Tarski claim exists outside the residual register",
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
const frozenCaseShapes = new Map<string, string | null>([
  ["direct-literal", null],
  ["acyclic-alias-chain", null],
  ["missing-reference-fallback", null],
  ["plain-two-cycle", "mutualReferenceWithoutFallback"],
  ["outer-fallback-after-invalid-dependency", null],
  ["mutually-recursive-fallback-chain", "mutuallyRecursiveFallbackChain"],
  ["cycle-through-fallback", "cycleThroughFallback"],
  ["three-node-fallback-cycle-entered-mid-chain", "threeNodeFallbackCycleEnteredMidChain"],
]);
assert.ok(corpus.cases.length >= 6, "the witness corpus may grow but must not shrink");
for (const [id, cycleShape] of frozenCaseShapes) {
  const witnessCase = corpus.cases.find((entry) => entry.id === id);
  assert.ok(witnessCase, `the frozen witness case ${id} must remain`);
  assert.equal(witnessCase.cycleShape, cycleShape, `the frozen cycle shape for ${id} changed`);
}
assert.equal(
  corpus.cases.filter((entry) => entry.expectedDisposition === "agreement").length,
  corpus.cases.length,
);
assert.equal(corpus.cases.filter((entry) => entry.expectedDisposition === "finding").length, 0);
assert.deepEqual(
  corpus.cases
    .map((entry) => entry.cycleShape)
    .filter((cycleShape): cycleShape is string => cycleShape !== null),
  [
    "mutualReferenceWithoutFallback",
    "mutuallyRecursiveFallbackChain",
    "cycleThroughFallback",
    "threeNodeFallbackCycleEnteredMidChain",
  ],
  "the named non-degenerate cycle-shape allowlist changed",
);
const novelCycleShapeAllowlist = new Set(
  [...frozenCaseShapes.values()].filter(
    (cycleShape): cycleShape is string =>
      cycleShape !== null && cycleShape !== "mutualReferenceWithoutFallback",
  ),
);
const evaluatorFunction = extractBetween(
  evaluator,
  "fn evaluate_from_all_bottom(",
  "\nfn to_cascade_value(",
);
assert.equal(
  sha256(evaluatorFunction),
  "5fb3d0b7ab8a5d5b80040cba27e04ef7bc4344d341f202cbb10766165d7c60bc",
  "the independent all-bottom evaluator kernel changed",
);
assert.equal(
  sha256(
    JSON.stringify(corpus.cases.map(({ id, expectedEvaluator }) => ({ id, expectedEvaluator }))),
  ),
  "96bea4cb6b02ad22d491b46b4ba632103f4539a27c788002f74523e57beaa486",
  "the independent expectedEvaluator projections changed",
);
const canonicalBindingProjection = canonicalJson(
  corpus.cases.map(({ id, cycleShape, bindings }) => ({ id, cycleShape, bindings })),
);
assert.equal(
  sha256(JSON.stringify(canonicalBindingProjection)),
  "92feed5fada2ae87cb0aa47fb995bae358f610c5ef375b000b487a2256d18836",
  "the frozen witness bindings or cycle shapes changed",
);
assert.ok(
  evaluator.includes(
    'const PERTURBATION_ENV: &str = "OMENA_CUSTOM_PROPERTY_WITNESS_PERTURBATION"',
  ) &&
    evaluator.includes('value == "reordered-in-place"') &&
    evaluator.includes("for (name, value) in bindings.iter().rev()") &&
    evaluator.includes("evaluate_fixture_value(value, &in_place)"),
  "the evaluator perturbation must remain an order-sensitive in-place transfer mutation",
);

if (args.has("--inject-reordered-evaluator")) {
  const injected = runWitness(repoRoot, "reordered-in-place", "inherit");
  process.exitCode = injected.status ?? 1;
} else if (args.has("--inject-fallback-rescue")) {
  relayMutationResult(runFallbackRescueMutation());
} else if (args.has("--inject-delete-scc")) {
  relayMutationResult(runSccDeletionMutation());
} else if (args.has("--inject-always-valid-validator")) {
  relayMutationResult(runAlwaysValidValidatorMutation());
} else if (args.has("--inject-literal-only-verdicts")) {
  relayMutationResult(runLiteralOnlyVerdictMutation());
} else if (args.has("--inject-matcher-coverage-promotion")) {
  relayMutationResult(runMatcherCoveragePromotionMutation());
} else if (args.has("--inject-nonconverged-return")) {
  relayMutationResult(runNonconvergedReturnMutation());
} else if (args.has("--inject-closed-world-certificate-disable")) {
  relayMutationResult(runClosedWorldCertificateDisableMutation());
} else if (args.has("--inject-close-open-function-kind")) {
  relayMutationResult(runCloseOpenFunctionKindMutation());
} else if (args.has("--inject-paint-context-keyword-removal")) {
  relayMutationResult(runPaintContextKeywordRemovalMutation());
} else if (args.has("--inject-linear-component-rescan")) {
  relayMutationResult(runLinearComponentRescanMutation());
} else if (args.has("--inject-keyword-reference-closure-drop")) {
  relayMutationResult(
    runValueGrammarDifferentialFault("OMENA_VALUE_GRAMMAR_TEST_DROP_REFERENCE_CLOSURE"),
  );
} else if (args.has("--inject-accepted-keyword-removal")) {
  relayMutationResult(runAcceptedKeywordRemovalMutation());
} else if (args.has("--inject-owned-wrong-definite")) {
  relayMutationResult(
    runValueGrammarDifferentialFault("OMENA_VALUE_GRAMMAR_TEST_INJECT_ORACLE_VALID_DEFINITE"),
  );
} else if (args.size === 0) {
  const normal = runWitness(repoRoot, null, "inherit");
  assert.equal(normal.status, 0, "the independent all-bottom witness corpus must pass");
  const directGrammar = runCargo(repoRoot, [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-abstract-value",
    "cascade_validator_adapter_preserves_spec_grammar_outcomes",
    "--",
    "--nocapture",
  ]);
  assert.equal(directGrammar.status, 0, "the direct post-substitution grammar arm must pass");
  const salsaGrammar = runSalsaGrammarTest(repoRoot, "inherit");
  assert.equal(salsaGrammar.status, 0, "the salsa-fed post-substitution grammar arm must pass");
  const coverageCorpus = runCargo(
    repoRoot,
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "tracked_style_var_sites_have_the_pinned_product_status_distribution",
      "--",
      "--nocapture",
    ],
    "pipe",
  );
  assert.equal(coverageCorpus.status, 0, "the tracked var() site status corpus must pass");
  const coverageOutput = `${coverageCorpus.stdout ?? ""}\n${coverageCorpus.stderr ?? ""}`;
  const coverageMatch = coverageOutput.match(/trackedVarSiteCount=(\d+) statuses=(\{[^\n]+\})/u);
  assert.ok(coverageMatch, "the tracked var() site receipt is absent");
  const trackedVarSiteCount = Number(coverageMatch[1]);
  const trackedVarSiteStatuses = JSON.parse(coverageMatch[2]) as Record<string, number>;
  const diagnosticCorpus = runCargo(
    repoRoot,
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "tracked_style_diagnostics_preserve_the_pinned_rule_census",
      "--",
      "--nocapture",
    ],
    "pipe",
  );
  assert.equal(diagnosticCorpus.status, 0, "the tracked style-diagnostics census must pass");
  const diagnosticOutput = `${diagnosticCorpus.stdout ?? ""}\n${diagnosticCorpus.stderr ?? ""}`;
  const diagnosticMatch = diagnosticOutput.match(
    /trackedStyleDiagnosticsFileCount=(\d+) ruleCounts=(\{[^\n]+\})/u,
  );
  assert.ok(diagnosticMatch, "the tracked style-diagnostics receipt is absent");
  const trackedStyleDiagnosticsFileCount = Number(diagnosticMatch[1]);
  const trackedStyleDiagnosticCounts = JSON.parse(diagnosticMatch[2]) as Record<string, number>;
  const cliDiagnosticCorpus = runCargo(
    repoRoot,
    [
      "test",
      "--release",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "tracked_style_diagnostics_default_cli_preserves_the_pinned_rule_census",
      "--",
      "--ignored",
      "--nocapture",
    ],
    "pipe",
  );
  assert.equal(cliDiagnosticCorpus.status, 0, "the tracked CLI style-diagnostics census must pass");
  const cliDiagnosticOutput = `${cliDiagnosticCorpus.stdout ?? ""}\n${cliDiagnosticCorpus.stderr ?? ""}`;
  const cliDiagnosticMatch = cliDiagnosticOutput.match(
    /trackedStyleDiagnosticsDefaultCliFileCount=(\d+) ruleCounts=(\{[^\n]+\})/u,
  );
  assert.ok(cliDiagnosticMatch, "the tracked CLI style-diagnostics receipt is absent");
  const trackedStyleDiagnosticsCliFileCount = Number(cliDiagnosticMatch[1]);
  const trackedStyleDiagnosticsCliCounts = JSON.parse(cliDiagnosticMatch[2]) as Record<
    string,
    number
  >;
  const currentDiagnosticPin = diagnosticCensus.pins.find(
    (pin) => pin.sourcePin === "26450dbf146807be958b14547393dd57852dbc45",
  );
  assert.ok(currentDiagnosticPin, "the current implementation diagnostic pin is absent");
  assert.equal(
    trackedStyleDiagnosticsFileCount,
    diagnosticCensus.corpus.styleFileCount,
    "the query diagnostic corpus size diverged from the five-pin census",
  );
  assert.deepEqual(
    trackedStyleDiagnosticCounts,
    currentDiagnosticPin.measurements.queryStyleDiagnostics.countsByRule,
    "the query diagnostic counts diverged from the current five-pin measurement",
  );
  assert.equal(
    trackedStyleDiagnosticsCliFileCount,
    diagnosticCensus.corpus.styleFileCount,
    "the default CLI diagnostic corpus size diverged from the five-pin census",
  );
  assert.deepEqual(
    trackedStyleDiagnosticsCliCounts,
    currentDiagnosticPin.measurements.defaultStyleDiagnosticsCli.countsByRule,
    "the default CLI diagnostic counts diverged from the current five-pin measurement",
  );
  const diagnosticTransitionAttribution = spawnSync(
    process.execPath,
    ["--import", "tsx", diagnosticTransitionAttributionPath],
    {
      cwd: repoRoot,
      encoding: "utf8",
    },
  );
  const diagnosticTransitionAttributionOutput = `${diagnosticTransitionAttribution.stdout ?? ""}\n${diagnosticTransitionAttribution.stderr ?? ""}`;
  assert.equal(
    diagnosticTransitionAttribution.status,
    0,
    `the measured diagnostic transition attribution must pass:\n${diagnosticTransitionAttributionOutput.slice(-8_000)}`,
  );
  const diagnosticTransitionAttributionMatch = diagnosticTransitionAttributionOutput.match(
    /(\{"schemaVersion":"1","product":"omena-query\.diagnostic-transition-attribution"[^\n]+\})/u,
  );
  assert.ok(
    diagnosticTransitionAttributionMatch,
    "the measured diagnostic transition attribution receipt is absent",
  );
  const diagnosticTransitionAttributionReceipt = JSON.parse(
    diagnosticTransitionAttributionMatch[1],
  ) as {
    addedLocationCount: number;
    removedLocationCount: number;
    attribution: string;
    locationSubstitutionMutation: string;
  };
  assert.deepEqual(
    {
      addedLocationCount: diagnosticTransitionAttributionReceipt.addedLocationCount,
      removedLocationCount: diagnosticTransitionAttributionReceipt.removedLocationCount,
      attribution: diagnosticTransitionAttributionReceipt.attribution,
      locationSubstitutionMutation:
        diagnosticTransitionAttributionReceipt.locationSubstitutionMutation,
    },
    {
      addedLocationCount: 12,
      removedLocationCount: 0,
      attribution: "measured-two-pin-diff:GREEN",
      locationSubstitutionMutation: "RED",
    },
  );
  const longChain = runCargo(repoRoot, [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cascade",
    "resolves_a_hundred_thousand_binding_alias_chain_without_recursion",
    "--",
    "--nocapture",
  ]);
  assert.equal(longChain.status, 0, "the 100k iterative alias-chain arm must pass");
  const variableEnvironmentPerformance = runCargo(repoRoot, [
    "test",
    "--release",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cascade",
    "variable_environment_resolution_and_summary_stay_within_linear_growth_noise_budget",
    "--",
    "--ignored",
    "--nocapture",
  ]);
  assert.equal(
    variableEnvironmentPerformance.status,
    0,
    "the variable-environment request and summary normalized linear-growth budget must pass",
  );
  const traceSchedule = runCargo(repoRoot, [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-rg-flow",
    "fixed_point_trace_verification_requires_a_complete_component_schedule",
    "--",
    "--nocapture",
  ]);
  assert.equal(traceSchedule.status, 0, "the component-schedule trace invariant must pass");

  const mutationTargetScratch = mkdtempSync(
    path.join(tmpdir(), "omena-custom-property-mutation-suite-"),
  );
  let mutationReceipts: string[];
  try {
    sharedMutationTargetDirectory = path.join(mutationTargetScratch, "target");
    mutationReceipts = [
      expectScriptMutationRed(
        "--inject-reordered-evaluator",
        "acyclic-alias-chain evaluator iterations",
      ),
      expectScriptMutationRed(
        "--inject-case-replacement",
        "frozen witness case cycle-through-fallback",
      ),
      expectScriptMutationRed(
        "--inject-oracle-weakening",
        "independent all-bottom evaluator kernel changed",
      ),
      expectScriptMutationRed(
        "--inject-fallback-rescue",
        "fallback_edges_make_a_three_node_cycle_invalid_when_entered_mid_chain",
      ),
      expectScriptMutationRed(
        "--inject-delete-scc",
        "the component schedule must settle dependencies before their consumers",
      ),
      expectScriptMutationRed("--inject-always-valid-validator", "12px"),
      expectScriptMutationRed(
        "--inject-literal-only-verdicts",
        "source_element_computed_value_revalidates_a_substituted_standard_value",
      ),
      expectScriptMutationRed("--inject-matcher-coverage-promotion", "MatcherCoverageIncomplete"),
      expectScriptMutationRed(
        "--inject-nonconverged-return",
        "summarizes_custom_property_least_fixed_point",
      ),
      expectScriptMutationRed(
        "--inject-cycle-member-shrink",
        "the frozen witness bindings or cycle shapes changed",
      ),
      expectScriptMutationRed(
        "--inject-closed-world-certificate-disable",
        "cascade_validator_adapter_preserves_spec_grammar_outcomes",
      ),
      expectScriptMutationRed(
        "--inject-close-open-function-kind",
        "outside the derived open/closed token profile",
      ),
      expectScriptMutationRed("--inject-paint-context-keyword-removal", "context-fill"),
      expectScriptMutationRed(
        "--inject-linear-component-rescan",
        "exceeded the 1.10 linear-growth ceiling",
      ),
      expectScriptMutationRed(
        "--inject-keyword-reference-closure-drop",
        "the pinned css-tree type/property reference closure changed",
      ),
      expectScriptMutationRed(
        "--inject-accepted-keyword-removal",
        "an oracle-accepted matcher gap cannot become a definite rejection",
      ),
      expectScriptMutationRed(
        "--inject-owned-wrong-definite",
        "1 oracle-valid rows were definitely rejected",
      ),
    ];
  } finally {
    sharedMutationTargetDirectory = null;
    rmSync(mutationTargetScratch, { recursive: true, force: true });
  }

  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: "1",
      product: "omena-cascade.custom-property-fixed-point-formalization",
      correspondenceSymbolCount: correspondenceSymbols.length,
      gapCount: gapRegister.rows.length,
      caseCount: corpus.cases.length,
      agreementCount: corpus.cases.length,
      findingCount: 0,
      novelCycleCaseCount: corpus.cases.filter(
        (entry) => entry.cycleShape !== null && novelCycleShapeAllowlist.has(entry.cycleShape),
      ).length,
      postSubstitutionGrammar: "direct-and-salsa:GREEN",
      trackedVarSiteCount,
      trackedVarSiteStatuses,
      trackedStyleDiagnosticsFileCount,
      trackedStyleDiagnosticCounts,
      trackedStyleDiagnosticsCliFileCount,
      trackedStyleDiagnosticsCliCounts,
      diagnosticFivePinCensus: "GREEN",
      diagnosticTransitionAttribution: diagnosticTransitionAttributionReceipt.attribution,
      diagnosticLocationSubstitutionMutation:
        diagnosticTransitionAttributionReceipt.locationSubstitutionMutation,
      iterativeAliasBoundary: "100000:GREEN",
      variableEnvironmentPerformanceCeiling:
        "three-size-flat-control-normalized-log-log-growth-exponent<=1.10:GREEN",
      componentScheduleTraceInvariant: "GREEN",
      mutations: mutationReceipts,
      rfcDisposition: gapRegister.rfcDisposition.status,
    })}\n`,
  );
}

function relayMutationResult(result: ReturnType<typeof spawnSync>): void {
  if (result.stdout) {
    process.stdout.write(result.stdout);
  }
  if (result.stderr) {
    process.stderr.write(result.stderr);
  }
  process.exitCode = result.status ?? 1;
}

function canonicalJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonicalJson);
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([left], [right]) => Buffer.compare(Buffer.from(left), Buffer.from(right)))
        .map(([key, entry]) => [key, canonicalJson(entry)]),
    );
  }
  return value;
}

function expectScriptMutationRed(option: string, outputNeedle: string): string {
  const result = spawnSync(process.execPath, ["--import", "tsx", scriptPath, option], {
    cwd: repoRoot,
    encoding: "utf8",
    env:
      sharedMutationTargetDirectory === null
        ? process.env
        : {
            ...process.env,
            [sharedMutationTargetEnvironment]: sharedMutationTargetDirectory,
          },
  });
  const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
  assert.notEqual(result.status, 0, `${option} must be RED`);
  assert.ok(output.includes(outputNeedle), `${option} must be caught by ${outputNeedle}`);
  return `${option.slice("--inject-".length)}:RED`;
}

function runFallbackRescueMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    customPropertyPath,
    (source) =>
      replaceExactly(
        source,
        "resolved_values[*node] = Some(resolved);",
        `let rescued = match input {
                    CascadeValue::Var {
                        fallback: Some(fallback),
                        ..
                    } => (**fallback).clone(),
                    _ => CascadeValue::GuaranteedInvalid,
                };
                resolved_values[*node] = Some(rescued);`,
      ),
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cascade",
      "fallback_edges_make_a_three_node_cycle_invalid_when_entered_mid_chain",
      "--",
      "--nocapture",
    ],
  );
}

function runSccDeletionMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    customPropertyPath,
    (source) =>
      replaceExactly(
        source,
        "if component_is_cyclic(component, &dependency_graph) {",
        "if false && component_is_cyclic(component, &dependency_graph) {",
      ),
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
  );
}

function runAlwaysValidValidatorMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    grammarPath,
    (source) => {
      const start = source.indexOf(
        "impl CascadeStandardValueValidatorV0 for SpecStandardPropertyValueValidatorV0 {",
      );
      const end = source.indexOf("\npub fn validate_registered_property_value_v0", start);
      assert.ok(start >= 0 && end > start, "cannot locate the standard-value validator adapter");
      const replacement = `impl CascadeStandardValueValidatorV0 for SpecStandardPropertyValueValidatorV0 {
    fn validate_standard_property_value(
        &self,
        _property: &PropertyNameV0,
        _value: &str,
    ) -> CascadeStandardValueVerdictV0 {
        CascadeStandardValueVerdictV0::Matched
    }
}
`;
      return source.slice(0, start) + replacement + source.slice(end);
    },
    salsaGrammarCargoArgs(),
  );
}

function runLiteralOnlyVerdictMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    salsaPath,
    (source) =>
      replaceExactly(
        source,
        ".filter(|declaration| declaration.property_key.as_custom().is_none())",
        `.filter(|declaration| {
                declaration.property_key.as_custom().is_none()
                    && matches!(declaration.value, CascadeValue::Literal(_))
            })`,
      ),
    salsaGrammarCargoArgs(),
  );
}

function runMatcherCoveragePromotionMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    grammarPath,
    (source) =>
      replaceExactly(
        source,
        `CssValueGrammarVerdictV0::Unmatched { .. }
                if !matcher_coverage_complete && !closed_world_token_kind_mismatch =>
            {`,
        `CssValueGrammarVerdictV0::Unmatched { .. } if false => {`,
      ),
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-abstract-value",
      "incomplete_matcher_coverage_cannot_promote_unmatched_to_invalid",
      "--",
      "--nocapture",
    ],
  );
}

function runNonconvergedReturnMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    customPropertyPath,
    (source) =>
      replaceExactly(
        source,
        `iteration_bound: components.len().max(1),
        reached_fixed_point: true,`,
        `iteration_bound: components.len().max(1),
        reached_fixed_point: false,`,
      ),
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cascade",
      "summarizes_custom_property_least_fixed_point",
      "--",
      "--nocapture",
    ],
  );
}

function runClosedWorldCertificateDisableMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    grammarPath,
    (source) =>
      replaceExactly(
        source,
        `let closed_world_token_kind_mismatch =
        matches!(verdict, CssValueGrammarVerdictV0::Unmatched { .. })
            && standard_property_closed_world_token_kind_mismatch(&property, value, registry);`,
        `let closed_world_token_kind_mismatch = false;`,
      ),
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-abstract-value",
      "cascade_validator_adapter_preserves_spec_grammar_outcomes",
      "--",
      "--nocapture",
    ],
  );
}

function runCloseOpenFunctionKindMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    grammarPath,
    (source) =>
      replaceExactly(
        source,
        `let Some(profile) = cached_standard_property_closed_world_token_profile(property, registry)
    else {
        return false;
    };`,
        `let Some(mut profile) = cached_standard_property_closed_world_token_profile(property, registry)
    else {
        return false;
    };
    profile.function_name.open = false;`,
      ),
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-abstract-value",
      "valid_declaration_corpus_covers_closed_ident_edges_without_definite_rejection",
      "--",
      "--nocapture",
    ],
  );
}

function runPaintContextKeywordRemovalMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    grammarOverridePath,
    (source) =>
      replaceExactly(
        source,
        '"replacementSyntax": "none | <color> | <url> [ none | <color> ]? | context-fill | context-stroke"',
        '"replacementSyntax": "none | <color> | <url> [ none | <color> ]? | context-stroke"',
      ),
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-query",
      "read_cascade_at_position_resolves_paint_values_through_the_pinned_matcher",
      "--",
      "--nocapture",
    ],
  );
}

function runAcceptedKeywordRemovalMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    grammarPath,
    (source) =>
      replaceExactly(
        source,
        `property_test.accepted_keywords.iter().cloned().collect(),`,
        `property_test
                    .accepted_keywords
                    .iter()
                    .filter(|keyword| {
                        !(property_test.property == "content"
                            && keyword.as_str() == "open-quote")
                    })
                    .cloned()
                    .collect(),`,
      ),
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-abstract-value",
      "accepted_keyword_authority_prevents_oracle_valid_ident_rejection",
      "--",
      "--nocapture",
    ],
  );
}

function runLinearComponentRescanMutation(): ReturnType<typeof spawnSync> {
  return runSourceMutation(
    customPropertyPath,
    (source) =>
      replaceExactly(
        source,
        "for component_index in component_schedule {",
        `for component_index in component_schedule {
        let _remaining_var_count = std::hint::black_box(
            env.values()
                .filter(|value| cascade_value_contains_var_reference(value))
                .count(),
        );`,
      ),
    [
      "test",
      "--release",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cascade",
      "variable_environment_resolution_and_summary_stay_within_linear_growth_noise_budget",
      "--",
      "--ignored",
      "--nocapture",
    ],
  );
}

function runValueGrammarDifferentialFault(environmentName: string): ReturnType<typeof spawnSync> {
  return spawnSync(process.execPath, ["--import", "tsx", valueGrammarDifferentialPath], {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      [environmentName]: "1",
    },
  });
}

function runSourceMutation(
  targetPath: string,
  mutate: (source: string) => string,
  cargoArgs: string[],
): ReturnType<typeof spawnSync> {
  const scratch = mkdtempSync(path.join(tmpdir(), "omena-custom-property-mutation-"));
  const worktree = path.join(scratch, "worktree");
  const added = spawnSync("git", ["worktree", "add", "--detach", worktree, "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(added.status, 0, `cannot create mutation worktree: ${added.stderr}`);
  try {
    for (const file of [
      customPropertyPath,
      computedValuePath,
      cascadeTestsPath,
      cascadeLibPath,
      modelPath,
      evaluatorPath,
      corpusPath,
      grammarPath,
      queryCorePath,
      salsaPath,
      cascadePositionPath,
    ]) {
      copyFileSync(path.join(repoRoot, file), path.join(worktree, file));
    }
    const mutationTarget = path.join(worktree, targetPath);
    writeFileSync(mutationTarget, mutate(readFileSync(mutationTarget, "utf8")));
    return runCargo(
      worktree,
      cargoArgs,
      "pipe",
      process.env,
      process.env[sharedMutationTargetEnvironment] ?? path.join(scratch, "target"),
    );
  } finally {
    spawnSync("git", ["worktree", "remove", "--force", worktree], {
      cwd: repoRoot,
      encoding: "utf8",
    });
    rmSync(scratch, { recursive: true, force: true });
  }
}

function replaceExactly(source: string, needle: string, replacement: string): string {
  assert.equal(
    source.split(needle).length - 1,
    1,
    `expected one source mutation needle: ${needle}`,
  );
  return source.replace(needle, replacement);
}

function runWitness(
  cwd: string,
  perturbation: string | null,
  stdio: "inherit" | "pipe",
): ReturnType<typeof spawnSync> {
  const environment = { ...process.env };
  delete environment.OMENA_CUSTOM_PROPERTY_WITNESS_PERTURBATION;
  if (perturbation !== null) {
    environment.OMENA_CUSTOM_PROPERTY_WITNESS_PERTURBATION = perturbation;
  }
  return runCargo(
    cwd,
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
    stdio,
    environment,
  );
}

function salsaGrammarCargoArgs(): string[] {
  return [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "--features",
    "salsa-memo",
    "source_element_computed_value_revalidates_a_substituted_standard_value",
    "--",
    "--nocapture",
  ];
}

function runSalsaGrammarTest(cwd: string, stdio: "inherit" | "pipe"): ReturnType<typeof spawnSync> {
  return runCargo(cwd, salsaGrammarCargoArgs(), stdio);
}

function runCargo(
  cwd: string,
  cargoArgs: string[],
  stdio: "inherit" | "pipe" = "inherit",
  environment: NodeJS.ProcessEnv = process.env,
  targetDirectory: string = path.join(repoRoot, "rust/target"),
): ReturnType<typeof spawnSync> {
  return spawnSync("cargo", cargoArgs, {
    cwd,
    encoding: "utf8",
    env: {
      ...environment,
      CARGO_TARGET_DIR: targetDirectory,
    },
    stdio,
  });
}

function validateDiagnosticCensus(census: DiagnosticCensus, lintSource: string): void {
  const expectedPins = [
    ["pre-structural-evaluator-baseline", "8017f20a60c622161fbdcabb38c2065ce9efc7d1"],
    ["structural-evaluator-and-grammar-port", "caa53005e4414c7c0d89bb12f1a42427e91b8407"],
    ["closed-world-token-certificate", "ceb18cd8a723b5153a86ce932d6a88528b76e781"],
    ["type-expanded-keyword-closure", "76574230a2e807f243f04e09944f6696d51541b8"],
    ["accepted-keyword-rejection-authority", "26450dbf146807be958b14547393dd57852dbc45"],
  ];
  const surfaceIds: DiagnosticSurfaceId[] = [
    "recommendedLint",
    "defaultStyleDiagnosticsCli",
    "queryStyleDiagnostics",
  ];

  assert.equal(census.schemaVersion, "0");
  assert.equal(census.product, "omena-query.tracked-style-diagnostics-five-pin-census");
  assert.equal(census.measuredAt, "2026-08-25");
  assert.equal(census.corpus.styleFileCount, 200);
  assert.equal(census.corpus.recommendedLintSourceFileCount, 1_021);
  assert.deepEqual(
    census.pins.map(({ id, sourcePin }) => [id, sourcePin]),
    expectedPins,
    "the five diagnostic pins changed",
  );

  for (const pin of census.pins) {
    assert.match(pin.sourcePin, /^[0-9a-f]{40}$/u);
    assert.deepEqual(Object.keys(pin.measurements).sort(), [...surfaceIds].sort());
    for (const surface of surfaceIds) {
      const measurement = pin.measurements[surface];
      assert.equal(
        Object.values(measurement.countsByRule).reduce((sum, count) => sum + count, 0),
        measurement.findingCount,
        `${pin.id}/${surface} finding count is not the sum of its rule counts`,
      );
    }
  }

  const [baseline, structuralPort, closedWorldCertificate, typeExpandedClosure, current] =
    census.pins;
  assert.deepEqual(
    baseline.measurements,
    structuralPort.measurements,
    "the baseline-to-port diagnostic delta must remain empty",
  );
  const transitions = [
    {
      from: structuralPort,
      to: closedWorldCertificate,
      expected: [
        "defaultStyleDiagnosticsCli\u0000invalidPropertyValue\u0000-3",
        "defaultStyleDiagnosticsCli\u0000sassModuleSymlinkResolution\u00004",
        "defaultStyleDiagnosticsCli\u0000unresolvedExternalReference\u0000-4",
        "queryStyleDiagnostics\u0000invalidPropertyValue\u0000-3",
        "recommendedLint\u0000invalid-property-value\u0000-3",
      ].sort(),
    },
    {
      from: closedWorldCertificate,
      to: typeExpandedClosure,
      expected: [
        "defaultStyleDiagnosticsCli\u0000invalidPropertyValue\u0000-12",
        "queryStyleDiagnostics\u0000invalidPropertyValue\u0000-12",
        "recommendedLint\u0000invalid-property-value\u0000-12",
      ].sort(),
    },
    {
      from: typeExpandedClosure,
      to: current,
      expected: [
        "defaultStyleDiagnosticsCli\u0000invalidPropertyValue\u000012",
        "queryStyleDiagnostics\u0000invalidPropertyValue\u000012",
        "recommendedLint\u0000invalid-property-value\u000012",
      ].sort(),
    },
  ];
  for (const transition of transitions) {
    const observed = diagnosticRuleDeltas(transition.from, transition.to);
    const declared = census.declaredDeltas
      .filter(
        (entry) =>
          entry.fromPin === transition.from.sourcePin && entry.toPin === transition.to.sourcePin,
      )
      .flatMap((entry) => entry.ruleDeltas)
      .map(({ surface, ruleId, delta }) => `${surface}\u0000${ruleId}\u0000${delta}`)
      .sort();
    assert.deepEqual(observed, declared, "every diagnostic count change must have an owner");
    assert.deepEqual(observed, transition.expected, "the measured diagnostic deltas changed");
  }
  assert.equal(
    census.declaredDeltas.length,
    4,
    "only the three measured transitions may own diagnostic deltas",
  );

  const certificateDelta = census.declaredDeltas.find(
    (entry) => entry.owner === "closed-world-token-kind-certificate",
  );
  assert.ok(certificateDelta, "the closed-world diagnostic delta owner is absent");
  assert.deepEqual(
    certificateDelta.locations?.map(({ path: sourcePath, line, character }) => ({
      sourcePath,
      line,
      character,
    })),
    [
      {
        sourcePath: "examples/src/scenarios/18-less-module/LessModule.module.less",
        line: 8,
        character: 3,
      },
      {
        sourcePath: "examples/src/scenarios/18-less-module/LessModule.module.less",
        line: 13,
        character: 5,
      },
      {
        sourcePath: "test/_fixtures/sdk-cross-surface-parity/mixins.module.less",
        line: 7,
        character: 5,
      },
    ],
    "the three reviewed uncertainty reductions changed",
  );

  const locationOwner = census.declaredDeltas.find(
    (entry) => entry.owner === "type-expanded-keyword-closure",
  );
  const replayOwner = census.declaredDeltas.find(
    (entry) => entry.owner === "accepted-keyword-rejection-authority",
  );
  assert.ok(locationOwner?.locations, "the measured diagnostic location set is absent");
  assert.equal(locationOwner.locationSetOwner, undefined);
  assert.ok(replayOwner, "the accepted-keyword diagnostic delta owner is absent");
  assert.equal(replayOwner.locations, undefined, "the diagnostic location set must not be copied");
  assert.equal(replayOwner.locationSetOwner, locationOwner.owner);
  assert.equal(locationOwner.locations.length, 12);
  assert.equal(
    locationOwner.locations.filter(({ classification }) => classification === "knownFalsePositive")
      .length,
    11,
  );
  assert.deepEqual(
    locationOwner.locations
      .filter(({ classification }) => classification === "truePositive")
      .map(({ path: sourcePath, line, character }) => ({ sourcePath, line, character })),
    [
      {
        sourcePath: "scripts/fixtures/real-workspace-lint-corpus/src/styles/Card.module.scss",
        line: 12,
        character: 3,
      },
    ],
    "the pinned true-positive diagnostic attribution changed",
  );
  assert.ok(
    locationOwner.locations.every(({ coordinateSystem }) => coordinateSystem === "one-based"),
    "diagnostic attribution coordinates must remain one-based",
  );

  assert.ok(
    lintSource.includes("fn tracked_workspace_recommended_lint_preserves_the_pinned_rule_census()"),
    "the full recommended-lint receipt arm is absent",
  );
  assert.ok(lintSource.includes("assert_eq!(report.style_file_count, 200);"));
  assert.ok(lintSource.includes("assert_eq!(report.source_file_count, 1_021);"));
  assert.ok(
    lintSource.includes(
      `assert_eq!(report.finding_count, ${current.measurements.recommendedLint.findingCount});`,
    ),
  );
  for (const [ruleId, count] of Object.entries(current.measurements.recommendedLint.countsByRule)) {
    assert.ok(
      lintSource.includes(`("${ruleId}".to_string(), ${count}),`),
      `the full recommended-lint arm does not pin ${ruleId}=${count}`,
    );
  }
}

function diagnosticRuleDeltas(from: DiagnosticPin, to: DiagnosticPin): string[] {
  const surfaces = Object.keys(from.measurements) as DiagnosticSurfaceId[];
  return surfaces
    .flatMap((surface) => {
      const fromCounts = from.measurements[surface].countsByRule;
      const toCounts = to.measurements[surface].countsByRule;
      return [...new Set([...Object.keys(fromCounts), ...Object.keys(toCounts)])].flatMap(
        (ruleId) => {
          const delta = (toCounts[ruleId] ?? 0) - (fromCounts[ruleId] ?? 0);
          return delta === 0 ? [] : [`${surface}\u0000${ruleId}\u0000${delta}`];
        },
      );
    })
    .sort();
}

function read(filePath: string): string {
  return readFileSync(path.join(repoRoot, filePath), "utf8");
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
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

type DiagnosticSurfaceId =
  | "recommendedLint"
  | "defaultStyleDiagnosticsCli"
  | "queryStyleDiagnostics";

type DiagnosticMeasurement = {
  findingCount: number;
  countsByRule: Record<string, number>;
};

type DiagnosticPin = {
  id: string;
  sourcePin: string;
  measurements: Record<DiagnosticSurfaceId, DiagnosticMeasurement>;
};

type DiagnosticCensus = {
  schemaVersion: string;
  product: string;
  measuredAt: string;
  corpus: {
    styleFileCount: number;
    recommendedLintSourceFileCount: number;
  };
  pins: DiagnosticPin[];
  declaredDeltas: Array<{
    owner: string;
    fromPin: string;
    toPin: string;
    ruleDeltas: Array<{
      surface: DiagnosticSurfaceId;
      ruleId: string;
      delta: number;
    }>;
    locations?: Array<{
      path: string;
      line: number;
      character: number;
      coordinateSystem?: string;
      classification?: string;
    }>;
    locationSetOwner?: string;
  }>;
};

type WitnessCase = {
  id: string;
  cycleShape: string | null;
  bindings: Record<string, unknown>;
  expectedDisposition: string;
  expectedEvaluator: unknown;
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
  cases: WitnessCase[];
};
