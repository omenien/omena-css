import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const lawverePath = "rust/crates/omena-lawvere/src/lib.rs";
const executorPath = "rust/crates/omena-transform-passes/src/runtime/executor.rs";
const independencePath = "rust/crates/omena-lawvere/src/independence.rs";
const dataPath = "rust/crates/omena-lawvere/data/transform-catalog-independence-v0.json";
const args = new Set(process.argv.slice(2));
const injectRankHint = args.has("--inject-rank-hint");
const injectO1Bypass = args.has("--inject-o1-bypass");
const injectDependentPair = args.has("--inject-dependent-pair");
const injectEmptyIndependence = args.has("--inject-empty-independence");

assert.ok(
  [injectRankHint, injectO1Bypass, injectDependentPair, injectEmptyIndependence].filter(Boolean)
    .length <= 1,
  "only one proof-kernel falsifier may run at once",
);

let lawvereSource = read(lawverePath);
let executorSource = read(executorPath);
const independenceSource = read(independencePath);
const independenceData = JSON.parse(read(dataPath)) as {
  entries: readonly { disposition: string }[];
  profiles: readonly unknown[];
};

if (injectRankHint) {
  lawvereSource = lawvereSource.replace(
    "rank_clusters: transform_catalog_independence_clusters_v0(requested),",
    "rank_clusters: transform_catalog_equation_clusters_v0(requested_pass_ids.as_slice()),",
  );
}
if (injectO1Bypass) {
  executorSource = executorSource.replace(
    "checked_token_ownership_admission_v0(census, module_instance, pass_kind).is_none()",
    "false",
  );
}

const planBody = functionBody(lawvereSource, "plan_transform_catalog_parallel_layers_v0");
const layerBody = functionBody(lawvereSource, "transform_catalog_independence_clusters_v0");
const o1Body = functionBody(executorSource, "closed_world_admission_o1_reasons");
const tokenConsumerBody = functionBody(executorSource, "checked_token_ownership_admission_v0");

assert.match(planBody, /transform_catalog_independence_clusters_v0\(requested\)/);
assert.doesNotMatch(planBody, /transform_catalog_equation_clusters_v0/);
assert.doesNotMatch(planBody, /transform_catalog_execution_rank_hint/);
assert.doesNotMatch(layerBody, /transform_catalog_equation_clusters_v0/);
assert.doesNotMatch(layerBody, /transform_catalog_execution_rank_hint/);
assert.match(planBody, /scheduler_status: "independenceDataReady"/);
assert.match(planBody, /executor_consumes_plan: false/);
assert.match(
  lawvereSource,
  /TRANSFORM_CATALOG_PLAN_NON_CONSUMPTION_REASON_V0:[\s\S]*executorKeepsValidatedSerialDagUntilParallelApplicationSemanticsLand/,
);
assert.match(o1Body, /checked_token_ownership_admission_v0/);
assert.match(o1Body, /proofKernelToken:/);
assert.match(tokenConsumerBody, /check_rewrite_certificate_v0/);
assert.match(tokenConsumerBody, /matches_endpoints_v0/);
assert.match(independenceSource, /default_transform_observation_matrix_v0\(\)/);
assert.match(independenceSource, /checked_adjacent_swap_token_v0/);
assert.match(independenceSource, /canonicalize_transform_catalog_schedule_v0/);
assert.equal(independenceData.profiles.length, 1);
assert.equal(
  independenceData.entries.filter((entry) => entry.disposition === "independent").length,
  1,
);

const kernelOutput = cargoTest(
  "omena-cascade-proof",
  "proof_kernel::tests::transform_independence_requires_observation_and_precondition_halves",
);
const ownershipOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::proof_kernel_token_closes_favourable_ownership_count_bypass",
  true,
);
const observationOutput = cargoTest(
  "omena-transform-cst",
  "observation_equivalence::tests::cascade_winner_change_only_reddens_profiles_that_observe_it",
);
const r18Output = cargoTest(
  "omena-lawvere",
  "independence::tests::canonical_schedule_agrees_with_independent_bounded_oracle",
  true,
);
const productRowsOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::committed_independence_rows_match_product_transform_outputs",
  true,
);
const r19ProductOutput = cargoTest(
  "omena-transform-passes",
  "tests::runtime_boundary::nested_color_lowering_conflict_has_swapped_order_output_divergence",
  true,
);
const r19CheckerOutput = cargoTest(
  "omena-lawvere",
  "independence::tests::dependent_pair_injection_is_rejected_by_data_and_s1_checker",
  true,
);
const r20Output = cargoTest(
  "omena-lawvere",
  "independence::tests::empty_independence_data_collapses_parallel_width_to_one",
  true,
);

assert.match(kernelOutput, /test result: ok/);
assert.match(ownershipOutput, /test result: ok/);
assert.match(observationOutput, /test result: ok/);
assert.match(r18Output, /bound=4 permutations=24[\s\S]*canonicalOracleAgreement=true/);
assert.match(productRowsOutput, /test result: ok/);
assert.match(r19ProductOutput, /equal=false/);
assert.match(r19CheckerOutput, /dataValidation=Err/);
assert.match(r19CheckerOutput, /S1 checker rejected reorder certificate/);
assert.match(r20Output, /entries=0 layers=2 maxParallelWidth=1/);

if (injectDependentPair) {
  assert.doesNotMatch(
    r19CheckerOutput,
    /dataValidation=Err|S1 checker rejected reorder certificate/,
    "injected dependent pair was rejected instead of being admitted",
  );
}
if (injectEmptyIndependence) {
  assert.match(
    r20Output,
    /maxParallelWidth=[2-9]/,
    "empty independence data unexpectedly collapsed parallel width to one",
  );
}

process.stdout.write(
  [
    "proof-kernel gate: ok",
    "checkerSideConditions=tokenOwnershipSeparability,transformIndependence",
    "observationProfiles=1 independentPairs=1 dependentPairs=1",
    "scheduleOracleBound=4 schedulePermutations=24",
    "executorConsumesPlan=false nonConsumptionReason=executorKeepsValidatedSerialDagUntilParallelApplicationSemanticsLand",
    "o1Consumer=closed_world_admission_o1_reasons",
    "",
  ].join("\n"),
);

function cargoTest(packageName: string, testName: string, allFeatures = false): string {
  const cargoArgs = ["test", "--manifest-path", "rust/Cargo.toml", "-p", packageName];
  if (allFeatures) cargoArgs.push("--all-features");
  cargoArgs.push(testName, "--", "--exact", "--nocapture");
  return execFileSync("cargo", cargoArgs, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function read(sourcePath: string): string {
  return fs.readFileSync(path.join(repoRoot, sourcePath), "utf8");
}

function functionBody(source: string, functionName: string): string {
  const anchor = source.indexOf(`fn ${functionName}`);
  assert.notEqual(anchor, -1, `missing function ${functionName}`);
  const brace = source.indexOf("{", anchor);
  assert.notEqual(brace, -1, `missing body for ${functionName}`);
  let depth = 0;
  for (let index = brace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(brace + 1, index);
    }
  }
  assert.fail(`unterminated function ${functionName}`);
}
