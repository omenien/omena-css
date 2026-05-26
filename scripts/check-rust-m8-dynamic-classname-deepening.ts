import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const packageJson = readFileSync("package.json", "utf8");
const runnerSource = readFileSync("rust/crates/engine-shadow-runner/src/main.rs", "utf8");

assert.ok(
  packageJson.includes("check:rust-m8-dynamic-classname-deepening"),
  "package.json must expose the M8 dynamic className deepening gate",
);
assert.ok(
  runnerSource.includes("omena-checker-k-limited-flow-m-tier-evaluations"),
  "engine-shadow-runner must expose k-limited flow M-tier checker evaluations",
);
assert.ok(
  runnerSource.includes("analyze_k_limited_call_site_flows"),
  "k-limited flow M-tier evaluations must invoke omena-abstract-value k-CFA analysis",
);
assert.ok(
  runnerSource.includes("evaluate_omena_checker_m_tier_rules"),
  "k-limited flow M-tier evaluations must feed checker M-tier enforcement",
);

const zeroCfa = runKLimitedFlowEvaluation(0);
const twoCfa = runKLimitedFlowEvaluation(2);

assert.equal(zeroCfa.product, "omena-checker.k-limited-flow-m-tier-evaluations");
assert.equal(zeroCfa.flowProduct, "omena-abstract-value.k-limited-call-site-flow");
assert.equal(zeroCfa.contextSensitivity, "0-cfa");
assert.equal(twoCfa.contextSensitivity, "2-cfa");

const zeroPrimary = contextBySuffix(zeroCfa, "<root>");
const twoPrimary = contextBySuffix(twoCfa, "RouteA.tsx:render > PrimaryButton.tsx:className");
const twoSecondary = contextBySuffix(twoCfa, "RouteB.tsx:render > SecondaryButton.tsx:className");

assert.ok(
  zeroPrimary.ruleCodeNames.includes("no-impossible-selector"),
  "0-CFA root join must surface a missing secondary selector against a primary-only universe",
);
assert.equal(
  twoPrimary.evaluationCount,
  0,
  "2-CFA must narrow the primary call-site to a clean exact selector",
);
assert.ok(
  twoSecondary.ruleCodeNames.includes("no-impossible-selector"),
  "2-CFA must keep the secondary call-site diagnostic instead of globally suppressing it",
);
assert.ok(
  zeroCfa.evaluationCount > twoCfa.evaluationCount,
  "Increasing context depth must change checker output, not just metadata",
);

process.stdout.write(
  [
    "validated m8 dynamic className deepening:",
    `zeroCfaEvaluations=${zeroCfa.evaluationCount}`,
    `twoCfaEvaluations=${twoCfa.evaluationCount}`,
    "mechanism=omena-abstract-value.k-limited-call-site-flow",
    "product=omena-checker.k-limited-flow-m-tier-evaluations",
  ].join(" "),
);
process.stdout.write("\n");

function runKLimitedFlowEvaluation(maxContextDepth: number): KLimitedFlowSummary {
  const input = {
    maxContextDepth,
    selectorUniverse: ["btn-primary"],
    contexts: [
      {
        calleeKey: "classForVariant",
        callSiteStack: ["RouteA.tsx:render", "PrimaryButton.tsx:className"],
        value: { kind: "exact", value: "btn-primary" },
      },
      {
        calleeKey: "classForVariant",
        callSiteStack: ["RouteB.tsx:render", "SecondaryButton.tsx:className"],
        value: { kind: "exact", value: "btn-secondary" },
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
      "omena-checker-k-limited-flow-m-tier-evaluations",
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
    `engine-shadow-runner k-limited flow command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as KLimitedFlowSummary;
}

function contextBySuffix(summary: KLimitedFlowSummary, suffix: string): KLimitedFlowContext {
  const context = summary.contexts.find((entry) => entry.contextKey.endsWith(suffix));
  assert.ok(context, `missing context ending with ${suffix}`);
  return context;
}

interface KLimitedFlowSummary {
  readonly product: string;
  readonly flowProduct: string;
  readonly contextSensitivity: string;
  readonly evaluationCount: number;
  readonly contexts: readonly KLimitedFlowContext[];
}

interface KLimitedFlowContext {
  readonly contextKey: string;
  readonly evaluationCount: number;
  readonly ruleCodeNames: readonly string[];
}
