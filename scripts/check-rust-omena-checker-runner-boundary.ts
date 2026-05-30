import { readFileSync } from "node:fs";
import path from "node:path";
import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

const RUNNER_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/src/main.rs");
const RUNNER_CARGO_PATH = path.join(process.cwd(), "rust/crates/engine-shadow-runner/Cargo.toml");
const PACKAGE_JSON_PATH = path.join(process.cwd(), "package.json");

const runnerSource = readFileSync(RUNNER_PATH, "utf8");
const runnerCargoToml = readFileSync(RUNNER_CARGO_PATH, "utf8");
const packageJsonSource = readFileSync(PACKAGE_JSON_PATH, "utf8");
const commandBodies = extractCommandBodies(runnerSource);

const checkerMTierBody = commandBodies.get("omena-checker-m-tier-evaluations");
const checkerCascadeBody = commandBodies.get("omena-checker-cascade-evaluations");
const checkerGrnBody = commandBodies.get("omena-checker-grn-evaluations");
const checkerSmtBody = commandBodies.get("omena-checker-smt-evaluations");
const checkerMdlBody = commandBodies.get("omena-checker-mdl-evaluations");
const checkerStreamingIfdsBody = commandBodies.get("omena-checker-streaming-ifds-evaluations");
const checkerRgFlowBody = commandBodies.get("omena-checker-rg-flow-evaluations");
const checkerReplicaEnsembleBody = commandBodies.get("omena-checker-replica-ensemble-evaluations");
const checkerCategoricalBody = commandBodies.get("omena-checker-categorical-evaluations");
assert.ok(
  checkerMTierBody,
  "missing engine-shadow-runner command arm: omena-checker-m-tier-evaluations",
);
assert.ok(
  checkerCascadeBody,
  "missing engine-shadow-runner command arm: omena-checker-cascade-evaluations",
);
assert.ok(
  checkerGrnBody,
  "missing engine-shadow-runner command arm: omena-checker-grn-evaluations",
);
assert.ok(
  checkerSmtBody,
  "missing engine-shadow-runner command arm: omena-checker-smt-evaluations",
);
assert.ok(
  checkerMdlBody,
  "missing engine-shadow-runner command arm: omena-checker-mdl-evaluations",
);
assert.ok(
  checkerStreamingIfdsBody,
  "missing engine-shadow-runner command arm: omena-checker-streaming-ifds-evaluations",
);
assert.ok(
  checkerRgFlowBody,
  "missing engine-shadow-runner command arm: omena-checker-rg-flow-evaluations",
);
assert.ok(
  checkerReplicaEnsembleBody,
  "missing engine-shadow-runner command arm: omena-checker-replica-ensemble-evaluations",
);
assert.ok(
  checkerCategoricalBody,
  "missing engine-shadow-runner command arm: omena-checker-categorical-evaluations",
);
assert.ok(
  checkerMTierBody.includes("OmenaCheckerMTierEvaluationInputV0"),
  "omena-checker-m-tier-evaluations must deserialize the checker M-tier input product",
);
assert.ok(
  checkerCascadeBody.includes("OmenaCheckerCascadeInputV0"),
  "omena-checker-cascade-evaluations must deserialize the checker cascade input product",
);
assert.ok(
  checkerGrnBody.includes("OmenaCheckerGrnInputV0"),
  "omena-checker-grn-evaluations must deserialize the checker GRN input product",
);
assert.ok(
  checkerSmtBody.includes("OmenaCheckerSmtInputV0"),
  "omena-checker-smt-evaluations must deserialize the checker SMT input product",
);
assert.ok(
  checkerMdlBody.includes("OmenaCheckerMdlEvaluationInputV0"),
  "omena-checker-mdl-evaluations must deserialize the runner MDL input product",
);
assert.ok(
  checkerStreamingIfdsBody.includes("OmenaCheckerStreamingIfdsEvaluationInputV0"),
  "omena-checker-streaming-ifds-evaluations must deserialize the runner streaming IFDS input product",
);
assert.ok(
  checkerRgFlowBody.includes("OmenaCheckerRgFlowEvaluationInputV0"),
  "omena-checker-rg-flow-evaluations must deserialize the runner RG-flow input product",
);
assert.ok(
  checkerReplicaEnsembleBody.includes("OmenaCheckerReplicaEnsembleEvaluationInputV0"),
  "omena-checker-replica-ensemble-evaluations must deserialize the runner replica-ensemble input product",
);
assert.ok(
  checkerCategoricalBody.includes("OmenaCheckerCategoricalEvaluationInputV0"),
  "omena-checker-categorical-evaluations must deserialize the runner categorical input product",
);
assert.ok(
  checkerMTierBody.includes("summarize_omena_checker_m_tier_evaluations"),
  "omena-checker-m-tier-evaluations must route through the runner-owned checker summary wrapper",
);
assert.ok(
  checkerCascadeBody.includes("summarize_omena_checker_cascade_evaluations"),
  "omena-checker-cascade-evaluations must route through the runner-owned checker cascade summary wrapper",
);
assert.ok(
  checkerGrnBody.includes("summarize_omena_checker_grn_evaluations"),
  "omena-checker-grn-evaluations must route through the runner-owned checker GRN summary wrapper",
);
assert.ok(
  checkerSmtBody.includes("summarize_omena_checker_smt_evaluations"),
  "omena-checker-smt-evaluations must route through the runner-owned checker SMT summary wrapper",
);
assert.ok(
  checkerMdlBody.includes("summarize_omena_checker_mdl_evaluations"),
  "omena-checker-mdl-evaluations must route through the runner-owned checker MDL summary wrapper",
);
assert.ok(
  checkerStreamingIfdsBody.includes("summarize_omena_checker_streaming_ifds_evaluations"),
  "omena-checker-streaming-ifds-evaluations must route through the runner-owned checker streaming IFDS summary wrapper",
);
assert.ok(
  checkerRgFlowBody.includes("summarize_omena_checker_rg_flow_evaluations"),
  "omena-checker-rg-flow-evaluations must route through the runner-owned checker RG-flow summary wrapper",
);
assert.ok(
  checkerReplicaEnsembleBody.includes("summarize_omena_checker_replica_ensemble_evaluations"),
  "omena-checker-replica-ensemble-evaluations must route through the runner-owned checker replica-ensemble summary wrapper",
);
assert.ok(
  checkerCategoricalBody.includes("summarize_omena_checker_categorical_evaluations"),
  "omena-checker-categorical-evaluations must route through the runner-owned checker categorical summary wrapper",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_m_tier_rules"),
  "engine-shadow-runner must call omena-checker's M-tier evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_cascade_rules"),
  "engine-shadow-runner must call omena-checker's cascade-aware evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_grn_rules"),
  "engine-shadow-runner must call omena-checker's GRN evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_smt_rules"),
  "engine-shadow-runner must call omena-checker's SMT evaluator",
);
assert.ok(
  runnerSource.includes("summarize_omena_query_design_system_minimum_description"),
  "engine-shadow-runner must build MDL evidence from omena-query before checker evaluation",
);
assert.ok(
  runnerSource.includes("run_streaming_ifds_exact_v0"),
  "engine-shadow-runner must build streaming IFDS evidence before checker evaluation",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_mdl_rules"),
  "engine-shadow-runner must call omena-checker's MDL evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_streaming_ifds_rules"),
  "engine-shadow-runner must call omena-checker's streaming IFDS evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_rg_flow_rules"),
  "engine-shadow-runner must call omena-checker's RG-flow evaluator",
);
assert.ok(
  runnerSource.includes("build_cross_file_inconsistency_report"),
  "engine-shadow-runner must build replica-ensemble evidence with the real overlap-Q/SBM algorithm before checker evaluation",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_replica_ensemble_rules"),
  "engine-shadow-runner must call omena-checker's replica-ensemble evaluator",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_categorical_rules"),
  "engine-shadow-runner must call omena-checker's categorical functor-verdict evaluator",
);
assert.ok(
  runnerSource.includes('"omena-checker-m-tier-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-m-tier-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-cascade-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-cascade-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-grn-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-grn-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-smt-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-smt-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-mdl-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-mdl-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-streaming-ifds-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-streaming-ifds-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-rg-flow-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-rg-flow-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-replica-ensemble-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-replica-ensemble-evaluations",
);
assert.ok(
  runnerSource.includes('"omena-checker-categorical-evaluations" =>'),
  "engine-shadow-runner daemon must support omena-checker-categorical-evaluations",
);
assert.ok(
  /^\s*omena-checker\s*=/m.test(runnerCargoToml),
  "engine-shadow-runner must depend on omena-checker for M-tier evaluations",
);
assert.ok(
  /^\s*omena-abstract-value\s*=/m.test(runnerCargoToml),
  "engine-shadow-runner must depend on omena-abstract-value for M-tier value input construction",
);
assert.ok(
  /^\s*omena-streaming-ifds\s*=/m.test(runnerCargoToml),
  "engine-shadow-runner must depend on omena-streaming-ifds for streaming precision evidence",
);
assert.ok(
  /^\s*omena-ensemble\s*=/m.test(runnerCargoToml),
  "engine-shadow-runner must depend on omena-ensemble for replica-ensemble overlap-Q/SBM evidence",
);
assert.ok(
  packageJsonSource.includes("check:rust-omena-checker-runner-boundary"),
  "package.json must expose the checker runner boundary gate",
);
assert.ok(
  packageJsonSource.includes("rust/omena-checker/runner-boundary"),
  "rust/omena-checker/boundary must include the checker runner boundary gate",
);

const cascadeSummary = runCascadeEvaluationFixture();
assert.equal(cascadeSummary.product, "omena-checker.cascade-evaluations");
assert.equal(cascadeSummary.declarationCount, 5);
assert.equal(cascadeSummary.customPropertyCount, 3);
for (const code of [
  "unreachable-declaration",
  "dead-cascade-layer",
  "iacvt-prone",
  "circular-var",
  "unspecified-cascade-tie",
]) {
  assert.ok(
    cascadeSummary.ruleCodeNames.includes(code),
    `cascade runner output must include ${code}`,
  );
}
const grnSummary = runGrnEvaluationFixture();
assert.equal(grnSummary.product, "omena-checker.grn-evaluations");
assert.equal(grnSummary.vertexCount, 3);
for (const code of ["cascade.deep-conflict", "cascade.unreachable-rule"]) {
  assert.ok(grnSummary.ruleCodeNames.includes(code), `GRN runner output must include ${code}`);
}
const smtSummary = runSmtEvaluationFixture();
assert.equal(smtSummary.product, "omena-checker.smt-evaluations");
assert.equal(smtSummary.obligationCount, 1);
assert.ok(
  smtSummary.ruleCodeNames.includes("cascade.smt-violation"),
  "SMT runner output must include cascade.smt-violation",
);
// End-to-end mechanism-depth proof: drive the real entropy/log MDL through the
// product path (runner builds omena-query MDL, checker gates on it). Both inputs
// have IDENTICAL ruleCount(4) + observationCount(6); a degenerate count-sum MDL
// would be 10 for both. Only the value distribution differs. The uniform [3,3]
// histogram (max entropy) yields totalBits 10 > budget 9 and emits the diagnostic;
// the peaked [15,1] histogram (low entropy / compressible) yields ~6.02 < 9 and
// emits nothing. If the runner replaced the MDL with rule_count+observation_count,
// both would tie and the clear case would wrongly emit.
const mdlEmitSummary = runMdlEvaluationFixture([3, 3]);
assert.equal(mdlEmitSummary.product, "omena-checker.mdl-evaluations");
assert.equal(mdlEmitSummary.totalBits, 10);
assert.equal(mdlEmitSummary.evaluationCount, 1);
assert.ok(
  mdlEmitSummary.ruleCodeNames.includes("design-system-mdl-budget"),
  "MDL runner output must include design-system-mdl-budget",
);
const mdlClearSummary = runMdlEvaluationFixture([15, 1]);
assert.ok(
  mdlClearSummary.totalBits < mdlEmitSummary.budgetBits,
  `peaked value distribution must compress below budget, got ${mdlClearSummary.totalBits}`,
);
assert.equal(mdlClearSummary.evaluationCount, 0);
// The discriminating distribution MUST move total_bits and the diagnostic outcome.
assert.notEqual(mdlClearSummary.totalBits, mdlEmitSummary.totalBits);
assert.notEqual(mdlClearSummary.evaluationCount, mdlEmitSummary.evaluationCount);
// End-to-end mechanism-depth proof: parity is a real equality of two distinct
// computations (incremental reuse vs batch recompute), not f(x) == f(x). Both
// runs carry the SAME prior fact-key cache {a, b, c}; only the current graph
// differs by ONE load-bearing hyperedge (edge-b-c). When edge-b-c is present the
// batch oracle still produces c, so the reused prior c agrees and parity holds.
// When edge-b-c is removed the batch oracle drops c while the incremental path
// reuses the now-stale c (it is outside the dirty region), so the two fact sets
// diverge and the diagnostic fires. No precisionParityWithBatch literal is fed
// in; the runner computes it from the two real runs.
const streamingSummary = runStreamingIfdsEvaluationFixture(true);
assert.equal(streamingSummary.product, "omena-checker.streaming-ifds-evaluations");
assert.equal(streamingSummary.reportProduct, "omena-streaming-ifds.analysis-report");
assert.equal(streamingSummary.precisionParityWithBatch, true);
assert.equal(streamingSummary.evaluationCount, 0);
const streamingDivergeSummary = runStreamingIfdsEvaluationFixture(false);
assert.equal(streamingDivergeSummary.product, "omena-checker.streaming-ifds-evaluations");
assert.equal(streamingDivergeSummary.precisionParityWithBatch, false);
assert.equal(streamingDivergeSummary.evaluationCount, 1);
// The load-bearing edge change MUST move both the parity verdict and the
// diagnostic outcome between the two runs.
assert.notEqual(
  streamingDivergeSummary.precisionParityWithBatch,
  streamingSummary.precisionParityWithBatch,
);
assert.notEqual(streamingDivergeSummary.evaluationCount, streamingSummary.evaluationCount);
const rgFlowSummary = runRgFlowEvaluationFixture();
assert.equal(rgFlowSummary.product, "omena-checker.rg-flow-evaluations");
assert.equal(rgFlowSummary.flowCount, 2);
assert.ok(
  rgFlowSummary.ruleCodeNames.includes("rg-flow-relevant-operator"),
  "RG-flow runner output must include rg-flow-relevant-operator",
);
assert.equal(rgFlowSummary.evaluationCount, 1);

// End-to-end mechanism-depth proof: drive the real overlap-Q/SBM algorithm with
// two ensembles that differ ONLY in their replica winners. Disagreeing winners
// reduce overlap-Q below 1.0 and flip the recommendation to investigateRsbBroken;
// identical winners settle overlap-Q at 1.0 with no action. If the runner replaced
// build_cross_file_inconsistency_report with a constant, both runs would be equal.
const replicaDisagreeSummary = runReplicaEnsembleEvaluationFixture(false);
const replicaAgreeSummary = runReplicaEnsembleEvaluationFixture(true);
assert.equal(replicaDisagreeSummary.product, "omena-checker.replica-ensemble-evaluations");
assert.equal(replicaAgreeSummary.product, "omena-checker.replica-ensemble-evaluations");
assert.equal(
  replicaDisagreeSummary.reportProduct,
  "omena-ensemble.cross-file-inconsistency-report",
);
assert.equal(replicaDisagreeSummary.replicaCount, 5);
assert.equal(replicaAgreeSummary.replicaCount, 5);
// Disagreeing winners: real overlap-Q falls below 1.0 (computed 0.4) and the
// bimodal P(q) over a detectable composes graph yields investigateRsbBroken.
assert.ok(
  replicaDisagreeSummary.meanQ < 1.0,
  `disagreeing replicas must drive overlap-Q below 1.0, got ${replicaDisagreeSummary.meanQ}`,
);
assert.equal(replicaDisagreeSummary.meanQ, 0.4);
assert.equal(replicaDisagreeSummary.recommendation, "investigateRsbBroken");
assert.equal(replicaDisagreeSummary.evaluationCount, 1);
assert.ok(
  replicaDisagreeSummary.ruleCodeNames.includes("replica-ensemble-inconsistency"),
  "replica-ensemble runner output must include replica-ensemble-inconsistency",
);
// Identical winners: overlap-Q settles at 1.0 with the unimodal distribution and
// no recommended action. Only the load-bearing winners changed between the runs.
assert.equal(replicaAgreeSummary.meanQ, 1.0);
assert.equal(replicaAgreeSummary.recommendation, "noActionNeeded");
// The discriminating fields MUST differ between the two runs.
assert.notEqual(replicaDisagreeSummary.meanQ, replicaAgreeSummary.meanQ);
assert.notEqual(replicaDisagreeSummary.recommendation, replicaAgreeSummary.recommendation);

// End-to-end mechanism-depth proof: drive the real categorical functor through the
// product path. Both mappings carry distinct primitive->role pairs; the ONLY
// difference is the number of pairs, which decides whether the functor can witness
// composition. Three pairs give two composable non-identity morphisms, so the
// functor verdict is `accepted` and emits nothing. Two pairs give a single
// non-identity morphism that cannot be composed, so the functor verdict is rejected
// and the diagnostic fires. No `accepted`/`compositionPreserved` literal is fed in;
// the runner computes the verdict from the real functor. A constant verdict would
// make both runs tie and one of these assertions would fail.
const categoricalClearSummary = runCategoricalEvaluationFixture([
  ["cascade_property", "cosheaf colimit witness"],
  ["prove_layer_flatten_candidate", "beck-chevalley witness"],
  ["evaluate_static_supports_condition", "site decidability witness"],
]);
assert.equal(categoricalClearSummary.product, "omena-checker.categorical-evaluations");
assert.equal(categoricalClearSummary.mappingCount, 1);
assert.equal(categoricalClearSummary.evaluationCount, 0);
const categoricalEmitSummary = runCategoricalEvaluationFixture([
  ["cascade_property", "cosheaf colimit witness"],
  ["prove_layer_flatten_candidate", "beck-chevalley witness"],
]);
assert.equal(categoricalEmitSummary.evaluationCount, 1);
assert.ok(
  categoricalEmitSummary.ruleCodeNames.includes("categorical-cascade-evidence-inconsistency"),
  "categorical runner output must include categorical-cascade-evidence-inconsistency",
);
// The load-bearing morphism structure MUST move the functor verdict and the
// diagnostic outcome between the two runs.
assert.notEqual(categoricalEmitSummary.evaluationCount, categoricalClearSummary.evaluationCount);

process.stdout.write(
  [
    "validated omena-checker runner boundary:",
    "mTierCommand=omena-checker-m-tier-evaluations",
    "cascadeCommand=omena-checker-cascade-evaluations",
    "grnCommand=omena-checker-grn-evaluations",
    "smtCommand=omena-checker-smt-evaluations",
    "mdlCommand=omena-checker-mdl-evaluations",
    "streamingIfdsCommand=omena-checker-streaming-ifds-evaluations",
    "rgFlowCommand=omena-checker-rg-flow-evaluations",
    "replicaEnsembleCommand=omena-checker-replica-ensemble-evaluations",
    "categoricalCommand=omena-checker-categorical-evaluations",
    "runtime=engine-shadow-runner",
    "owner=omena-checker",
  ].join(" "),
);
process.stdout.write("\n");

function extractCommandBodies(source: string): Map<string, string> {
  const commandMatches = [...source.matchAll(/Some\("([^"]+)"\)\s*=>\s*\{/g)];
  const bodies = new Map<string, string>();

  for (const match of commandMatches) {
    const command = match[1];
    const bodyStart = match.index === undefined ? -1 : match.index + match[0].length;
    if (!command || bodyStart < 0) continue;
    bodies.set(command, readBraceBody(source, bodyStart));
  }

  return bodies;
}

function readBraceBody(source: string, bodyStart: number): string {
  let depth = 1;
  let index = bodyStart;
  while (index < source.length && depth > 0) {
    const char = source[index];
    if (char === "{") depth += 1;
    if (char === "}") depth -= 1;
    index += 1;
  }
  return source.slice(bodyStart, index - 1);
}

interface CascadeEvaluationSummary {
  readonly product: string;
  readonly declarationCount: number;
  readonly customPropertyCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface GrnEvaluationSummary {
  readonly product: string;
  readonly vertexCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface SmtEvaluationSummary {
  readonly product: string;
  readonly obligationCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface MdlEvaluationSummary {
  readonly product: string;
  readonly totalBits: number;
  readonly budgetBits: number;
  readonly evaluationCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface StreamingIfdsEvaluationSummary {
  readonly product: string;
  readonly reportProduct: string;
  readonly precisionParityWithBatch: boolean;
  readonly evaluationCount: number;
}

interface RgFlowEvaluationSummary {
  readonly product: string;
  readonly flowCount: number;
  readonly evaluationCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface ReplicaEnsembleEvaluationSummary {
  readonly product: string;
  readonly reportProduct: string;
  readonly replicaCount: number;
  readonly meanQ: number;
  readonly recommendation: string;
  readonly evaluationCount: number;
  readonly ruleCodeNames: readonly string[];
}

interface CategoricalEvaluationSummary {
  readonly product: string;
  readonly mappingCount: number;
  readonly evaluationCount: number;
  readonly ruleCodeNames: readonly string[];
}

function runCascadeEvaluationFixture(): CascadeEvaluationSummary {
  const input = {
    declarations: [
      cascadeDeclaration("base-color", ".btn", "color", "red", 1, "base", 0, false, []),
      cascadeDeclaration("override-color", ".btn", "color", "blue", 2, "overrides", 1, false, []),
      cascadeDeclaration("gap-use", ".card", "margin", "var(--gap)", 3, "components", 1, false, [
        "--gap",
      ]),
      cascadeDeclaration("tie-a", ".tie", "color", "red", 4, "utilities", 2, false, []),
      cascadeDeclaration("tie-b", ".tie", "color", "green", 5, "utilities", 2, false, []),
    ],
    customProperties: [
      { name: "--gap", dependencies: [], guaranteedInvalid: true },
      { name: "--a", dependencies: ["--b"], guaranteedInvalid: false },
      { name: "--b", dependencies: ["--a"], guaranteedInvalid: false },
    ],
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-cascade-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner cascade command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as CascadeEvaluationSummary;
}

function runGrnEvaluationFixture(): GrnEvaluationSummary {
  const input = {
    vertices: [
      grnVertex("winner", ".btn", "color", "applied"),
      grnVertex("losing-eligible", ".btn", "color", "losingButEligible"),
      grnVertex("inactive-rule", ".card", "display", "inactive"),
    ],
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-grn-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner GRN command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as GrnEvaluationSummary;
}

function runSmtEvaluationFixture(): SmtEvaluationSummary {
  const input = {
    obligations: [
      {
        obligationId: "bad-layer-flatten",
        l1Primitive: "layerFlattenCandidate",
        canonicalTerms: ["require:closed-bundle=true", "require:no-unlayered-rule=false"],
      },
    ],
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-smt-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner SMT command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as SmtEvaluationSummary;
}

function runMdlEvaluationFixture(valueFrequencies: readonly number[]): MdlEvaluationSummary {
  const input = {
    sourceUri: "file:///workspace/Button.module.css",
    sourceHash: "fixture-mdl",
    ruleCount: 4,
    observationCount: 6,
    valueFrequencies,
    budgetBits: 9,
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-mdl-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner MDL command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as MdlEvaluationSummary;
}

function runStreamingIfdsEvaluationFixture(consistent: boolean): StreamingIfdsEvaluationSummary {
  // Both variants re-seed node a with the same exact value and carry the same
  // prior fact-key cache {a, b, c} from an earlier revision. The ONLY difference
  // is whether the current graph still has edge-b-c. With it, c is reachable and
  // the reused prior c is consistent (parity holds). Without it, c is no longer
  // reachable from the batch oracle, but the incremental path still reuses the
  // stale prior c, so the two computations diverge (parity false, diagnostic).
  const hyperedges = [
    { hyperedgeId: "edge-a-b", from: "a", to: "b", edgeKind: "foreignReference" },
  ];
  if (consistent) {
    hyperedges.push({
      hyperedgeId: "edge-b-c",
      from: "b",
      to: "c",
      edgeKind: "foreignReference",
    });
  }
  const input = {
    updateId: consistent ? "streaming-update-consistent" : "streaming-update-stale",
    startNodeId: "a",
    hyperedges,
    events: [
      {
        eventId: "event-a",
        revision: 2,
        nodeId: "a",
        value: { kind: "exact", value: "button" },
      },
    ],
    previousFactKeys: ["a|exact:button", "b|exact:button", "c|exact:button"],
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-streaming-ifds-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner streaming IFDS command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as StreamingIfdsEvaluationSummary;
}

function runRgFlowEvaluationFixture(): RgFlowEvaluationSummary {
  const input = {
    flows: [
      {
        workspacePath: "workspace://critical-token-graph",
        before: rgFlowCoupling(1, 1, 0, 0),
        after: rgFlowCoupling(5, 0, 0, 8),
      },
      {
        workspacePath: "workspace://settled-token-graph",
        before: rgFlowCoupling(4, 2, 0, 0),
        after: rgFlowCoupling(3, 1, 0, 0),
      },
    ],
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-rg-flow-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner RG-flow command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as RgFlowEvaluationSummary;
}

function runReplicaEnsembleEvaluationFixture(agree: boolean): ReplicaEnsembleEvaluationSummary {
  // Two replica clusters over a composes module graph. In the disagreeing case the
  // second cluster declares different winners, so the real overlap-Q algorithm sees
  // a bimodal P(q). In the agreeing case every replica declares the same winners,
  // so overlap-Q is 1.0 everywhere. Only the winners of the second cluster change.
  const secondClusterWinners = agree ? ["red", "blue"] : ["green", "orange"];
  const input = {
    workspaceRoot: agree ? "workspace://settled" : "workspace://rsb",
    replicas: [
      replicaSnapshot("src/a.module.css", ["red", "blue"]),
      replicaSnapshot("src/b.module.css", ["red", "blue"]),
      replicaSnapshot("src/c.module.css", ["red", "blue"]),
      replicaSnapshot("src/d.module.css", secondClusterWinners),
      replicaSnapshot("src/e.module.css", secondClusterWinners),
    ],
    graphEdges: [
      moduleGraphEdge("src/a.module.css", "src/b.module.css", "composes"),
      moduleGraphEdge("src/b.module.css", "src/c.module.css", "composes"),
      moduleGraphEdge("src/a.module.css", "src/c.module.css", "composes"),
      moduleGraphEdge("src/d.module.css", "src/e.module.css", "composes"),
    ],
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-replica-ensemble-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner replica-ensemble command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as ReplicaEnsembleEvaluationSummary;
}

function runCategoricalEvaluationFixture(
  primitiveRolePairs: readonly (readonly [string, string])[],
): CategoricalEvaluationSummary {
  const input = {
    mappings: [
      {
        mappingId: "cascade-role-mapping",
        primitiveRolePairs: primitiveRolePairs.map(([primitiveName, categoricalRole]) => ({
          primitiveName,
          categoricalRole,
        })),
      },
    ],
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-categorical-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner categorical command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as CategoricalEvaluationSummary;
}

function cascadeDeclaration(
  declarationId: string,
  selector: string,
  property: string,
  value: string,
  sourceOrder: number,
  layerName: string,
  layerOrder: number,
  important: boolean,
  varReferences: readonly string[],
) {
  return {
    declarationId,
    selector,
    property,
    value,
    sourceOrder,
    layerName,
    layerOrder,
    important,
    varReferences,
  };
}

function grnVertex(vertexId: string, selector: string, property: string, state: string) {
  return {
    vertexId,
    selector,
    property,
    state,
  };
}

function rgFlowCoupling(kEnv: number, kDecl: number, kCycle: number, kDirty: number) {
  return {
    kEnv,
    kDecl,
    kCycle,
    kDirty,
  };
}

function replicaSnapshot(modulePath: string, winners: readonly string[]) {
  return {
    path: modulePath,
    winners,
  };
}

function moduleGraphEdge(fromModule: string, toModule: string, edgeKind: string) {
  return {
    fromModule,
    toModule,
    edgeKind,
  };
}
