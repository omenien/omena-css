import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

interface StrictBuildProfileV0 {
  readonly profileId: string;
  readonly passIds: readonly string[];
}

interface StrictPolicyEventV0 {
  readonly passId: string;
  readonly reasons: readonly {
    readonly kind: string;
  }[];
}

interface ConsumerBuildSummaryV0 {
  readonly unknownPassIds: readonly string[];
  readonly execution: {
    readonly outputCss: string;
    readonly executedPassIds: readonly string[];
    readonly decisions: readonly {
      readonly kind: string;
      readonly reason?: {
        readonly kind: string;
      };
    }[];
    readonly strictPolicy: {
      readonly profileId: string | null;
      readonly refusedCount: number;
      readonly rolledBackCount: number;
      readonly refusalReasons: readonly StrictPolicyEventV0[];
      readonly rollbackReasons: readonly StrictPolicyEventV0[];
    };
  };
}

function runRunner<T>(command: string, input: unknown): T {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--",
      command,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 16 * 1024 * 1024,
    },
  );

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.error, undefined);
  return JSON.parse(result.stdout) as T;
}

function runRustTest(packageName: string, testName: string, testTarget?: string): void {
  const args = ["test", "--manifest-path", "rust/Cargo.toml", "-p", packageName];
  if (testTarget) {
    args.push("--test", testTarget);
  }
  args.push(testName, "--", "--exact");
  const result = spawnSync("cargo", args, {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /1 passed; 0 failed/u);
}

const expectedStrictPassIds = [
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "nesting-unwrap",
  "scope-flatten",
  "layer-flatten",
];
const profile = runRunner<StrictBuildProfileV0>("consumer-strict-verification-profile", {});
assert.equal(profile.profileId, "strict-verification");
assert.deepEqual(profile.passIds, expectedStrictPassIds);
assert.ok(!profile.passIds.includes("empty-rule-removal"));

const styleSource = ".a { color: red; } .a { background: blue; }";
const strictRefusal = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-source", {
  stylePath: "fixture.css",
  styleSource,
  requestedPassIds: ["rule-merging"],
  verificationProfile: "strict",
});
assert.equal(strictRefusal.execution.outputCss, styleSource);
assert.equal(strictRefusal.execution.strictPolicy.profileId, "strict-verification");
assert.equal(strictRefusal.execution.strictPolicy.refusedCount, 1);
assert.equal(strictRefusal.execution.strictPolicy.rolledBackCount, 0);
assert.equal(strictRefusal.execution.strictPolicy.refusalReasons[0]?.passId, "rule-merging");
assert.deepEqual(strictRefusal.execution.strictPolicy.refusalReasons[0]?.reasons, [
  { kind: "cascadeEnvironmentUnavailable" },
]);
assert.equal(strictRefusal.execution.decisions[0]?.kind, "blocked");
assert.equal(strictRefusal.execution.decisions[0]?.reason?.kind, "strictVerification");

const strictClean = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-source", {
  stylePath: "fixture.css",
  styleSource,
  requestedPassIds: ["rule-merging"],
  verificationProfile: "strict",
  transformContext: {
    cascadeEnvironment: {
      stylesheetSourceOrderBase: 0,
      declarations: [],
    },
  },
});
assert.equal(strictClean.execution.strictPolicy.refusedCount, 0);
assert.equal(strictClean.execution.strictPolicy.rolledBackCount, 0);
assert.deepEqual(strictClean.execution.strictPolicy.refusalReasons, []);
assert.deepEqual(strictClean.execution.strictPolicy.rollbackReasons, []);
assert.deepEqual(strictClean.execution.executedPassIds, ["rule-merging"]);
assert.notEqual(strictClean.execution.outputCss, styleSource);

const descriptive = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-source", {
  stylePath: "fixture.css",
  styleSource,
  requestedPassIds: ["rule-merging"],
});
assert.equal(descriptive.execution.strictPolicy.profileId, null);
assert.equal(descriptive.execution.strictPolicy.refusedCount, 0);
assert.equal(descriptive.execution.strictPolicy.rolledBackCount, 0);
assert.deepEqual(descriptive.execution.strictPolicy.refusalReasons, []);
assert.deepEqual(descriptive.execution.strictPolicy.rollbackReasons, []);
assert.notEqual(descriptive.execution.outputCss, styleSource);

const unknown = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-source", {
  stylePath: "fixture.css",
  styleSource,
  requestedPassIds: ["not-a-transform-pass"],
  verificationProfile: "strict",
});
assert.deepEqual(unknown.unknownPassIds, ["not-a-transform-pass"]);
assert.equal(unknown.execution.strictPolicy.refusedCount, 1);
assert.deepEqual(unknown.execution.strictPolicy.refusalReasons[0]?.reasons, [
  { kind: "unknownPass" },
]);
assert.deepEqual(unknown.execution.decisions, []);

runRustTest(
  "omena-transform-passes",
  "runtime::executor::dispatch_table_tests::strict_winner_difference_restores_the_input_ir",
);
runRustTest(
  "omena-transform-passes",
  "tests::rule_optimization::strict_verification_rolls_back_when_required_observation_is_unavailable",
);
runRustTest(
  "omena-query",
  "explain::tests::transform_explanation_surfaces_strict_policy_counts_and_reasons",
);
runRustTest("omena-query", "tests::shared_build_admission_reports_preflight_and_coverage_failures");
runRustTest(
  "omena-query",
  "workspace_runtime_executes_every_typed_workflow",
  "sdk_workflow_contract",
);
runRustTest("omena-cli", "tests::build_strict_verification_refuses_unestablished_winner_evidence");
runRustTest("omena-napi", "tests::strict_build_surfaces_typed_refusal");
runRustTest("omena-wasm", "tests::strict_build_surfaces_typed_refusal");

console.log(
  JSON.stringify(
    {
      product: "omena-query-strict-verification-gate",
      strictPassIds: profile.passIds,
      refusalReason: "cascadeEnvironmentUnavailable",
      rollbackTest: "passed",
      explainSurfaceTest: "passed",
      sharedAdmissionTest: "passed",
      sdkEnvelopeTest: "passed",
      cliSurfaceTest: "passed",
      napiSurfaceTest: "passed",
      wasmSurfaceTest: "passed",
      descriptiveCounts: {
        refused: descriptive.execution.strictPolicy.refusedCount,
        rolledBack: descriptive.execution.strictPolicy.rolledBackCount,
      },
    },
    null,
    2,
  ),
);
