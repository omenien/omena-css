import { strict as assert } from "node:assert";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";

import {
  buildCheckPlan,
  loadCheckManifest,
  resolveGateTarget,
} from "../packages/check-orchestrator/src";

const OMENA_CHECK_TARGET_REF =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(run|bundle)\s+([A-Za-z0-9:_@/.-]+)/g;
const OMENA_CHECK_MATRIX_TARGET_REF = /^\s+target:\s+([A-Za-z0-9:_@/.-]+)\s*$/gm;
const OMENA_CHECK_MATRIX_TARGET_BINDING =
  /^\s*OMENA_CHECK_TARGET:\s*\$\{\{\s*matrix\.target\s*\}\}\s*$/m;
const OMENA_CHECK_MATRIX_TARGET_INVOCATION =
  /\bpnpm\s+(?:run\s+)?omena-check\s+(?:run|bundle)\s+["']?\$OMENA_CHECK_TARGET\b/;

const REQUIRED_BENCHMARK_GATES = [
  "rust/bundler-productization-benchmark",
  "rust/benchmark/emitted-css-golden-gate",
  "rust/benchmark/headline-axis",
  "rust/benchmark/instruction-count-advisory",
  "rust/benchmark/transform-relex-baseline",
  "rust/omena-diff-test-wpt-perf",
  "rust/omena-diff-test-wpt-perf-record",
  "rust/z5-parser-product-cutover",
  "rust/z5-perf-baseline",
  "rust/z5-perf-complexity-slope",
  "rust/demand-sliced-monotone-fact-propagation-relocation-gate-bound",
  "rust/z5-perf-no-regression",
  "rust/z5-perf-per-file-invariant",
  "rust/z5-perf-warmup-wave-count",
  "rust/z5-performance-baseline-micro",
  "rust/z5-performance-baseline-macro",
  "rust/z5-performance-baseline-readiness",
] as const;

const root = process.cwd();
const manifest = loadCheckManifest(root);
const reachable = collectWorkflowReachableGateIds();
const missing = REQUIRED_BENCHMARK_GATES.filter((id) => !reachable.has(id));

assert.deepEqual(
  missing,
  [],
  `benchmark check id(s) are not reachable from any workflow: ${missing.join(", ")}`,
);

const ci = read(".github/workflows/ci.yml");
assert.ok(
  ci.includes(
    "OMENA_DEMAND_SLICED_MONOTONE_FACT_PROPAGATION_RELOCATION_APPROVAL_REPORT: .omena-ci/demand-sliced-monotone-fact-propagation-relocation-approval.json",
  ),
  "CI must name the live relocation approval artifact consumed by the demand route",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/benchmark/emitted-css-golden-gate"),
  "CI must hard-run the emitted CSS golden gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/benchmark/headline-axis"),
  "CI must hard-run the headline-axis fidelity gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/benchmark/transform-relex-baseline"),
  "CI must hard-run the transform re-lex baseline gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/z5-parser-product-cutover"),
  "CI must hard-run the parser-product cutover gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/z5-perf-baseline"),
  "CI must hard-run the z5 perf baseline gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/bundler-productization-benchmark"),
  "CI must hard-run the bundler productization benchmark gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/z5-perf-per-file-invariant"),
  "CI must hard-run the z5 per-file invalidation perf gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/z5-perf-complexity-slope"),
  "CI must hard-run the z5 complexity-slope perf gate",
);
assert.ok(
  ci.includes(
    "pnpm omena-check run rust/demand-sliced-monotone-fact-propagation-relocation-gate-bound",
  ),
  "CI must hard-run the demand-sliced monotone fact propagation bound relocation gate after the slope report is produced",
);
assert.ok(
  ci.indexOf("pnpm omena-check run rust/z5-perf-complexity-slope") <
    ci.indexOf(
      "pnpm omena-check run rust/demand-sliced-monotone-fact-propagation-relocation-gate-bound",
    ),
  "CI must run the complexity-slope producer before the bound relocation gate consumes its report",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/z5-perf-warmup-wave-count"),
  "CI must hard-run the z5 warm-up wave-count perf gate",
);
assert.ok(
  ci.includes("pnpm omena-check run rust/z5-perf-no-regression"),
  "CI must hard-run the z5 instruction-count regression perf gate",
);

const nightlySoak = read(".github/workflows/nightly-soak.yml");
const scheduledFactPropagationStart = nightlySoak.indexOf(
  "  demand-sliced-monotone-fact-propagation-settle-soak:",
);
assert.ok(
  scheduledFactPropagationStart >= 0,
  "nightly soak must retain the demand-sliced monotone fact propagation job",
);
const scheduledFactPropagationEnd = nightlySoak.indexOf(
  "\n  # NOTE:",
  scheduledFactPropagationStart,
);
const scheduledFactPropagationJob = nightlySoak.slice(
  scheduledFactPropagationStart,
  scheduledFactPropagationEnd === -1 ? undefined : scheduledFactPropagationEnd,
);
assert.ok(
  scheduledFactPropagationJob.includes(
    "OMENA_Z5_COMPLEXITY_SLOPE_REPORT: .omena-ci/z5-complexity-slope-report.json",
  ),
  "scheduled demand-sliced monotone fact propagation soak must name the live slope artifact",
);
assert.ok(
  scheduledFactPropagationJob.includes(
    "OMENA_DEMAND_SLICED_MONOTONE_FACT_PROPAGATION_RELOCATION_APPROVAL_REPORT: .omena-ci/demand-sliced-monotone-fact-propagation-relocation-approval.json",
  ),
  "scheduled demand-sliced monotone fact propagation soak must name the live relocation approval artifact",
);
const scheduledSlopeIndex = scheduledFactPropagationJob.indexOf(
  "pnpm omena-check run rust/z5-perf-complexity-slope",
);
const scheduledApprovalIndex = scheduledFactPropagationJob.indexOf(
  "pnpm omena-check run rust/demand-sliced-monotone-fact-propagation-relocation-gate-bound",
);
const scheduledSettleIndex = scheduledFactPropagationJob.indexOf(
  "pnpm omena-check run rust/demand-sliced-monotone-fact-propagation-settle-soak",
);
assert.ok(scheduledSlopeIndex >= 0, "scheduled soak must produce live slope evidence");
assert.ok(
  scheduledApprovalIndex > scheduledSlopeIndex,
  "scheduled soak must bind the demand route after live slope evidence is produced",
);
assert.ok(
  scheduledSettleIndex > scheduledApprovalIndex,
  "scheduled soak must exercise settle stability after the approval-bound route",
);

const benchmarkRegression = read(".github/workflows/benchmark-regression.yml");
const wptPolicyIndex = benchmarkRegression.indexOf(
  "pnpm omena-check run rust/omena-diff-test-wpt-perf",
);
const wptRecordIndex = benchmarkRegression.indexOf(
  "pnpm omena-check run rust/omena-diff-test-wpt-perf-record",
);
assert.ok(wptPolicyIndex >= 0, "benchmark regression must validate the WPT perf policy");
assert.ok(
  wptRecordIndex > wptPolicyIndex,
  "benchmark regression must validate WPT perf policy before recording a sample",
);

const drift = read(".github/workflows/omena-css-drift.yml");
assert.ok(
  !drift.includes("continue-on-error: true"),
  "Omena CSS drift workflow must not mask benchmark readiness failures",
);

console.log(
  JSON.stringify({
    schemaVersion: "0",
    product: "rust.benchmark-ci-reachability",
    requiredBenchmarkGateCount: REQUIRED_BENCHMARK_GATES.length,
    reachableBenchmarkGateCount: REQUIRED_BENCHMARK_GATES.length - missing.length,
    requiredBenchmarkGates: REQUIRED_BENCHMARK_GATES,
  }),
);

function collectWorkflowReachableGateIds(): Set<string> {
  const workflowsDir = path.join(root, ".github", "workflows");
  const ids = new Set<string>();
  if (!existsSync(workflowsDir)) return ids;

  for (const fileName of readdirSync(workflowsDir).toSorted()) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;
    const workflowText = read(path.join(".github", "workflows", fileName));
    const targets = Array.from(workflowText.matchAll(OMENA_CHECK_TARGET_REF), (match) => match[2]);
    if (
      OMENA_CHECK_MATRIX_TARGET_BINDING.test(workflowText) &&
      OMENA_CHECK_MATRIX_TARGET_INVOCATION.test(workflowText)
    ) {
      targets.push(
        ...Array.from(workflowText.matchAll(OMENA_CHECK_MATRIX_TARGET_REF), (match) => match[1]),
      );
    }

    for (const target of targets) {
      if (!target) continue;
      const gate = resolveGateTarget(manifest, target);
      if (!gate) {
        throw new Error(`${fileName} references unknown omena-check target: ${target}`);
      }
      for (const step of buildCheckPlan(manifest, gate).steps) {
        ids.add(step.id);
      }
    }
  }
  return ids;
}

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}
