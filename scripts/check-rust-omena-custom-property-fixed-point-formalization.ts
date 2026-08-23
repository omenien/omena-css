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
]);
assert.deepEqual(
  [...args].filter((argument) => argument.startsWith("--") && !allowedOptions.has(argument)),
  [],
  "unknown custom-property formalization option",
);
assert.ok(args.size <= 1, "inject exactly one custom-property mutation at a time");

const documentPath = "docs/concepts/custom-property-bounded-substitution.md";
const gapRegisterPath = "rust/omena-custom-property-fixed-point-gap-register.json";
const corpusPath =
  "rust/crates/omena-cascade/tests/fixtures/custom-property-fixed-point-witness-v1.json";
const evaluatorPath = "rust/crates/omena-cascade/tests/custom_property_fixed_point_witness.rs";
const customPropertyPath = "rust/crates/omena-cascade/src/custom_property.rs";
const computedValuePath = "rust/crates/omena-cascade/src/computed_value.rs";
const cascadeTestsPath = "rust/crates/omena-cascade/src/tests.rs";
const cascadeLibPath = "rust/crates/omena-cascade/src/lib.rs";
const modelPath = "rust/crates/omena-cascade/src/model.rs";
const grammarPath = "rust/crates/omena-abstract-value/src/value_grammar.rs";
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
const corpus = JSON.parse(read(corpusPath)) as WitnessCorpus;

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
if (args.has("--inject-nonconverged-return")) {
  customProperty = customProperty.replace(
    "reached_fixed_point: true,",
    "reached_fixed_point: false,",
  );
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
    customProperty.includes("resolved_env.insert(name.clone(), CascadeValue::GuaranteedInvalid);"),
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
  ["plain-two-cycle", null],
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
    "mutuallyRecursiveFallbackChain",
    "cycleThroughFallback",
    "threeNodeFallbackCycleEnteredMidChain",
  ],
  "the named non-degenerate cycle-shape allowlist changed",
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
} else if (args.size === 0) {
  const normal = runWitness(repoRoot, null, "inherit");
  assert.equal(normal.status, 0, "the independent all-bottom witness corpus must pass");
  const directGrammar = runCargo(repoRoot, [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cascade",
    "standard_property_syntax_is_revalidated_after_var_substitution",
    "--",
    "--nocapture",
  ]);
  assert.equal(directGrammar.status, 0, "the direct post-substitution grammar arm must pass");
  const salsaGrammar = runSalsaGrammarTest(repoRoot, "inherit");
  assert.equal(salsaGrammar.status, 0, "the salsa-fed post-substitution grammar arm must pass");
  const coverageCorpus = runCargo(repoRoot, [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "tracked_thirty_six_var_sites_have_no_undeclared_status_delta",
    "--",
    "--nocapture",
  ]);
  assert.equal(coverageCorpus.status, 0, "the 36-site status corpus must pass");
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
  const flatPerformance = runCargo(repoRoot, [
    "test",
    "--release",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cascade",
    "flat_environment_resolution_stays_within_twice_the_clone_pickup",
    "--",
    "--ignored",
    "--nocapture",
  ]);
  assert.equal(flatPerformance.status, 0, "the flat-environment 2x performance ceiling must pass");
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

  const mutationReceipts = [
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
    expectScriptMutationRed("--inject-delete-scc", "implementation projection"),
    expectScriptMutationRed("--inject-always-valid-validator", "12px"),
    expectScriptMutationRed(
      "--inject-literal-only-verdicts",
      "source_element_computed_value_revalidates_a_substituted_standard_value",
    ),
    expectScriptMutationRed("--inject-matcher-coverage-promotion", "MatcherCoverageIncomplete"),
    expectScriptMutationRed(
      "--inject-nonconverged-return",
      "retired iterate-and-rescue code remains: reached_fixed_point: false",
    ),
  ];

  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: "1",
      product: "omena-cascade.custom-property-fixed-point-formalization",
      correspondenceSymbolCount: correspondenceSymbols.length,
      gapCount: gapRegister.rows.length,
      caseCount: corpus.cases.length,
      agreementCount: corpus.cases.length,
      findingCount: 0,
      novelCycleCaseCount: corpus.cases.filter((entry) => entry.cycleShape !== null).length,
      postSubstitutionGrammar: "direct-and-salsa:GREEN",
      trackedVarSiteStatusCorpus: "36/36:GREEN",
      iterativeAliasBoundary: "100000:GREEN",
      flatEnvironmentPerformanceCeiling: "2x:GREEN",
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

function expectScriptMutationRed(option: string, outputNeedle: string): string {
  const result = spawnSync(process.execPath, ["--import", "tsx", scriptPath, option], {
    cwd: repoRoot,
    encoding: "utf8",
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
        "resolved_env.insert(name.clone(), CascadeValue::GuaranteedInvalid);",
        `let rescued = env
                    .get(name)
                    .and_then(|value| match value {
                        CascadeValue::Var {
                            fallback: Some(fallback),
                            ..
                        } => Some((**fallback).clone()),
                        _ => None,
                    })
                    .unwrap_or(CascadeValue::GuaranteedInvalid);
                resolved_env.insert(name.clone(), rescued);`,
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
        "CssValueGrammarVerdictV0::Unmatched { .. } if !matcher_coverage_complete => (",
        "CssValueGrammarVerdictV0::Unmatched { .. } if false => (",
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
    return runCargo(worktree, cargoArgs, "pipe", process.env, path.join(scratch, "target"));
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

type WitnessCase = {
  id: string;
  cycleShape: string | null;
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
